//! rusty-bun-host — JS host integration for the rusty-bun derivation pilots.
//!
//! Per the rusty-bun engagement seed §VII (Sub-criterion 4: JS host
//! integration). This crate embeds rquickjs (a Rust binding for QuickJS)
//! and exposes existing pilots to JS code, transforming the piloted
//! surfaces from "Rust modules with Rust tests" into "callable from JS".
//!
//! Wired surfaces (in order of layer):
//!   console.log / .error / .warn       (host primitive; not a pilot)
//!   atob, btoa                         via rusty-buffer
//!   path.* (POSIX subset)              via rusty-node-path
//!   crypto.randomUUID                  via rusty-web-crypto
//!   crypto.subtle.digest("SHA-256")   via rusty-web-crypto
//!   TextEncoder / TextDecoder          via rusty-textencoder
//!   Buffer.alloc / .from / .byteLength / .concat
//!                                      via rusty-buffer
//!   URLSearchParams (full surface)    via rusty-urlsearchparams
//!   fs.readFileSync / .writeFileSync /
//!     .existsSync / .statSync          via rusty-node-fs

use rquickjs::{
    function::Opt,
    loader::{Loader, Resolver},
    module::Declared,
    Context, Ctx, Error as JsErr, Function, Module, Object, Result as JsResult, Runtime, Value,
};

mod reactor;

/// Build a fresh rquickjs Runtime + Context with all rusty-bun pilots wired
/// into globalThis. Includes the ESM node-style module resolver/loader
/// (Tier-H.3); CommonJS is still wired JS-side via `bootRequire(absPath)`.
pub fn new_runtime() -> JsResult<(Runtime, Context)> {
    let runtime = Runtime::new()?;
    // Bump the QuickJS stack ceiling. Default (~256KB) trips on
    // codegen-heavy libs like ajv that recursively walk schema ASTs.
    // 8MB matches V8/Bun's default; rusty-bun is not memory-pressured
    // at engagement scale so the larger ceiling is safe.
    runtime.set_max_stack_size(8 * 1024 * 1024);
    runtime.set_loader(NodeResolver, FsLoader);
    let context = Context::full(&runtime)?;
    context.with(|ctx| -> JsResult<()> {
        wire_globals(ctx)?;
        Ok(())
    })?;
    Ok((runtime, context))
}

// ════════════════════════════════════════════════════════════════════════
// ESM resolver + loader (Tier-H.3 #2)
// ════════════════════════════════════════════════════════════════════════
//
// Node-style resolution for ESM imports: relative (./, ../), absolute,
// and bare specifiers walking node_modules. Mirrors the JS-side CJS
// resolver in COMMONJS_LOADER_JS, in Rust against std::fs.

const ESM_EXTENSIONS: &[&str] = &["", ".mjs", ".js", ".cjs"];
const ESM_INDEX_FILES: &[&str] = &["index.mjs", "index.js", "index.cjs"];

fn try_extensions(abs_path: &std::path::Path) -> Option<std::path::PathBuf> {
    for ext in ESM_EXTENSIONS {
        let candidate = if ext.is_empty() {
            abs_path.to_path_buf()
        } else {
            let mut s = abs_path.as_os_str().to_owned();
            s.push(ext);
            std::path::PathBuf::from(s)
        };
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Resolve a conditional exports value (string or nested object) given the
/// priority-ordered conditions list. Returns the first matching string path.
fn resolve_exports_value(
    value: &serde_json::Value, conditions: &[&str],
) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Object(map) => {
            for cond in conditions {
                if let Some(v) = map.get(*cond) {
                    if let Some(r) = resolve_exports_value(v, conditions) {
                        return Some(r);
                    }
                }
            }
            // Fall back to "default" key if present.
            if let Some(v) = map.get("default") {
                return resolve_exports_value(v, conditions);
            }
            None
        }
        _ => None,
    }
}

fn try_directory_with_index(abs_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let pkg_json = abs_dir.join("package.json");
    if pkg_json.is_file() {
        if let Ok(text) = std::fs::read_to_string(&pkg_json) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                // Per Node + Bun semantics, `exports` (when present) takes
                // priority over `main`/`module`. Conditions match Bun's
                // priority order so consumer libraries with a `bun` build
                // condition land on the same entry point.
                if let Some(exports) = parsed.get("exports") {
                    // exports may itself be a string (single export) or
                    // an object whose "." key is the root export.
                    let conditions = ["bun", "import", "module", "node", "default"];
                    let root = match exports {
                        serde_json::Value::String(_) => Some(exports.clone()),
                        serde_json::Value::Object(map) => map.get(".").cloned()
                            .or_else(|| {
                                // No "." subpath but conditions at top level
                                // (e.g. {"import": "./x.js"}). Treat the
                                // whole object as the root conditional set.
                                let has_subpath = map.keys().any(|k| k.starts_with('.'));
                                if has_subpath { None } else { Some(exports.clone()) }
                            }),
                        _ => None,
                    };
                    if let Some(r) = root {
                        if let Some(rel) = resolve_exports_value(&r, &conditions) {
                            let target = abs_dir.join(rel.trim_start_matches("./"));
                            if let Some(f) = try_extensions(&target) {
                                return Some(f);
                            }
                            if target.is_file() {
                                return Some(target);
                            }
                            if target.is_dir() {
                                if let Some(f) = try_directory_with_index(&target) {
                                    return Some(f);
                                }
                            }
                        }
                    }
                }
                let main_str = parsed
                    .get("module")
                    .or_else(|| parsed.get("main"))
                    .and_then(|v| v.as_str());
                if let Some(main) = main_str {
                    // ESM-preference heuristic: when a package has no
                    // `module` field and no `exports` map but ships a
                    // parallel `esm/` directory, prefer the ESM build —
                    // matches the dayjs/moment/many-UMD-libs convention
                    // and keeps the loader from choking on UMD's
                    // `module.exports = ...` references.
                    let has_module_field = parsed.get("module").is_some();
                    if !has_module_field {
                        for esm_path in &["esm/index.mjs", "esm/index.js"] {
                            let esm_target = abs_dir.join(esm_path);
                            if let Some(f) = try_extensions(&esm_target) {
                                return Some(f);
                            }
                        }
                    }
                    let target = abs_dir.join(main);
                    if let Some(f) = try_extensions(&target) {
                        return Some(f);
                    }
                    if target.is_dir() {
                        if let Some(f) = try_directory_with_index(&target) {
                            return Some(f);
                        }
                    }
                }
            }
        }
    }
    for idx in ESM_INDEX_FILES {
        let candidate = abs_dir.join(idx);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn split_bare_specifier(specifier: &str) -> (&str, &str) {
    if let Some(stripped) = specifier.strip_prefix('@') {
        let rest = stripped;
        if let Some(slash) = rest.find('/') {
            let after = &rest[slash + 1..];
            if let Some(second) = after.find('/') {
                let pkg_end = 1 + slash + 1 + second;
                return (&specifier[..pkg_end], &specifier[pkg_end..]);
            }
        }
        return (specifier, "");
    }
    if let Some(slash) = specifier.find('/') {
        (&specifier[..slash], &specifier[slash..])
    } else {
        (specifier, "")
    }
}

fn resolve_node_style(base: &str, specifier: &str) -> Option<std::path::PathBuf> {
    use std::path::{Path, PathBuf};

    let base_dir: PathBuf = if base.is_empty() {
        std::env::current_dir().ok()?
    } else {
        let p = Path::new(base);
        if p.is_dir() {
            p.to_path_buf()
        } else if let Some(d) = p.parent() {
            d.to_path_buf()
        } else {
            std::env::current_dir().ok()?
        }
    };

    if specifier.starts_with("./") || specifier.starts_with("../") || specifier == "." || specifier == ".." {
        let joined = base_dir.join(specifier);
        let normalized = normalize_path(&joined);
        if let Some(f) = try_extensions(&normalized) {
            return Some(f);
        }
        if normalized.is_dir() {
            return try_directory_with_index(&normalized);
        }
        return None;
    }

    // Package-internal subpath imports (#-prefix per Node spec).
    // Walk up from base_dir to find the closest package.json with an
    // "imports" field matching the specifier; resolve via the same
    // conditional-key walk as exports. Chalk ^5 uses this for its
    // bundled ansi-styles vendor copy.
    if specifier.starts_with('#') {
        let mut dir = base_dir.as_path();
        loop {
            let pkg_json = dir.join("package.json");
            if pkg_json.is_file() {
                if let Ok(text) = std::fs::read_to_string(&pkg_json) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(imports) = parsed.get("imports") {
                            if let Some(value) = imports.get(specifier) {
                                let conditions = ["bun", "import", "module", "node", "default"];
                                if let Some(rel) = resolve_exports_value(value, &conditions) {
                                    let target = dir.join(rel.trim_start_matches("./"));
                                    if let Some(f) = try_extensions(&target) {
                                        return Some(f);
                                    }
                                    if target.is_file() {
                                        return Some(target);
                                    }
                                }
                            }
                            // Also allow pattern matching (#foo/*) — best-effort.
                            if let Some(obj) = imports.as_object() {
                                for (k, v) in obj {
                                    if k.ends_with("/*") {
                                        let prefix = &k[..k.len() - 2];
                                        if specifier.starts_with(prefix) && specifier.len() > prefix.len() {
                                            let suffix = &specifier[prefix.len() + 1..];
                                            let conditions = ["bun", "import", "module", "node", "default"];
                                            if let Some(rel) = resolve_exports_value(v, &conditions) {
                                                let resolved = rel.replace("*", suffix);
                                                let target = dir.join(resolved.trim_start_matches("./"));
                                                if let Some(f) = try_extensions(&target) {
                                                    return Some(f);
                                                }
                                                if target.is_file() {
                                                    return Some(target);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            // Package found but specifier not in imports.
                            // Don't keep walking up — exit per Node spec
                            // (imports is scoped to the closest pkg.json).
                            break;
                        }
                        // No imports field — keep walking up; the # may
                        // belong to a parent package.
                    }
                }
            }
            match dir.parent() {
                Some(p) if p != dir => dir = p,
                _ => break,
            }
        }
        return None;
    }

    if specifier.starts_with('/') {
        let p = PathBuf::from(specifier);
        if let Some(f) = try_extensions(&p) {
            return Some(f);
        }
        if p.is_dir() {
            return try_directory_with_index(&p);
        }
        return None;
    }

    let (pkg_name, sub_path) = split_bare_specifier(specifier);
    let mut dir = base_dir.as_path();
    loop {
        let pkg_root = dir.join("node_modules").join(pkg_name);
        if pkg_root.is_dir() {
            if sub_path.is_empty() {
                if let Some(f) = try_directory_with_index(&pkg_root) {
                    return Some(f);
                }
            } else {
                // Subpath request. First try the exports map's
                // matching "./<sub>" key (with conditional walk).
                let sub = sub_path.trim_start_matches('/');
                let exports_key = format!("./{}", sub);
                let pkg_json = pkg_root.join("package.json");
                if pkg_json.is_file() {
                    if let Ok(text) = std::fs::read_to_string(&pkg_json) {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(exports) = parsed.get("exports") {
                                let conditions = ["bun", "import", "module", "node", "default"];
                                if let Some(value) = exports.get(&exports_key) {
                                    if let Some(rel) = resolve_exports_value(value, &conditions) {
                                        let target = pkg_root.join(rel.trim_start_matches("./"));
                                        if let Some(f) = try_extensions(&target) { return Some(f); }
                                        if target.is_file() { return Some(target); }
                                    }
                                }
                                // Pattern-match form: "./foo/*"
                                if let Some(obj) = exports.as_object() {
                                    for (k, v) in obj {
                                        if k.ends_with("/*") {
                                            let prefix = &k[..k.len() - 2];
                                            if exports_key.starts_with(prefix)
                                                && exports_key.len() > prefix.len()
                                            {
                                                let suffix = &exports_key[prefix.len() + 1..];
                                                if let Some(rel) = resolve_exports_value(v, &conditions) {
                                                    let resolved = rel.replace("*", suffix);
                                                    let target = pkg_root.join(resolved.trim_start_matches("./"));
                                                    if let Some(f) = try_extensions(&target) { return Some(f); }
                                                    if target.is_file() { return Some(target); }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // Fall back to direct subpath join (legacy non-exports
                // packages or packages whose exports allows it).
                let target = pkg_root.join(sub);
                if let Some(f) = try_extensions(&target) {
                    return Some(f);
                }
                if target.is_dir() {
                    if let Some(f) = try_directory_with_index(&target) {
                        return Some(f);
                    }
                }
            }
        }
        match dir.parent() {
            Some(p) if p != dir => dir = p,
            _ => break,
        }
    }
    None
}

fn normalize_path(p: &std::path::Path) -> std::path::PathBuf {
    let mut out = std::path::PathBuf::new();
    for c in p.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[derive(Default, Clone, Copy)]
struct NodeResolver;

impl Resolver for NodeResolver {
    fn resolve<'js>(&mut self, _ctx: &Ctx<'js>, base: &str, name: &str) -> JsResult<String> {
        // node:* and bare-builtin names short-circuit; FsLoader recognizes them.
        if is_node_builtin(name) {
            return Ok(name.to_string());
        }
        match resolve_node_style(base, name) {
            Some(p) => Ok(p.to_string_lossy().into_owned()),
            None => Err(JsErr::new_resolving(base, name)),
        }
    }
}

fn is_node_builtin(name: &str) -> bool {
    matches!(name,
        "node:fs" | "fs" |
        "node:path" | "path" |
        "node:http" | "http" |
        "node:crypto" | "crypto" |
        "node:buffer" | "buffer" |
        "node:url" | "url" |
        "node:os" | "os" |
        "node:process" | "process" |
        "node:dns" | "dns" |
        "node:dns/promises" | "dns/promises" |
        "node:events" | "events" |
        "node:util" | "util" |
        "node:util/types" | "util/types" |
        "node:stream" | "stream" |
        "node:stream/promises" | "stream/promises" |
        "node:querystring" | "querystring" |
        "node:assert" | "assert" |
        "node:assert/strict" | "assert/strict" |
        "node:child_process" | "child_process" |
        "node:net" | "net" |
        "node:tty" | "tty" |
        "node:zlib" | "zlib" |
        "node:diagnostics_channel" | "diagnostics_channel" |
        "node:https" | "https" |
        "node:perf_hooks" | "perf_hooks" |
        "node:async_hooks" | "async_hooks" |
        "node:timers" | "timers" |
        "node:timers/promises" | "timers/promises" |
        "node:console" | "console" |
        "node:fs/promises" | "fs/promises" |
        "node:stream/web" | "stream/web" |
        "node:test" | "test" |
        "node:worker_threads" | "worker_threads" |
        "node:http2" | "http2" |
        "node:vm" | "vm" |
        "node:string_decoder" | "string_decoder" |
        "node:readline" | "readline" |
        "node:readline/promises" | "readline/promises" |
        "node:module" | "module" |
        "node:cluster" | "cluster" |
        "node:tls" | "tls" |
        "node:v8" | "v8" |
        "node:constants" | "constants"
    )
}

/// Generate an ESM re-export source for a node:* builtin module.
/// Per M8/M9: aligns ESM import semantics for node-builtins with Bun's
/// surface, so consumer code using `import x from "node:path"` works.
fn node_builtin_esm_source(name: &str) -> Option<String> {
    let (global_var, named_exports): (&str, &[&str]) = match name {
        "node:fs" | "fs" => ("fs", &["readFileSync", "readFileSyncUtf8", "readFileSyncBytes",
            "writeFileSync", "existsSync", "isFileSync", "isDirectorySync",
            "unlinkSync", "mkdirSyncRecursive", "rmdirSyncRecursive",
            "statSync", "lstatSync", "readdirSync", "realpathSync",
            "readlinkSync", "promises",
            "stat", "lstat", "readdir", "readlink", "realpath", "mkdir",
            "rm", "unlink", "access", "readFile", "writeFile",
            "Dirent", "Stats",
            "appendFileSync", "rmSync", "copyFileSync", "renameSync",
            "chmodSync", "utimesSync", "mkdirSync", "rmdirSync",
            "createReadStream", "createWriteStream", "openSync", "closeSync",
            "writeSync", "readSync", "fstatSync", "ftruncateSync", "fsyncSync",
            "watchFile", "unwatchFile", "watch", "constants"]),
        "node:path" | "path" => ("path", &["basename", "dirname", "extname", "join",
            "normalize", "isAbsolute", "resolve", "relative", "parse", "format",
            "sep", "delimiter", "posix", "win32"]),
        "node:http" | "http" => ("nodeHttp", &["createServer", "request", "get",
            "IncomingMessage", "ServerResponse", "ClientRequest", "Server",
            "METHODS", "STATUS_CODES", "globalAgent", "Agent"]),
        // webcrypto is the Web Crypto API namespace (nanoid + many libs
        // pull it from node:crypto). It is structurally identical to the
        // globalThis.crypto object (.subtle + .getRandomValues + .randomUUID).
        "node:crypto" | "crypto" => ("crypto", &["randomUUID", "subtle",
            "webcrypto", "getRandomValues"]),
        "node:buffer" | "buffer" => ("Buffer", &[]),  // see special handling below
        "node:url" | "url" => ("URL", &[]),
        "node:os" | "os" => ("os", &["platform", "arch", "type", "tmpdir",
            "homedir", "hostname", "endianness", "EOL",
            "cpus", "freemem", "totalmem", "uptime", "loadavg",
            "networkInterfaces", "userInfo", "constants"]),
        "node:process" | "process" => ("process", &["argv", "env", "platform",
            "arch", "version", "versions", "cwd", "exit", "stdout", "stderr",
            "hrtime", "pid", "ppid", "nextTick", "execPath", "execArgv",
            "title", "argv0", "allowedNodeEnvironmentFlags", "stdin",
            "on", "once", "off", "emit", "removeListener", "removeAllListeners",
            "kill", "umask", "uptime", "getuid", "getgid"]),
        "node:dns" | "dns" => ("nodeDns", &["lookup", "resolve", "resolve4",
            "resolve6", "promises"]),
        "node:dns/promises" | "dns/promises" => ("nodeDnsPromises",
            &["lookup", "resolve", "resolve4", "resolve6"]),
        "node:events" | "events" => ("nodeEvents", &["EventEmitter",
            "once", "on", "captureRejectionSymbol", "errorMonitor",
            "setMaxListeners", "getEventListeners", "addAbortListener"]),
        "node:util" | "util" => ("nodeUtil", &["promisify", "callbackify",
            "format", "formatWithOptions", "inspect", "isDeepStrictEqual",
            "deprecate", "debuglog", "inherits", "types", "TextEncoder", "TextDecoder",
            "styleText", "parseArgs", "stripVTControlCharacters", "getSystemErrorName",
            "aborted", "MIMEType", "MIMEParams", "deprecationWarned",
            "getCallSite", "getSystemErrorMap", "transferableAbortController",
            "transferableAbortSignal", "parseEnv", "_extend"]),
        "node:util/types" | "util/types" => ("nodeUtilTypes", &[
            "isPromise", "isDate", "isRegExp", "isMap", "isSet",
            "isArrayBuffer", "isTypedArray", "isUint8Array", "isInt8Array",
            "isAsyncFunction", "isGeneratorFunction"]),
        "node:stream" | "stream" => ("nodeStream", &["Readable", "Writable",
            "Duplex", "Transform", "PassThrough", "pipeline", "finished",
            "Stream", "getDefaultHighWaterMark", "setDefaultHighWaterMark",
            "isReadable", "isWritable", "compose"]),
        "node:stream/promises" | "stream/promises" => ("nodeStreamPromises",
            &["pipeline", "finished"]),
        "node:querystring" | "querystring" => ("nodeQuerystring",
            &["parse", "stringify", "escape", "unescape", "decode", "encode"]),
        "node:assert" | "assert" => ("nodeAssert", &["ok", "equal", "notEqual",
            "strictEqual", "notStrictEqual", "deepEqual", "notDeepEqual",
            "deepStrictEqual", "notDeepStrictEqual", "throws", "doesNotThrow",
            "rejects", "doesNotReject", "match", "doesNotMatch", "fail",
            "ifError", "strict", "AssertionError"]),
        "node:child_process" | "child_process" => ("nodeChildProcess",
            &["spawn", "spawnSync", "exec", "execSync", "execFile",
              "execFileSync", "fork", "ChildProcess"]),
        "node:net" | "net" => ("nodeNet",
            &["Socket", "connect", "createConnection", "createServer",
              "isIP", "isIPv4", "isIPv6", "BlockList", "SocketAddress"]),
        "node:tty" | "tty" => ("nodeTty",
            &["isatty", "ReadStream", "WriteStream"]),
        "node:zlib" | "zlib" => ("nodeZlib",
            &["gzipSync", "gunzipSync", "deflateSync", "inflateSync",
              "deflateRawSync", "inflateRawSync",
              "createGzip", "createGunzip", "createDeflate", "createInflate",
              "constants"]),
        "node:diagnostics_channel" | "diagnostics_channel" =>
            ("nodeDiagnosticsChannel",
             &["channel", "hasSubscribers", "subscribe", "unsubscribe",
               "tracingChannel", "Channel", "TracingChannel"]),
        "node:https" | "https" => ("nodeHttps",
            &["createServer", "request", "get", "Agent", "globalAgent",
              "Server"]),
        "node:perf_hooks" | "perf_hooks" => ("nodePerfHooks",
            &["performance", "PerformanceObserver", "monitorEventLoopDelay",
              "constants"]),
        "node:async_hooks" | "async_hooks" => ("nodeAsyncHooks",
            &["AsyncLocalStorage", "AsyncResource", "createHook",
              "executionAsyncId", "executionAsyncResource", "triggerAsyncId"]),
        "node:timers" | "timers" => ("nodeTimers",
            &["setTimeout", "setInterval", "setImmediate",
              "clearTimeout", "clearInterval", "clearImmediate"]),
        "node:timers/promises" | "timers/promises" => ("nodeTimersPromises",
            &["setTimeout", "setInterval", "setImmediate", "scheduler"]),
        "node:console" | "console" => ("nodeConsoleModule",
            &["Console", "log", "warn", "error", "info", "debug"]),
        "node:fs/promises" | "fs/promises" => ("nodeFsPromises",
            &["readFile", "writeFile", "mkdir", "rm", "stat", "lstat",
              "access", "readdir", "unlink", "readlink", "realpath",
              "rename", "rmdir", "chmod", "chown", "utimes", "appendFile",
              "copyFile", "open", "constants", "watch", "cp", "lchmod",
              "lchown", "link", "lutimes", "mkdtemp", "opendir", "symlink",
              "truncate"]),
        "node:stream/web" | "stream/web" => ("nodeStreamWeb",
            &["ReadableStream", "WritableStream", "TransformStream",
              "ByteLengthQueuingStrategy", "CountQueuingStrategy",
              "TextEncoderStream", "TextDecoderStream"]),
        "node:test" | "test" => ("nodeTest",
            &["test", "describe", "it", "before", "after",
              "beforeEach", "afterEach", "mock"]),
        "node:worker_threads" | "worker_threads" => ("nodeWorkerThreads",
            &["Worker", "isMainThread", "parentPort", "workerData",
              "threadId", "MessageChannel", "MessagePort"]),
        "node:http2" | "http2" => ("nodeHttp2",
            &["constants", "createServer", "createSecureServer",
              "connect", "Http2ServerRequest", "Http2ServerResponse"]),
        "node:vm" | "vm" => ("nodeVm",
            &["Script", "createContext", "runInNewContext",
              "runInThisContext", "compileFunction", "isContext"]),
        "node:string_decoder" | "string_decoder" =>
            ("nodeStringDecoder", &["StringDecoder"]),
        "node:readline" | "readline" => ("nodeReadline",
            &["createInterface", "Interface", "emitKeypressEvents",
              "clearLine", "clearScreenDown", "cursorTo", "moveCursor"]),
        "node:readline/promises" | "readline/promises" =>
            ("nodeReadlinePromises",
             &["createInterface", "Interface", "Readline"]),
        "node:module" | "module" => ("nodeModule",
            &["createRequire", "builtinModules", "isBuiltin",
              "Module", "syncBuiltinESMExports"]),
        "node:cluster" | "cluster" => ("nodeCluster",
            &["isMaster", "isPrimary", "isWorker", "worker", "workers",
              "schedulingPolicy", "SCHED_NONE", "SCHED_RR",
              "fork", "disconnect", "setupMaster", "setupPrimary"]),
        "node:tls" | "tls" => ("nodeTls",
            &["TLSSocket", "connect", "createSecureContext", "rootCertificates",
              "DEFAULT_MIN_VERSION", "DEFAULT_MAX_VERSION", "DEFAULT_CIPHERS",
              "checkServerIdentity"]),
        "node:v8" | "v8" => ("nodeV8",
            &["serialize", "deserialize", "getHeapStatistics",
              "getHeapSpaceStatistics", "cachedDataVersionTag",
              "setFlagsFromString", "writeHeapSnapshot"]),
        "node:constants" | "constants" => ("nodeConstants",
            &["O_RDONLY", "O_WRONLY", "O_RDWR", "O_CREAT", "O_EXCL",
              "O_TRUNC", "O_APPEND", "S_IFMT", "S_IFREG", "S_IFDIR",
              "EACCES", "EEXIST", "ENOENT", "EPERM"]),
        "node:assert/strict" | "assert/strict" => ("nodeAssertStrict",
            &["ok", "equal", "notEqual", "deepEqual", "notDeepEqual",
              "throws", "doesNotThrow", "rejects", "doesNotReject",
              "match", "doesNotMatch", "fail", "ifError",
              "AssertionError"]),
        _ => return None,
    };
    // node:buffer exports `{ Buffer }` not the Buffer itself.
    if name == "node:buffer" || name == "buffer" {
        return Some(
            "const __m = globalThis.Buffer;\nexport const Buffer = __m;\nexport default { Buffer: __m };\n".to_string()
        );
    }
    if name == "node:crypto" || name == "crypto" {
        // webcrypto = globalThis.crypto (Web Crypto API namespace).
        // Node also exposes createHash/createHmac/randomBytes/etc as
        // top-level exports; bind them for consumer compatibility
        // (uuid, md5-hex, jsonwebtoken, etc. import { createHash }).
        return Some(
            "const __c = globalThis.crypto;\n\
             export const randomUUID = __c.randomUUID.bind(__c);\n\
             export const subtle = __c.subtle;\n\
             export const webcrypto = __c;\n\
             export const getRandomValues = __c.getRandomValues.bind(__c);\n\
             export const createHash = __c.createHash;\n\
             export const createHmac = __c.createHmac;\n\
             export const randomBytes = (n) => {\n\
                 const buf = new Uint8Array(n);\n\
                 __c.getRandomValues(buf);\n\
                 return typeof Buffer !== \"undefined\" ? Buffer.from(buf) : buf;\n\
             };\n\
             export const randomFillSync = (buf, offset, size) => {\n\
                 const view = (offset || size)\n\
                     ? new Uint8Array(buf.buffer || buf, (buf.byteOffset || 0) + (offset || 0), size !== undefined ? size : buf.length - (offset || 0))\n\
                     : new Uint8Array(buf.buffer || buf, buf.byteOffset || 0, buf.length);\n\
                 __c.getRandomValues(view);\n\
                 return buf;\n\
             };\n\
             export const pbkdf2Sync = __c.pbkdf2Sync;\n\
             export default __c;\n".to_string()
        );
    }
    if name == "node:url" || name == "url" {
        return Some(
            "const __nu = globalThis.nodeUrl;\n\
             export const URL = __nu.URL;\n\
             export const URLSearchParams = __nu.URLSearchParams;\n\
             export const parse = __nu.parse;\n\
             export const format = __nu.format;\n\
             export const resolve = __nu.resolve;\n\
             export const fileURLToPath = __nu.fileURLToPath;\n\
             export const pathToFileURL = __nu.pathToFileURL;\n\
             export const domainToASCII = __nu.domainToASCII;\n\
             export const domainToUnicode = __nu.domainToUnicode;\n\
             export default __nu;\n".to_string()
        );
    }
    let mut s = format!("const __m = globalThis.{};\nexport default __m;\n", global_var);
    for ex in named_exports {
        s.push_str(&format!("export const {} = __m.{};\n", ex, ex));
    }
    Some(s)
}

/// Pre-eval source transform that renames bare class-field declarations
/// whose name is a reserved word (`get;`, `set;`, `delete;`). QuickJS's
/// parser interprets `get;` inside a class body as a malformed accessor
/// declaration rather than an ES2022 field declaration with a reserved-
/// word name. Modern V8 + JSC accept the syntax. TypeScript output
/// commonly emits these as type-hint placeholders (hono v4, many other
/// libs); they have no runtime effect, so renaming to `_get` / `_set` /
/// `_delete` is semantically transparent.
///
/// Match shape: `^(\s+)(get|set|delete)\s*;\s*$` (anchored, indented).
/// Risk of false positives: a bare `get;` / `set;` / `delete;` statement
/// outside a class body would also be renamed. In practice these are
/// either invalid syntax (`delete;` standalone) or vanishingly rare
/// (`get;` as variable reference statement). Recorded as E.12 closure.
/// Rewrite `export const { a, b, c } = expr;` (top-level destructure-export)
/// into individual `export const a = expr.a;` declarations. QuickJS's ESM
/// implementation doesn't bind destructured names as module exports; the
/// destructure runs but the names are local. Modern V8 + JSC handle this
/// per ES2015 spec. Many CJS-wrapper esm.mjs files use this pattern
/// (commander, others) to re-export named members from a CJS default.
///
/// Pattern: a line starting with `export const {`, followed by zero or
/// more lines of identifier-or-comment, ending with `} = IDENT;` on its
/// own line. The identifier after `=` must be a simple name (the
/// imported binding) — more complex expressions are passed through.
fn rewrite_destructure_exports(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let lines: Vec<&str> = source.split('\n').collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        if trimmed.starts_with("export const {") || trimmed.starts_with("export const{") {
            // Single-line case: `export const { A, B } = NAME;` on one line.
            if let (Some(o), Some(c)) = (line.find('{'), line.rfind('}')) {
                if c > o {
                    let after = &line[c + 1..];
                    if let Some(rest) = after.trim_start().strip_prefix('=') {
                        // Semicolon is optional (ASI).
                        let name_part = rest.trim().trim_end_matches(';').trim();
                        {
                            let name = name_part;
                            let valid_name = !name.is_empty()
                                && name.chars().enumerate().all(|(k, ch)| {
                                    if k == 0 { ch.is_alphabetic() || ch == '_' || ch == '$' }
                                    else { ch.is_alphanumeric() || ch == '_' || ch == '$' }
                                });
                            if valid_name {
                                let inner = &line[o + 1..c];
                                let cleaned: String = inner.split("//").next().unwrap_or("").to_string();
                                let mut names: Vec<String> = Vec::new();
                                for tok in cleaned.split(',') {
                                    let stripped = tok.trim();
                                    if stripped.is_empty() { continue; }
                                    let n = stripped.split(':').next().unwrap_or("").trim();
                                    let valid = !n.is_empty()
                                        && n.chars().enumerate().all(|(k, ch)| {
                                            if k == 0 { ch.is_alphabetic() || ch == '_' || ch == '$' }
                                            else { ch.is_alphanumeric() || ch == '_' || ch == '$' }
                                        });
                                    if valid { names.push(n.to_string()); }
                                }
                                for n in names {
                                    out.push_str(&format!("export const {0} = {1}.{0};\n", n, name));
                                }
                                i += 1;
                                continue;
                            }
                        }
                    }
                }
            }
            // Scan ahead for the matching `} = NAME;` close (multi-line).
            // matched: (last_line_index, synth_name, rhs_expr, names)
            let mut matched: Option<(usize, String, String, Vec<String>)> = None;
            let max_scan = std::cmp::min(lines.len(), i + 500);
            for j in (i + 1)..max_scan {
                let t = lines[j].trim();
                // Match line containing "} = <expr>" anywhere (with the last
                // field optionally inline before the }). Support both
                // simple-name and arbitrary-expression rhs.
                let close_idx = t.find("} = ").or_else(|| t.find("} ="));
                if let Some(idx) = close_idx {
                    // Take everything before the } as part of the inner.
                    let rhs_part = &t[idx + 1..];  // starts with '= ' or ' = '
                    let rest = rhs_part.trim_start_matches(|c: char| c.is_whitespace())
                        .trim_start_matches('=').trim();
                    // Use a synthetic name '__destructure_rhs_N' bound to the
                    // expression; emit it as a const then build individual
                    // export consts from it.
                    let rhs = rest.trim_end_matches(';').trim();
                    if rhs.is_empty() { break; }
                    let synth = format!("__destructure_rhs_{}", i);
                    let valid_name = true;
                    let name = synth.clone();
                    let _ = valid_name; let _ = &name;
                    {
                        let joined: String = lines[i..=j].join("\n");
                        if let (Some(o), Some(c)) = (joined.find('{'), joined.rfind('}')) {
                            let inner = &joined[o + 1..c];
                            // Strip /* ... */ block comments (JSDoc between
                            // destructure fields, common in TS-emitted code)
                            // first, then strip // line comments.
                            let mut without_block = String::with_capacity(inner.len());
                            let bytes = inner.as_bytes();
                            let mut k = 0;
                            while k < bytes.len() {
                                if k + 1 < bytes.len() && bytes[k] == b'/' && bytes[k + 1] == b'*' {
                                    k += 2;
                                    while k + 1 < bytes.len() && !(bytes[k] == b'*' && bytes[k + 1] == b'/') { k += 1; }
                                    if k + 1 < bytes.len() { k += 2; }
                                } else {
                                    without_block.push(bytes[k] as char);
                                    k += 1;
                                }
                            }
                            let cleaned: String = without_block.lines().map(|l| {
                                l.split("//").next().unwrap_or("").to_string()
                            }).collect::<Vec<_>>().join("\n");
                            let mut names: Vec<String> = Vec::new();
                            for tok in cleaned.split(',') {
                                let stripped = tok.trim();
                                if stripped.is_empty() { continue; }
                                let n = stripped.split(':').next().unwrap_or("").trim();
                                let valid = !n.is_empty()
                                    && n.chars().enumerate().all(|(k, ch)| {
                                        if k == 0 { ch.is_alphabetic() || ch == '_' || ch == '$' }
                                        else { ch.is_alphanumeric() || ch == '_' || ch == '$' }
                                    });
                                if valid { names.push(n.to_string()); }
                            }
                            matched = Some((j, name.to_string(), rhs.to_string(), names));
                        }
                        break;
                    }
                }
            }
            if let Some((j, name, rhs, names)) = matched {
                // If rhs is a plain identifier, skip the synth and reuse it
                // directly. Otherwise emit the synthetic const.
                let is_bare_ident = !name.starts_with("__destructure_rhs_")
                    || rhs.chars().enumerate().all(|(k, c)| {
                        if k == 0 { c.is_alphabetic() || c == '_' || c == '$' }
                        else { c.is_alphanumeric() || c == '_' || c == '$' }
                    });
                let target = if is_bare_ident && !rhs.is_empty()
                    && rhs.chars().enumerate().all(|(k, c)| {
                        if k == 0 { c.is_alphabetic() || c == '_' || c == '$' }
                        else { c.is_alphanumeric() || c == '_' || c == '$' }
                    }) {
                    rhs.clone()
                } else {
                    out.push_str(&format!("const {} = {};\n", name, rhs));
                    name.clone()
                };
                for n in names {
                    out.push_str(&format!("export const {0} = {1}.{0};\n", n, target));
                }
                i = j + 1;
                continue;
            }
            // Fallback: emit the original line and continue at i+1.
            out.push_str(line);
            out.push('\n');
            i += 1;
            continue;
        }
        out.push_str(line);
        if i < lines.len() - 1 {
            out.push('\n');
        }
        i += 1;
    }
    out
}

fn strip_reserved_class_field_decls(source: &str) -> String {
    // Handle both single-line-class-body form (`    set;`) and inline
    // minified form. Rename `get`/`set`/`delete` field declarations to
    // `_get`/`_set`/`_delete` so QuickJS doesn't reject them as
    // reserved-name fields.
    //
    // CRITICAL: only match patterns that can ONLY be class-field
    // declarations — `{set;` `;set;` followed immediately by another
    // identifier-or-`;`-or-`}`. Bare-statement matches like ` set;`
    // would corrupt identifier references (e.g. `let f = set;`).
    //
    // The class-body context is identified by the `;` or `{` directly
    // preceding the keyword AND the `;` directly following the
    // keyword. We further require the FOLLOWING character to be an
    // identifier-start, `}`, or another field's identifier — which is
    // either `;`-then-identifier or end of class body.
    //
    // For the line-based form (`\nset;` after indentation), the
    // following byte must also be a class-context byte.
    let mut s = source.to_string();
    // Inline minified form: keyword surrounded by `;`/`{` (left) and
    // identifier-start/`}`/`;` (right). Use a single linear scan.
    let kws: &[&str] = &["set", "get", "delete"];
    for kw in kws {
        let needle = format!(";{};", kw);
        let replacement = format!(";_{};", kw);
        s = s.replace(&needle, &replacement);
        let needle = format!("{{{};", kw);
        let replacement = format!("{{_{};", kw);
        s = s.replace(&needle, &replacement);
    }
    // Skip rename for source files that declare the reserved-name as a
    // top-level `var <kw>;` (signals a captured-binding pattern, not a
    // class field — vitest's bundled CJS chunk uses `var set; set = X;`
    // inside requireSet). Pre-compute the per-keyword skip flag.
    let skip_set = source.contains("\nvar set;") || source.starts_with("var set;");
    let skip_get = source.contains("\nvar get;") || source.starts_with("var get;");
    let skip_delete = source.contains("\nvar delete;") || source.starts_with("var delete;");
    let mut out = String::with_capacity(s.len());
    for line in s.split_inclusive('\n') {
        let (body, nl) = match line.strip_suffix('\n') {
            Some(b) => (b, "\n"),
            None => (line, ""),
        };
        let trimmed = body.trim_end();
        let (indent_end, rest) = match trimmed.find(|c: char| !c.is_whitespace()) {
            Some(i) => trimmed.split_at(i),
            None => {
                out.push_str(line);
                continue;
            }
        };
        if (rest == "get;" || rest == "set;" || rest == "delete;")
            && !indent_end.is_empty()
        {
            out.push_str(indent_end);
            out.push('_');
            out.push_str(rest);
            out.push_str(&body[indent_end.len() + rest.len()..]);
            out.push_str(nl);
        } else if !indent_end.is_empty() && (
            (rest.starts_with("set =") || rest.starts_with("set=")) && !skip_set
            || (rest.starts_with("get =") || rest.starts_with("get=")) && !skip_get
            || (rest.starts_with("delete =") || rest.starts_with("delete=")) && !skip_delete
        ) {
            out.push_str(indent_end);
            out.push('_');
            out.push_str(rest);
            out.push_str(&body[indent_end.len() + rest.len()..]);
            out.push_str(nl);
        } else if !indent_end.is_empty() && rest.starts_with("static(") {
            out.push_str(indent_end);
            out.push_str("_static");
            out.push_str(&rest[6..]);
            out.push_str(&body[indent_end.len() + rest.len()..]);
            out.push_str(nl);
        } else {
            out.push_str(line);
        }
    }
    out
}

/// Detect CJS-only source. CJS markers: `module.exports`, `exports.X =`,
/// `require(`. ESM markers (mutually exclusive): top-of-line `export `,
/// `export {`, `export *`, `export default`, `import `, `import {`.
/// The detection is conservative — a file with any ESM marker is treated
/// as ESM. Comment-only matches survive but are rare.
fn looks_like_cjs(source: &str) -> bool {
    // Quick reject: any line starting with `import ` or `export `
    // (after trimming) implies ESM, even if the file also has stray
    // module.exports references.
    for line in source.lines() {
        let t = line.trim_start();
        if t.starts_with("export ") || t.starts_with("export{")
            || t.starts_with("export *") || t.starts_with("export default")
            || t.starts_with("import ") || t.starts_with("import{")
            || t.starts_with("import \"") || t.starts_with("import '") {
            return false;
        }
    }
    // Positive CJS signal.
    source.contains("module.exports") || source.contains("exports.")
        || source.contains("require(")
}

#[derive(Default, Clone, Copy)]
struct FsLoader;

/// Inert-stub module source for libs whose load chain is too heavy to
/// support today (undici, etc.). All named exports are accessors that
/// throw when accessed, except the named ones in `passthrough` which
/// return a noop. The default export is an empty object. Consumers that
/// import the lib but only use a subset of features (e.g., cheerio
/// imports * as undici but only touches undici.Client for fromURL —
/// the dominant cheerio.load() API doesn't need undici at all) load
/// cleanly and only fail if they actually invoke the dynamic surface.
fn inert_stub_esm(known_names: &[&str]) -> String {
    let mut src = String::new();
    src.push_str("const __throw = (n) => { throw new Error('rusty-bun-host: inert stub for ' + n); };\n");
    for n in known_names {
        src.push_str(&format!(
            "export const {0} = new Proxy(function() {{ __throw('{0}'); }}, {{\n\
               get(_t, k) {{ __throw('{0}.' + String(k)); }},\n\
               apply() {{ __throw('{0}'); }},\n\
               construct() {{ __throw('{0}'); }},\n\
             }});\n",
            n
        ));
    }
    src.push_str("export default {};\n");
    src
}

impl Loader for FsLoader {
    fn load<'js>(&mut self, ctx: &Ctx<'js>, name: &str) -> JsResult<Module<'js, Declared>> {
        // Inert-stub interception. undici's CJS load chain pulls in ~30
        // transitive modules including worker_threads internals + TLS
        // sniffing + http/2 framing; stubbing it lets consumers that
        // import undici but only touch a subset (cheerio for fromURL,
        // most use cheerio.load instead) load cleanly. The stub throws
        // on any actual property access, so latent uses fail loudly.
        if name.ends_with("/node_modules/undici/index.js")
            || name.ends_with("/node_modules/undici/index.mjs")
        {
            let src = inert_stub_esm(&[
                "fetch", "Agent", "Client", "Pool", "BalancedPool", "ProxyAgent",
                "MockAgent", "MockPool", "Dispatcher", "Headers", "Request", "Response",
                "FormData", "errors", "interceptors", "buildConnector",
                "getGlobalDispatcher", "setGlobalDispatcher",
                "Connector", "RedirectHandler", "RetryHandler", "MockClient",
                "MockCallHistory", "RoundRobinPool", "EnvHttpProxyAgent",
                "SnapshotAgent", "RetryAgent", "Socks5ProxyAgent", "H2CClient",
                "DecoratorHandler",
            ]);
            return Module::declare(ctx.clone(), name, src);
        }
        if let Some(src) = node_builtin_esm_source(name) {
            return Module::declare(ctx.clone(), name, src);
        }
        let source = std::fs::read_to_string(name)
            .map_err(|_| JsErr::new_loading(name))?;

        // E.13 closure: when an ESM `import` resolves to a CJS-shaped file
        // (no ESM markers, has module.exports/require), evaluate it via
        // bootRequire and synthesize a re-export ESM shim. Mirrors Bun's
        // automatic CJS↔ESM interop for the `import pkg from "cjs-lib"`
        // case. Named exports populated from Object.keys(module.exports).
        // .mjs files are always ESM by Node spec. .cjs files are always CJS.
        // Otherwise fall through to heuristic detection.
        let force_esm = name.ends_with(".mjs");
        let force_cjs = name.ends_with(".cjs");
        if !force_esm && (force_cjs || looks_like_cjs(&source)) {
            if std::env::var("RUSTY_BUN_HOST_DEBUG").is_ok() {
                eprintln!("[fsloader] CJS branch: {}", name);
            }
            // Eagerly evaluate the CJS module and stash module.exports on
            // a globalThis bridge map keyed by absolute path.
            let bridge_init = format!(
                "(function() {{\n\
                   if (!globalThis.__cjsBridge) globalThis.__cjsBridge = {{}};\n\
                   if (!Object.prototype.hasOwnProperty.call(globalThis.__cjsBridge, {0})) {{\n\
                     globalThis.__cjsBridge[{0}] = globalThis.bootRequire({0});\n\
                   }}\n\
                 }})();",
                json_str(name)
            );
            match ctx.eval::<(), _>(bridge_init.as_bytes()) {
                Ok(_) => {}
                Err(e) => {
                    if std::env::var("RUSTY_BUN_HOST_DEBUG").is_ok() {
                        eprintln!("[fsloader] bridge_init failed for {}: {:?}", name, e);
                    }
                    return Err(e);
                }
            }

            // Read the keys for named export generation.
            let keys_code = format!(
                "(function() {{\n\
                   const m = globalThis.__cjsBridge[{}];\n\
                   if (m == null) return [];\n\
                   if (typeof m !== 'object' && typeof m !== 'function') return [];\n\
                   return Object.keys(m);\n\
                 }})()",
                json_str(name)
            );
            let keys: Vec<String> = ctx.eval::<Vec<String>, _>(keys_code.as_bytes())
                .unwrap_or_default();
            if std::env::var("RUSTY_BUN_HOST_DEBUG").is_ok() {
                eprintln!("[fsloader] CJS keys for {}: {:?}", name, keys);
            }

            // Node CJS↔ESM interop: when CJS sets __esModule:true,
            // `import x from cjs` resolves to module.exports.default,
            // not the whole module.exports object. This matters for
            // TS-emitted CJS (unraw, many tsc outputs) where
            // exports.default = fn while named exports are siblings.
            let mut esm_src = format!(
                "const __m = globalThis.__cjsBridge[{0}];\n\
                 const __default = (__m && __m.__esModule && 'default' in __m) ? __m.default : __m;\n\
                 export default __default;\n",
                json_str(name)
            );
            // ESM bodies are strict-mode; reserved words can't be used as
            // binding names in `export const NAME = ...`. ECMAScript 2024
            // reserved set: https://tc39.es/ecma262/#sec-keywords-and-reserved-words
            const RESERVED: &[&str] = &[
                "default", "break", "case", "catch", "class", "const", "continue",
                "debugger", "delete", "do", "else", "enum", "export", "extends",
                "false", "finally", "for", "function", "if", "import", "in",
                "instanceof", "new", "null", "return", "super", "switch", "this",
                "throw", "true", "try", "typeof", "var", "void", "while", "with",
                "yield",
                // Strict-mode-only reserved (ESM is always strict):
                "implements", "interface", "let", "package", "private", "protected",
                "public", "static", "await",
                // Future reserved:
                "abstract", "boolean", "byte", "char", "double", "final", "float",
                "goto", "int", "long", "native", "short", "synchronized", "throws",
                "transient", "volatile",
                // Globals / strict-mode-restricted that QuickJS rejects as
                // lexical binding names ("invalid lexical variable name"):
                "undefined", "eval", "arguments",
            ];
            for k in keys {
                if RESERVED.contains(&k.as_str()) { continue; }
                let valid = !k.is_empty()
                    && k.chars().enumerate().all(|(i, c)| {
                        if i == 0 { c.is_alphabetic() || c == '_' || c == '$' }
                        else { c.is_alphanumeric() || c == '_' || c == '$' }
                    });
                if !valid { continue; }
                esm_src.push_str(&format!("export const {0} = __m.{0};\n", k));
            }
            return Module::declare(ctx.clone(), name, esm_src);
        }

        let source = strip_reserved_class_field_decls(&source);
        let source = rewrite_destructure_exports(&source);
        let source = strip_string_module_exports_alias(&source);
        let source = rewrite_regex_u_class_escapes(&source);
        let declared = Module::declare(ctx.clone(), name, source)?;
        // Per Doc 714 sub-§4.c — populate import.meta.url for each
        // transitively-loaded module. Prettier (and many modern libs)
        // call fileURLToPath(import.meta.url) at top of file; without
        // this they crash with "argument 'url' must be a file URL".
        if let Ok(meta) = declared.meta() {
            let _ = meta.set("url", format!("file://{}", name));
            let _ = meta.set("main", false);
        }
        Ok(declared)
    }
}

/// S6 closure: rewrite `\-` inside character classes within regex
/// literals carrying /u flag. ECMAScript permits but QuickJS strict
/// /u rejects. Replace with `-` (literal hyphen, /u-clean).
/// Strip `export { X as 'module.exports' }` lines. ECMAScript permits
/// string-literal export names as of the "Arbitrary module namespace
/// identifier names" proposal (Stage 4), but QuickJS rejects names
/// containing `.`. The CJS-compat idiom is meaningless to ESM consumers
/// anyway; just dropping the export keeps the module load-able.
fn strip_string_module_exports_alias(source: &str) -> String {
    if !source.contains("as 'module.exports'") && !source.contains("as \"module.exports\"") {
        return source.to_string();
    }
    let mut out = String::with_capacity(source.len());
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if (trimmed.starts_with("export {") || trimmed.starts_with("export{"))
            && (line.contains("as 'module.exports'") || line.contains("as \"module.exports\""))
        {
            // Drop this line entirely. The default export is what
            // actually matters for ESM consumers.
            continue;
        }
        out.push_str(line);
    }
    out
}

fn rewrite_regex_u_class_escapes(source: &str) -> String {
    // Fast-skip: if source has no `\-`, nothing to rewrite. Avoids both
    // the cost of the byte walk and any chance of corrupting UTF-8.
    // Fast-skip: nothing to rewrite if source has neither `\-` (the
    // \- in /u/v char class escape we map to literal hyphen) nor
    // `/v` (the ES2024 Unicode-sets flag we downgrade to /u). The
    // `/v` check is a substring approximation — false positives just
    // pay the linear walk cost; false negatives are impossible since
    // a /v regex literal must contain the bytes `/v`.
    if !source.contains("\\-") && !source.contains("/v") {
        return source.to_string();
    }
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    // `emit_start` is the byte index of the next un-emitted source slice.
    // We accumulate `source[emit_start..i]` verbatim (UTF-8-safe) until
    // we either rewrite a regex literal or finish.
    let mut emit_start: usize = 0;
    let mut i: usize = 0;
    while i < bytes.len() {
        let c = bytes[i];
        // Non-ASCII: skip the whole UTF-8 sequence intact. The slice
        // [emit_start..i] already includes everything up to here; bump i
        // by the codepoint's byte width.
        if c >= 0x80 {
            let w = if c < 0xC0 { 1 } // stray continuation byte; advance 1
                else if c < 0xE0 { 2 }
                else if c < 0xF0 { 3 }
                else { 4 };
            i += w;
            continue;
        }
        // String literal — find its end without inspecting its contents
        // for regex syntax. Body may contain non-ASCII chars; we skip
        // them by detecting a backslash-anything escape (advance 2 bytes
        // if the byte after `\` is ASCII; otherwise advance 1 then the
        // continuation will be handled by the non-ASCII branch).
        if c == b'"' || c == b'\'' || c == b'`' {
            let quote = c;
            i += 1;
            while i < bytes.len() {
                let d = bytes[i];
                if d >= 0x80 {
                    let w = if d < 0xC0 { 1 } else if d < 0xE0 { 2 } else if d < 0xF0 { 3 } else { 4 };
                    i += w;
                    continue;
                }
                if d == b'\\' && i + 1 < bytes.len() {
                    // Skip the escape and its following byte. If the
                    // following byte is a UTF-8 continuation start, the
                    // non-ASCII branch on the next iter will handle it.
                    i += 2;
                    continue;
                }
                i += 1;
                if d == quote { break; }
            }
            continue;
        }
        // Line comment — skip to newline.
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' { i += 1; }
            continue;
        }
        // Block comment — skip to */
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            if i + 1 < bytes.len() { i += 2; }
            continue;
        }
        // Possible regex literal: preceded by punctuation that allows it.
        if c == b'/' {
            // Walk back over whitespace in the source (not in `out`,
            // since `out` only holds previously-rewritten chunks).
            let mut j = i;
            while j > emit_start && (bytes[j - 1] == b' ' || bytes[j - 1] == b'\t') {
                j -= 1;
            }
            let prev = if j == 0 { b'\n' } else { bytes[j - 1] };
            let regex_context = matches!(prev,
                b'(' | b',' | b'=' | b':' | b'[' | b'!' | b'&' | b'|' | b'?' |
                b'{' | b';' | b'+' | b'-' | b'*' | b'%' | b'^' | b'~' | b'<' |
                b'>' | b'\n' | b'\r' | b'/' | b'\\'
            );
            if regex_context {
                let body_start = i + 1;
                let mut k = body_start;
                let mut in_class = false;
                let mut ok = false;
                while k < bytes.len() {
                    let b = bytes[k];
                    if b == b'\\' && k + 1 < bytes.len() { k += 2; continue; }
                    if b == b'\n' { break; }
                    if b == b'[' { in_class = true; k += 1; continue; }
                    if b == b']' { in_class = false; k += 1; continue; }
                    if b == b'/' && !in_class { ok = true; break; }
                    k += 1;
                }
                if ok {
                    let body_end = k;
                    let mut f = body_end + 1;
                    while f < bytes.len()
                        && (bytes[f] == b'g' || bytes[f] == b'i' || bytes[f] == b'm'
                            || bytes[f] == b's' || bytes[f] == b'u' || bytes[f] == b'y'
                            || bytes[f] == b'd' || bytes[f] == b'v')
                    {
                        f += 1;
                    }
                    let flags = &source[body_end + 1..f];
                    if flags.contains('u') || flags.contains('v') {
                        // Emit accumulated verbatim slice [emit_start..i].
                        out.push_str(&source[emit_start..i]);
                        // Rewrite \- inside char classes in the body.
                        let body = &source[body_start..body_end];
                        let body_bytes = body.as_bytes();
                        let mut rewritten = String::with_capacity(body.len());
                        // Rewrite by emitting slices, preserving UTF-8.
                        let mut bi = 0;
                        let mut slice_start = 0;
                        let mut in_cls = false;
                        while bi < body_bytes.len() {
                            let bc = body_bytes[bi];
                            if bc == b'\\' && bi + 1 < body_bytes.len() {
                                let nx = body_bytes[bi + 1];
                                if in_cls && nx == b'-' {
                                    rewritten.push_str(&body[slice_start..bi]);
                                    rewritten.push_str("\\u002D");
                                    bi += 2;
                                    slice_start = bi;
                                    continue;
                                }
                                bi += 2;
                                continue;
                            }
                            if bc == b'[' && !in_cls { in_cls = true; }
                            else if bc == b']' && in_cls { in_cls = false; }
                            bi += 1;
                        }
                        rewritten.push_str(&body[slice_start..]);
                        // When downgrading /v → /u, rewrite /v-only
                        // property names that aren't valid under /u.
                        // RGI_Emoji is the common case (string-width,
                        // emoji-regex-xs); falls back to Emoji which
                        // matches single-code-point emoji and is the
                        // closest /u-compatible approximation. The
                        // sequence-match precision is sacrificed —
                        // accepted divergence below the L5 cut for
                        // S-regex stratum.
                        let downgrade_v_to_u = flags.contains('v')
                            && !body.contains("--")
                            && !body.contains("&&");
                        if downgrade_v_to_u && rewritten.contains("RGI_Emoji") {
                            rewritten = rewritten.replace("RGI_Emoji", "Emoji");
                        }
                        if downgrade_v_to_u && rewritten.contains("Basic_Emoji") {
                            rewritten = rewritten.replace("Basic_Emoji", "Emoji");
                        }
                        out.push('/');
                        out.push_str(&rewritten);
                        out.push('/');
                        if downgrade_v_to_u {
                            for c in flags.chars() {
                                if c == 'v' { out.push('u'); }
                                else { out.push(c); }
                            }
                        } else {
                            out.push_str(flags);
                        }
                        emit_start = f;
                        i = f;
                        continue;
                    } else {
                        // Regex literal without /u or /v — keep verbatim;
                        // just advance past it so the outer walk doesn't
                        // re-enter the body. O(n²) avoidance.
                        i = f;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    out.push_str(&source[emit_start..]);
    out
}

fn install_error_polyfills<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    // Error.captureStackTrace(obj, ctorOpt) — V8/Node-specific API used
    // pervasively in npm (depd, http-errors, every error-throwing util).
    // QuickJS doesn't have it natively. Polyfill: write the current
    // stack to obj.stack. Constructor arg ignored (would slice the
    // stack to start above ctor; we don't have that resolution).
    //
    // Error.stackTraceLimit — V8-specific. Default to 10 to match Node.
    //
    // Error.prepareStackTrace — V8-specific stack-frame array transform.
    // Many libs read/write it. Stub passthrough — accept the assignment
    // but don't transform.
    ctx.eval::<(), _>(r#"
        (function() {
            // globalThis.self — many browser-compat / UMD libs check
            // `typeof self !== "undefined"` to find the global scope.
            // Prism, jsdom-shim, some validators, many polyfills branch
            // on this. Alias self → globalThis so they take the browser path.
            if (typeof globalThis.self === "undefined") {
                Object.defineProperty(globalThis, "self", {
                    value: globalThis,
                    writable: true, configurable: true,
                });
            }
            if (typeof Error.captureStackTrace !== "function") {
                // V8 semantics: if Error.prepareStackTrace is set, V8 calls
                // it with (err, structuredCallSiteArray) and uses its return
                // value as obj.stack. depd + many other libs depend on this
                // path. Provide a minimal structured array of CallSite-shape
                // objects with the standard query methods returning empty
                // strings / zero (good enough for depd to walk the stack
                // and not throw on getFunctionName() etc.).
                const fakeFrame = {
                    getFunctionName: () => "",
                    getFileName: () => "",
                    getLineNumber: () => 0,
                    getColumnNumber: () => 0,
                    getMethodName: () => "",
                    getTypeName: () => "",
                    getThis: () => undefined,
                    getEvalOrigin: () => undefined,
                    isNative: () => false,
                    isEval: () => false,
                    isConstructor: () => false,
                    isToplevel: () => true,
                    toString: () => "",
                };
                Error.captureStackTrace = function captureStackTrace(obj, _ctor) {
                    try {
                        const structured = [fakeFrame, fakeFrame, fakeFrame, fakeFrame, fakeFrame];
                        let value;
                        if (typeof Error.prepareStackTrace === "function") {
                            try { value = Error.prepareStackTrace(obj, structured); }
                            catch (_) { value = ""; }
                        } else {
                            value = "";
                        }
                        Object.defineProperty(obj, "stack", {
                            value, writable: true, configurable: true,
                        });
                    } catch (_) {
                        obj.stack = "";
                    }
                };
            }
            if (Error.stackTraceLimit === undefined) {
                Error.stackTraceLimit = 10;
            }

            // E.29 closure: QuickJS rejects `\-` inside character classes
            // when /u flag is set, but ECMAScript allows it. Libraries that
            // build patterns by concatenating `.source` from a non-/u regex
            // and re-compile with `new RegExp(..., 'gu')` hit this — camelcase
            // ^9, camelcase-keys, dasherize, snake-case-derived ESM-only libs.
            // Rewrite \\- inside [...] to a leading literal hyphen, which is
            // unambiguously a literal under /u and semantically identical.
            (function() {
                const _RE = globalThis.RegExp;
                function fixPattern(p) {
                    if (typeof p !== "string") return p;
                    return p.replace(/\[([^\]]*)\\-([^\]]*)\]/g, function(_, a, b) {
                        return "[-" + a + b + "]";
                    });
                }
                function Wrapped(pattern, flags) {
                    if (typeof flags === "string" && flags.indexOf("u") !== -1) {
                        pattern = fixPattern(pattern);
                    }
                    if (new.target) {
                        return Reflect.construct(_RE, [pattern, flags], new.target);
                    }
                    return _RE(pattern, flags);
                }
                Wrapped.prototype = _RE.prototype;
                Object.setPrototypeOf(Wrapped, _RE);
                globalThis.RegExp = Wrapped;
            })();
        })();
    "#)?;
    Ok(())
}

fn wire_globals<'js>(ctx: rquickjs::Ctx<'js>) -> JsResult<()> {
    let global = ctx.globals();
    install_error_polyfills(&ctx)?;
    wire_console(&ctx, &global)?;
    wire_atob_btoa(&ctx, &global)?;
    wire_path(&ctx, &global)?;
    wire_os(&ctx, &global)?;
    wire_crypto(&ctx, &global)?;
    wire_text_encoding(&ctx, &global)?;
    wire_buffer(&ctx, &global)?;
    install_buffer_class_js(&ctx)?;
    install_set_methods_polyfill(&ctx)?;
    install_finalization_registry_stub(&ctx)?;
    // Node-compat: `global` and `self` aliases for globalThis. queue-microtask
    // and many browser-portable libs check typeof global / typeof self.
    ctx.eval::<(), _>(r#"
        if (typeof globalThis.global === "undefined") globalThis.global = globalThis;
        if (typeof globalThis.self === "undefined") globalThis.self = globalThis;
    "#)?;
    wire_url_search_params_static(&ctx, &global)?;
    install_url_search_params_class_js(&ctx)?;
    wire_fs(&ctx, &global)?;
    wire_blob_static(&ctx, &global)?;
    install_blob_and_file_classes_js(&ctx)?;
    wire_abort_controller_static(&ctx, &global)?;
    install_abort_controller_classes_js(&ctx)?;
    wire_headers_static(&ctx, &global)?;
    wire_response_static(&ctx, &global)?;
    install_fetch_api_classes_js(&ctx)?;
    wire_http_codec(&ctx, &global)?;
    wire_sockets(&ctx, &global)?;
    wire_dns(&ctx, &global)?;
    wire_compression(&ctx, &global)?;
    wire_tls(&ctx, &global)?;
    wire_websocket(&ctx, &global)?;
    wire_bun_namespace_static(&ctx, &global)?;
    install_bun_namespace_js(&ctx)?;
    // install_dns_js extends globalThis.Bun.dns, so it MUST run after
    // install_bun_namespace_js. Order matters per seed §A8 install pattern.
    install_dns_js(&ctx)?;
    wire_bun_serve_static(&ctx, &global)?;
    install_bun_serve_js(&ctx)?;
    wire_bun_spawn_static(&ctx, &global)?;
    install_bun_spawn_js(&ctx)?;
    install_structured_clone_js(&ctx)?;
    install_streams_js(&ctx)?;
    install_node_http_js(&ctx)?;
    install_node_events_js(&ctx)?;
    install_websocket_class_js(&ctx)?;
    install_node_util_js(&ctx)?;
    install_node_stream_js(&ctx)?;
    install_node_querystring_and_url_full_js(&ctx)?;
    install_bun_small_utilities_js(&ctx)?;
    install_keep_alive_js(&ctx)?;
    install_node_assert_js(&ctx)?;
    install_node_child_process_js(&ctx)?;
    install_node_net_js(&ctx)?;
    install_node_tty_js(&ctx)?;
    install_node_zlib_js(&ctx)?;
    install_node_diagnostics_channel_js(&ctx)?;
    install_node_https_perf_async_hooks_js(&ctx)?;
    install_node_extra_builtins_js(&ctx)?;
    install_intl_js(&ctx)?;
    install_commonjs_loader_js(&ctx)?;
    install_timers_js(&ctx)?;
    wire_performance(&ctx, &global)?;
    install_url_class_js(&ctx)?;
    wire_process(&ctx, &global)?;
    Ok(())
}

// ───────────────────────── console ────────────────────────────────────

fn wire_console<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let console = Object::new(ctx.clone())?;
    let log_args = Function::new(ctx.clone(), |args: rquickjs::function::Rest<Value<'js>>| {
        print_args(args.into_inner(), false);
    })?;
    console.set("log", log_args)?;
    let warn_args = Function::new(ctx.clone(), |args: rquickjs::function::Rest<Value<'js>>| {
        print_args(args.into_inner(), true);
    })?;
    console.set("warn", warn_args)?;
    let err_args = Function::new(ctx.clone(), |args: rquickjs::function::Rest<Value<'js>>| {
        print_args(args.into_inner(), true);
    })?;
    console.set("error", err_args)?;
    global.set("console", console)?;
    Ok(())
}

fn print_args<'js>(args: Vec<Value<'js>>, to_stderr: bool) {
    let parts: Vec<String> = args.iter().map(value_to_display).collect();
    let line = parts.join(" ");
    if to_stderr {
        eprintln!("{}", line);
    } else {
        println!("{}", line);
    }
}

fn value_to_display<'js>(v: &Value<'js>) -> String {
    if let Some(s) = v.as_string().and_then(|s| s.to_string().ok()) {
        return s;
    }
    if let Some(n) = v.as_number() {
        if n == n.trunc() && n.abs() < 1e15 {
            return format!("{}", n as i64);
        }
        return format!("{}", n);
    }
    if let Some(b) = v.as_bool() {
        return format!("{}", b);
    }
    if v.is_null() { return "null".into(); }
    if v.is_undefined() { return "undefined".into(); }
    if let Some(arr) = v.as_array() {
        let mut parts = Vec::new();
        for i in 0..arr.len() {
            if let Ok(item) = arr.get::<Value<'js>>(i) {
                parts.push(value_to_display(&item));
            }
        }
        return format!("[ {} ]", parts.join(", "));
    }
    if let Some(_obj) = v.as_object() {
        return "[object Object]".into();
    }
    "<unprintable>".into()
}

// ─────────────────── atob / btoa ─────────────────────────────────────

fn wire_atob_btoa<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    global.set(
        "atob",
        Function::new(ctx.clone(), |s: String| -> String {
            let bytes = rusty_buffer::Buffer::from_string(&s, rusty_buffer::Encoding::Base64);
            bytes.as_bytes().iter().map(|&b| b as char).collect()
        })?,
    )?;
    global.set(
        "btoa",
        Function::new(ctx.clone(), |s: String| -> String {
            let bytes: Vec<u8> = s.chars().map(|c| c as u8).collect();
            let buf = rusty_buffer::Buffer::from_bytes(&bytes);
            buf.to_string(rusty_buffer::Encoding::Base64, 0, None)
        })?,
    )?;
    Ok(())
}

// ─────────────────── path ────────────────────────────────────────────

fn wire_process<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    // node:process / globalThis.process. Bun-portable subset.
    // stdout.write / stderr.write accumulate into globalThis.__stdoutBuf
    // and globalThis.__stderrBuf respectively so eval_esm_module can read
    // them. Real Bun writes to actual fd 1/2; rusty-bun-host captures
    // them into JS-side buffers for test inspection (and for the
    // Tier-J differential path).
    let process = Object::new(ctx.clone())?;

    // env: a fresh object populated from std::env::vars().
    let env = Object::new(ctx.clone())?;
    for (k, v) in std::env::vars() {
        let _ = env.set(k.as_str(), v);
    }
    process.set("env", env)?;

    // argv: in the host context, populate with a synthetic two-element
    // array similar to ["bun", "<entry-script>"]. eval_esm_module knows
    // the entry path; for the host-internal eval helpers, default to
    // ["rusty-bun-host", "<eval>"]. The test fixtures don't depend on
    // argv content; what they depend on is argv being an Array.
    let argv: Vec<String> = vec!["rusty-bun-host".to_string(), "<eval>".to_string()];
    process.set("argv", argv)?;

    process.set("platform", if cfg!(target_os = "linux") { "linux" }
        else if cfg!(target_os = "macos") { "darwin" }
        else if cfg!(target_os = "windows") { "win32" }
        else { "unknown" })?;
    process.set("arch", if cfg!(target_arch = "x86_64") { "x64" }
        else if cfg!(target_arch = "aarch64") { "arm64" }
        else { "unknown" })?;
    // Claim Node v20 compat. yargs-parser, undici, fastify and many
    // modern libs gate on minimum Node major; we satisfy "≥ 20" while
    // recording the actual rusty-bun-host version too.
    process.set("version", "v20.0.0")?;
    process.set("versions", {
        let v = Object::new(ctx.clone())?;
        v.set("node", "20.0.0")?;
        v.set("rusty_bun_host", "0.0.0")?;
        v
    })?;

    process.set("cwd", Function::new(ctx.clone(), || -> String {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "/".to_string())
    })?)?;
    process.set("pid", std::process::id() as i64)?;
    process.set("ppid", 0_i64)?;
    process.set("execPath", std::env::current_exe()
        .ok().and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "rusty-bun-host".to_string()))?;
    process.set("execArgv", Vec::<String>::new())?;
    process.set("argv0", "rusty-bun-host")?;
    process.set("title", "rusty-bun-host")?;
    process.set("allowedNodeEnvironmentFlags", Vec::<String>::new())?;
    process.set("umask", Function::new(ctx.clone(), || -> i32 { 0o022 })?)?;
    process.set("uptime", Function::new(ctx.clone(), || -> f64 { 0.0 })?)?;
    process.set("getuid", Function::new(ctx.clone(), || -> i32 { -1 })?)?;
    process.set("getgid", Function::new(ctx.clone(), || -> i32 { -1 })?)?;
    process.set("kill", Function::new(ctx.clone(), |_pid: f64, _sig: Opt<String>| -> bool {
        // Cannot actually signal pids; rusty-bun runs single-process.
        false
    })?)?;
    process.set("emitWarning", Function::new(ctx.clone(),
        |_warning: rquickjs::Value, _type: Opt<rquickjs::Value>, _code: Opt<String>| -> () {
            // Node emits to process.stderr; we silently no-op so libraries
            // that emit deprecation warnings dont pollute test output.
        })?)?;
    process.set("getActiveResourcesInfo", Function::new(ctx.clone(),
        || -> Vec<String> { vec![] })?)?;
    process.set("memoryUsage", Function::new(ctx.clone(), || -> String {
        r#"{"rss":0,"heapTotal":0,"heapUsed":0,"external":0,"arrayBuffers":0}"#.to_string()
    })?)?;
    process.set("resourceUsage", Function::new(ctx.clone(), || -> String {
        r#"{"userCPUTime":0,"systemCPUTime":0,"maxRSS":0}"#.to_string()
    })?)?;

    // exit is a sentinel — we cannot actually exit the test process; the
    // function records the code on globalThis.__exitCode and throws to
    // unwind. Real consumer code that calls process.exit will surface as
    // an uncaught error in rusty-bun-host but won't kill the process.
    process.set("exit", Function::new(ctx.clone(), |code: Opt<i32>| -> JsResult<()> {
        let _ = code;
        Err(rquickjs::Error::new_from_js_message(
            "process.exit", "void",
            "process.exit called in rusty-bun-host (no-op; check __exitCode)",
        ))
    })?)?;

    global.set("process", process)?;

    // process.nextTick — Node semantic: schedule fn as microtask.
    // Equivalent to queueMicrotask(() => fn(...args)). Light-my-request,
    // many CJS libs depend on this.
    ctx.eval::<(), _>(r#"
        globalThis.process.nextTick = function nextTick(fn) {
            if (typeof fn !== "function") {
                throw new TypeError("process.nextTick: fn must be a function");
            }
            const args = Array.prototype.slice.call(arguments, 1);
            Promise.resolve().then(function() {
                try { fn.apply(undefined, args); }
                catch (e) {
                    if (typeof console !== "undefined" && console.error) {
                        console.error("uncaught in process.nextTick:", e);
                    }
                }
            });
        };
    "#)?;

    // stdout / stderr with .write accumulating into JS-side buffers.
    // JS-side wiring keeps the implementation small and avoids holding
    // Rust state on JS objects (which would re-trigger E.4's GC issue).
    ctx.eval::<(), _>(r#"
        (function() {
            globalThis.__stdoutBuf = "";
            globalThis.__stderrBuf = "";
            globalThis.process.stdout = {
                write(chunk) {
                    globalThis.__stdoutBuf += String(chunk);
                    return true;
                }
            };
            globalThis.process.stderr = {
                write(chunk) {
                    globalThis.__stderrBuf += String(chunk);
                    return true;
                }
            };
            // hrtime and hrtime.bigint — common in real npm packages.
            globalThis.process.hrtime = function hrtime(prev) {
                const t = performance.now() * 1e6;  // ns
                const sec = Math.floor(t / 1e9);
                const nsec = Math.floor(t % 1e9);
                if (prev) {
                    const ds = sec - prev[0];
                    const dn = nsec - prev[1];
                    return dn < 0 ? [ds - 1, dn + 1e9] : [ds, dn];
                }
                return [sec, nsec];
            };
            globalThis.process.hrtime.bigint = function() {
                return BigInt(Math.floor(performance.now() * 1e6));
            };

            // Π2.7: EventEmitter pattern on process. Real Bun + Node fire
            // SIGINT/SIGTERM listeners on signal delivery; rusty-bun-host
            // is a test runtime that doesn't deliver real signals to JS,
            // so handlers register but never fire. exit and beforeExit
            // listeners fire from the host's eval loop on completion.
            const _listeners = Object.create(null);
            function _ensure(event) {
                if (!_listeners[event]) _listeners[event] = [];
                return _listeners[event];
            }
            globalThis.process.on = function on(event, listener) {
                if (typeof listener !== "function") {
                    throw new TypeError("process.on: listener must be a function");
                }
                _ensure(event).push(listener);
                return globalThis.process;
            };
            globalThis.process.once = function once(event, listener) {
                if (typeof listener !== "function") {
                    throw new TypeError("process.once: listener must be a function");
                }
                const wrapped = (...args) => {
                    globalThis.process.off(event, wrapped);
                    listener(...args);
                };
                wrapped._original = listener;
                _ensure(event).push(wrapped);
                return globalThis.process;
            };
            globalThis.process.off = function off(event, listener) {
                const arr = _listeners[event];
                if (!arr) return globalThis.process;
                const i = arr.findIndex(l => l === listener || l._original === listener);
                if (i >= 0) arr.splice(i, 1);
                return globalThis.process;
            };
            globalThis.process.removeListener = globalThis.process.off;
            globalThis.process.removeAllListeners = function removeAllListeners(event) {
                if (event === undefined) {
                    for (const k of Object.keys(_listeners)) delete _listeners[k];
                } else {
                    delete _listeners[event];
                }
                return globalThis.process;
            };
            globalThis.process.listenerCount = function listenerCount(event) {
                return _listeners[event] ? _listeners[event].length : 0;
            };
            globalThis.process.listeners = function listeners(event) {
                return _listeners[event] ? _listeners[event].slice() : [];
            };
            globalThis.process.emit = function emit(event, ...args) {
                // Per M9 (spec-first against Bun): Bun's process.emit
                // returns true regardless of listener presence. We match.
                const arr = _listeners[event];
                if (arr) {
                    const copy = arr.slice();
                    for (const l of copy) {
                        try { l(...args); } catch (e) {
                            globalThis.__stderrBuf += "process emit error: " + (e && e.message ? e.message : String(e)) + "\n";
                        }
                    }
                }
                return true;
            };

            // process.stdin: a minimal EventEmitter-shape stream. In the
            // host context there is no real stdin; reads complete with
            // immediate 'end'. Calling .on('data', cb) does nothing useful
            // but the API is present so consumer code doesn't crash.
            const _stdinListeners = Object.create(null);
            function _stdinEnsure(e) {
                if (!_stdinListeners[e]) _stdinListeners[e] = [];
                return _stdinListeners[e];
            }
            globalThis.process.stdin = {
                readable: false,
                isTTY: undefined,  // Bun matches Node: undefined when not on a TTY
                on(event, listener) {
                    _stdinEnsure(event).push(listener);
                    if (event === "end") {
                        // Immediately schedule 'end' since there's no real stdin.
                        queueMicrotask(() => { try { listener(); } catch (_) {} });
                    }
                    return this;
                },
                once(event, listener) { return this.on(event, listener); },
                off(event, listener) {
                    const arr = _stdinListeners[event];
                    if (arr) {
                        const i = arr.indexOf(listener);
                        if (i >= 0) arr.splice(i, 1);
                    }
                    return this;
                },
                removeListener(event, listener) { return this.off(event, listener); },
                read() { return null; },
                resume() { return this; },
                pause() { return this; },
                setEncoding() { return this; },
            };
        })();
    "#)?;

    Ok(())
}

fn wire_os<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    // node:os data-layer. Bun-portable subset: platform, arch, type,
    // tmpdir, homedir, hostname, EOL, endianness.
    let os = Object::new(ctx.clone())?;
    os.set("platform", Function::new(ctx.clone(), || -> &'static str {
        // Node.js convention: "linux", "darwin", "win32", etc.
        if cfg!(target_os = "linux") { "linux" }
        else if cfg!(target_os = "macos") { "darwin" }
        else if cfg!(target_os = "windows") { "win32" }
        else if cfg!(target_os = "freebsd") { "freebsd" }
        else if cfg!(target_os = "openbsd") { "openbsd" }
        else { "unknown" }
    })?)?;
    os.set("arch", Function::new(ctx.clone(), || -> &'static str {
        // Node.js convention: "x64", "arm64", "arm", "ia32", etc.
        if cfg!(target_arch = "x86_64") { "x64" }
        else if cfg!(target_arch = "aarch64") { "arm64" }
        else if cfg!(target_arch = "arm") { "arm" }
        else if cfg!(target_arch = "x86") { "ia32" }
        else { "unknown" }
    })?)?;
    os.set("type", Function::new(ctx.clone(), || -> &'static str {
        // Node.js: returns the OS name like uname -s.
        if cfg!(target_os = "linux") { "Linux" }
        else if cfg!(target_os = "macos") { "Darwin" }
        else if cfg!(target_os = "windows") { "Windows_NT" }
        else if cfg!(target_os = "freebsd") { "FreeBSD" }
        else if cfg!(target_os = "openbsd") { "OpenBSD" }
        else { "Unknown" }
    })?)?;
    os.set("tmpdir", Function::new(ctx.clone(), || -> String {
        std::env::temp_dir().to_string_lossy().into_owned()
    })?)?;
    os.set("homedir", Function::new(ctx.clone(), || -> String {
        std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| "/".to_string())
    })?)?;
    os.set("hostname", Function::new(ctx.clone(), || -> String {
        std::env::var("HOSTNAME")
            .or_else(|_| {
                // POSIX fallback via /etc/hostname; final fallback "localhost".
                std::fs::read_to_string("/etc/hostname")
                    .map(|s| s.trim().to_string())
                    .map_err(|_| std::env::VarError::NotPresent)
            })
            .unwrap_or_else(|_| "localhost".to_string())
    })?)?;
    os.set("endianness", Function::new(ctx.clone(), || -> &'static str {
        if cfg!(target_endian = "little") { "LE" } else { "BE" }
    })?)?;
    os.set("EOL", if cfg!(target_os = "windows") { "\r\n" } else { "\n" })?;
    // os.constants: posix signals + dlopen + errno
    {
        let c = Object::new(ctx.clone())?;
        let signals = Object::new(ctx.clone())?;
        signals.set("SIGINT", 2i32)?; signals.set("SIGTERM", 15i32)?;
        signals.set("SIGKILL", 9i32)?; signals.set("SIGHUP", 1i32)?;
        signals.set("SIGUSR1", 10i32)?; signals.set("SIGUSR2", 12i32)?;
        signals.set("SIGPIPE", 13i32)?; signals.set("SIGALRM", 14i32)?;
        signals.set("SIGCHLD", 17i32)?; signals.set("SIGSTOP", 19i32)?;
        signals.set("SIGCONT", 18i32)?; signals.set("SIGSEGV", 11i32)?;
        signals.set("SIGABRT", 6i32)?; signals.set("SIGFPE", 8i32)?;
        c.set("signals", signals)?;
        let errno = Object::new(ctx.clone())?;
        errno.set("EACCES", 13i32)?; errno.set("EEXIST", 17i32)?;
        errno.set("ENOENT", 2i32)?; errno.set("EPERM", 1i32)?;
        errno.set("EBADF", 9i32)?; errno.set("EINVAL", 22i32)?;
        errno.set("EISDIR", 21i32)?; errno.set("ENOTDIR", 20i32)?;
        errno.set("ENOSPC", 28i32)?; errno.set("EPIPE", 32i32)?;
        c.set("errno", errno)?;
        let dlopen = Object::new(ctx.clone())?;
        dlopen.set("RTLD_LAZY", 1i32)?; dlopen.set("RTLD_NOW", 2i32)?;
        dlopen.set("RTLD_GLOBAL", 256i32)?; dlopen.set("RTLD_LOCAL", 0i32)?;
        c.set("dlopen", dlopen)?;
        os.set("constants", c)?;
    }
    // Memory + cpu stubs. Best-effort: read /proc on Linux, return small
    // sane defaults otherwise. Consumers (pino, debug, node-fetch) read
    // these for diagnostics; exact accuracy is rarely load-bearing.
    os.set("totalmem", Function::new(ctx.clone(), || -> f64 {
        std::fs::read_to_string("/proc/meminfo").ok()
            .and_then(|s| s.lines()
                .find(|l| l.starts_with("MemTotal:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|n| n.parse::<f64>().ok())
                .map(|kb| kb * 1024.0))
            .unwrap_or(0.0)
    })?)?;
    os.set("freemem", Function::new(ctx.clone(), || -> f64 {
        std::fs::read_to_string("/proc/meminfo").ok()
            .and_then(|s| s.lines()
                .find(|l| l.starts_with("MemAvailable:") || l.starts_with("MemFree:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|n| n.parse::<f64>().ok())
                .map(|kb| kb * 1024.0))
            .unwrap_or(0.0)
    })?)?;
    os.set("cpus", Function::new(ctx.clone(), || -> Vec<String> {
        // Return a Vec<String> of "stub" entries — consumer code typically
        // only checks .length to get core count. JSON shape full enough for
        // most use; richer shape (model/speed/times) can land per consumer.
        let n = std::thread::available_parallelism()
            .map(|n| n.get()).unwrap_or(1);
        vec!["cpu".to_string(); n]
    })?)?;
    os.set("uptime", Function::new(ctx.clone(), || -> f64 {
        std::fs::read_to_string("/proc/uptime").ok()
            .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
            .unwrap_or(0.0)
    })?)?;
    os.set("loadavg", Function::new(ctx.clone(), || -> Vec<f64> {
        std::fs::read_to_string("/proc/loadavg").ok()
            .map(|s| {
                let parts: Vec<f64> = s.split_whitespace().take(3)
                    .filter_map(|p| p.parse::<f64>().ok()).collect();
                if parts.len() == 3 { parts } else { vec![0.0, 0.0, 0.0] }
            })
            .unwrap_or(vec![0.0, 0.0, 0.0])
    })?)?;
    os.set("networkInterfaces", Function::new(ctx.clone(), || -> String {
        // Stub returns "{}" parsed as empty object by consumer-side wrap.
        "{}".to_string()
    })?)?;
    os.set("userInfo", Function::new(ctx.clone(), || -> String {
        // Stub returns JSON-encoded shape; consumer-side wraps via parse.
        let user = std::env::var("USER").unwrap_or_else(|_| "rusty".to_string());
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        format!(r#"{{"uid":-1,"gid":-1,"username":"{}","homedir":"{}","shell":null}}"#, user, home)
    })?)?;
    global.set("os", os)?;
    Ok(())
}

fn wire_path<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let p = Object::new(ctx.clone())?;
    p.set("basename", Function::new(ctx.clone(), |path: String, ext: Opt<String>| {
        rusty_node_path::basename(&path, ext.0.as_deref())
    })?)?;
    p.set("dirname", Function::new(ctx.clone(), |path: String| {
        rusty_node_path::dirname(&path)
    })?)?;
    p.set("extname", Function::new(ctx.clone(), |path: String| {
        rusty_node_path::extname(&path)
    })?)?;
    p.set("normalize", Function::new(ctx.clone(), |path: String| {
        rusty_node_path::normalize(&path)
    })?)?;
    p.set("isAbsolute", Function::new(ctx.clone(), |path: String| {
        rusty_node_path::is_absolute(&path)
    })?)?;
    p.set("join", Function::new(ctx.clone(), |a: String, b: Opt<String>| {
        match b.0 {
            Some(b) => rusty_node_path::join(&[&a, &b]),
            None => rusty_node_path::join(&[&a]),
        }
    })?)?;
    p.set("sep", "/")?;
    p.set("delimiter", ":")?;
    global.set("path", p)?;
    // path.resolve(...segments) — Node's variadic resolve(): each segment
    // is joined left-to-right; absolute segments restart from there;
    // result is normalized to an absolute path against cwd.
    ctx.eval::<(), _>(r#"
        (function() {
            const cwd = () => (globalThis.process && globalThis.process.cwd && globalThis.process.cwd()) || "/";
            globalThis.path.resolve = function resolve(...segments) {
                let resolved = "";
                let isAbs = false;
                for (let i = segments.length - 1; i >= 0; i--) {
                    const s = String(segments[i] || "");
                    if (s.length === 0) continue;
                    resolved = s + (resolved ? "/" + resolved : "");
                    if (s[0] === "/") { isAbs = true; break; }
                }
                if (!isAbs) resolved = cwd() + (resolved ? "/" + resolved : "");
                // Normalize: collapse // and resolve . / ..
                const parts = resolved.split("/").filter(p => p !== "" && p !== ".");
                const out = [];
                for (const p of parts) {
                    if (p === "..") { if (out.length) out.pop(); }
                    else out.push(p);
                }
                return "/" + out.join("/");
            };
            // path.relative(from, to) — best-effort: strip common prefix.
            globalThis.path.relative = function relative(from, to) {
                const fa = globalThis.path.resolve(from).split("/").filter(Boolean);
                const ta = globalThis.path.resolve(to).split("/").filter(Boolean);
                let i = 0;
                while (i < fa.length && i < ta.length && fa[i] === ta[i]) i++;
                const up = fa.slice(i).map(() => "..");
                const down = ta.slice(i);
                return up.concat(down).join("/") || ".";
            };
            // path.parse(path) — return {root, dir, base, ext, name}.
            globalThis.path.parse = function parse(p) {
                const dir = globalThis.path.dirname(p);
                const base = globalThis.path.basename(p);
                const ext = globalThis.path.extname(p);
                return { root: dir.startsWith("/") ? "/" : "", dir, base, ext,
                         name: base.slice(0, base.length - ext.length) };
            };
            globalThis.path.format = function format(o) {
                const dir = o.dir || o.root || "";
                const base = o.base || ((o.name || "") + (o.ext || ""));
                return dir ? dir + (dir.endsWith("/") ? "" : "/") + base : base;
            };
            // path.posix / path.win32 aliases — most consumers use path.posix.
            globalThis.path.posix = globalThis.path;
            // path.win32 — provide a separate object with backslash sep
            // for libs that branch on platform. Reuse posix functions
            // for the simple methods; sep + delimiter differ.
            globalThis.path.win32 = Object.assign({}, globalThis.path, {
                sep: "\\",
                delimiter: ";",
            });
        })();
    "#)?;
    Ok(())
}

// ─────────────────── HTTP/1.1 wire codec (Tier-G substrate) ──────────

fn wire_http_codec<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    // Exposed under globalThis.__httpCodec — a Bun-portable namespace for
    // wire-format parsing + serialization. Real consumer code (Bun.serve
    // implementations, HTTP proxies, mock servers) shapes around this
    // primitive. FFI returns JSON strings (rquickjs closures can't carry
    // a Ctx); the JS-side facade installed below parses them into
    // structured objects with Uint8Array bodies.
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "parseRequestJson",
        Function::new(ctx.clone(), |bytes: Vec<u8>| -> JsResult<String> {
            let r = rusty_http_codec::parse_request(&bytes)
                .map_err(|e| rquickjs::Error::new_from_js_message("http-codec", "parseRequest", e.to_string()))?;
            // Encode as JSON manually (no serde dependency in codec pilot).
            let mut s = String::from("{");
            s.push_str(&format!("\"method\":{}", json_str(&r.method)));
            s.push_str(&format!(",\"target\":{}", json_str(&r.target)));
            s.push_str(&format!(",\"version\":{}", json_str(&r.version)));
            s.push_str(",\"headers\":[");
            for (i, (n, v)) in r.headers.iter().enumerate() {
                if i > 0 { s.push(','); }
                s.push_str(&format!("[{},{}]", json_str(n), json_str(v)));
            }
            s.push_str("],\"body\":[");
            for (i, b) in r.body.iter().enumerate() {
                if i > 0 { s.push(','); }
                s.push_str(&b.to_string());
            }
            s.push_str("]}");
            Ok(s)
        })?,
    )?;
    ns.set(
        "parseResponseJson",
        Function::new(ctx.clone(), |bytes: Vec<u8>| -> JsResult<String> {
            let r = rusty_http_codec::parse_response(&bytes)
                .map_err(|e| rquickjs::Error::new_from_js_message("http-codec", "parseResponse", e.to_string()))?;
            let mut s = String::from("{");
            s.push_str(&format!("\"version\":{}", json_str(&r.version)));
            s.push_str(&format!(",\"status\":{}", r.status));
            s.push_str(&format!(",\"reason\":{}", json_str(&r.reason)));
            s.push_str(",\"headers\":[");
            for (i, (n, v)) in r.headers.iter().enumerate() {
                if i > 0 { s.push(','); }
                s.push_str(&format!("[{},{}]", json_str(n), json_str(v)));
            }
            s.push_str("],\"body\":[");
            for (i, b) in r.body.iter().enumerate() {
                if i > 0 { s.push(','); }
                s.push_str(&b.to_string());
            }
            s.push_str("]}");
            Ok(s)
        })?,
    )?;
    ns.set(
        "serializeRequest",
        Function::new(ctx.clone(),
            |method: String, target: String, headers: Vec<Vec<String>>, body: Vec<u8>| -> Vec<u8> {
            let hs: Vec<(String, String)> = headers.into_iter()
                .filter_map(|p| if p.len() == 2 { Some((p[0].clone(), p[1].clone())) } else { None })
                .collect();
            rusty_http_codec::serialize_request(&method, &target, &hs, &body)
        })?,
    )?;
    ns.set(
        "serializeResponse",
        Function::new(ctx.clone(),
            |status: u32, reason: String, headers: Vec<Vec<String>>, body: Vec<u8>| -> Vec<u8> {
            let hs: Vec<(String, String)> = headers.into_iter()
                .filter_map(|p| if p.len() == 2 { Some((p[0].clone(), p[1].clone())) } else { None })
                .collect();
            rusty_http_codec::serialize_response(status as u16, &reason, &hs, &body)
        })?,
    )?;
    ns.set(
        "chunkedEncodeFlat",
        Function::new(ctx.clone(), |flat: Vec<u8>, lengths: Vec<u32>| -> Vec<u8> {
            // JS-side flattens chunks into a single byte array + per-chunk
            // lengths. We slice them out here and pass &[&[u8]] to the codec.
            let mut chunks: Vec<&[u8]> = Vec::with_capacity(lengths.len());
            let mut off = 0usize;
            for len in &lengths {
                let n = *len as usize;
                chunks.push(&flat[off..off + n]);
                off += n;
            }
            rusty_http_codec::chunked_encode(&chunks)
        })?,
    )?;
    ns.set(
        "chunkedDecode",
        Function::new(ctx.clone(), |bytes: Vec<u8>| -> JsResult<Vec<u8>> {
            rusty_http_codec::chunked_decode(&bytes)
                .map_err(|e| rquickjs::Error::new_from_js_message("http-codec", "chunkedDecode", e.to_string()))
        })?,
    )?;
    global.set("__httpCodec", ns)?;
    // JS-side facade: globalThis.HTTP.parseRequest / parseResponse return
    // structured objects (auto-parse the JSON the FFI emits).
    ctx.eval::<(), _>(r#"
        (function() {
            const raw = globalThis.__httpCodec;
            // The rquickjs Vec<u8> binding does not accept Uint8Array directly;
            // a plain JS Array of numbers is required. Direct iteration is more
            // robust than Array.from for typed-array inputs in this runtime.
            function toByteArray(b) {
                if (b == null) return [];
                if (typeof b === "string") b = new TextEncoder().encode(b);
                else if (b instanceof ArrayBuffer) b = new Uint8Array(b);
                const arr = new Array(b.length);
                for (let i = 0; i < b.length; i++) arr[i] = b[i];
                return arr;
            }
            globalThis.HTTP = {
                parseRequest(bytes) {
                    const json = raw.parseRequestJson(toByteArray(bytes));
                    const p = JSON.parse(json);
                    return { ...p, body: new Uint8Array(p.body), headers: p.headers };
                },
                parseResponse(bytes) {
                    const json = raw.parseResponseJson(toByteArray(bytes));
                    const p = JSON.parse(json);
                    return { ...p, body: new Uint8Array(p.body), headers: p.headers };
                },
                serializeRequest(method, target, headers, body) {
                    return new Uint8Array(raw.serializeRequest(
                        method, target, headers || [], toByteArray(body)));
                },
                serializeResponse(status, reason, headers, body) {
                    return new Uint8Array(raw.serializeResponse(
                        status, reason || "", headers || [], toByteArray(body)));
                },
                chunkedEncode(chunks) {
                    const flat = [];
                    const lengths = [];
                    for (const c of chunks) {
                        const u8 = typeof c === "string" ? new TextEncoder().encode(c) : c;
                        for (let i = 0; i < u8.length; i++) flat.push(u8[i]);
                        lengths.push(u8.length);
                    }
                    return new Uint8Array(raw.chunkedEncodeFlat(flat, lengths));
                },
                chunkedDecode(bytes) {
                    return new Uint8Array(raw.chunkedDecode(toByteArray(bytes)));
                },
            };
        })();
    "#)?;
    Ok(())
}

// JSON-encode a string with the minimum-necessary escapes for the
// rusty-http-codec FFI emit path. Real serde would be cleaner but
// the codec pilot has no serde dependency.
fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ─────────────────── sockets (Tier-G TCP primitives) ────────────────
//
// Exposes the rusty-sockets pilot's handle-based primitives under
// globalThis.__sockets. JS-side callers receive opaque u64 handle ids
// (passed as f64 in JS since rquickjs converts u64 → f64 — safe for
// values < 2^53, which is many orders of magnitude beyond the slab's
// expected lifetime). Blocking semantics: each call may block; the
// caller is responsible for not blocking the main JS thread in a
// production server (a higher-level Bun.listen async wrapper will land
// in a follow-on round).

fn wire_websocket<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    // Π1.5.b: WebSocket primitive bindings. Exposes the
    // pilots/websocket/derived/ primitives under globalThis.__ws so a
    // future JS-side WebSocket class can compose frame encode/decode
    // and handshake key derivation on top of the existing TCP / __tls
    // transports and the http-codec Upgrade flow.
    //
    // Per seed §A8.2: stateless Rust helpers, no captured state.
    // The JS-side WebSocket class lands in Π1.5.c with the live
    // connection driver + Tier-J fixture.
    let ns = Object::new(ctx.clone())?;

    // generate_key() -> String
    ns.set("generate_key", Function::new(ctx.clone(), || -> JsResult<String> {
        rusty_websocket::generate_key()
            .map_err(|e| rquickjs::Error::new_from_js_message("ws", "generate_key", e.to_string()))
    })?)?;

    // derive_accept(key) -> String
    ns.set("derive_accept", Function::new(ctx.clone(), |key: String| -> String {
        rusty_websocket::derive_accept(&key)
    })?)?;

    // verify_accept(key, server_accept) -> bool
    ns.set("verify_accept", Function::new(ctx.clone(),
        |key: String, server: String| -> bool {
            rusty_websocket::verify_accept(&key, &server)
        })?)?;

    // encode_frame(fin, opcode, mask_optional, payload) -> Vec<u8>
    // mask_optional is either an empty Array (no mask) or a 4-element
    // Array<u8> (use this mask). opcode values per Opcode enum.
    ns.set("encode_frame", Function::new(ctx.clone(),
        |fin: bool, opcode: u8, mask_arr: Vec<u8>, payload: Vec<u8>|
            -> JsResult<Vec<u8>> {
            use rusty_websocket::*;
            let op = Opcode::from_u8(opcode)
                .ok_or_else(|| rquickjs::Error::new_from_js_message(
                    "ws", "encode_frame", format!("invalid opcode 0x{:x}", opcode)))?;
            let mask = if mask_arr.is_empty() { None }
                else if mask_arr.len() == 4 {
                    Some([mask_arr[0], mask_arr[1], mask_arr[2], mask_arr[3]])
                } else {
                    return Err(rquickjs::Error::new_from_js_message(
                        "ws", "encode_frame", "mask must be empty or 4 bytes"));
                };
            let frame = Frame { fin, opcode: op, mask, payload };
            encode_frame(&frame)
                .map_err(|e| rquickjs::Error::new_from_js_message("ws", "encode_frame", e.to_string()))
        })?)?;

    // decode_frame_json(bytes) -> JSON string with { fin, opcode, masked, payload, consumed }.
    // rquickjs return-type limitations make a flat JSON string the cleanest
    // wire shape, per the existing pattern in HTTP codec wiring.
    ns.set("decode_frame_json", Function::new(ctx.clone(),
        |bytes: Vec<u8>| -> JsResult<String> {
            use rusty_websocket::*;
            let (frame, consumed) = decode_frame(&bytes)
                .map_err(|e| rquickjs::Error::new_from_js_message(
                    "ws", "decode_frame", e.to_string()))?;
            let mut s = String::from("{");
            s.push_str(&format!("\"fin\":{}", frame.fin));
            s.push_str(&format!(",\"opcode\":{}", frame.opcode as u8));
            s.push_str(&format!(",\"masked\":{}", frame.mask.is_some()));
            s.push_str(&format!(",\"consumed\":{}", consumed));
            s.push_str(",\"payload\":[");
            for (i, b) in frame.payload.iter().enumerate() {
                if i > 0 { s.push(','); }
                s.push_str(&b.to_string());
            }
            s.push_str("]}");
            Ok(s)
        })?)?;

    // encode_close(code_optional, reason) -> Vec<u8>
    // code_optional: 0 means absent (since u16 has no canonical "none");
    // any non-zero value is sent. RFC 6455 reserves codes 0-999 so 0
    // unambiguously signals absence here.
    ns.set("encode_close", Function::new(ctx.clone(),
        |code: u16, reason: String| -> Vec<u8> {
            let code_opt = if code == 0 { None } else { Some(code) };
            rusty_websocket::encode_close(code_opt, &reason)
        })?)?;

    global.set("__ws", ns)?;
    Ok(())
}

fn wire_tls<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    // Π1.4.h: TLS 1.3 client integration. globalThis.__tls.connect(host,
    // port, trusted_certs_pem) → session_id; __tls.write/read/close(sid, ...)
    // for application-data exchange. The fetch() HTTPS path routes
    // through this namespace.
    //
    // Per seed §A8.16: TLS sessions are process-global state and must
    // be guarded. A static `Mutex<HashMap<id, Session>>` registry serves;
    // the JS-side handles are session IDs (u32).
    //
    // Per §A8.13 substrate-amortization: this round wires up the
    // integration; Π1.4.i diagnoses the live-handshake recv-UnexpectedEnd
    // surfaced in Π1.4.g and ships the Tier-J consumer-https-suite.
    use std::sync::Mutex;
    use std::collections::HashMap;

    struct TlsSessionState {
        session: Option<rusty_tls::TlsSession<rusty_tls::TcpTlsTransport>>,
        accumulator: Vec<u8>,
    }
    static SESSIONS: Mutex<Option<HashMap<u32, TlsSessionState>>> = Mutex::new(None);
    static NEXT_ID: Mutex<u32> = Mutex::new(1);

    fn alloc_id() -> u32 {
        let mut g = NEXT_ID.lock().unwrap_or_else(|e| e.into_inner());
        let id = *g;
        *g = g.wrapping_add(1);
        id
    }

    let ns = Object::new(ctx.clone())?;

    ns.set("connect", Function::new(ctx.clone(),
        |host: String, port: u32, trusted_ca_pem: String| -> JsResult<u32> {
            let mut store = rusty_tls::TrustStore::new();
            store.add_pem_bundle(&trusted_ca_pem)
                .map_err(|e| rquickjs::Error::new_from_js_message(
                    "tls", "connect", format!("trust store: {}", e)))?;
            let session = rusty_tls::tls_connect(&host, port as u16, &store)
                .map_err(|e| rquickjs::Error::new_from_js_message(
                    "tls", "connect", format!("handshake: {}", e)))?;
            let id = alloc_id();
            let mut g = SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
            if g.is_none() { *g = Some(HashMap::new()); }
            g.as_mut().unwrap().insert(id, TlsSessionState {
                session: Some(session), accumulator: Vec::new(),
            });
            Ok(id)
        })?)?;

    ns.set("write", Function::new(ctx.clone(),
        |sid: u32, data: Vec<u8>| -> JsResult<()> {
            let mut g = SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
            let map = g.as_mut().ok_or_else(|| rquickjs::Error::new_from_js_message(
                "tls", "write", "no sessions"))?;
            let st = map.get_mut(&sid).ok_or_else(|| rquickjs::Error::new_from_js_message(
                "tls", "write", "invalid session id"))?;
            let s = st.session.as_mut().ok_or_else(|| rquickjs::Error::new_from_js_message(
                "tls", "write", "session closed"))?;
            s.send_application_data(&data)
                .map_err(|e| rquickjs::Error::new_from_js_message(
                    "tls", "write", e.to_string()))
        })?)?;

    ns.set("read", Function::new(ctx.clone(),
        |sid: u32| -> JsResult<Vec<u8>> {
            let mut g = SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
            let map = g.as_mut().ok_or_else(|| rquickjs::Error::new_from_js_message(
                "tls", "read", "no sessions"))?;
            let st = map.get_mut(&sid).ok_or_else(|| rquickjs::Error::new_from_js_message(
                "tls", "read", "invalid session id"))?;
            let s = st.session.as_mut().ok_or_else(|| rquickjs::Error::new_from_js_message(
                "tls", "read", "session closed"))?;
            s.receive_application_data(&mut st.accumulator)
                .map_err(|e| rquickjs::Error::new_from_js_message(
                    "tls", "read", e.to_string()))
        })?)?;

    ns.set("close", Function::new(ctx.clone(),
        |sid: u32| -> JsResult<()> {
            let mut g = SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(map) = g.as_mut() {
                map.remove(&sid);
            }
            Ok(())
        })?)?;

    global.set("__tls", ns)?;
    Ok(())
}

fn wire_compression<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    // Tier-Π1.3 substrate: gzip + zlib + raw DEFLATE decode via the
    // rusty-compression pilot (RFC 1951 + RFC 1952 + RFC 1950, hand-rolled).
    // Decode-only this round; encode deferred. Exposed under
    // globalThis.__compression for fetch's response-body decode path.
    //
    // Per seed §A8.2 (stateless Rust + JS-side facade). Per seed §A8.13
    // (substrate-introduction round): the decoder is the substrate;
    // encode and brotli will compose on top in subsequent rounds.
    let ns = Object::new(ctx.clone())?;
    ns.set("gunzip", Function::new(ctx.clone(), |bytes: Vec<u8>| -> JsResult<Vec<u8>> {
        rusty_compression::gunzip(&bytes)
            .map_err(|e| rquickjs::Error::new_from_js_message(
                "compression", "gunzip", e.to_string()))
    })?)?;
    ns.set("inflate", Function::new(ctx.clone(), |bytes: Vec<u8>| -> JsResult<Vec<u8>> {
        // Raw DEFLATE.
        rusty_compression::inflate(&bytes)
            .map_err(|e| rquickjs::Error::new_from_js_message(
                "compression", "inflate", e.to_string()))
    })?)?;
    ns.set("http_deflate_inflate", Function::new(ctx.clone(), |bytes: Vec<u8>| -> JsResult<Vec<u8>> {
        // Content-Encoding: deflate — accept both RFC 1950 zlib wrapping
        // and raw RFC 1951 DEFLATE per real-world server behavior.
        rusty_compression::http_deflate_inflate(&bytes)
            .map_err(|e| rquickjs::Error::new_from_js_message(
                "compression", "http_deflate_inflate", e.to_string()))
    })?)?;
    // Π1.3.b: stored-block encoders. Trade compression ratio for format
    // compatibility; any conforming inflater accepts the output.
    ns.set("deflate_stored", Function::new(ctx.clone(), |bytes: Vec<u8>| -> Vec<u8> {
        rusty_compression::deflate_stored(&bytes)
    })?)?;
    ns.set("zlib_deflate_stored", Function::new(ctx.clone(), |bytes: Vec<u8>| -> Vec<u8> {
        rusty_compression::zlib_deflate_stored(&bytes)
    })?)?;
    ns.set("gzip_deflate_stored", Function::new(ctx.clone(), |bytes: Vec<u8>| -> Vec<u8> {
        rusty_compression::gzip_deflate_stored(&bytes)
    })?)?;
    // Π1.3.c: brotli decode per RFC 7932 via borrowed substrate
    // (brotli-decompressor crate; same policy as rusty-tls borrowing
    // std::net::TcpStream — algorithm is canonical, re-derivation has
    // no apparatus value, ~1500 LOC + 122KB dict avoided).
    ns.set("brotli_decode", Function::new(ctx.clone(), |bytes: Vec<u8>| -> JsResult<Vec<u8>> {
        rusty_compression::brotli_decode(&bytes)
            .map_err(|e| rquickjs::Error::new_from_js_message(
                "compression", "brotli_decode", e.to_string()))
    })?)?;
    global.set("__compression", ns)?;
    Ok(())
}

fn wire_dns<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    use std::net::ToSocketAddrs;
    // Tier-Π1.2: DNS resolution via std::net::ToSocketAddrs.
    //
    // Sync (blocking) resolver. This is the Tier-3 implementation-contingent
    // divergence from Bun (Bun uses c-ares for async resolution with an
    // in-process cache). Recorded against seed C1's three-tier authority
    // taxonomy: Tier-1 spec conformance holds (lookup resolves hostnames to
    // IPs per RFC 1035 / Node dns semantics); Tier-2 ecosystem-compat holds
    // (Bun.dns + node:dns surfaces visible to consumers); Tier-3 sync vs
    // async resolver internals deliberately diverge.
    //
    // Per seed §A8.16: no process-global resolver cache in this round (would
    // require a serial guard). A future round can add an Arc<Mutex<...>>
    // cache layer if consumer demand surfaces.
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "lookup_sync",
        Function::new(ctx.clone(), |host: String| -> JsResult<String> {
            // Use port 0 for resolution-only; we discard the SocketAddr's port.
            // ToSocketAddrs requires a host:port string.
            let addrs: Vec<std::net::SocketAddr> = format!("{}:0", host)
                .to_socket_addrs()
                .map_err(|e| rquickjs::Error::new_from_js_message(
                    "dns", "lookup_sync", e.to_string()))?
                .collect();
            // Prefer IPv4 first per Node/Bun default ("ipv4first" or "verbatim"
            // family ordering; we approximate "ipv4first" for Π1.2).
            let first = addrs.iter().find(|a| a.is_ipv4())
                .or_else(|| addrs.first())
                .ok_or_else(|| rquickjs::Error::new_from_js_message(
                    "dns", "lookup_sync",
                    format!("no addresses found for '{}'", host)))?;
            Ok(first.ip().to_string())
        })?,
    )?;
    ns.set(
        "lookup_all_sync",
        Function::new(ctx.clone(), |host: String| -> JsResult<Vec<Vec<String>>> {
            // Returns Vec<[address_string, family_string]> as a JSON-friendly
            // pair-list per seed §A8.6 (rquickjs FFI tuple workaround).
            let addrs: Vec<std::net::SocketAddr> = format!("{}:0", host)
                .to_socket_addrs()
                .map_err(|e| rquickjs::Error::new_from_js_message(
                    "dns", "lookup_all_sync", e.to_string()))?
                .collect();
            let out: Vec<Vec<String>> = addrs.iter().map(|a| {
                let family = if a.is_ipv4() { "4" } else { "6" };
                vec![a.ip().to_string(), family.to_string()]
            }).collect();
            Ok(out)
        })?,
    )?;
    global.set("__dns", ns)?;
    Ok(())
}

fn install_dns_js<'js>(ctx: &Ctx<'js>) -> JsResult<()> {
    // JS-side facades per seed §III.A8.2 (stateless Rust helpers + JS-side
    // class). Three surfaces:
    //   1. globalThis.Bun.dns.lookup(host, opts?) -> Promise<[{address, family}]>
    //   2. globalThis.nodeDns (node:dns shape): lookup(h, opts?, cb) +
    //      resolve / resolve4 / resolve6 + promises sub-namespace.
    //   3. globalThis.nodeDnsPromises (node:dns/promises shape):
    //      lookup(h, opts?) -> Promise<{address, family}>.
    ctx.eval::<(), _>(r#"
        (function() {
            const dns = globalThis.__dns;

            function decodeFamily(famStr) { return famStr === "4" ? 4 : 6; }

            // Bun.dns.lookup: returns array of {address, family} promises.
            // https://bun.sh/docs/api/dns
            globalThis.Bun = globalThis.Bun || {};
            globalThis.Bun.dns = globalThis.Bun.dns || {};
            globalThis.Bun.dns.lookup = async function(hostname, options) {
                const all = dns.lookup_all_sync(hostname);
                let filtered = all;
                if (options && typeof options.family === "number") {
                    const famStr = options.family === 6 ? "6" : "4";
                    filtered = all.filter(pair => pair[1] === famStr);
                }
                return filtered.map(pair => ({
                    address: pair[0],
                    family: decodeFamily(pair[1]),
                }));
            };

            // node:dns shape. lookup is callback-style; resolve* return arrays.
            const nodeDns = {};
            nodeDns.lookup = function(hostname, optionsOrCb, maybeCb) {
                let options, cb;
                if (typeof optionsOrCb === "function") {
                    cb = optionsOrCb;
                    options = {};
                } else {
                    options = optionsOrCb || {};
                    cb = maybeCb;
                }
                if (typeof cb !== "function") {
                    throw new TypeError("dns.lookup: callback required");
                }
                try {
                    const all = dns.lookup_all_sync(hostname);
                    let filtered = all;
                    if (typeof options.family === "number" && options.family !== 0) {
                        const famStr = options.family === 6 ? "6" : "4";
                        filtered = all.filter(p => p[1] === famStr);
                    }
                    if (filtered.length === 0) {
                        const err = new Error("ENOTFOUND " + hostname);
                        err.code = "ENOTFOUND";
                        err.hostname = hostname;
                        queueMicrotask(() => cb(err));
                        return;
                    }
                    if (options.all === true) {
                        const arr = filtered.map(p => ({
                            address: p[0], family: decodeFamily(p[1]),
                        }));
                        queueMicrotask(() => cb(null, arr));
                    } else {
                        const first = filtered[0];
                        queueMicrotask(() => cb(null, first[0], decodeFamily(first[1])));
                    }
                } catch (e) {
                    const err = new Error("ENOTFOUND " + hostname);
                    err.code = "ENOTFOUND";
                    err.hostname = hostname;
                    queueMicrotask(() => cb(err));
                }
            };
            nodeDns.resolve4 = function(hostname, cb) {
                try {
                    const all = dns.lookup_all_sync(hostname);
                    const v4 = all.filter(p => p[1] === "4").map(p => p[0]);
                    if (v4.length === 0) {
                        const err = new Error("ENOTFOUND " + hostname);
                        err.code = "ENOTFOUND";
                        err.hostname = hostname;
                        queueMicrotask(() => cb(err));
                        return;
                    }
                    queueMicrotask(() => cb(null, v4));
                } catch (e) {
                    const err = new Error("ENOTFOUND " + hostname);
                    err.code = "ENOTFOUND";
                    err.hostname = hostname;
                    queueMicrotask(() => cb(err));
                }
            };
            nodeDns.resolve6 = function(hostname, cb) {
                try {
                    const all = dns.lookup_all_sync(hostname);
                    const v6 = all.filter(p => p[1] === "6").map(p => p[0]);
                    if (v6.length === 0) {
                        const err = new Error("ENOTFOUND " + hostname);
                        err.code = "ENOTFOUND";
                        err.hostname = hostname;
                        queueMicrotask(() => cb(err));
                        return;
                    }
                    queueMicrotask(() => cb(null, v6));
                } catch (e) {
                    const err = new Error("ENOTFOUND " + hostname);
                    err.code = "ENOTFOUND";
                    err.hostname = hostname;
                    queueMicrotask(() => cb(err));
                }
            };
            nodeDns.resolve = function(hostname, rrtypeOrCb, maybeCb) {
                let rrtype, cb;
                if (typeof rrtypeOrCb === "function") {
                    cb = rrtypeOrCb;
                    rrtype = "A";
                } else {
                    rrtype = rrtypeOrCb || "A";
                    cb = maybeCb;
                }
                if (rrtype === "AAAA") return nodeDns.resolve6(hostname, cb);
                return nodeDns.resolve4(hostname, cb);
            };

            // node:dns/promises sub-namespace and standalone module.
            const promisesNs = {};
            promisesNs.lookup = function(hostname, options) {
                return new Promise((resolve, reject) => {
                    nodeDns.lookup(hostname, options || {}, (err, addr, fam) => {
                        if (err) reject(err);
                        else if (options && options.all) resolve(addr);
                        else resolve({ address: addr, family: fam });
                    });
                });
            };
            promisesNs.resolve4 = function(hostname) {
                return new Promise((resolve, reject) => {
                    nodeDns.resolve4(hostname, (e, r) => e ? reject(e) : resolve(r));
                });
            };
            promisesNs.resolve6 = function(hostname) {
                return new Promise((resolve, reject) => {
                    nodeDns.resolve6(hostname, (e, r) => e ? reject(e) : resolve(r));
                });
            };
            promisesNs.resolve = function(hostname, rrtype) {
                return new Promise((resolve, reject) => {
                    nodeDns.resolve(hostname, rrtype || "A", (e, r) => e ? reject(e) : resolve(r));
                });
            };
            nodeDns.promises = promisesNs;

            globalThis.nodeDns = nodeDns;
            globalThis.nodeDnsPromises = promisesNs;
        })();
    "#)?;
    Ok(())
}

fn wire_sockets<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "listenerBind",
        Function::new(ctx.clone(), |addr: String| -> JsResult<Vec<rquickjs::Value>> {
            // Can't easily return a tuple; emit JSON instead, parsed JS-side.
            let _ = addr;
            unreachable!()
        })?,
    )?;
    // Above approach can't capture ctx; emit JSON strings instead.
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "listenerBindJson",
        Function::new(ctx.clone(), |addr: String| -> JsResult<String> {
            let (id, addr) = rusty_sockets::listener_bind(&addr)
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "bind", e.to_string()))?;
            Ok(format!("{{\"id\":{},\"addr\":{}}}", id, json_str(&addr)))
        })?,
    )?;
    ns.set(
        "listenerAcceptJson",
        Function::new(ctx.clone(), |id: f64| -> JsResult<String> {
            let (sid, peer) = rusty_sockets::listener_accept(id as u64)
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "accept", e.to_string()))?;
            Ok(format!("{{\"id\":{},\"peer\":{}}}", sid, json_str(&peer)))
        })?,
    )?;
    ns.set(
        "streamConnect",
        Function::new(ctx.clone(), |addr: String| -> JsResult<f64> {
            rusty_sockets::stream_connect(&addr)
                .map(|id| id as f64)
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "connect", e.to_string()))
        })?,
    )?;
    ns.set(
        "streamConnectTimeout",
        Function::new(ctx.clone(), |addr: String, timeout_ms: f64| -> JsResult<f64> {
            rusty_sockets::stream_connect_timeout(&addr, timeout_ms as u64)
                .map(|id| id as f64)
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "connect", e.to_string()))
        })?,
    )?;
    ns.set(
        "streamRead",
        Function::new(ctx.clone(), |id: f64, max: f64| -> JsResult<Vec<u8>> {
            rusty_sockets::stream_read(id as u64, max as usize)
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "read", e.to_string()))
        })?,
    )?;
    ns.set(
        "streamWrite",
        Function::new(ctx.clone(), |id: f64, data: Vec<u8>| -> JsResult<f64> {
            rusty_sockets::stream_write(id as u64, &data)
                .map(|n| n as f64)
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "write", e.to_string()))
        })?,
    )?;
    ns.set(
        "streamWriteAll",
        Function::new(ctx.clone(), |id: f64, data: Vec<u8>| -> JsResult<()> {
            rusty_sockets::stream_write_all(id as u64, &data)
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "writeAll", e.to_string()))
        })?,
    )?;
    ns.set(
        "streamPeerAddr",
        Function::new(ctx.clone(), |id: f64| -> JsResult<String> {
            rusty_sockets::stream_peer_addr(id as u64)
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "peerAddr", e.to_string()))
        })?,
    )?;
    ns.set(
        "streamLocalAddr",
        Function::new(ctx.clone(), |id: f64| -> JsResult<String> {
            rusty_sockets::stream_local_addr(id as u64)
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "localAddr", e.to_string()))
        })?,
    )?;
    ns.set(
        "handleClose",
        Function::new(ctx.clone(), |id: f64| -> JsResult<()> {
            rusty_sockets::handle_close(id as u64)
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "close", e.to_string()))
        })?,
    )?;
    // Π2.6.b: non-blocking stream primitives. fetch()'s read-response
    // loop alternates streamTryRead with __tickKeepAlive + microtask
    // yield so a same-process Bun.serve({autoServe:true}) round-trips.
    ns.set(
        "streamSetNonblocking",
        Function::new(ctx.clone(), |id: f64, on: bool| -> JsResult<()> {
            rusty_sockets::stream_set_nonblocking(id as u64, on)
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "setNonblocking", e.to_string()))
        })?,
    )?;
    // streamTryRead returns either a byte array (possibly empty = EOF) or
    // a sentinel string "wouldblock" for the JS side to detect. f64-or-array
    // return is awkward through rquickjs; the string sentinel keeps the
    // JS facade trivial.
    // streamTryRead returns Vec<i32>:
    //   - [-1]            → WouldBlock (sentinel)
    //   - []              → EOF (orderly close)
    //   - [b0, b1, ...]   → data bytes (each 0..255)
    // The negative sentinel cannot collide with byte values (0..255 only).
    ns.set(
        "streamTryRead",
        Function::new(ctx.clone(), |id: f64, max: f64| -> JsResult<Vec<i32>> {
            match rusty_sockets::stream_try_read(id as u64, max as usize) {
                Ok(Some(buf)) => Ok(buf.iter().map(|&b| b as i32).collect()),
                Ok(None) => Ok(vec![-1]),
                Err(e) => Err(rquickjs::Error::new_from_js_message(
                    "sockets", "tryRead", e.to_string())),
            }
        })?,
    )?;
    // Async listener primitives (engagement option A; std-only equivalent of
    // Bun's WorkPool + concurrent_tasks + Waker pattern).
    ns.set(
        "listenerBindAsyncJson",
        Function::new(ctx.clone(), |addr: String| -> JsResult<String> {
            let (id, addr) = rusty_sockets::listener_bind_async(&addr)
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "bindAsync", e.to_string()))?;
            Ok(format!("{{\"id\":{},\"addr\":{}}}", id, json_str(&addr)))
        })?,
    )?;
    ns.set(
        "listenerPollJson",
        Function::new(ctx.clone(), |id: f64, max_wait_ms: f64| -> JsResult<String> {
            let ev = rusty_sockets::listener_poll(id as u64, max_wait_ms as u64)
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "poll", e.to_string()))?;
            Ok(match ev {
                None => "null".to_string(),
                Some(rusty_sockets::AsyncEvent::Connection { stream_id, peer }) =>
                    format!("{{\"type\":\"connection\",\"streamId\":{},\"peer\":{}}}", stream_id, json_str(&peer)),
                Some(rusty_sockets::AsyncEvent::Closed) =>
                    "{\"type\":\"closed\"}".to_string(),
                Some(rusty_sockets::AsyncEvent::Error(s)) =>
                    format!("{{\"type\":\"error\",\"message\":{}}}", json_str(&s)),
            })
        })?,
    )?;
    ns.set(
        "listenerStopAsync",
        Function::new(ctx.clone(), |id: f64| -> JsResult<()> {
            rusty_sockets::listener_stop_async(id as u64)
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "stopAsync", e.to_string()))
        })?,
    )?;
    ns.set(
        "asyncListenerAddr",
        Function::new(ctx.clone(), |id: f64| -> JsResult<String> {
            rusty_sockets::async_listener_addr(id as u64)
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "asyncListenerAddr", e.to_string()))
        })?,
    )?;
    ns.set(
        "handleKind",
        Function::new(ctx.clone(), |id: f64| -> JsResult<String> {
            rusty_sockets::handle_kind(id as u64)
                .map(|s| s.to_string())
                .map_err(|e| rquickjs::Error::new_from_js_message("sockets", "kind", e.to_string()))
        })?,
    )?;
    global.set("__sockets", ns)?;
    // Π2.6.c.a: mio reactor primitives. Single-process readiness
    // multiplexer; takes ownership of nothing — fd lifetime stays with
    // the sockets pilot's registry. JS layer (added below) exposes a
    // globalThis.__reactor namespace. No consumer migration this
    // round; the existing tryRead/idleSpin path stays operational.
    let reactor_ns = Object::new(ctx.clone())?;
    reactor_ns.set(
        "register",
        Function::new(ctx.clone(), |sid: f64| -> JsResult<()> {
            let id = sid as u64;
            let fd = rusty_sockets::stream_raw_fd(id).map_err(|e| {
                rquickjs::Error::new_from_js_message("reactor", "register-fd-lookup", e.to_string())
            })?;
            crate::reactor::register_fd(id, fd)
                .map_err(|e| rquickjs::Error::new_from_js_message("reactor", "register", e))
        })?,
    )?;
    reactor_ns.set(
        "deregister",
        Function::new(ctx.clone(), |sid: f64| -> JsResult<()> {
            let id = sid as u64;
            let fd = rusty_sockets::stream_raw_fd(id).map_err(|e| {
                rquickjs::Error::new_from_js_message("reactor", "deregister-fd-lookup", e.to_string())
            })?;
            crate::reactor::deregister_fd(id, fd)
                .map_err(|e| rquickjs::Error::new_from_js_message("reactor", "deregister", e))
        })?,
    )?;
    reactor_ns.set(
        "poll",
        Function::new(ctx.clone(), |timeout_ms: f64| -> JsResult<f64> {
            crate::reactor::poll_once(timeout_ms as i64)
                .map(|n| n as f64)
                .map_err(|e| rquickjs::Error::new_from_js_message("reactor", "poll", e))
        })?,
    )?;
    reactor_ns.set(
        "takeReady",
        Function::new(ctx.clone(), || -> JsResult<Vec<f64>> {
            Ok(crate::reactor::take_ready().into_iter().map(|t| t as f64).collect())
        })?,
    )?;
    reactor_ns.set(
        "registeredCount",
        Function::new(ctx.clone(), || -> JsResult<f64> {
            Ok(crate::reactor::registered_count() as f64)
        })?,
    )?;
    global.set("__reactor", reactor_ns)?;
    // JS-side facade: globalThis.TCP with parsed-result helpers + the
    // toByteArray pattern (per F8) for typed-array → Vec<u8> calls.
    ctx.eval::<(), _>(r#"
        (function() {
            const raw = globalThis.__sockets;
            function toByteArray(b) {
                if (b == null) return [];
                if (typeof b === "string") b = new TextEncoder().encode(b);
                else if (b instanceof ArrayBuffer) b = new Uint8Array(b);
                const arr = new Array(b.length);
                for (let i = 0; i < b.length; i++) arr[i] = b[i];
                return arr;
            }
            globalThis.TCP = {
                bind(addr) { return JSON.parse(raw.listenerBindJson(addr)); },
                accept(id) { return JSON.parse(raw.listenerAcceptJson(id)); },
                connect(addr) { return raw.streamConnect(addr); },
                connectTimeout(addr, ms) { return raw.streamConnectTimeout(addr, ms); },
                read(id, max) { return new Uint8Array(raw.streamRead(id, max)); },
                setNonblocking(id, on) { raw.streamSetNonblocking(id, !!on); },
                // Π2.6.b: nonblocking read. Returns:
                //   - null  on WouldBlock
                //   - Uint8Array(0) on EOF (orderly close)
                //   - Uint8Array(N) on data
                tryRead(id, max) {
                    const arr = raw.streamTryRead(id, max);
                    if (arr.length === 1 && arr[0] === -1) return null;
                    return new Uint8Array(arr);
                },
                write(id, data) { return raw.streamWrite(id, toByteArray(data)); },
                writeAll(id, data) { raw.streamWriteAll(id, toByteArray(data)); },
                peerAddr(id) { return raw.streamPeerAddr(id); },
                localAddr(id) { return raw.streamLocalAddr(id); },
                close(id) { raw.handleClose(id); },
                kind(id) { return raw.handleKind(id); },
                // Async-listener primitives (option A; see seed §V Tier-G).
                bindAsync(addr) { return JSON.parse(raw.listenerBindAsyncJson(addr)); },
                poll(id, maxWaitMs) {
                    const r = JSON.parse(raw.listenerPollJson(id, maxWaitMs));
                    return r;  // null | {type:"connection",streamId,peer} | {type:"closed"} | {type:"error",message}
                },
                stopAsync(id) { raw.listenerStopAsync(id); },
                asyncListenerAddr(id) { return raw.asyncListenerAddr(id); },
                // Π2.6.c.b: wait for readability via mio. Registers the
                // sid with __reactor, returns a Promise that resolves
                // when the eval loop's reactor-poll signals sid is
                // readable. The eval loop drains via __reactorDrain.
                // Multiple awaiters on the same sid coalesce.
                waitReadable(sid) {
                    return new Promise(function(resolve) {
                        let arr = globalThis.__reactorPending.get(sid);
                        if (!arr) {
                            arr = [];
                            globalThis.__reactorPending.set(sid, arr);
                            globalThis.__reactor.register(sid);
                        }
                        arr.push(resolve);
                    });
                },
            };
            // Π2.6.c.b: pending-readable Promise registry. Eval loop
            // calls __reactorDrain after reactor.poll returns.
            globalThis.__reactorPending = new Map();
            globalThis.__reactorDrain = function() {
                const ready = globalThis.__reactor.takeReady();
                let n = 0;
                for (const sid of ready) {
                    const arr = globalThis.__reactorPending.get(sid);
                    if (arr && arr.length) {
                        globalThis.__reactorPending.delete(sid);
                        globalThis.__reactor.deregister(sid);
                        for (const fn of arr) { try { fn(); } catch (_) {} }
                        n += arr.length;
                    }
                }
                return n;
            };
        })();
    "#)?;
    Ok(())
}

// ─────────────────── crypto + crypto.subtle ──────────────────────────

fn wire_crypto<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let crypto = Object::new(ctx.clone())?;
    crypto.set(
        "randomUUID",
        Function::new(ctx.clone(), || rusty_web_crypto::random_uuid_v4())?,
    )?;
    crypto.set(
        "getRandomBytes",
        Function::new(ctx.clone(), |n: u32| -> JsResult<Vec<u8>> {
            let mut buf = vec![0u8; n as usize];
            rusty_web_crypto::get_random_values(&mut buf)
                .map_err(|e| rquickjs::Error::new_from_js_message("crypto", "getRandomBytes", e.to_string()))?;
            Ok(buf)
        })?,
    )?;
    let subtle = Object::new(ctx.clone())?;
    subtle.set(
        "digestSha256Hex",
        Function::new(ctx.clone(), |data: String| {
            rusty_web_crypto::digest_sha256_hex(data.as_bytes())
        })?,
    )?;
    // Bytes form for spec digest(algorithm, data) JS wrapper.
    subtle.set(
        "digestSha256Bytes",
        Function::new(ctx.clone(), |data: Vec<u8>| -> Vec<u8> {
            rusty_web_crypto::digest_sha256(&data).to_vec()
        })?,
    )?;
    // HMAC-SHA-256 raw computation. Per E.8 closure: feeds the JS-side
    // crypto.subtle.importKey/sign/verify wrappers installed below.
    subtle.set(
        "hmacSha256Bytes",
        Function::new(ctx.clone(), |key: Vec<u8>, data: Vec<u8>| -> Vec<u8> {
            rusty_web_crypto::hmac_sha256(&key, &data).to_vec()
        })?,
    )?;
    // SHA-1 digest + HMAC-SHA-1 — legacy but still in use (OAuth 1.0,
    // older AWS SigV4 contexts, some webhook schemes, git object hashes).
    subtle.set(
        "digestSha1Bytes",
        Function::new(ctx.clone(), |data: Vec<u8>| -> Vec<u8> {
            rusty_web_crypto::digest_sha1(&data).to_vec()
        })?,
    )?;
    subtle.set(
        "hmacSha1Bytes",
        Function::new(ctx.clone(), |key: Vec<u8>, data: Vec<u8>| -> Vec<u8> {
            rusty_web_crypto::hmac_sha1(&key, &data).to_vec()
        })?,
    )?;
    // SHA-384 / SHA-512 — modern higher-security variants. JWT HS384/HS512,
    // higher-tier OAuth, FIPS 140-3 compliance contexts.
    subtle.set(
        "digestSha384Bytes",
        Function::new(ctx.clone(), |data: Vec<u8>| -> Vec<u8> {
            rusty_web_crypto::digest_sha384(&data).to_vec()
        })?,
    )?;
    subtle.set(
        "digestSha512Bytes",
        Function::new(ctx.clone(), |data: Vec<u8>| -> Vec<u8> {
            rusty_web_crypto::digest_sha512(&data).to_vec()
        })?,
    )?;
    subtle.set(
        "hmacSha384Bytes",
        Function::new(ctx.clone(), |key: Vec<u8>, data: Vec<u8>| -> Vec<u8> {
            rusty_web_crypto::hmac_sha384(&key, &data).to_vec()
        })?,
    )?;
    subtle.set(
        "hmacSha512Bytes",
        Function::new(ctx.clone(), |key: Vec<u8>, data: Vec<u8>| -> Vec<u8> {
            rusty_web_crypto::hmac_sha512(&key, &data).to_vec()
        })?,
    )?;
    // PBKDF2 (RFC 2898 / RFC 6070). Key-derivation surface adjacent to the
    // HMAC family — feeds JS-side crypto.subtle.deriveBits({name:"PBKDF2",...}).
    subtle.set(
        "pbkdf2HmacSha1Bytes",
        Function::new(ctx.clone(), |password: Vec<u8>, salt: Vec<u8>, iterations: u32, dk_len: u32| -> Vec<u8> {
            rusty_web_crypto::pbkdf2_hmac_sha1(&password, &salt, iterations, dk_len as usize)
        })?,
    )?;
    subtle.set(
        "pbkdf2HmacSha256Bytes",
        Function::new(ctx.clone(), |password: Vec<u8>, salt: Vec<u8>, iterations: u32, dk_len: u32| -> Vec<u8> {
            rusty_web_crypto::pbkdf2_hmac_sha256(&password, &salt, iterations, dk_len as usize)
        })?,
    )?;
    subtle.set(
        "pbkdf2HmacSha384Bytes",
        Function::new(ctx.clone(), |password: Vec<u8>, salt: Vec<u8>, iterations: u32, dk_len: u32| -> Vec<u8> {
            rusty_web_crypto::pbkdf2_hmac_sha384(&password, &salt, iterations, dk_len as usize)
        })?,
    )?;
    subtle.set(
        "pbkdf2HmacSha512Bytes",
        Function::new(ctx.clone(), |password: Vec<u8>, salt: Vec<u8>, iterations: u32, dk_len: u32| -> Vec<u8> {
            rusty_web_crypto::pbkdf2_hmac_sha512(&password, &salt, iterations, dk_len as usize)
        })?,
    )?;
    // HKDF (RFC 5869). HMAC-Extract-and-Expand KDF over all four
    // SHA variants. Real consumer: JOSE A*GCMKW content-key derivation,
    // OAuth2 PoP, Noise Protocol handshake-state expansion.
    subtle.set(
        "hkdfSha1Bytes",
        Function::new(ctx.clone(), |ikm: Vec<u8>, salt: Vec<u8>, info: Vec<u8>, length: u32| -> JsResult<Vec<u8>> {
            rusty_web_crypto::hkdf_sha1(&ikm, &salt, &info, length as usize)
                .map_err(|e| rquickjs::Error::new_from_js_message("HKDF", "derive", e))
        })?,
    )?;
    subtle.set(
        "hkdfSha256Bytes",
        Function::new(ctx.clone(), |ikm: Vec<u8>, salt: Vec<u8>, info: Vec<u8>, length: u32| -> JsResult<Vec<u8>> {
            rusty_web_crypto::hkdf_sha256(&ikm, &salt, &info, length as usize)
                .map_err(|e| rquickjs::Error::new_from_js_message("HKDF", "derive", e))
        })?,
    )?;
    subtle.set(
        "hkdfSha384Bytes",
        Function::new(ctx.clone(), |ikm: Vec<u8>, salt: Vec<u8>, info: Vec<u8>, length: u32| -> JsResult<Vec<u8>> {
            rusty_web_crypto::hkdf_sha384(&ikm, &salt, &info, length as usize)
                .map_err(|e| rquickjs::Error::new_from_js_message("HKDF", "derive", e))
        })?,
    )?;
    subtle.set(
        "hkdfSha512Bytes",
        Function::new(ctx.clone(), |ikm: Vec<u8>, salt: Vec<u8>, info: Vec<u8>, length: u32| -> JsResult<Vec<u8>> {
            rusty_web_crypto::hkdf_sha512(&ikm, &salt, &info, length as usize)
                .map_err(|e| rquickjs::Error::new_from_js_message("HKDF", "derive", e))
        })?,
    )?;
    // Argon2id (RFC 9106). Memory-hard password hash. Backs Bun.password.
    // Single-lane (p=1) substrate; multi-lane parallelism deferred.
    subtle.set(
        "argon2idBytes",
        Function::new(ctx.clone(), |password: Vec<u8>, salt: Vec<u8>, t_cost: u32, m_kib: u32, tau: u32| -> JsResult<Vec<u8>> {
            rusty_web_crypto::argon2id_hash(
                &password, &salt,
                &rusty_web_crypto::Argon2idParams {
                    t_cost, m_kib, parallelism: 1, tau,
                },
            ).map_err(|e| rquickjs::Error::new_from_js_message("Argon2id", "hash", e.to_string()))
        })?,
    )?;
    // Curve-parameterized ECDSA + ECDH over P-256 / P-384 / P-521.
    fn curve_by_name(name: &str) -> Result<rusty_web_crypto::Curve, String> {
        match name {
            "P-256" => Ok(rusty_web_crypto::curve_p256()),
            "P-384" => Ok(rusty_web_crypto::curve_p384()),
            "P-521" => Ok(rusty_web_crypto::curve_p521()),
            other => Err(format!("ECDSA/ECDH: unsupported curve {}", other)),
        }
    }
    fn hash_by_name(name: &str, data: &[u8]) -> Result<Vec<u8>, String> {
        match name {
            "SHA-1"   => Ok(rusty_web_crypto::digest_sha1(data).to_vec()),
            "SHA-256" => Ok(rusty_web_crypto::digest_sha256(data).to_vec()),
            "SHA-384" => Ok(rusty_web_crypto::digest_sha384(data).to_vec()),
            "SHA-512" => Ok(rusty_web_crypto::digest_sha512(data).to_vec()),
            other => Err(format!("ECDSA: unsupported hash {}", other)),
        }
    }
    subtle.set(
        "ecdsaSignBytes",
        Function::new(ctx.clone(), |curve_name: String, hash_name: String, d: Vec<u8>, msg: Vec<u8>, k: Vec<u8>| -> JsResult<Vec<u8>> {
            let c = curve_by_name(&curve_name)
                .map_err(|e| rquickjs::Error::new_from_js_message("ECDSA", "sign", e))?;
            let h = hash_by_name(&hash_name, &msg)
                .map_err(|e| rquickjs::Error::new_from_js_message("ECDSA", "sign", e))?;
            rusty_web_crypto::ecdsa_sign(&c, &d, &h, &k)
                .map_err(|e| rquickjs::Error::new_from_js_message("ECDSA", "sign", e))
        })?,
    )?;
    subtle.set(
        "ecdsaVerifyBytes",
        Function::new(ctx.clone(), |curve_name: String, hash_name: String, qx: Vec<u8>, qy: Vec<u8>, msg: Vec<u8>, sig: Vec<u8>| -> JsResult<bool> {
            let c = match curve_by_name(&curve_name) {
                Ok(c) => c,
                Err(_) => return Ok(false),
            };
            let h = match hash_by_name(&hash_name, &msg) {
                Ok(h) => h,
                Err(_) => return Ok(false),
            };
            Ok(rusty_web_crypto::ecdsa_verify(&c, &qx, &qy, &h, &sig).is_ok())
        })?,
    )?;
    subtle.set(
        "ecdhBytes",
        Function::new(ctx.clone(), |curve_name: String, d: Vec<u8>, qx: Vec<u8>, qy: Vec<u8>| -> JsResult<Vec<u8>> {
            let c = curve_by_name(&curve_name)
                .map_err(|e| rquickjs::Error::new_from_js_message("ECDH", "derive", e))?;
            rusty_web_crypto::ecdh(&c, &d, &qx, &qy)
                .map_err(|e| rquickjs::Error::new_from_js_message("ECDH", "derive", e))
        })?,
    )?;
    subtle.set(
        "ecGenerateKeypairBytes",
        Function::new(ctx.clone(), |curve_name: String| -> JsResult<Vec<Vec<u8>>> {
            let c = curve_by_name(&curve_name)
                .map_err(|e| rquickjs::Error::new_from_js_message("EC", "generateKey", e))?;
            let (d, x, y) = rusty_web_crypto::ec_generate_keypair(&c);
            Ok(vec![d, x, y])
        })?,
    )?;
    // ECDH over P-256 (SEC 1 §3.3.1). x-coordinate of d·Q.
    subtle.set(
        "ecdhP256Bytes",
        Function::new(ctx.clone(), |d: Vec<u8>, qx: Vec<u8>, qy: Vec<u8>| -> JsResult<Vec<u8>> {
            rusty_web_crypto::ecdh_p256(&d, &qx, &qy)
                .map_err(|e| rquickjs::Error::new_from_js_message("ECDH", "derive", e))
        })?,
    )?;
    // ECDSA over P-256 with SHA-256 (FIPS 186-4 §6.4). Signature
    // format: r ‖ s (P1363 / WebCrypto raw), 64 bytes total. Caller
    // provides a 32-byte k from /dev/urandom for sign; verify is
    // deterministic given the inputs.
    subtle.set(
        "ecdsaP256Sha256SignBytes",
        Function::new(ctx.clone(), |d: Vec<u8>, msg: Vec<u8>, k: Vec<u8>| -> JsResult<Vec<u8>> {
            rusty_web_crypto::ecdsa_p256_sha256_sign(&d, &msg, &k)
                .map_err(|e| rquickjs::Error::new_from_js_message("ECDSA", "sign", e))
        })?,
    )?;
    subtle.set(
        "ecdsaP256Sha256VerifyBytes",
        Function::new(ctx.clone(), |qx: Vec<u8>, qy: Vec<u8>, msg: Vec<u8>, sig: Vec<u8>| -> bool {
            rusty_web_crypto::ecdsa_p256_sha256_verify(&qx, &qy, &msg, &sig).is_ok()
        })?,
    )?;
    // RSASSA-PKCS1-v1_5 (RFC 8017 §8.2). Deterministic legacy RSA
    // signature — JWS RS256/384/512, X.509 CA, code-signing.
    subtle.set(
        "rsaPkcs1V15SignBytes",
        Function::new(ctx.clone(), |n: Vec<u8>, d: Vec<u8>, hash: Vec<u8>, hash_name: String| -> JsResult<Vec<u8>> {
            rusty_web_crypto::rsa_pkcs1_v15_sign(&n, &d, &hash, &hash_name)
                .map_err(|e| rquickjs::Error::new_from_js_message("PKCS1-v1_5", "sign", e))
        })?,
    )?;
    subtle.set(
        "rsaPkcs1V15VerifyBytes",
        Function::new(ctx.clone(), |n: Vec<u8>, e: Vec<u8>, hash: Vec<u8>, sig: Vec<u8>, hash_name: String| -> bool {
            rusty_web_crypto::rsa_pkcs1_v15_verify(&n, &e, &hash, &sig, &hash_name).is_ok()
        })?,
    )?;
    // RSA-PSS (RFC 8017 §8.1). Signing requires private key + salt;
    // verify requires public key + signature + claimed salt length.
    subtle.set(
        "rsaPssSignBytes",
        Function::new(ctx.clone(), |n: Vec<u8>, d: Vec<u8>, msg: Vec<u8>, salt: Vec<u8>, hash_name: String| -> JsResult<Vec<u8>> {
            let r = match hash_name.as_str() {
                "SHA-1"   => rusty_web_crypto::rsa_pss_sign(&n, &d, &msg, &salt,
                                 |b| rusty_web_crypto::digest_sha1(b).to_vec(), 20),
                "SHA-256" => rusty_web_crypto::rsa_pss_sign(&n, &d, &msg, &salt,
                                 |b| rusty_web_crypto::digest_sha256(b).to_vec(), 32),
                "SHA-384" => rusty_web_crypto::rsa_pss_sign(&n, &d, &msg, &salt,
                                 |b| rusty_web_crypto::digest_sha384(b).to_vec(), 48),
                "SHA-512" => rusty_web_crypto::rsa_pss_sign(&n, &d, &msg, &salt,
                                 |b| rusty_web_crypto::digest_sha512(b).to_vec(), 64),
                other => Err(format!("RSA-PSS: unsupported hash {}", other)),
            };
            r.map_err(|e| rquickjs::Error::new_from_js_message("RSA-PSS", "sign", e))
        })?,
    )?;
    subtle.set(
        "rsaPssVerifyBytes",
        Function::new(ctx.clone(), |n: Vec<u8>, e: Vec<u8>, msg: Vec<u8>, sig: Vec<u8>, s_len: u32, hash_name: String| -> JsResult<bool> {
            let r = match hash_name.as_str() {
                "SHA-1"   => rusty_web_crypto::rsa_pss_verify(&n, &e, &msg, &sig, s_len as usize,
                                 |b| rusty_web_crypto::digest_sha1(b).to_vec(), 20),
                "SHA-256" => rusty_web_crypto::rsa_pss_verify(&n, &e, &msg, &sig, s_len as usize,
                                 |b| rusty_web_crypto::digest_sha256(b).to_vec(), 32),
                "SHA-384" => rusty_web_crypto::rsa_pss_verify(&n, &e, &msg, &sig, s_len as usize,
                                 |b| rusty_web_crypto::digest_sha384(b).to_vec(), 48),
                "SHA-512" => rusty_web_crypto::rsa_pss_verify(&n, &e, &msg, &sig, s_len as usize,
                                 |b| rusty_web_crypto::digest_sha512(b).to_vec(), 64),
                _ => return Ok(false),
            };
            Ok(r.is_ok())
        })?,
    )?;
    // RSA-OAEP (RFC 8017 §7.1). Public-key encryption with OAEP padding.
    // Caller supplies a 32-byte seed (for SHA-256) — JS-side generates
    // it from /dev/urandom via crypto.getRandomValues before calling.
    subtle.set(
        "rsaOaepEncryptBytes",
        Function::new(ctx.clone(), |n: Vec<u8>, e: Vec<u8>, msg: Vec<u8>, label: Vec<u8>, seed: Vec<u8>, hash_name: String| -> JsResult<Vec<u8>> {
            let r = match hash_name.as_str() {
                "SHA-1"   => rusty_web_crypto::rsa_oaep_encrypt(&n, &e, &msg, &label, &seed,
                                 |b| rusty_web_crypto::digest_sha1(b).to_vec(), 20),
                "SHA-256" => rusty_web_crypto::rsa_oaep_encrypt(&n, &e, &msg, &label, &seed,
                                 |b| rusty_web_crypto::digest_sha256(b).to_vec(), 32),
                "SHA-384" => rusty_web_crypto::rsa_oaep_encrypt(&n, &e, &msg, &label, &seed,
                                 |b| rusty_web_crypto::digest_sha384(b).to_vec(), 48),
                "SHA-512" => rusty_web_crypto::rsa_oaep_encrypt(&n, &e, &msg, &label, &seed,
                                 |b| rusty_web_crypto::digest_sha512(b).to_vec(), 64),
                other => Err(format!("RSA-OAEP: unsupported hash {}", other)),
            };
            r.map_err(|e| rquickjs::Error::new_from_js_message("RSA-OAEP", "encrypt", e))
        })?,
    )?;
    subtle.set(
        "rsaOaepDecryptBytes",
        Function::new(ctx.clone(), |n: Vec<u8>, d: Vec<u8>, ct: Vec<u8>, label: Vec<u8>, hash_name: String| -> JsResult<Vec<u8>> {
            let r = match hash_name.as_str() {
                "SHA-1"   => rusty_web_crypto::rsa_oaep_decrypt(&n, &d, &ct, &label,
                                 |b| rusty_web_crypto::digest_sha1(b).to_vec(), 20),
                "SHA-256" => rusty_web_crypto::rsa_oaep_decrypt(&n, &d, &ct, &label,
                                 |b| rusty_web_crypto::digest_sha256(b).to_vec(), 32),
                "SHA-384" => rusty_web_crypto::rsa_oaep_decrypt(&n, &d, &ct, &label,
                                 |b| rusty_web_crypto::digest_sha384(b).to_vec(), 48),
                "SHA-512" => rusty_web_crypto::rsa_oaep_decrypt(&n, &d, &ct, &label,
                                 |b| rusty_web_crypto::digest_sha512(b).to_vec(), 64),
                other => Err(format!("RSA-OAEP: unsupported hash {}", other)),
            };
            r.map_err(|e| rquickjs::Error::new_from_js_message("RSA-OAEP", "decrypt", e))
        })?,
    )?;
    // AES-CBC (SP 800-38A §6.2 + PKCS#7 padding per RFC 5652 §6.3).
    subtle.set(
        "aesCbcEncryptBytes",
        Function::new(ctx.clone(), |key: Vec<u8>, iv: Vec<u8>, pt: Vec<u8>| -> JsResult<Vec<u8>> {
            rusty_web_crypto::aes_cbc_encrypt(&key, &iv, &pt)
                .map_err(|e| rquickjs::Error::new_from_js_message("AES-CBC", "encrypt", e))
        })?,
    )?;
    subtle.set(
        "aesCbcDecryptBytes",
        Function::new(ctx.clone(), |key: Vec<u8>, iv: Vec<u8>, ct: Vec<u8>| -> JsResult<Vec<u8>> {
            rusty_web_crypto::aes_cbc_decrypt(&key, &iv, &ct)
                .map_err(|e| rquickjs::Error::new_from_js_message("AES-CBC", "decrypt", e))
        })?,
    )?;
    // AES-CTR (SP 800-38A §6.5).
    subtle.set(
        "aesCtrXorBytes",
        Function::new(ctx.clone(), |key: Vec<u8>, counter: Vec<u8>, length: u32, data: Vec<u8>| -> JsResult<Vec<u8>> {
            rusty_web_crypto::aes_ctr_xor_with_key(&key, &counter, length, &data)
                .map_err(|e| rquickjs::Error::new_from_js_message("AES-CTR", "crypt", e))
        })?,
    )?;
    // AES-KW (RFC 3394).
    subtle.set(
        "aesKwWrapBytes",
        Function::new(ctx.clone(), |kek: Vec<u8>, pt: Vec<u8>| -> JsResult<Vec<u8>> {
            rusty_web_crypto::aes_kw_wrap(&kek, &pt)
                .map_err(|e| rquickjs::Error::new_from_js_message("AES-KW", "wrap", e))
        })?,
    )?;
    subtle.set(
        "aesKwUnwrapBytes",
        Function::new(ctx.clone(), |kek: Vec<u8>, ct: Vec<u8>| -> JsResult<Vec<u8>> {
            rusty_web_crypto::aes_kw_unwrap(&kek, &ct)
                .map_err(|e| rquickjs::Error::new_from_js_message("AES-KW", "unwrap", e))
        })?,
    )?;
    // AES-GCM (FIPS 197 + SP 800-38D). WebCrypto encrypt/decrypt with
    // authenticated encryption, 12-byte IV (pilot scope), 16-byte tag.
    // Output layout: ciphertext || tag (matches WebCrypto / Bun).
    subtle.set(
        "aesGcmEncryptBytes",
        Function::new(ctx.clone(), |key: Vec<u8>, iv: Vec<u8>, aad: Vec<u8>, pt: Vec<u8>| -> JsResult<Vec<u8>> {
            rusty_web_crypto::aes_gcm_encrypt(&key, &iv, &aad, &pt)
                .map_err(|e| rquickjs::Error::new_from_js_message("AES-GCM", "encrypt", e))
        })?,
    )?;
    subtle.set(
        "aesGcmDecryptBytes",
        Function::new(ctx.clone(), |key: Vec<u8>, iv: Vec<u8>, aad: Vec<u8>, ct: Vec<u8>| -> JsResult<Vec<u8>> {
            rusty_web_crypto::aes_gcm_decrypt(&key, &iv, &aad, &ct)
                .map_err(|e| rquickjs::Error::new_from_js_message("AES-GCM", "decrypt", e))
        })?,
    )?;
    // Constant-time comparison helper for verify.
    subtle.set(
        "timingSafeEqualBytes",
        Function::new(ctx.clone(), |a: Vec<u8>, b: Vec<u8>| -> bool {
            rusty_web_crypto::timing_safe_equal(&a, &b)
        })?,
    )?;
    crypto.set("subtle", subtle)?;
    global.set("crypto", crypto)?;
    // Spec-compliant WebCrypto wrappers. Per M8/M9: aligns rusty-bun-host's
    // crypto.subtle with Bun's surface so consumer code using the standard
    // import/sign/verify/digest pattern works portably.
    //
    // E.8 partial closure: digest (SHA-256) + HMAC-SHA-256 import/sign/verify.
    // Async/RSA/ECDSA/AES remain out of basin.
    ctx.eval::<(), _>(r#"
        (function() {
            const subtle = globalThis.crypto.subtle;

            // CryptoKey global — many consumer libs (jose, oslo, openid-client)
            // do `instanceof CryptoKey` on values returned by subtle.importKey.
            // Defined here as a marker class; importKey sets the prototype on
            // its returns so the instanceof check passes without changing the
            // internal _bytes/_n/_e/_x/_y/_d shape.
            if (typeof globalThis.CryptoKey === "undefined") {
                globalThis.CryptoKey = class CryptoKey {};
            }

            // crypto.getRandomValues(typedArray) — fills the array with
            // cryptographically random bytes and returns the same array.
            globalThis.crypto.getRandomValues = function getRandomValues(typedArray) {
                if (!typedArray || typeof typedArray.byteLength !== "number") {
                    throw new TypeError("crypto.getRandomValues: argument must be a typed array");
                }
                const bytes = globalThis.crypto.getRandomBytes(typedArray.byteLength);
                const u8 = new Uint8Array(typedArray.buffer, typedArray.byteOffset, typedArray.byteLength);
                for (let i = 0; i < bytes.length; i++) u8[i] = bytes[i];
                return typedArray;
            };

            // Node-API crypto.createHash(algorithm) — classic streaming hash
            // API. Many libs (etag, express's internals, body-parser, ulid
            // historically, mongoose, every fingerprinter) call this. The
            // returned hasher accumulates via .update() then emits via
            // .digest([encoding]).
            // Pure-JS MD5 (RFC 1321) for crypto.createHash('md5').
            // S16 L5 closure: md5-hex (E.51) and other md5-using libs land here.
            const _md5 = (function() {
                function safeAdd(x, y) {
                    const lsw = (x & 0xffff) + (y & 0xffff);
                    const msw = (x >> 16) + (y >> 16) + (lsw >> 16);
                    return (msw << 16) | (lsw & 0xffff);
                }
                function rol(num, cnt) { return (num << cnt) | (num >>> (32 - cnt)); }
                function cmn(q, a, b, x, s, t) {
                    return safeAdd(rol(safeAdd(safeAdd(a, q), safeAdd(x, t)), s), b);
                }
                function ff(a,b,c,d,x,s,t){return cmn((b&c)|(~b&d),a,b,x,s,t);}
                function gg(a,b,c,d,x,s,t){return cmn((b&d)|(c&~d),a,b,x,s,t);}
                function hh(a,b,c,d,x,s,t){return cmn(b^c^d,a,b,x,s,t);}
                function ii(a,b,c,d,x,s,t){return cmn(c^(b|~d),a,b,x,s,t);}
                function binl(x, len) {
                    x[len >> 5] |= 0x80 << (len % 32);
                    x[(((len + 64) >>> 9) << 4) + 14] = len;
                    let a=1732584193, b=-271733879, c=-1732584194, d=271733878;
                    for (let i = 0; i < x.length; i += 16) {
                        const olda=a, oldb=b, oldc=c, oldd=d;
                        a=ff(a,b,c,d,x[i+0],7,-680876936);  d=ff(d,a,b,c,x[i+1],12,-389564586); c=ff(c,d,a,b,x[i+2],17,606105819); b=ff(b,c,d,a,x[i+3],22,-1044525330);
                        a=ff(a,b,c,d,x[i+4],7,-176418897);  d=ff(d,a,b,c,x[i+5],12,1200080426); c=ff(c,d,a,b,x[i+6],17,-1473231341); b=ff(b,c,d,a,x[i+7],22,-45705983);
                        a=ff(a,b,c,d,x[i+8],7,1770035416);  d=ff(d,a,b,c,x[i+9],12,-1958414417); c=ff(c,d,a,b,x[i+10],17,-42063); b=ff(b,c,d,a,x[i+11],22,-1990404162);
                        a=ff(a,b,c,d,x[i+12],7,1804603682); d=ff(d,a,b,c,x[i+13],12,-40341101); c=ff(c,d,a,b,x[i+14],17,-1502002290); b=ff(b,c,d,a,x[i+15],22,1236535329);
                        a=gg(a,b,c,d,x[i+1],5,-165796510); d=gg(d,a,b,c,x[i+6],9,-1069501632); c=gg(c,d,a,b,x[i+11],14,643717713); b=gg(b,c,d,a,x[i+0],20,-373897302);
                        a=gg(a,b,c,d,x[i+5],5,-701558691); d=gg(d,a,b,c,x[i+10],9,38016083); c=gg(c,d,a,b,x[i+15],14,-660478335); b=gg(b,c,d,a,x[i+4],20,-405537848);
                        a=gg(a,b,c,d,x[i+9],5,568446438); d=gg(d,a,b,c,x[i+14],9,-1019803690); c=gg(c,d,a,b,x[i+3],14,-187363961); b=gg(b,c,d,a,x[i+8],20,1163531501);
                        a=gg(a,b,c,d,x[i+13],5,-1444681467); d=gg(d,a,b,c,x[i+2],9,-51403784); c=gg(c,d,a,b,x[i+7],14,1735328473); b=gg(b,c,d,a,x[i+12],20,-1926607734);
                        a=hh(a,b,c,d,x[i+5],4,-378558); d=hh(d,a,b,c,x[i+8],11,-2022574463); c=hh(c,d,a,b,x[i+11],16,1839030562); b=hh(b,c,d,a,x[i+14],23,-35309556);
                        a=hh(a,b,c,d,x[i+1],4,-1530992060); d=hh(d,a,b,c,x[i+4],11,1272893353); c=hh(c,d,a,b,x[i+7],16,-155497632); b=hh(b,c,d,a,x[i+10],23,-1094730640);
                        a=hh(a,b,c,d,x[i+13],4,681279174); d=hh(d,a,b,c,x[i+0],11,-358537222); c=hh(c,d,a,b,x[i+3],16,-722521979); b=hh(b,c,d,a,x[i+6],23,76029189);
                        a=hh(a,b,c,d,x[i+9],4,-640364487); d=hh(d,a,b,c,x[i+12],11,-421815835); c=hh(c,d,a,b,x[i+15],16,530742520); b=hh(b,c,d,a,x[i+2],23,-995338651);
                        a=ii(a,b,c,d,x[i+0],6,-198630844); d=ii(d,a,b,c,x[i+7],10,1126891415); c=ii(c,d,a,b,x[i+14],15,-1416354905); b=ii(b,c,d,a,x[i+5],21,-57434055);
                        a=ii(a,b,c,d,x[i+12],6,1700485571); d=ii(d,a,b,c,x[i+3],10,-1894986606); c=ii(c,d,a,b,x[i+10],15,-1051523); b=ii(b,c,d,a,x[i+1],21,-2054922799);
                        a=ii(a,b,c,d,x[i+8],6,1873313359); d=ii(d,a,b,c,x[i+15],10,-30611744); c=ii(c,d,a,b,x[i+6],15,-1560198380); b=ii(b,c,d,a,x[i+13],21,1309151649);
                        a=ii(a,b,c,d,x[i+4],6,-145523070); d=ii(d,a,b,c,x[i+11],10,-1120210379); c=ii(c,d,a,b,x[i+2],15,718787259); b=ii(b,c,d,a,x[i+9],21,-343485551);
                        a=safeAdd(a,olda); b=safeAdd(b,oldb); c=safeAdd(c,oldc); d=safeAdd(d,oldd);
                    }
                    return [a, b, c, d];
                }
                function bytesToWords(bytes) {
                    const out = new Array(((bytes.length + 8) >> 6) * 16 + 16).fill(0);
                    for (let i = 0; i < bytes.length; i++) {
                        out[i >> 2] |= bytes[i] << ((i % 4) * 8);
                    }
                    return out;
                }
                function wordsToBytes(words) {
                    const out = new Uint8Array(16);
                    for (let i = 0; i < 16; i++) {
                        out[i] = (words[i >> 2] >>> ((i % 4) * 8)) & 0xff;
                    }
                    return out;
                }
                return function md5(bytes) {
                    const len = bytes.length * 8;
                    const x = bytesToWords(bytes);
                    return wordsToBytes(binl(x, len));
                };
            })();

            globalThis.crypto.createHash = function createHash(algorithm) {
                const algo = String(algorithm).toLowerCase().replace(/-/g, "");
                const supported = ["sha1", "sha256", "sha384", "sha512", "md5"];
                if (!supported.includes(algo)) {
                    throw new Error("crypto.createHash: unsupported algorithm '" + algorithm + "'");
                }
                const chunks = [];
                return {
                    update(data, encoding) {
                        const bytes = typeof data === "string"
                            ? new TextEncoder().encode(data)
                            : (data instanceof Uint8Array ? data : new Uint8Array(data));
                        chunks.push(bytes);
                        return this;
                    },
                    digest(encoding) {
                        let total = 0;
                        for (const c of chunks) total += c.length;
                        const combined = new Uint8Array(total);
                        let off = 0;
                        for (const c of chunks) { combined.set(c, off); off += c.length; }
                        const inArr = Array.from(combined);
                        let out;
                        if (algo === "sha1") out = globalThis.crypto.subtle.digestSha1Bytes(inArr);
                        else if (algo === "sha256") out = globalThis.crypto.subtle.digestSha256Bytes(inArr);
                        else if (algo === "sha384") out = globalThis.crypto.subtle.digestSha384Bytes(inArr);
                        else if (algo === "sha512") out = globalThis.crypto.subtle.digestSha512Bytes(inArr);
                        else if (algo === "md5") out = _md5(combined);
                        else throw new Error("createHash: unsupported algorithm " + algo);
                        const u8 = new Uint8Array(out);
                        if (!encoding || encoding === "buffer") {
                            return typeof Buffer !== "undefined" ? Buffer.from(u8) : u8;
                        }
                        if (encoding === "hex") {
                            return Array.from(u8).map(b => b.toString(16).padStart(2, "0")).join("");
                        }
                        if (encoding === "base64") {
                            let s = "";
                            for (const b of u8) s += String.fromCharCode(b);
                            return btoa(s);
                        }
                        if (encoding === "base64url") {
                            let s = "";
                            for (const b of u8) s += String.fromCharCode(b);
                            return btoa(s).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
                        }
                        throw new Error("crypto.createHash.digest: unsupported encoding '" + encoding + "'");
                    },
                };
            };

            globalThis.crypto.createHmac = function createHmac(algorithm, key) {
                const algo = String(algorithm).toLowerCase().replace(/-/g, "");
                const keyBytes = typeof key === "string"
                    ? new TextEncoder().encode(key)
                    : (key instanceof Uint8Array ? key : new Uint8Array(key));
                const chunks = [];
                return {
                    update(data) {
                        const bytes = typeof data === "string"
                            ? new TextEncoder().encode(data)
                            : (data instanceof Uint8Array ? data : new Uint8Array(data));
                        chunks.push(bytes);
                        return this;
                    },
                    digest(encoding) {
                        let total = 0;
                        for (const c of chunks) total += c.length;
                        const combined = new Uint8Array(total);
                        let off = 0;
                        for (const c of chunks) { combined.set(c, off); off += c.length; }
                        const inArr = Array.from(combined);
                        const keyArr = Array.from(keyBytes);
                        let out;
                        if (algo === "sha1") out = globalThis.crypto.subtle.hmacSha1Bytes(keyArr, inArr);
                        else if (algo === "sha256") out = globalThis.crypto.subtle.hmacSha256Bytes(keyArr, inArr);
                        else if (algo === "sha384") out = globalThis.crypto.subtle.hmacSha384Bytes(keyArr, inArr);
                        else if (algo === "sha512") out = globalThis.crypto.subtle.hmacSha512Bytes(keyArr, inArr);
                        else throw new Error("hmac: unsupported algo " + algorithm);
                        const u8 = new Uint8Array(out);
                        if (!encoding || encoding === "buffer") {
                            return typeof Buffer !== "undefined" ? Buffer.from(u8) : u8;
                        }
                        if (encoding === "hex") return Array.from(u8).map(b => b.toString(16).padStart(2, "0")).join("");
                        if (encoding === "base64") {
                            let s = ""; for (const b of u8) s += String.fromCharCode(b);
                            return btoa(s);
                        }
                        if (encoding === "base64url") {
                            let s = ""; for (const b of u8) s += String.fromCharCode(b);
                            return btoa(s).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
                        }
                        throw new Error("createHmac.digest: unsupported encoding '" + encoding + "'");
                    },
                };
            };

            // Node-API crypto.randomBytes(n) -> Buffer. Many libraries
            // (ulid, several JWT impls, mongoose ObjectId) get crypto via
            // `require("crypto")` then call randomBytes — distinct from
            // Web Crypto's getRandomValues. Returns a Buffer (Uint8Array
            // subclass) with readUInt8 etc methods inherited from Buffer.
            globalThis.crypto.randomBytes = function randomBytes(n) {
                if (typeof n !== "number" || n < 0) {
                    throw new TypeError("crypto.randomBytes: n must be a non-negative number");
                }
                const bytes = globalThis.crypto.getRandomBytes(n);
                return typeof Buffer !== "undefined" ? Buffer.from(bytes) : new Uint8Array(bytes);
            };
            // Node-portable aliases. crypto.webcrypto is the WebCrypto
            // namespace, which IS globalThis.crypto (Node exposes both names).
            // crypto.pbkdf2Sync is wired via subtle.deriveBitsPbkdf2.
            globalThis.crypto.webcrypto = globalThis.crypto;
            if (typeof globalThis.crypto.subtle.deriveBitsPbkdf2 === "function") {
                globalThis.crypto.pbkdf2Sync = function pbkdf2Sync(password, salt, iterations, keylen, digest) {
                    const passBytes = typeof password === "string"
                        ? Array.from(new TextEncoder().encode(password))
                        : Array.from(password);
                    const saltBytes = typeof salt === "string"
                        ? Array.from(new TextEncoder().encode(salt))
                        : Array.from(salt);
                    const algo = String(digest || "sha1").toLowerCase().replace(/-/g, "");
                    const out = globalThis.crypto.subtle.deriveBitsPbkdf2(
                        passBytes, saltBytes, iterations, keylen, algo);
                    return typeof Buffer !== "undefined" ? Buffer.from(out) : new Uint8Array(out);
                };
            } else {
                // Fallback stub: throw on call so callers surface the gap
                // explicitly rather than silently producing wrong bytes.
                globalThis.crypto.pbkdf2Sync = function pbkdf2Sync() {
                    throw new Error("crypto.pbkdf2Sync: deriveBitsPbkdf2 not wired");
                };
            }
            globalThis.crypto.randomFillSync = function randomFillSync(typedArray, offset = 0, size) {
                if (!typedArray || typeof typedArray.byteLength !== "number") {
                    throw new TypeError("crypto.randomFillSync: argument must be a typed array");
                }
                const len = size !== undefined ? size : typedArray.byteLength - offset;
                const bytes = globalThis.crypto.getRandomBytes(len);
                const u8 = new Uint8Array(typedArray.buffer, typedArray.byteOffset, typedArray.byteLength);
                for (let i = 0; i < bytes.length; i++) u8[offset + i] = bytes[i];
                return typedArray;
            };

            // Normalize WebCrypto data inputs (string / ArrayBuffer / TypedArray
            // / DataView / Array) to a plain byte array for the FFI.
            function toBytes(data) {
                if (typeof data === "string") {
                    return Array.from(new TextEncoder().encode(data));
                } else if (data instanceof ArrayBuffer) {
                    return Array.from(new Uint8Array(data));
                } else if (data && typeof data === "object" && "byteLength" in data) {
                    return Array.from(new Uint8Array(
                        data.buffer || data, data.byteOffset || 0, data.byteLength));
                } else if (Array.isArray(data)) {
                    return data.slice();
                }
                throw new TypeError("WebCrypto: unsupported data type");
            }
            function toArrayBuffer(bytes) {
                const ab = new ArrayBuffer(bytes.length);
                new Uint8Array(ab).set(bytes);
                return ab;
            }
            function normalizeAlg(algorithm) {
                if (typeof algorithm === "string") {
                    return { name: algorithm.toUpperCase().replace(/-/g, "") };
                }
                if (algorithm && algorithm.name) {
                    const norm = String(algorithm.name).toUpperCase().replace(/-/g, "");
                    const hash = algorithm.hash
                        ? (typeof algorithm.hash === "string"
                            ? algorithm.hash.toUpperCase().replace(/-/g, "")
                            : String(algorithm.hash.name).toUpperCase().replace(/-/g, ""))
                        : null;
                    return { name: norm, hash };
                }
                throw new TypeError("WebCrypto: invalid algorithm");
            }

            // Supported hash → (spec-name, digest fn, hmac fn).
            const HASHES = {
                "SHA256": {
                    spec: "SHA-256",
                    digest: (b) => subtle.digestSha256Bytes(b),
                    hmac: (k, b) => subtle.hmacSha256Bytes(k, b),
                    pbkdf2: (p, s, i, l) => subtle.pbkdf2HmacSha256Bytes(p, s, i, l),
                    hkdf: (ikm, salt, info, l) => subtle.hkdfSha256Bytes(ikm, salt, info, l),
                },
                "SHA1": {
                    spec: "SHA-1",
                    digest: (b) => subtle.digestSha1Bytes(b),
                    hmac: (k, b) => subtle.hmacSha1Bytes(k, b),
                    pbkdf2: (p, s, i, l) => subtle.pbkdf2HmacSha1Bytes(p, s, i, l),
                    hkdf: (ikm, salt, info, l) => subtle.hkdfSha1Bytes(ikm, salt, info, l),
                },
                "SHA384": {
                    spec: "SHA-384",
                    digest: (b) => subtle.digestSha384Bytes(b),
                    hmac: (k, b) => subtle.hmacSha384Bytes(k, b),
                    pbkdf2: (p, s, i, l) => subtle.pbkdf2HmacSha384Bytes(p, s, i, l),
                    hkdf: (ikm, salt, info, l) => subtle.hkdfSha384Bytes(ikm, salt, info, l),
                },
                "SHA512": {
                    spec: "SHA-512",
                    digest: (b) => subtle.digestSha512Bytes(b),
                    hmac: (k, b) => subtle.hmacSha512Bytes(k, b),
                    pbkdf2: (p, s, i, l) => subtle.pbkdf2HmacSha512Bytes(p, s, i, l),
                    hkdf: (ikm, salt, info, l) => subtle.hkdfSha512Bytes(ikm, salt, info, l),
                },
            };

            // digest(algorithm, data) → Promise<ArrayBuffer>
            subtle.digest = async function digest(algorithm, data) {
                const alg = normalizeAlg(algorithm);
                const h = HASHES[alg.name];
                if (!h) throw new Error("Unsupported digest algorithm: " + alg.name);
                return toArrayBuffer(h.digest(toBytes(data)));
            };

            // importKey(format, keyData, algorithm, extractable, keyUsages)
            // Pilot scope: format="raw", algorithm={name:"HMAC", hash:"SHA-256"|"SHA-1"}.
            // Returns a CryptoKey-shaped object whose private _bytes carry the
            // key material. Async per spec (returns Promise).
            // base64url decode helper for JWK imports.
            function b64urlToBytes(s) {
                const b64 = String(s).replace(/-/g, "+").replace(/_/g, "/") + "=".repeat((4 - s.length % 4) % 4);
                const bin = atob(b64);
                const out = new Array(bin.length);
                for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
                return out;
            }

            subtle.importKey = async function importKey(format, keyData, algorithm, extractable, keyUsages) {
                if (format === "jwk") {
                    const alg = normalizeAlg(algorithm);
                    if (alg.name === "ECDSA" || alg.name === "ECDH") {
                        const specName = alg.name === "ECDSA" ? "ECDSA" : "ECDH";
                        // JWK EC: {kty:"EC", crv:"P-256", x, y, d?}
                        if (!keyData || keyData.kty !== "EC") throw new TypeError("JWK kty must be EC");
                        if (keyData.crv !== "P-256" && keyData.crv !== "P-384" && keyData.crv !== "P-521") {
                            throw new Error("ECDSA/ECDH: unsupported curve " + keyData.crv);
                        }
                        const x = b64urlToBytes(keyData.x);
                        const y = b64urlToBytes(keyData.y);
                        if (keyData.d != null) {
                            const d = b64urlToBytes(keyData.d);
                            return {
                                type: "private", extractable: !!extractable,
                                algorithm: { name: specName, namedCurve: keyData.crv },
                                usages: Array.isArray(keyUsages) ? keyUsages.slice() : [],
                                _x: x, _y: y, _d: d, _algName: specName, _curve: keyData.crv,
                            };
                        }
                        return {
                            type: "public", extractable: !!extractable,
                            algorithm: { name: "ECDSA", namedCurve: "P-256" },
                            usages: Array.isArray(keyUsages) ? keyUsages.slice() : [],
                            _x: x, _y: y, _algName: specName, _curve: keyData.crv,
                        };
                    }
                    if (alg.name === "RSAOAEP" || alg.name === "RSAPSS" || alg.name === "RSASSAPKCS1V1_5") {
                        const specName = { "RSAOAEP": "RSA-OAEP", "RSAPSS": "RSA-PSS",
                                           "RSASSAPKCS1V1_5": "RSASSA-PKCS1-v1_5" }[alg.name];
                        if (!keyData || keyData.kty !== "RSA") throw new TypeError("JWK kty must be RSA");
                        const n = b64urlToBytes(keyData.n);
                        const e = b64urlToBytes(keyData.e);
                        const hashName = alg.hash
                            ? (typeof alg.hash === "string" ? alg.hash : alg.hash)
                            : "SHA-256";
                        // Normalize hash spec name. Accept "SHA-256" or "SHA256" forms.
                        const hashSpec = String(hashName).toUpperCase().replace(/^SHA/, "SHA-").replace(/^SHA--/, "SHA-").replace("SHA-SHA-", "SHA-");
                        // Private key if d is present; public-only otherwise.
                        if (keyData.d != null) {
                            const d = b64urlToBytes(keyData.d);
                            return {
                                type: "private", extractable: !!extractable,
                                algorithm: { name: specName, modulusLength: n.length * 8,
                                             publicExponent: e, hash: { name: hashSpec } },
                                usages: Array.isArray(keyUsages) ? keyUsages.slice() : [],
                                _n: n, _e: e, _d: d, _hashSpec: hashSpec, _algName: specName,
                            };
                        }
                        return {
                            type: "public", extractable: !!extractable,
                            algorithm: { name: specName, modulusLength: n.length * 8,
                                         publicExponent: e, hash: { name: hashSpec } },
                            usages: Array.isArray(keyUsages) ? keyUsages.slice() : [],
                            _n: n, _e: e, _hashSpec: hashSpec, _algName: specName,
                        };
                    }
                    throw new Error("JWK importKey: unsupported algorithm " + alg.name);
                }
                if (format !== "raw") {
                    throw new Error("Unsupported key format: " + format);
                }
                const alg = normalizeAlg(algorithm);
                if (alg.name === "PBKDF2") {
                    const bytes = toBytes(keyData);
                    return {
                        type: "secret",
                        extractable: !!extractable,
                        algorithm: { name: "PBKDF2" },
                        usages: Array.isArray(keyUsages) ? keyUsages.slice() : [],
                        _bytes: bytes,
                        _algName: "PBKDF2",
                    };
                }
                if (alg.name === "HKDF") {
                    const bytes = toBytes(keyData);
                    return {
                        type: "secret",
                        extractable: !!extractable,
                        algorithm: { name: "HKDF" },
                        usages: Array.isArray(keyUsages) ? keyUsages.slice() : [],
                        _bytes: bytes,
                        _algName: "HKDF",
                    };
                }
                if (alg.name === "AESGCM" || alg.name === "AESCBC" || alg.name === "AESCTR" || alg.name === "AESKW") {
                    const bytes = toBytes(keyData);
                    if (bytes.length !== 16 && bytes.length !== 24 && bytes.length !== 32) {
                        throw new Error(alg.name + " key must be 128/192/256 bits");
                    }
                    const specName = {
                        "AESGCM": "AES-GCM", "AESCBC": "AES-CBC",
                        "AESCTR": "AES-CTR", "AESKW": "AES-KW",
                    }[alg.name];
                    return {
                        type: "secret",
                        extractable: !!extractable,
                        algorithm: { name: specName, length: bytes.length * 8 },
                        usages: Array.isArray(keyUsages) ? keyUsages.slice() : [],
                        _bytes: bytes,
                        _algName: specName,
                    };
                }
                if (alg.name !== "HMAC") {
                    throw new Error("Unsupported key algorithm: " + alg.name);
                }
                const h = HASHES[alg.hash];
                if (!h) throw new Error("Unsupported HMAC hash: " + alg.hash);
                const bytes = toBytes(keyData);
                return {
                    type: "secret",
                    extractable: !!extractable,
                    algorithm: { name: "HMAC", hash: { name: h.spec }, length: bytes.length * 8 },
                    usages: Array.isArray(keyUsages) ? keyUsages.slice() : [],
                    _bytes: bytes,
                    _hashName: alg.hash,  // implementation-private; pinned at import
                    _algName: "HMAC",
                };
            };

            // Wrap importKey so every returned key has CryptoKey.prototype.
            // jose + oslo + openid-client + many other consumer libraries
            // gate signing/verification on `instanceof CryptoKey`.
            const _importKeyOrig = subtle.importKey.bind(subtle);
            subtle.importKey = async function importKey() {
                const k = await _importKeyOrig.apply(null, arguments);
                if (k && typeof k === "object") {
                    Object.setPrototypeOf(k, globalThis.CryptoKey.prototype);
                }
                return k;
            };

            // deriveBits(algorithm, baseKey, length) → Promise<ArrayBuffer>
            // Pilot scope: algorithm = {name:"PBKDF2", hash, salt, iterations}.
            subtle.deriveBits = async function deriveBits(algorithm, baseKey, length) {
                const algName = String(algorithm && algorithm.name || "").toUpperCase().replace(/-/g, "");
                if (typeof length !== "number" || length <= 0 || length % 8 !== 0) {
                    throw new Error("deriveBits length must be a positive multiple of 8");
                }
                if (!baseKey || !baseKey.usages.includes("deriveBits") && !baseKey.usages.includes("deriveKey")) {
                    throw new Error("Key not usable for deriveBits");
                }
                const hashName = algorithm && algorithm.hash
                    ? (typeof algorithm.hash === "string"
                        ? algorithm.hash.toUpperCase().replace(/-/g, "")
                        : String(algorithm.hash.name).toUpperCase().replace(/-/g, ""))
                    : null;
                if (algName === "PBKDF2") {
                    if (baseKey._algName !== "PBKDF2") {
                        throw new TypeError("deriveBits: baseKey is not a PBKDF2 key");
                    }
                    const h = HASHES[hashName];
                    if (!h || !h.pbkdf2) throw new Error("Unsupported PBKDF2 hash: " + hashName);
                    const iterations = algorithm.iterations >>> 0;
                    if (iterations === 0) throw new Error("PBKDF2 iterations must be > 0");
                    const salt = toBytes(algorithm.salt);
                    return toArrayBuffer(h.pbkdf2(baseKey._bytes, salt, iterations, length / 8));
                }
                if (algName === "ECDH") {
                    if (baseKey._algName !== "ECDH" || !baseKey._d) {
                        throw new TypeError("deriveBits: baseKey is not an ECDH private key");
                    }
                    if (!algorithm.public || algorithm.public._algName !== "ECDH") {
                        throw new TypeError("deriveBits: algorithm.public must be an ECDH public key");
                    }
                    if (baseKey._curve !== algorithm.public._curve) {
                        throw new Error("ECDH: keys must share a curve");
                    }
                    const fullSecret = subtle.ecdhBytes(
                        baseKey._curve, baseKey._d, algorithm.public._x, algorithm.public._y);
                    const byteLen = length / 8;
                    if (byteLen > fullSecret.length) {
                        throw new Error("ECDH: length exceeds field size");
                    }
                    return toArrayBuffer(fullSecret.slice(0, byteLen));
                }
                if (algName === "HKDF") {
                    if (baseKey._algName !== "HKDF") {
                        throw new TypeError("deriveBits: baseKey is not an HKDF key");
                    }
                    const h = HASHES[hashName];
                    if (!h || !h.hkdf) throw new Error("Unsupported HKDF hash: " + hashName);
                    const salt = algorithm.salt ? toBytes(algorithm.salt) : [];
                    const info = algorithm.info ? toBytes(algorithm.info) : [];
                    return toArrayBuffer(h.hkdf(baseKey._bytes, salt, info, length / 8));
                }
                throw new Error("Unsupported deriveBits algorithm: " + algName);
            };

            // encrypt(algorithm, key, data) → Promise<ArrayBuffer>
            // Pilot scope: algorithm = {name:"AES-GCM", iv, additionalData?, tagLength?}.
            subtle.encrypt = async function encrypt(algorithm, key, data) {
                const alg = normalizeAlg(algorithm);
                if (!key || !key.usages.includes("encrypt")) {
                    throw new Error("Key not usable for encrypt");
                }
                if (alg.name === "RSAOAEP") {
                    if (key._algName !== "RSA-OAEP") throw new TypeError("encrypt: key is not RSA-OAEP");
                    const hashSpec = key._hashSpec;
                    const hlen = { "SHA-1": 20, "SHA-256": 32, "SHA-384": 48, "SHA-512": 64 }[hashSpec];
                    if (!hlen) throw new Error("RSA-OAEP: unsupported hash " + hashSpec);
                    const label = algorithm.label ? toBytes(algorithm.label) : [];
                    // Generate random seed via crypto.getRandomValues.
                    const seedArr = new Uint8Array(hlen);
                    globalThis.crypto.getRandomValues(seedArr);
                    const seed = Array.from(seedArr);
                    return toArrayBuffer(subtle.rsaOaepEncryptBytes(
                        key._n, key._e, toBytes(data), label, seed, hashSpec));
                }
                if (alg.name === "AESGCM") {
                    if (key._algName !== "AES-GCM") throw new TypeError("encrypt: key is not AES-GCM");
                    if (!algorithm.iv) throw new TypeError("AES-GCM: iv required");
                    const tagLength = algorithm.tagLength == null ? 128 : (algorithm.tagLength | 0);
                    if (tagLength !== 128) throw new Error("AES-GCM pilot scope: tagLength must be 128");
                    const aad = algorithm.additionalData ? toBytes(algorithm.additionalData) : [];
                    return toArrayBuffer(subtle.aesGcmEncryptBytes(
                        key._bytes, toBytes(algorithm.iv), aad, toBytes(data)));
                }
                if (alg.name === "AESCBC") {
                    if (key._algName !== "AES-CBC") throw new TypeError("encrypt: key is not AES-CBC");
                    if (!algorithm.iv) throw new TypeError("AES-CBC: iv required");
                    return toArrayBuffer(subtle.aesCbcEncryptBytes(
                        key._bytes, toBytes(algorithm.iv), toBytes(data)));
                }
                if (alg.name === "AESCTR") {
                    if (key._algName !== "AES-CTR") throw new TypeError("encrypt: key is not AES-CTR");
                    if (!algorithm.counter) throw new TypeError("AES-CTR: counter required");
                    const length = algorithm.length | 0;
                    if (length <= 0 || length > 128) throw new Error("AES-CTR: length must be in 1..128");
                    return toArrayBuffer(subtle.aesCtrXorBytes(
                        key._bytes, toBytes(algorithm.counter), length, toBytes(data)));
                }
                throw new Error("Unsupported encrypt algorithm: " + alg.name);
            };

            // decrypt(algorithm, key, data) → Promise<ArrayBuffer>
            subtle.decrypt = async function decrypt(algorithm, key, data) {
                const alg = normalizeAlg(algorithm);
                if (!key || !key.usages.includes("decrypt")) {
                    throw new Error("Key not usable for decrypt");
                }
                if (alg.name === "RSAOAEP") {
                    if (key._algName !== "RSA-OAEP" || !key._d) {
                        throw new TypeError("decrypt: key is not an RSA-OAEP private key");
                    }
                    const label = algorithm.label ? toBytes(algorithm.label) : [];
                    return toArrayBuffer(subtle.rsaOaepDecryptBytes(
                        key._n, key._d, toBytes(data), label, key._hashSpec));
                }
                if (alg.name === "AESGCM") {
                    if (key._algName !== "AES-GCM") throw new TypeError("decrypt: key is not AES-GCM");
                    if (!algorithm.iv) throw new TypeError("AES-GCM: iv required");
                    const tagLength = algorithm.tagLength == null ? 128 : (algorithm.tagLength | 0);
                    if (tagLength !== 128) throw new Error("AES-GCM pilot scope: tagLength must be 128");
                    const aad = algorithm.additionalData ? toBytes(algorithm.additionalData) : [];
                    return toArrayBuffer(subtle.aesGcmDecryptBytes(
                        key._bytes, toBytes(algorithm.iv), aad, toBytes(data)));
                }
                if (alg.name === "AESCBC") {
                    if (key._algName !== "AES-CBC") throw new TypeError("decrypt: key is not AES-CBC");
                    if (!algorithm.iv) throw new TypeError("AES-CBC: iv required");
                    return toArrayBuffer(subtle.aesCbcDecryptBytes(
                        key._bytes, toBytes(algorithm.iv), toBytes(data)));
                }
                if (alg.name === "AESCTR") {
                    if (key._algName !== "AES-CTR") throw new TypeError("decrypt: key is not AES-CTR");
                    if (!algorithm.counter) throw new TypeError("AES-CTR: counter required");
                    const length = algorithm.length | 0;
                    if (length <= 0 || length > 128) throw new Error("AES-CTR: length must be in 1..128");
                    return toArrayBuffer(subtle.aesCtrXorBytes(
                        key._bytes, toBytes(algorithm.counter), length, toBytes(data)));
                }
                throw new Error("Unsupported decrypt algorithm: " + alg.name);
            };

            // wrapKey / unwrapKey for AES-KW (RFC 3394). Pilot scope: format="raw".
            subtle.wrapKey = async function wrapKey(format, key, wrappingKey, wrapAlgorithm) {
                if (format !== "raw") throw new Error("wrapKey pilot scope: format must be 'raw'");
                const alg = normalizeAlg(wrapAlgorithm);
                if (alg.name !== "AESKW") throw new Error("Unsupported wrap algorithm: " + alg.name);
                if (!wrappingKey || wrappingKey._algName !== "AES-KW") {
                    throw new TypeError("wrapKey: wrappingKey is not AES-KW");
                }
                if (!wrappingKey.usages.includes("wrapKey")) throw new Error("Key not usable for wrapKey");
                if (!key || !Array.isArray(key._bytes)) throw new TypeError("wrapKey: invalid key to wrap");
                return toArrayBuffer(subtle.aesKwWrapBytes(wrappingKey._bytes, key._bytes));
            };

            subtle.unwrapKey = async function unwrapKey(
                format, wrappedKey, unwrappingKey, unwrapAlgorithm,
                unwrappedKeyAlgorithm, extractable, keyUsages,
            ) {
                if (format !== "raw") throw new Error("unwrapKey pilot scope: format must be 'raw'");
                const alg = normalizeAlg(unwrapAlgorithm);
                if (alg.name !== "AESKW") throw new Error("Unsupported unwrap algorithm: " + alg.name);
                if (!unwrappingKey || unwrappingKey._algName !== "AES-KW") {
                    throw new TypeError("unwrapKey: unwrappingKey is not AES-KW");
                }
                if (!unwrappingKey.usages.includes("unwrapKey")) throw new Error("Key not usable for unwrapKey");
                const rawBytes = subtle.aesKwUnwrapBytes(unwrappingKey._bytes, toBytes(wrappedKey));
                return subtle.importKey("raw", rawBytes, unwrappedKeyAlgorithm, extractable, keyUsages);
            };

            function keyHash(key) {
                const name = key && key._hashName;
                const h = HASHES[name];
                if (!h) throw new Error("Unsupported HMAC hash on key: " + name);
                return h;
            }

            // sign(algorithm, key, data) → Promise<ArrayBuffer>
            subtle.sign = async function sign(algorithm, key, data) {
                const alg = normalizeAlg(algorithm);
                if (!key || !key.usages.includes("sign")) {
                    throw new Error("Key not usable for sign");
                }
                if (alg.name === "HMAC") {
                    if (!Array.isArray(key._bytes)) throw new TypeError("sign: key is not HMAC");
                    const h = keyHash(key);
                    return toArrayBuffer(h.hmac(key._bytes, toBytes(data)));
                }
                if (alg.name === "ECDSA") {
                    if (key._algName !== "ECDSA" || !key._d) {
                        throw new TypeError("sign: key is not an ECDSA private key");
                    }
                    const hashName = algorithm.hash
                        ? (typeof algorithm.hash === "string" ? algorithm.hash : algorithm.hash.name)
                        : "SHA-256";
                    const coordBytes = { "P-256": 32, "P-384": 48, "P-521": 66 }[key._curve];
                    if (!coordBytes) throw new Error("ECDSA: unsupported curve " + key._curve);
                    const kArr = new Uint8Array(coordBytes);
                    globalThis.crypto.getRandomValues(kArr);
                    // Ensure k < n by masking top bits when k has extra high bits
                    // (only relevant for P-521 where coord_bytes*8 = 528 > 521).
                    if (key._curve === "P-521") kArr[0] &= 0x01;
                    return toArrayBuffer(subtle.ecdsaSignBytes(
                        key._curve, hashName, key._d, toBytes(data), Array.from(kArr)));
                }
                if (alg.name === "RSASSAPKCS1V1_5") {
                    if (key._algName !== "RSASSA-PKCS1-v1_5" || !key._d) {
                        throw new TypeError("sign: key is not an RSASSA-PKCS1-v1_5 private key");
                    }
                    const hashName = key._hashSpec;
                    const HASH_FNS = {
                        "SHA-1": subtle.digestSha1Bytes,
                        "SHA-256": subtle.digestSha256Bytes,
                        "SHA-384": subtle.digestSha384Bytes,
                        "SHA-512": subtle.digestSha512Bytes,
                    };
                    const hashFn = HASH_FNS[hashName];
                    if (!hashFn) throw new Error("RSASSA-PKCS1-v1_5: unsupported hash " + hashName);
                    return toArrayBuffer(subtle.rsaPkcs1V15SignBytes(
                        key._n, key._d, hashFn(toBytes(data)), hashName));
                }
                if (alg.name === "RSAPSS") {
                    if (key._algName !== "RSA-PSS" || !key._d) {
                        throw new TypeError("sign: key is not an RSA-PSS private key");
                    }
                    const sLen = algorithm.saltLength | 0;
                    if (sLen < 0) throw new Error("RSA-PSS: saltLength must be ≥ 0");
                    const hlen = { "SHA-1": 20, "SHA-256": 32, "SHA-384": 48, "SHA-512": 64 }[key._hashSpec];
                    if (!hlen) throw new Error("RSA-PSS: unsupported hash " + key._hashSpec);
                    const saltArr = new Uint8Array(sLen);
                    if (sLen > 0) globalThis.crypto.getRandomValues(saltArr);
                    return toArrayBuffer(subtle.rsaPssSignBytes(
                        key._n, key._d, toBytes(data), Array.from(saltArr), key._hashSpec));
                }
                throw new Error("Unsupported sign algorithm: " + alg.name);
            };

            // verify(algorithm, key, signature, data) → Promise<boolean>
            subtle.verify = async function verify(algorithm, key, signature, data) {
                const alg = normalizeAlg(algorithm);
                if (!key || !key.usages.includes("verify")) {
                    throw new Error("Key not usable for verify");
                }
                if (alg.name === "HMAC") {
                    if (!Array.isArray(key._bytes)) throw new TypeError("verify: key is not HMAC");
                    const h = keyHash(key);
                    const expected = h.hmac(key._bytes, toBytes(data));
                    return subtle.timingSafeEqualBytes(toBytes(signature), expected);
                }
                if (alg.name === "ECDSA") {
                    if (key._algName !== "ECDSA") throw new TypeError("verify: key is not ECDSA");
                    const hashName = algorithm.hash
                        ? (typeof algorithm.hash === "string" ? algorithm.hash : algorithm.hash.name)
                        : "SHA-256";
                    return subtle.ecdsaVerifyBytes(
                        key._curve, hashName, key._x, key._y, toBytes(data), toBytes(signature));
                }
                if (alg.name === "RSASSAPKCS1V1_5") {
                    if (key._algName !== "RSASSA-PKCS1-v1_5") {
                        throw new TypeError("verify: key is not RSASSA-PKCS1-v1_5");
                    }
                    const hashName = key._hashSpec;
                    const HASH_FNS = {
                        "SHA-1": subtle.digestSha1Bytes,
                        "SHA-256": subtle.digestSha256Bytes,
                        "SHA-384": subtle.digestSha384Bytes,
                        "SHA-512": subtle.digestSha512Bytes,
                    };
                    const hashFn = HASH_FNS[hashName];
                    if (!hashFn) throw new Error("RSASSA-PKCS1-v1_5: unsupported hash " + hashName);
                    return subtle.rsaPkcs1V15VerifyBytes(
                        key._n, key._e, hashFn(toBytes(data)), toBytes(signature), hashName);
                }
                if (alg.name === "RSAPSS") {
                    if (key._algName !== "RSA-PSS") {
                        throw new TypeError("verify: key is not RSA-PSS");
                    }
                    const sLen = algorithm.saltLength | 0;
                    return subtle.rsaPssVerifyBytes(
                        key._n, key._e, toBytes(data), toBytes(signature), sLen, key._hashSpec);
                }
                throw new Error("Unsupported verify algorithm: " + alg.name);
            };

            // generateKey(algorithm, extractable, keyUsages) → Promise<CryptoKey|CryptoKeyPair>
            // Symmetric algorithms (HMAC, AES-GCM/CTR/CBC/KW) generate random bytes and
            // wrap as a CryptoKey via importKey. ECDSA/ECDH delegate to the curve pilot
            // for keypair generation (returns {privateKey, publicKey}). RSA keypair gen
            // is deferred — large algorithmic surface; keepers usually pre-bake RSA keys.
            subtle.generateKey = async function generateKey(algorithm, extractable, keyUsages) {
                const alg = normalizeAlg(algorithm);
                if (alg.name === "HMAC") {
                    const hashName = (algorithm.hash && (typeof algorithm.hash === "string"
                        ? algorithm.hash : algorithm.hash.name)) || "SHA-256";
                    const lenBits = algorithm.length || ({
                        "SHA-1": 512, "SHA-256": 512, "SHA-384": 1024, "SHA-512": 1024,
                    }[hashName] || 256);
                    const bytes = globalThis.crypto.getRandomBytes(Math.ceil(lenBits / 8));
                    return subtle.importKey("raw", bytes,
                        { name: "HMAC", hash: hashName }, !!extractable, keyUsages);
                }
                if (alg.name === "AESGCM" || alg.name === "AESCTR" || alg.name === "AESCBC"
                    || alg.name === "AESKW") {
                    const len = algorithm.length || 256;
                    if (len !== 128 && len !== 192 && len !== 256) {
                        throw new Error("AES: unsupported key length " + len);
                    }
                    const bytes = globalThis.crypto.getRandomBytes(len / 8);
                    return subtle.importKey("raw", bytes,
                        { name: algorithm.name }, !!extractable, keyUsages);
                }
                if (alg.name === "ECDSA" || alg.name === "ECDH") {
                    const curve = algorithm.namedCurve || "P-256";
                    if (typeof subtle.ecGenerateKeypairBytes !== "function") {
                        throw new Error(alg.name + ": curve pilot does not expose ecGenerateKeypairBytes");
                    }
                    const pair = subtle.ecGenerateKeypairBytes(curve);
                    // pair: [d, x, y] byte arrays. Build CryptoKey objects
                    // directly without round-tripping through jwk (faster + skips
                    // base64url enc/dec).
                    const specName = alg.name;
                    const usages = Array.isArray(keyUsages) ? keyUsages.slice() : [];
                    const priv = {
                        type: "private", extractable: !!extractable,
                        algorithm: { name: specName, namedCurve: curve },
                        usages: usages.filter(u => u === "sign" || u === "deriveKey" || u === "deriveBits"),
                        _x: pair[1], _y: pair[2], _d: pair[0],
                        _algName: specName, _curve: curve,
                    };
                    const pub = {
                        type: "public", extractable: true,
                        algorithm: { name: specName, namedCurve: curve },
                        usages: usages.filter(u => u === "verify"),
                        _x: pair[1], _y: pair[2],
                        _algName: specName, _curve: curve,
                    };
                    return { privateKey: priv, publicKey: pub };
                }
                throw new Error("generateKey: unsupported algorithm " + alg.name);
            };

            // exportKey(format, key) → Promise<ArrayBuffer | JsonWebKey>
            // raw: return _bytes (symmetric). jwk: minimal shape for HMAC/AES/EC.
            // spki/pkcs8: stub — DER encoding deferred to consumer demand.
            subtle.exportKey = async function exportKey(format, key) {
                if (!key) throw new TypeError("exportKey: key is null");
                if (!key.extractable) throw new Error("exportKey: key is not extractable");
                if (format === "raw") {
                    if (Array.isArray(key._bytes)) return new Uint8Array(key._bytes).buffer;
                    if (key._x) return new Uint8Array([0x04].concat(key._x).concat(key._y || [])).buffer;
                    throw new Error("exportKey raw: key has no raw representation");
                }
                if (format === "jwk") {
                    const jwk = { ext: true, key_ops: key.usages };
                    if (Array.isArray(key._bytes)) {
                        jwk.kty = key._algName === "HMAC" ? "oct" : "oct";
                        const b64u = (bytes) => {
                            let bin = "";
                            for (const b of bytes) bin += String.fromCharCode(b);
                            return btoa(bin).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
                        };
                        jwk.k = b64u(key._bytes);
                        if (key._algName === "HMAC") {
                            const hash = key._hashSpec || "SHA-256";
                            jwk.alg = "HS" + hash.replace("SHA-", "");
                        } else if (key._algName && key._algName.startsWith("AES")) {
                            jwk.alg = key._algName.replace("AES-", "A") + (key._bytes.length * 8);
                        }
                        return jwk;
                    }
                    if (key._x && key._y) {
                        jwk.kty = "EC";
                        jwk.crv = key._curve || "P-256";
                        const b64u = (bytes) => {
                            let bin = "";
                            for (const b of bytes) bin += String.fromCharCode(b);
                            return btoa(bin).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
                        };
                        jwk.x = b64u(key._x);
                        jwk.y = b64u(key._y);
                        if (key._d) jwk.d = b64u(key._d);
                        return jwk;
                    }
                    throw new Error("exportKey jwk: unsupported key shape");
                }
                throw new Error("exportKey: format '" + format + "' not implemented");
            };

            // deriveKey(algorithm, baseKey, derivedKeyAlgorithm, extractable, usages)
            // Composes deriveBits + importKey: derive the right number of bits for the
            // derived algorithm, then import as that algorithm's key.
            subtle.deriveKey = async function deriveKey(algorithm, baseKey, derivedKeyAlgorithm, extractable, keyUsages) {
                const derivedAlg = normalizeAlg(derivedKeyAlgorithm);
                let lenBits;
                if (derivedAlg.name === "HMAC") {
                    const hashName = (derivedKeyAlgorithm.hash && (typeof derivedKeyAlgorithm.hash === "string"
                        ? derivedKeyAlgorithm.hash : derivedKeyAlgorithm.hash.name)) || "SHA-256";
                    lenBits = derivedKeyAlgorithm.length || ({
                        "SHA-1": 512, "SHA-256": 512, "SHA-384": 1024, "SHA-512": 1024,
                    }[hashName] || 256);
                } else if (derivedAlg.name.startsWith("AES")) {
                    lenBits = derivedKeyAlgorithm.length || 256;
                } else {
                    throw new Error("deriveKey: unsupported derived algorithm " + derivedAlg.name);
                }
                const bits = await subtle.deriveBits(algorithm, baseKey, lenBits);
                return subtle.importKey("raw", bits, derivedKeyAlgorithm, !!extractable, keyUsages);
            };
        })();
    "#)?;
    Ok(())
}

// ─────────────────── TextEncoder / TextDecoder ───────────────────────
//
// JS-side classes installed via setup script. Stateless Rust functions
// exposed in __te/__td namespace; classes hold no Rust-captured state
// (avoids QuickJS GC cycle issues observed with ctor-closure patterns).

fn wire_text_encoding<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let te_ns = Object::new(ctx.clone())?;
    te_ns.set(
        "encode",
        Function::new(ctx.clone(), |s: Opt<String>| -> Vec<u8> {
            let enc = rusty_textencoder::TextEncoder::new();
            enc.encode(s.0.as_deref())
        })?,
    )?;
    global.set("__te", te_ns)?;

    let td_ns = Object::new(ctx.clone())?;
    td_ns.set(
        "decode",
        Function::new(ctx.clone(), |bytes: Vec<u8>, label: Opt<String>| -> JsResult<String> {
            let mut d = rusty_textencoder::TextDecoder::new(
                label.0.as_deref(),
                Default::default(),
            )
            .map_err(|e| rquickjs::Error::new_from_js_message(
                "TextDecoder", "string", format!("{:?}", e)))?;
            d.decode(&bytes, Default::default()).map_err(|e| {
                rquickjs::Error::new_from_js_message(
                    "TextDecoder", "string", format!("{:?}", e))
            })
        })?,
    )?;
    global.set("__td", td_ns)?;

    ctx.eval::<(), _>(r#"
        globalThis.TextEncoder = class TextEncoder {
            get encoding() { return "utf-8"; }
            encode(input) {
                // Per WHATWG Encoding: encode() returns Uint8Array, not Array.
                // The Rust binding returns a plain array (rquickjs FFI shape);
                // wrap so consumers checking `instanceof Uint8Array` (jose,
                // many crypto libs) accept it.
                const raw = (input === undefined) ? __te.encode() : __te.encode(input);
                return new Uint8Array(raw);
            }
            encodeInto(source, destination) {
                // Per WHATWG Encoding §encodeInto: encode source into existing
                // destination Uint8Array, return {read, written}. Spec
                // mandates UTF-8; we stop early if destination too small.
                const bytes = __te.encode(source);
                const written = Math.min(bytes.length, destination.length);
                for (let i = 0; i < written; i++) destination[i] = bytes[i];
                // 'read' counts source code units that fit; approximate by
                // counting characters whose UTF-8 expansion fits in written.
                let read = 0, used = 0;
                for (let i = 0; i < source.length && used < written; i++) {
                    const c = source.charCodeAt(i);
                    const w = c < 0x80 ? 1 : c < 0x800 ? 2 : (c >= 0xD800 && c <= 0xDBFF) ? 4 : 3;
                    if (used + w > written) break;
                    used += w;
                    read++;
                    if (w === 4) i++; // consume low surrogate
                }
                return { read, written };
            }
        };
        globalThis.TextDecoder = class TextDecoder {
            constructor(label) { this._label = label; }
            get encoding() { return "utf-8"; }
            decode(bytes) {
                // Normalize Uint8Array / Buffer / typed-array views / ArrayBuffer
                // → plain JS array (rquickjs Vec<u8> binding doesn't accept typed
                // arrays directly). ArrayBuffer is non-iterable so Array.from
                // returns empty; wrap in Uint8Array first.
                if (bytes === undefined || bytes === null) {
                    bytes = [];
                } else if (bytes instanceof ArrayBuffer) {
                    bytes = Array.from(new Uint8Array(bytes));
                } else if (bytes && typeof bytes === "object" && !Array.isArray(bytes)) {
                    bytes = Array.from(bytes);
                }
                if (this._label === undefined || this._label === null) {
                    return __td.decode(bytes);
                }
                return __td.decode(bytes, this._label);
            }
        };
    "#)?;
    Ok(())
}

// ─────────────────── Buffer (subset) ─────────────────────────────────

fn wire_buffer<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let buffer = Object::new(ctx.clone())?;
    buffer.set(
        "byteLength",
        Function::new(ctx.clone(), |s: String| -> usize {
            rusty_buffer::Buffer::byte_length(&s, rusty_buffer::Encoding::Utf8)
        })?,
    )?;
    buffer.set(
        "from",
        Function::new(ctx.clone(), |s: String| -> Vec<u8> {
            rusty_buffer::Buffer::from_string(&s, rusty_buffer::Encoding::Utf8)
                .as_bytes()
                .to_vec()
        })?,
    )?;
    buffer.set(
        "alloc",
        Function::new(ctx.clone(), |size: usize| -> Vec<u8> {
            rusty_buffer::Buffer::alloc(size).as_bytes().to_vec()
        })?,
    )?;
    buffer.set(
        "concat",
        Function::new(ctx.clone(), |chunks: Vec<Vec<u8>>| -> Vec<u8> {
            let bufs: Vec<rusty_buffer::Buffer> = chunks
                .into_iter()
                .map(|c| rusty_buffer::Buffer::from_bytes(&c))
                .collect();
            rusty_buffer::Buffer::concat(&bufs, None).as_bytes().to_vec()
        })?,
    )?;
    buffer.set(
        "decodeUtf8",
        Function::new(ctx.clone(), |bytes: Vec<u8>| -> String {
            rusty_buffer::Buffer::from_bytes(&bytes).to_string(
                rusty_buffer::Encoding::Utf8, 0, None,
            )
        })?,
    )?;
    buffer.set(
        "encodeBase64",
        Function::new(ctx.clone(), |bytes: Vec<u8>| -> String {
            rusty_buffer::Buffer::from_bytes(&bytes).to_string(
                rusty_buffer::Encoding::Base64, 0, None,
            )
        })?,
    )?;
    buffer.set(
        "encodeHex",
        Function::new(ctx.clone(), |bytes: Vec<u8>| -> String {
            rusty_buffer::Buffer::from_bytes(&bytes).to_string(
                rusty_buffer::Encoding::Hex, 0, None,
            )
        })?,
    )?;
    global.set("__bufferStatic", buffer)?;
    Ok(())
}

const BUFFER_CLASS_JS: &str = r#"
// Node-Buffer-shaped class wrapping the static helpers wired by wire_buffer.
// Per M8 reconciliation 2026-05-10: makes Buffer Bun-portable for the
// .toString(encoding) instance idiom used in the stream-processor fixture.
//
// Extends Uint8Array so indexed access + .length come for free.
(function() {
    const S = globalThis.__bufferStatic;
    class Buffer extends Uint8Array {
        static from(input, encoding, length) {
            if (typeof input === "string") {
                const enc = (encoding || "utf8").toLowerCase();
                if (enc === "utf8" || enc === "utf-8") {
                    return new Buffer(S.from(input));
                }
                if (enc === "latin1" || enc === "binary" || enc === "ascii") {
                    const buf = new Buffer(input.length);
                    for (let i = 0; i < input.length; i++) buf[i] = input.charCodeAt(i) & 0xff;
                    return buf;
                }
                if (enc === "hex") {
                    const clean = input.replace(/[^0-9a-fA-F]/g, "");
                    const buf = new Buffer(Math.floor(clean.length / 2));
                    for (let i = 0; i < buf.length; i++) buf[i] = parseInt(clean.substr(i*2, 2), 16);
                    return buf;
                }
                if (enc === "base64") {
                    // Browser atob accepts standard base64; tolerate URL-safe.
                    let s = input.replace(/-/g, "+").replace(/_/g, "/").replace(/\s+/g, "");
                    // Pad to multiple of 4.
                    const pad = (4 - (s.length % 4)) % 4;
                    s += "=".repeat(pad);
                    const decoded = atob(s);
                    const buf = new Buffer(decoded.length);
                    for (let i = 0; i < decoded.length; i++) buf[i] = decoded.charCodeAt(i);
                    return buf;
                }
                if (enc === "base64url") {
                    return Buffer.from(input, "base64");
                }
                if (enc === "utf16le" || enc === "ucs2" || enc === "ucs-2") {
                    const buf = new Buffer(input.length * 2);
                    for (let i = 0; i < input.length; i++) {
                        const c = input.charCodeAt(i);
                        buf[i * 2] = c & 0xff;
                        buf[i * 2 + 1] = (c >> 8) & 0xff;
                    }
                    return buf;
                }
                // Unknown encoding — treat as utf8.
                return new Buffer(S.from(input));
            }
            if (Array.isArray(input) || input instanceof Uint8Array || ArrayBuffer.isView(input)) {
                const buf = new Buffer(input.length || input.byteLength || 0);
                buf.set(input);
                return buf;
            }
            if (input && typeof input === "object" && input.type === "Buffer" && Array.isArray(input.data)) {
                // JSON-serialized Buffer ({type:"Buffer", data:[...]}).
                const buf = new Buffer(input.data.length);
                buf.set(input.data);
                return buf;
            }
            throw new TypeError("Buffer.from: unsupported input");
        }
        static alloc(size) { return new Buffer(size); }
        static allocUnsafe(size) { return new Buffer(size); }
        static allocUnsafeSlow(size) { return new Buffer(size); }
        static byteLength(s) { return S.byteLength(s); }
        static isBuffer(v) { return v instanceof Buffer; }
        // Buffer.compare(a, b) — lexicographic compare of two Buffer
        // (or Uint8Array) values. Returns -1/0/1. csv-parse uses this
        // to test escape === quote at parser init.
        static compare(a, b) {
            const len = Math.min(a.length, b.length);
            for (let i = 0; i < len; i++) {
                if (a[i] < b[i]) return -1;
                if (a[i] > b[i]) return 1;
            }
            if (a.length < b.length) return -1;
            if (a.length > b.length) return 1;
            return 0;
        }
        // Preserve the rusty-bun-only static-helper API alongside the
        // Bun-portable instance-method API. Both shapes work.
        static decodeUtf8(bytes) { return S.decodeUtf8(Array.from(bytes)); }
        static encodeBase64(bytes) { return S.encodeBase64(Array.from(bytes)); }
        static encodeHex(bytes) { return S.encodeHex(Array.from(bytes)); }
        static concat(chunks, totalLength) {
            let total = 0;
            for (const c of chunks) total += c.length;
            const out = new Buffer(totalLength !== undefined ? totalLength : total);
            let off = 0;
            for (const c of chunks) {
                if (off >= out.length) break;
                const slice = c.length > out.length - off ? c.subarray(0, out.length - off) : c;
                out.set(slice, off);
                off += slice.length;
            }
            return out;
        }
        toString(encoding, start, end) {
            const view = (start !== undefined || end !== undefined)
                ? this.subarray(start || 0, end !== undefined ? end : this.length)
                : this;
            const arr = Array.from(view);
            const enc = (encoding || "utf8").toLowerCase();
            if (enc === "utf8" || enc === "utf-8") return S.decodeUtf8(arr);
            if (enc === "base64") return S.encodeBase64(arr);
            if (enc === "hex") return S.encodeHex(arr);
            if (enc === "latin1" || enc === "binary") {
                let s = "";
                for (let i = 0; i < arr.length; i++) s += String.fromCharCode(arr[i]);
                return s;
            }
            if (enc === "ascii") {
                let s = "";
                for (let i = 0; i < arr.length; i++) s += String.fromCharCode(arr[i] & 0x7f);
                return s;
            }
            if (enc === "base64url") {
                return S.encodeBase64(arr).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
            }
            if (enc === "utf16le" || enc === "ucs2" || enc === "ucs-2") {
                let s = "";
                for (let i = 0; i + 1 < arr.length; i += 2) {
                    s += String.fromCharCode(arr[i] | (arr[i + 1] << 8));
                }
                return s;
            }
            throw new Error("Unsupported encoding: " + encoding);
        }
        // Buffer.prototype.write(string, [offset], [length], [encoding])
        // Per Node API, writes the string into the buffer starting at
        // offset using the given encoding, returns the number of bytes
        // written. base64url (E.40) and many encoding libs depend on it.
        write(string, ...args) {
            let offset = 0, length = this.length, encoding = "utf8";
            for (const a of args) {
                if (typeof a === "number" && offset === 0) offset = a;
                else if (typeof a === "number") length = a;
                else if (typeof a === "string") encoding = a;
            }
            const enc = (encoding || "utf8").toLowerCase();
            let bytes;
            if (enc === "utf8" || enc === "utf-8") {
                bytes = new TextEncoder().encode(String(string));
            } else if (enc === "latin1" || enc === "binary" || enc === "ascii") {
                bytes = new Uint8Array(String(string).length);
                for (let i = 0; i < bytes.length; i++) bytes[i] = String(string).charCodeAt(i) & 0xff;
            } else if (enc === "hex") {
                const s = String(string);
                bytes = new Uint8Array(Math.floor(s.length / 2));
                for (let i = 0; i < bytes.length; i++) bytes[i] = parseInt(s.substr(i*2, 2), 16);
            } else if (enc === "base64") {
                const decoded = atob(String(string).replace(/-/g, "+").replace(/_/g, "/"));
                bytes = new Uint8Array(decoded.length);
                for (let i = 0; i < decoded.length; i++) bytes[i] = decoded.charCodeAt(i);
            } else {
                bytes = new TextEncoder().encode(String(string));
            }
            const writeLen = Math.min(bytes.length, length, this.length - offset);
            for (let i = 0; i < writeLen; i++) this[offset + i] = bytes[i];
            return writeLen;
        }
        // Buffer.prototype.fill(value, [offset], [end], [encoding])
        fill(value, offset, end) {
            offset = offset || 0;
            end = end !== undefined ? end : this.length;
            if (typeof value === "string") {
                const tmp = Buffer.from(value);
                for (let i = offset; i < end; i++) this[i] = tmp[(i - offset) % tmp.length];
            } else {
                const v = (Number(value) || 0) & 0xff;
                for (let i = offset; i < end; i++) this[i] = v;
            }
            return this;
        }
        // Buffer.prototype.slice(start, end) — preserves Buffer-ness
        // (subarray returns Uint8Array; consumers that test isBuffer
        // need the result to be Buffer too).
        slice(start, end) {
            const u = this.subarray(start, end);
            return Buffer.from(u);
        }
        // Buffer.prototype.copy(target, targetStart, sourceStart, sourceEnd)
        copy(target, targetStart, sourceStart, sourceEnd) {
            targetStart = targetStart || 0;
            sourceStart = sourceStart || 0;
            sourceEnd = sourceEnd !== undefined ? sourceEnd : this.length;
            let written = 0;
            for (let i = sourceStart; i < sourceEnd && targetStart + written < target.length; i++) {
                target[targetStart + written] = this[i];
                written++;
            }
            return written;
        }
        // indexOf / includes (string or buffer search)
        indexOf(value, byteOffset) {
            byteOffset = byteOffset || 0;
            const needle = (typeof value === "string") ? Buffer.from(value)
                : (value instanceof Uint8Array ? value : Buffer.from([value & 0xff]));
            outer: for (let i = byteOffset; i <= this.length - needle.length; i++) {
                for (let j = 0; j < needle.length; j++) {
                    if (this[i + j] !== needle[j]) continue outer;
                }
                return i;
            }
            return -1;
        }
        includes(value, byteOffset) { return this.indexOf(value, byteOffset) !== -1; }
        // Node Buffer numeric readers — ulid + many bincode-shape libs
        // rely on these. Defaults to offset 0; throws on out-of-range
        // per Node convention (unless noAssert is true, deprecated).
        readUInt8(offset = 0) {
            if (offset >= this.length) throw new RangeError("readUInt8: offset out of range");
            return this[offset];
        }
        readInt8(offset = 0) {
            if (offset >= this.length) throw new RangeError("readInt8: offset out of range");
            const v = this[offset];
            return v >= 0x80 ? v - 0x100 : v;
        }
        readUInt16LE(offset = 0) {
            return this[offset] | (this[offset + 1] << 8);
        }
        readUInt16BE(offset = 0) {
            return (this[offset] << 8) | this[offset + 1];
        }
        readUInt32LE(offset = 0) {
            return (this[offset] | (this[offset + 1] << 8) |
                    (this[offset + 2] << 16) | (this[offset + 3] << 24)) >>> 0;
        }
        readUInt32BE(offset = 0) {
            return ((this[offset] << 24) | (this[offset + 1] << 16) |
                    (this[offset + 2] << 8) | this[offset + 3]) >>> 0;
        }
        writeUInt8(value, offset = 0) {
            this[offset] = value & 0xff;
            return offset + 1;
        }
        readInt32LE(offset = 0) {
            const u = (this[offset] | (this[offset + 1] << 8) |
                       (this[offset + 2] << 16) | (this[offset + 3] << 24)) >>> 0;
            return u >= 0x80000000 ? u - 0x100000000 : u;
        }
        readInt16LE(offset = 0) {
            const u = this[offset] | (this[offset + 1] << 8);
            return u >= 0x8000 ? u - 0x10000 : u;
        }
        writeUInt16LE(value, offset = 0) {
            this[offset] = value & 0xff;
            this[offset + 1] = (value >>> 8) & 0xff;
            return offset + 2;
        }
        writeUInt32LE(value, offset = 0) {
            this[offset] = value & 0xff;
            this[offset + 1] = (value >>> 8) & 0xff;
            this[offset + 2] = (value >>> 16) & 0xff;
            this[offset + 3] = (value >>> 24) & 0xff;
            return offset + 4;
        }
        writeUInt16BE(value, offset = 0) {
            this[offset] = (value >>> 8) & 0xff;
            this[offset + 1] = value & 0xff;
            return offset + 2;
        }
        writeUInt32BE(value, offset = 0) {
            this[offset] = (value >>> 24) & 0xff;
            this[offset + 1] = (value >>> 16) & 0xff;
            this[offset + 2] = (value >>> 8) & 0xff;
            this[offset + 3] = value & 0xff;
            return offset + 4;
        }
        readBigUInt64LE(offset = 0) {
            const lo = BigInt(this.readUInt32LE(offset));
            const hi = BigInt(this.readUInt32LE(offset + 4));
            return lo | (hi << 32n);
        }
        readBigUInt64BE(offset = 0) {
            const hi = BigInt(this.readUInt32BE(offset));
            const lo = BigInt(this.readUInt32BE(offset + 4));
            return (hi << 32n) | lo;
        }
        readDoubleLE(offset = 0) {
            const buf = new ArrayBuffer(8);
            const u8 = new Uint8Array(buf);
            for (let i = 0; i < 8; i++) u8[i] = this[offset + i];
            return new DataView(buf).getFloat64(0, true);
        }
        readDoubleBE(offset = 0) {
            const buf = new ArrayBuffer(8);
            const u8 = new Uint8Array(buf);
            for (let i = 0; i < 8; i++) u8[i] = this[offset + i];
            return new DataView(buf).getFloat64(0, false);
        }
        readFloatLE(offset = 0) {
            const buf = new ArrayBuffer(4);
            const u8 = new Uint8Array(buf);
            for (let i = 0; i < 4; i++) u8[i] = this[offset + i];
            return new DataView(buf).getFloat32(0, true);
        }
        writeDoubleLE(value, offset = 0) {
            const buf = new ArrayBuffer(8);
            new DataView(buf).setFloat64(0, value, true);
            const u8 = new Uint8Array(buf);
            for (let i = 0; i < 8; i++) this[offset + i] = u8[i];
            return offset + 8;
        }
        writeFloatLE(value, offset = 0) {
            const buf = new ArrayBuffer(4);
            new DataView(buf).setFloat32(0, value, true);
            const u8 = new Uint8Array(buf);
            for (let i = 0; i < 4; i++) this[offset + i] = u8[i];
            return offset + 4;
        }
        compare(target, targetStart, targetEnd, sourceStart, sourceEnd) {
            const me = this.subarray(sourceStart || 0,
                sourceEnd !== undefined ? sourceEnd : this.length);
            const them = target.subarray(targetStart || 0,
                targetEnd !== undefined ? targetEnd : target.length);
            const n = Math.min(me.length, them.length);
            for (let i = 0; i < n; i++) {
                if (me[i] < them[i]) return -1;
                if (me[i] > them[i]) return 1;
            }
            if (me.length < them.length) return -1;
            if (me.length > them.length) return 1;
            return 0;
        }
        toJSON() {
            const data = new Array(this.length);
            for (let i = 0; i < this.length; i++) data[i] = this[i];
            return { type: "Buffer", data };
        }
        equals(other) {
            if (this.length !== other.length) return false;
            for (let i = 0; i < this.length; i++) if (this[i] !== other[i]) return false;
            return true;
        }
    }
    // Make all static methods own enumerable properties so legacy
    // copy-with-for-in patterns (safer-buffer iterates `for (key in Buffer)`
    // to clone the static surface) pick them up. ES class statics are
    // non-enumerable by default.
    //
    // ALSO: replace static-method bindings with regular functions, because
    // some libraries do `new Buffer.allocUnsafeSlow(size)` (cbor-x) — a
    // class static method is not constructable, but a plain function is.
    // Plain functions retain the same call semantics.
    for (const name of Object.getOwnPropertyNames(Buffer)) {
        if (name === "length" || name === "name" || name === "prototype") continue;
        const desc = Object.getOwnPropertyDescriptor(Buffer, name);
        if (!desc) continue;
        if (typeof desc.value === "function") {
            const fn = desc.value;
            // Rebind as a plain function. ES classes mark static methods as
            // non-constructable; copying the body into a regular function
            // restores the legacy Buffer-as-function call semantics.
            const wrap = function (...args) { return fn.apply(Buffer, args); };
            Object.defineProperty(wrap, "name", { value: name });
            Object.defineProperty(Buffer, name, {
                value: wrap, writable: true, enumerable: true, configurable: true,
            });
        } else {
            Object.defineProperty(Buffer, name, { ...desc, enumerable: true });
        }
    }

    // Buffer can be called without `new` in legacy Node code
    // (safer-buffer's Safer.from fallback does `return Buffer(value, enc)`).
    // Wrap as a Proxy so both new Buffer(...) and Buffer(...) work; the
    // apply trap routes to the same dispatch logic the legacy Buffer(arg,
    // enc, len) signature used (Buffer.from for non-number, Buffer.alloc
    // for number).
    const BufferCallable = new Proxy(Buffer, {
        apply(_target, _thisArg, args) {
            const [value, encodingOrOffset, length] = args;
            if (typeof value === "number") {
                return Buffer.alloc(value);
            }
            return Buffer.from(value, encodingOrOffset, length);
        },
        construct(target, args, newTarget) {
            return Reflect.construct(target, args, newTarget);
        },
    });
    globalThis.Buffer = BufferCallable;
})();
"#;

fn install_buffer_class_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(BUFFER_CLASS_JS)?;
    Ok(())
}

// ─── Set.prototype ES2025 set-methods polyfill (closes E.10) ──────────
//
// Bun has these natively via JSC. rusty-bun-host's embedded QuickJS
// predates the TC39 set-methods Stage 4 merge. The polyfill is ~30
// LOC JS and brings the rusty-bun-host basin into alignment with Bun
// on the ES2025 Set algebra surface.

const SET_METHODS_POLYFILL_JS: &str = r#"
(function() {
    function toSetLike(other) {
        // Per spec, accept anything with .has(), .keys() iterator, and .size.
        // Simplification: accept Sets and iterables (collect into a fresh Set).
        if (other instanceof Set) return other;
        return new Set(other);
    }
    if (typeof Set.prototype.union !== "function") {
        Object.defineProperty(Set.prototype, "union", {
            value: function (other) {
                const o = toSetLike(other);
                const out = new Set(this);
                for (const v of o) out.add(v);
                return out;
            },
            writable: true, configurable: true,
        });
    }
    if (typeof Set.prototype.intersection !== "function") {
        Object.defineProperty(Set.prototype, "intersection", {
            value: function (other) {
                const o = toSetLike(other);
                const out = new Set();
                for (const v of this) if (o.has(v)) out.add(v);
                return out;
            },
            writable: true, configurable: true,
        });
    }
    if (typeof Set.prototype.difference !== "function") {
        Object.defineProperty(Set.prototype, "difference", {
            value: function (other) {
                const o = toSetLike(other);
                const out = new Set();
                for (const v of this) if (!o.has(v)) out.add(v);
                return out;
            },
            writable: true, configurable: true,
        });
    }
    if (typeof Set.prototype.symmetricDifference !== "function") {
        Object.defineProperty(Set.prototype, "symmetricDifference", {
            value: function (other) {
                const o = toSetLike(other);
                const out = new Set();
                for (const v of this) if (!o.has(v)) out.add(v);
                for (const v of o) if (!this.has(v)) out.add(v);
                return out;
            },
            writable: true, configurable: true,
        });
    }
    if (typeof Set.prototype.isSubsetOf !== "function") {
        Object.defineProperty(Set.prototype, "isSubsetOf", {
            value: function (other) {
                const o = toSetLike(other);
                for (const v of this) if (!o.has(v)) return false;
                return true;
            },
            writable: true, configurable: true,
        });
    }
    if (typeof Set.prototype.isSupersetOf !== "function") {
        Object.defineProperty(Set.prototype, "isSupersetOf", {
            value: function (other) {
                const o = toSetLike(other);
                for (const v of o) if (!this.has(v)) return false;
                return true;
            },
            writable: true, configurable: true,
        });
    }
    if (typeof Set.prototype.isDisjointFrom !== "function") {
        Object.defineProperty(Set.prototype, "isDisjointFrom", {
            value: function (other) {
                const o = toSetLike(other);
                for (const v of this) if (o.has(v)) return false;
                return true;
            },
            writable: true, configurable: true,
        });
    }
})();
"#;

fn install_set_methods_polyfill<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(SET_METHODS_POLYFILL_JS)?;
    Ok(())
}

// FinalizationRegistry + WeakRef stubs. QuickJS lacks GC-finalization
// hooks, so we provide structural shims: register/unregister are no-ops,
// the cleanup callback is never invoked (acceptable for libs that use
// these for opportunistic cleanup — p-throttle, undici, etc.). WeakRef
// stores a strong reference; deref always returns the held target. This
// trades the GC-aware behavior for API surface compat.
const FINALIZATION_REGISTRY_STUB_JS: &str = r#"
(function () {
    if (typeof globalThis.FinalizationRegistry === "function") return;
    class FinalizationRegistry {
        constructor(cleanupCallback) {
            this._cb = cleanupCallback;
        }
        register(_target, _heldValue, _unregisterToken) {}
        unregister(_unregisterToken) {}
    }
    globalThis.FinalizationRegistry = FinalizationRegistry;
})();
(function () {
    if (typeof globalThis.WeakRef === "function") return;
    class WeakRef {
        constructor(target) {
            if (target == null || (typeof target !== "object" && typeof target !== "function")) {
                throw new TypeError("WeakRef target must be an object");
            }
            this._t = target;
        }
        deref() { return this._t; }
    }
    globalThis.WeakRef = WeakRef;
})();
(function () {
    // MessagePort + MessageChannel + BroadcastChannel stubs. Worker-thread
    // and worker-API related; rusty-bun is single-thread so the messaging
    // is no-op but the classes need to exist for transitive deps that
    // top-level-check `typeof MessagePort` or call new MessageChannel().
    if (typeof globalThis.MessagePort !== "function") {
        class MessagePort {
            constructor() { this._listeners = new Map(); this._other = null; }
            postMessage(data) {
                if (!this._other) return;
                const arr = this._other._listeners.get("message");
                if (arr) {
                    for (const cb of arr) {
                        try { queueMicrotask(() => cb({ data })); } catch (_) {}
                    }
                }
            }
            addEventListener(type, cb) {
                if (!this._listeners.has(type)) this._listeners.set(type, []);
                this._listeners.get(type).push(cb);
            }
            removeEventListener(type, cb) {
                const arr = this._listeners.get(type);
                if (arr) this._listeners.set(type, arr.filter(l => l !== cb));
            }
            start() {}
            close() {}
            get onmessage() { return this._om; }
            set onmessage(fn) { this._om = fn; if (fn) this.addEventListener("message", fn); }
        }
        globalThis.MessagePort = MessagePort;
    }
    if (typeof globalThis.MessageChannel !== "function") {
        class MessageChannel {
            constructor() {
                this.port1 = new globalThis.MessagePort();
                this.port2 = new globalThis.MessagePort();
                this.port1._other = this.port2;
                this.port2._other = this.port1;
            }
        }
        globalThis.MessageChannel = MessageChannel;
    }
    if (typeof globalThis.EventTarget !== "function") {
        class EventTarget {
            constructor() { this.__listeners = new Map(); }
            addEventListener(type, cb, opts) {
                if (typeof cb !== "function" && !(cb && typeof cb.handleEvent === "function")) return;
                if (!this.__listeners.has(type)) this.__listeners.set(type, []);
                this.__listeners.get(type).push({ cb, once: !!(opts && opts.once) });
            }
            removeEventListener(type, cb) {
                const arr = this.__listeners.get(type);
                if (!arr) return;
                this.__listeners.set(type, arr.filter(l => l.cb !== cb));
            }
            dispatchEvent(event) {
                const arr = this.__listeners.get(event.type);
                if (!arr) return true;
                const snapshot = arr.slice();
                this.__listeners.set(event.type, arr.filter(l => !l.once));
                for (const l of snapshot) {
                    try {
                        if (typeof l.cb === "function") l.cb.call(this, event);
                        else if (l.cb && typeof l.cb.handleEvent === "function") l.cb.handleEvent(event);
                    } catch (_) {}
                }
                return !event.defaultPrevented;
            }
        }
        globalThis.EventTarget = EventTarget;
    }
    if (typeof globalThis.Event !== "function") {
        class Event {
            constructor(type, init) {
                this.type = String(type);
                init = init || {};
                this.bubbles = !!init.bubbles;
                this.cancelable = !!init.cancelable;
                this.defaultPrevented = false;
                this.target = null;
                this.currentTarget = null;
                this.timeStamp = Date.now();
            }
            preventDefault() { if (this.cancelable) this.defaultPrevented = true; }
            stopPropagation() {}
            stopImmediatePropagation() {}
        }
        globalThis.Event = Event;
        class CustomEvent extends Event {
            constructor(type, init) { super(type, init); this.detail = (init && init.detail); }
        }
        globalThis.CustomEvent = CustomEvent;
    }
    if (typeof globalThis.BroadcastChannel !== "function") {
        class BroadcastChannel {
            constructor(name) { this.name = name; this._listeners = new Map(); }
            postMessage(_data) {}
            addEventListener(type, cb) {
                if (!this._listeners.has(type)) this._listeners.set(type, []);
                this._listeners.get(type).push(cb);
            }
            removeEventListener(type, cb) {
                const arr = this._listeners.get(type);
                if (arr) this._listeners.set(type, arr.filter(l => l !== cb));
            }
            close() {}
        }
        globalThis.BroadcastChannel = BroadcastChannel;
    }
})();
"#;

fn install_finalization_registry_stub<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(FINALIZATION_REGISTRY_STUB_JS)?;
    Ok(())
}

// ─────────────────── URLSearchParams ─────────────────────────────────
//
// QuickJS does not GC-track Rust-side Rc<RefCell> captures held by JS-wrapped
// closures, so the previous instance-per-class approach (cycle-prone)
// triggered a GC assertion at runtime drop. Pattern used here: stateless
// Rust functions operating on plain JS-array pairs, with a JS-side class
// installed in globalThis that calls into them. No Rust-captured state;
// the JS class holds its own pairs array.

fn wire_url_search_params_static<'js>(
    ctx: &rquickjs::Ctx<'js>, global: &Object<'js>,
) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "parse",
        Function::new(ctx.clone(), |query: String| -> Vec<Vec<String>> {
            let p = rusty_urlsearchparams::URLSearchParams::from_query(&query);
            p.entries().map(|(n, v)| vec![n.to_string(), v.to_string()]).collect()
        })?,
    )?;
    ns.set(
        "serialize",
        Function::new(ctx.clone(), |pairs: Vec<Vec<String>>| -> String {
            let pair_refs: Vec<(&str, &str)> = pairs
                .iter()
                .filter_map(|p| if p.len() >= 2 { Some((p[0].as_str(), p[1].as_str())) } else { None })
                .collect();
            let urlsp = rusty_urlsearchparams::URLSearchParams::from_pairs(&pair_refs);
            urlsp.to_string()
        })?,
    )?;
    ns.set(
        "sort",
        Function::new(ctx.clone(), |pairs: Vec<Vec<String>>| -> Vec<Vec<String>> {
            let pair_refs: Vec<(&str, &str)> = pairs
                .iter()
                .filter_map(|p| if p.len() >= 2 { Some((p[0].as_str(), p[1].as_str())) } else { None })
                .collect();
            let mut urlsp = rusty_urlsearchparams::URLSearchParams::from_pairs(&pair_refs);
            urlsp.sort();
            urlsp.entries().map(|(n, v)| vec![n.to_string(), v.to_string()]).collect()
        })?,
    )?;
    global.set("__urlsp", ns)?;
    Ok(())
}

const URL_SEARCH_PARAMS_CLASS_JS: &str = r#"
globalThis.URLSearchParams = class URLSearchParams {
    constructor(init) {
        if (typeof init === "string") {
            this._pairs = __urlsp.parse(init);
        } else if (Array.isArray(init)) {
            this._pairs = init.map(p => [String(p[0]), String(p[1])]);
        } else if (init && typeof init === "object") {
            this._pairs = Object.entries(init).map(([k, v]) => [String(k), String(v)]);
        } else {
            this._pairs = [];
        }
    }
    append(name, value) {
        this._pairs.push([String(name), String(value)]);
    }
    delete(name) {
        const lookFor = String(name);
        this._pairs = this._pairs.filter(p => p[0] !== lookFor);
    }
    get(name) {
        const lookFor = String(name);
        const pair = this._pairs.find(p => p[0] === lookFor);
        return pair ? pair[1] : null;
    }
    getAll(name) {
        const lookFor = String(name);
        return this._pairs.filter(p => p[0] === lookFor).map(p => p[1]);
    }
    has(name) {
        const lookFor = String(name);
        return this._pairs.some(p => p[0] === lookFor);
    }
    set(name, value) {
        const lookFor = String(name);
        const newPairs = [];
        let placed = false;
        for (const p of this._pairs) {
            if (p[0] === lookFor) {
                if (!placed) { newPairs.push([lookFor, String(value)]); placed = true; }
            } else {
                newPairs.push(p);
            }
        }
        if (!placed) newPairs.push([lookFor, String(value)]);
        this._pairs = newPairs;
    }
    sort() {
        this._pairs = __urlsp.sort(this._pairs);
    }
    toString() {
        return __urlsp.serialize(this._pairs);
    }
    get size() { return this._pairs.length; }
    entries() { return this._pairs[Symbol.iterator](); }
    keys()    { return this._pairs.map(p => p[0])[Symbol.iterator](); }
    values()  { return this._pairs.map(p => p[1])[Symbol.iterator](); }
    forEach(cb) { for (const [k, v] of this._pairs) cb(v, k, this); }
    [Symbol.iterator]() { return this.entries(); }
};
"#;

fn install_url_search_params_class_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(URL_SEARCH_PARAMS_CLASS_JS)?;
    Ok(())
}

// ─────────────────── fs (sync subset) ────────────────────────────────

fn wire_fs<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let fs = Object::new(ctx.clone())?;
    fs.set(
        "readFileSync",
        Function::new(ctx.clone(), |path: String, encoding: Opt<String>| -> JsResult<Value<'js>> {
            // Without encoding: returns bytes as Vec<u8> (will surface as JS array).
            // With "utf-8": returns String.
            let bytes = rusty_node_fs::read_file_sync(&path).map_err(|e| {
                rquickjs::Error::new_from_js_message("readFileSync", "Buffer/string",
                    format!("{}", e))
            })?;
            // We can't easily return Value polymorphically without a Ctx
            // here; use a separate function for string-encoding.
            let _ = encoding;
            // For simplicity, return as Vec<u8>. Tests can use
            // readFileSyncUtf8 for the string variant.
            let _bytes_for_value = bytes;
            // Workaround: this branch isn't reachable in tests; we provide
            // readFileSyncUtf8 separately below.
            Err(rquickjs::Error::new_from_js_message(
                "readFileSync", "Buffer/string",
                "use readFileSyncUtf8 for string output, readFileSyncBytes for raw bytes",
            ))
        })?,
    )?;
    fs.set(
        "readFileSyncUtf8",
        Function::new(ctx.clone(), |path: String| -> JsResult<String> {
            rusty_node_fs::read_file_string_sync(&path).map_err(|e| {
                rquickjs::Error::new_from_js_message("readFileSyncUtf8", "string", format!("{}", e))
            })
        })?,
    )?;
    fs.set(
        "readFileSyncBytes",
        Function::new(ctx.clone(), |path: String| -> JsResult<Vec<u8>> {
            rusty_node_fs::read_file_sync(&path).map_err(|e| {
                rquickjs::Error::new_from_js_message("readFileSyncBytes", "Vec<u8>", format!("{}", e))
            })
        })?,
    )?;
    fs.set(
        "writeFileSync",
        Function::new(ctx.clone(), |path: String, data: String| -> JsResult<()> {
            rusty_node_fs::write_file_string_sync(&path, &data).map_err(|e| {
                rquickjs::Error::new_from_js_message("writeFileSync", "()", format!("{}", e))
            })
        })?,
    )?;
    fs.set(
        "appendFileSync",
        Function::new(ctx.clone(), |path: String, data: String| -> JsResult<()> {
            let prior = rusty_node_fs::read_file_string_sync(&path).unwrap_or_default();
            let combined = prior + &data;
            rusty_node_fs::write_file_string_sync(&path, &combined).map_err(|e| {
                rquickjs::Error::new_from_js_message("appendFileSync", "()", format!("{}", e))
            })
        })?,
    )?;
    fs.set(
        "rmSync",
        Function::new(ctx.clone(), |path: String, _opts: Opt<rquickjs::Value>| -> JsResult<()> {
            // Default to {force:true,recursive:true} semantics — fse and
            // many consumers expect rmSync to be idempotent on missing
            // paths. The opts argument is rquickjs Value (cant introspect
            // easily); accept the permissive default which matches Node
            // when force:true is set.
            let _ = rusty_node_fs::rm_sync_recursive(&path);
            let _ = std::fs::remove_file(&path);
            Ok(())
        })?,
    )?;
    fs.set(
        "copyFileSync",
        Function::new(ctx.clone(), |src: String, dest: String| -> JsResult<()> {
            std::fs::copy(&src, &dest).map(|_| ()).map_err(|e| {
                rquickjs::Error::new_from_js_message("copyFileSync", "()", e.to_string())
            })
        })?,
    )?;
    fs.set(
        "renameSync",
        Function::new(ctx.clone(), |src: String, dest: String| -> JsResult<()> {
            std::fs::rename(&src, &dest).map_err(|e| {
                rquickjs::Error::new_from_js_message("renameSync", "()", e.to_string())
            })
        })?,
    )?;
    fs.set(
        "chmodSync",
        Function::new(ctx.clone(), |path: String, mode: f64| -> JsResult<()> {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perm = std::fs::Permissions::from_mode(mode as u32);
                std::fs::set_permissions(&path, perm).map_err(|e| {
                    rquickjs::Error::new_from_js_message("chmodSync", "()", e.to_string())
                })
            }
            #[cfg(not(unix))] { let _ = (path, mode); Ok(()) }
        })?,
    )?;
    fs.set(
        "utimesSync",
        Function::new(ctx.clone(), |_path: String, _atime: f64, _mtime: f64| -> JsResult<()> {
            // Best-effort no-op (Rust std doesnt expose setting both
            // atime/mtime via stable API without filetime crate).
            Ok(())
        })?,
    )?;
    // Stream + file-descriptor surface — stub class + functions.
    // Many libs (execa, fs-extra-shim, etc.) typeof-check these for
    // capability detection without actually invoking them.
    ctx.eval::<(), _>(r#"
        (function() {
            if (typeof globalThis.fs === "undefined") return;
            class FsReadStream {
                constructor(path, opts) { this.path = path; this.opts = opts || {}; }
                pipe(dst) { return dst; }
                on() { return this; } off() { return this; } once() { return this; }
                emit() { return false; }
                close(cb) { if (cb) queueMicrotask(cb); }
                destroy() { return this; }
            }
            class FsWriteStream extends FsReadStream {
                write() { return true; } end() { return this; }
            }
            globalThis.fs.createReadStream = function(path, opts) { return new FsReadStream(path, opts); };
            globalThis.fs.createWriteStream = function(path, opts) { return new FsWriteStream(path, opts); };
            globalThis.fs.openSync = function(path, _flag, _mode) {
                // Return a fake file descriptor; we dont track fds.
                return 3;
            };
            globalThis.fs.closeSync = function(_fd) {};
            globalThis.fs.writeSync = function(_fd, buf) { return buf && buf.length || 0; };
            globalThis.fs.readSync = function(_fd, _buf, _offset, _len, _pos) { return 0; };
            globalThis.fs.fstatSync = function(_fd) {
                return { isFile: () => true, isDirectory: () => false, size: 0, mode: 0o644, mtimeMs: 0, atimeMs: 0, ctimeMs: 0 };
            };
            globalThis.fs.ftruncateSync = function() {};
            globalThis.fs.fsyncSync = function() {};
            globalThis.fs.watch = function(_path, _opts, _listener) {
                return { close() {}, on() { return this; }, ref() {}, unref() {} };
            };
            globalThis.fs.watchFile = function() {};
            globalThis.fs.unwatchFile = function() {};
            globalThis.fs.constants = {
                F_OK: 0, R_OK: 4, W_OK: 2, X_OK: 1,
                O_RDONLY: 0, O_WRONLY: 1, O_RDWR: 2,
                O_CREAT: 64, O_EXCL: 128, O_TRUNC: 512, O_APPEND: 1024,
                S_IFMT: 0o170000, S_IFREG: 0o100000, S_IFDIR: 0o40000,
                S_IFLNK: 0o120000, COPYFILE_EXCL: 1,
            };
            // F_OK/R_OK/W_OK/X_OK + ReadStream/WriteStream on the namespace
            // for libs that read them off fs.* directly.
            globalThis.fs.F_OK = 0; globalThis.fs.R_OK = 4;
            globalThis.fs.W_OK = 2; globalThis.fs.X_OK = 1;
            globalThis.fs.ReadStream = FsReadStream;
            globalThis.fs.WriteStream = FsWriteStream;
        })();
    "#)?;
    fs.set(
        "existsSync",
        Function::new(ctx.clone(), |path: String| -> bool {
            rusty_node_fs::exists_sync(&path)
        })?,
    )?;
    fs.set(
        "isFileSync",
        Function::new(ctx.clone(), |path: String| -> bool {
            std::path::Path::new(&path).is_file()
        })?,
    )?;
    fs.set(
        "isDirectorySync",
        Function::new(ctx.clone(), |path: String| -> bool {
            std::path::Path::new(&path).is_dir()
        })?,
    )?;
    // listDirectorySync — return entry names in a directory. Surface
    // glob/path-scurry's readdirSync depends on. Errors map to JS.
    fs.set(
        "listDirectorySync",
        Function::new(ctx.clone(), |path: String| -> JsResult<Vec<String>> {
            std::fs::read_dir(&path)
                .map_err(|e| rquickjs::Error::new_from_js_message(
                    "listDirectorySync", "()", format!("{}", e)))
                .map(|iter| iter.filter_map(|e| e.ok().and_then(|d|
                    d.file_name().into_string().ok()
                )).collect())
        })?,
    )?;
    // fileSizeSync — for stat shape.
    fs.set(
        "fileSizeSync",
        Function::new(ctx.clone(), |path: String| -> i64 {
            std::fs::metadata(&path).map(|m| m.len() as i64).unwrap_or(0)
        })?,
    )?;
    fs.set(
        "unlinkSync",
        Function::new(ctx.clone(), |path: String| -> JsResult<()> {
            rusty_node_fs::unlink_sync(&path).map_err(|e| {
                rquickjs::Error::new_from_js_message("unlinkSync", "()", format!("{}", e))
            })
        })?,
    )?;
    fs.set(
        "mkdirSyncRecursive",
        Function::new(ctx.clone(), |path: String| -> JsResult<()> {
            rusty_node_fs::mkdir_sync(&path, true).map_err(|e| {
                rquickjs::Error::new_from_js_message("mkdirSync", "()", format!("{}", e))
            })
        })?,
    )?;
    fs.set(
        "rmdirSyncRecursive",
        Function::new(ctx.clone(), |path: String| -> JsResult<()> {
            rusty_node_fs::rm_sync_recursive(&path).map_err(|e| {
                rquickjs::Error::new_from_js_message("rmSync", "()", format!("{}", e))
            })
        })?,
    )?;
    // Node-spec short aliases: fs.mkdirSync(p, opts) / fs.rmdirSync(p, opts).
    // Default is non-recursive; the {recursive:true} option routes to the
    // existing recursive helper. opts ignored for the bare-path case.
    fs.set(
        "mkdirSync",
        Function::new(ctx.clone(), |path: String, _opts: Opt<rquickjs::Value>| -> JsResult<()> {
            rusty_node_fs::mkdir_sync(&path, true).map_err(|e| {
                rquickjs::Error::new_from_js_message("mkdirSync", "()", format!("{}", e))
            })
        })?,
    )?;
    fs.set(
        "rmdirSync",
        Function::new(ctx.clone(), |path: String, _opts: Opt<rquickjs::Value>| -> JsResult<()> {
            rusty_node_fs::rm_sync_recursive(&path).map_err(|e| {
                rquickjs::Error::new_from_js_message("rmdirSync", "()", format!("{}", e))
            })
        })?,
    )?;
    global.set("fs", fs)?;
    // Node/Bun-portable readFileSync(path, encoding|options) override.
    // The Rust binding for readFileSync errors out telling you to use
    // readFileSyncUtf8/Bytes; this JS layer dispatches based on encoding
    // so consumer code using the standard Node shape just works.
    ctx.eval::<(), _>(r#"
        (function() {
            const orig = globalThis.fs;
            orig.readFileSync = function readFileSync(path, options) {
                let encoding;
                if (typeof options === "string") encoding = options;
                else if (options && typeof options === "object") encoding = options.encoding;
                if (encoding === "utf8" || encoding === "utf-8") {
                    return orig.readFileSyncUtf8(path);
                }
                if (encoding === undefined || encoding === null) {
                    // Node default: return Buffer (raw bytes).
                    const bytes = orig.readFileSyncBytes(path);
                    return typeof Buffer !== "undefined" ? Buffer.from(bytes) : bytes;
                }
                throw new Error("readFileSync: unsupported encoding " + encoding);
            };

            // S9 surface widening (E.20 glob): stat / lstat / readdir
            // sync variants. These compose on the existing
            // isFileSync / isDirectorySync / existsSync surface, with
            // best-effort stat shape mimicking Node's Stats object.
            function makeStats(path) {
                const isFile = orig.isFileSync && orig.isFileSync(path);
                const isDir = orig.isDirectorySync && orig.isDirectorySync(path);
                return {
                    isFile: () => isFile,
                    isDirectory: () => isDir,
                    isSymbolicLink: () => false,
                    isBlockDevice: () => false,
                    isCharacterDevice: () => false,
                    isFIFO: () => false,
                    isSocket: () => false,
                    size: isFile && orig.fileSizeSync ? orig.fileSizeSync(path) : 0,
                    mode: 0o644,
                    mtime: new Date(0),
                    ctime: new Date(0),
                    atime: new Date(0),
                    birthtime: new Date(0),
                    mtimeMs: 0, ctimeMs: 0, atimeMs: 0, birthtimeMs: 0,
                    dev: 0, ino: 0, nlink: 1, uid: 0, gid: 0, rdev: 0, blksize: 4096, blocks: 0,
                };
            }
            orig.statSync = function (path, _opts) {
                if (!orig.existsSync(path)) {
                    const e = new Error("ENOENT: no such file or directory, stat '" + path + "'");
                    e.code = "ENOENT";
                    throw e;
                }
                return makeStats(path);
            };
            orig.lstatSync = orig.statSync;
            // readdirSync: depend on listDirectorySync if exposed by the
            // Rust binding; otherwise throw with a clear message.
            orig.readdirSync = function (path, opts) {
                if (orig.listDirectorySync) {
                    const names = orig.listDirectorySync(path);
                    if (opts && opts.withFileTypes) {
                        return names.map(name => ({
                            name,
                            isFile: () => orig.isFileSync(path + "/" + name),
                            isDirectory: () => orig.isDirectorySync(path + "/" + name),
                            isSymbolicLink: () => false,
                        }));
                    }
                    return names;
                }
                throw new Error("fs.readdirSync: not yet implemented (no Rust binding)");
            };
            orig.realpathSync = function (path) { return path; };
            orig.realpathSync.native = orig.realpathSync;
            orig.readlinkSync = function (path) {
                throw new Error("fs.readlinkSync: not supported");
            };
            // fs.promises namespace — proxy through to Promise-wrapped sync.
            orig.promises = orig.promises || {
                readFile: async (p, opts) => orig.readFileSync(p, opts),
                writeFile: async (p, data, opts) => orig.writeFileSync(p, data, opts),
                stat: async p => orig.statSync(p),
                lstat: async p => orig.lstatSync(p),
                readdir: async (p, opts) => orig.readdirSync(p, opts),
                mkdir: async (p, opts) => orig.mkdirSyncRecursive ? orig.mkdirSyncRecursive(p) : null,
                rm: async (p, opts) => orig.unlinkSync ? orig.unlinkSync(p) : null,
                unlink: async p => orig.unlinkSync(p),
                access: async p => { if (!orig.existsSync(p)) throw new Error("ENOENT: " + p); },
                realpath: async p => p,
                readlink: async () => { throw new Error("readlink not supported"); },
            };

            // Async callback variants. Node's fs.X(path, cb) shape.
            // Promise-style wrappers compose via util.promisify.
            const cbify = (syncFn) => function (...args) {
                const cb = args.pop();
                try {
                    const r = syncFn.apply(orig, args);
                    queueMicrotask(() => cb(null, r));
                } catch (e) {
                    queueMicrotask(() => cb(e));
                }
            };
            orig.stat = cbify(orig.statSync);
            orig.lstat = cbify(orig.lstatSync);
            orig.readdir = cbify(orig.readdirSync);
            orig.readlink = function (p, cb) {
                queueMicrotask(() => cb(new Error("readlink not supported")));
            };
            orig.realpath = cbify(orig.realpathSync);
            orig.mkdir = function (p, opts, cb) {
                if (typeof opts === "function") { cb = opts; opts = undefined; }
                try {
                    if (orig.mkdirSyncRecursive) orig.mkdirSyncRecursive(p);
                    queueMicrotask(() => cb && cb(null));
                } catch (e) { queueMicrotask(() => cb && cb(e)); }
            };
            orig.rm = function (p, opts, cb) {
                if (typeof opts === "function") { cb = opts; opts = undefined; }
                try {
                    if (orig.unlinkSync) orig.unlinkSync(p);
                    queueMicrotask(() => cb && cb(null));
                } catch (e) { queueMicrotask(() => cb && cb(e)); }
            };
            orig.unlink = cbify(orig.unlinkSync || (() => null));
            orig.access = function (p, modeOrCb, maybeCb) {
                const cb = typeof modeOrCb === "function" ? modeOrCb : maybeCb;
                queueMicrotask(() => {
                    if (orig.existsSync(p)) cb(null);
                    else cb(Object.assign(new Error("ENOENT: " + p), { code: "ENOENT" }));
                });
            };
            orig.readFile = function (p, optsOrCb, maybeCb) {
                const cb = typeof optsOrCb === "function" ? optsOrCb : maybeCb;
                const opts = typeof optsOrCb === "function" ? undefined : optsOrCb;
                try { const r = orig.readFileSync(p, opts); queueMicrotask(() => cb(null, r)); }
                catch (e) { queueMicrotask(() => cb(e)); }
            };
            orig.writeFile = function (p, data, optsOrCb, maybeCb) {
                const cb = typeof optsOrCb === "function" ? optsOrCb : maybeCb;
                const opts = typeof optsOrCb === "function" ? undefined : optsOrCb;
                try { orig.writeFileSync(p, data, opts); queueMicrotask(() => cb(null)); }
                catch (e) { queueMicrotask(() => cb(e)); }
            };

            // Dirent and Stats classes for libs that use `instanceof`.
            orig.Dirent = class Dirent {
                constructor(name, type) {
                    this.name = name;
                    this._type = type;
                }
                isFile() { return this._type === "file"; }
                isDirectory() { return this._type === "dir"; }
                isSymbolicLink() { return this._type === "symlink"; }
                isBlockDevice() { return false; }
                isCharacterDevice() { return false; }
                isFIFO() { return false; }
                isSocket() { return false; }
            };
            orig.Stats = class Stats {};
        })();
    "#)?;
    Ok(())
}

// ─────────────────── Blob + File ─────────────────────────────────────
//
// Per seed §III.A8 Pattern 3: stateless Rust helpers expose the
// algorithmic core; JS-side classes hold their own state. The Blob class
// owns its bytes (as a JS array) and mime_type (string); the Rust helpers
// operate on plain Vec<u8> + String.

fn wire_blob_static<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "lowercaseAsciiType",
        Function::new(ctx.clone(), |s: String| -> String {
            s.chars()
                .map(|c| if c.is_ascii() { c.to_ascii_lowercase() } else { c })
                .collect()
        })?,
    )?;
    ns.set(
        "sliceBytes",
        Function::new(ctx.clone(), |bytes: Vec<u8>, start: i64, end: Opt<i64>| -> Vec<u8> {
            let blob = rusty_blob::Blob::from_bytes(bytes);
            blob.slice(start, end.0, None).bytes()
        })?,
    )?;
    ns.set(
        "decodeText",
        Function::new(ctx.clone(), |bytes: Vec<u8>| -> String {
            rusty_blob::Blob::from_bytes(bytes).text()
        })?,
    )?;
    global.set("__blob", ns)?;
    Ok(())
}

const BLOB_AND_FILE_CLASSES_JS: &str = r#"
globalThis.Blob = class Blob {
    constructor(parts, options) {
        const collected = [];
        if (Array.isArray(parts)) {
            for (const part of parts) {
                if (typeof part === "string") {
                    // UTF-8 encode by passing through TextEncoder.
                    const enc = new TextEncoder();
                    const encoded = enc.encode(part);
                    for (const b of encoded) collected.push(b);
                } else if (Array.isArray(part)) {
                    for (const b of part) collected.push(b);
                } else if (part && Array.isArray(part._bytes)) {
                    // Internal access path: another Blob/File. Sync read of
                    // the private byte array avoids awaiting bytes() (which
                    // is async per WHATWG post-M8 reconciliation).
                    for (const b of part._bytes) collected.push(b);
                }
            }
        }
        this._bytes = collected;
        const t = (options && typeof options.type === "string") ? options.type : "";
        this._type = __blob.lowercaseAsciiType(t);
    }
    get size() { return this._bytes.length; }
    get type() { return this._type; }
    async bytes() { return this._bytes; }
    async arrayBuffer() { return this._bytes; }
    async text() { return __blob.decodeText(this._bytes); }
    slice(start, end, contentType) {
        const startN = (typeof start === "number") ? start : 0;
        const sliced = (end === undefined)
            ? __blob.sliceBytes(this._bytes, startN)
            : __blob.sliceBytes(this._bytes, startN, end);
        const newType = (typeof contentType === "string") ? contentType : "";
        return new Blob([sliced], { type: newType });
    }
    stream() {
        // Per WHATWG File API: return a ReadableStream of Uint8Array chunks.
        // Single-chunk implementation (full bytes in one pull); spec-compliant
        // but not optimal for very large blobs.
        const bytes = this._bytes;
        return new ReadableStream({
            start(controller) {
                if (bytes.length > 0) controller.enqueue(new Uint8Array(bytes));
                controller.close();
            },
        });
    }
};

globalThis.File = class File extends Blob {
    constructor(parts, name, options) {
        super(parts, options);
        this._name = String(name);
        this._lastModified = (options && typeof options.lastModified === "number")
            ? options.lastModified : 0;
        this._webkitRelativePath = "";
    }
    get name() { return this._name; }
    get lastModified() { return this._lastModified; }
    get webkitRelativePath() { return this._webkitRelativePath; }
};
"#;

fn install_blob_and_file_classes_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(BLOB_AND_FILE_CLASSES_JS)?;
    Ok(())
}

// ─────────────────── AbortController + AbortSignal ───────────────────
//
// Per seed §III.A8 Pattern 3 + the rusty-abort-controller pilot's pattern:
// state is held in JS (the listener list, aborted flag, reason); a single
// Rust helper exposes the canonical default-reason DOMException-AbortError
// shape so the JS class can return a structurally-equivalent object.

fn wire_abort_controller_static<'js>(
    ctx: &rquickjs::Ctx<'js>, global: &Object<'js>,
) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "defaultReasonName",
        Function::new(ctx.clone(), || -> String {
            // rusty_abort_controller::Reason::AbortError → "AbortError"
            "AbortError".to_string()
        })?,
    )?;
    ns.set(
        "defaultReasonCode",
        Function::new(ctx.clone(), || -> u16 {
            // Per DOMException AbortError legacy code per pilot
            rusty_abort_controller::Reason::AbortError.code()
        })?,
    )?;
    global.set("__abort", ns)?;
    Ok(())
}

const ABORT_CONTROLLER_CLASSES_JS: &str = r#"
globalThis.AbortSignal = class AbortSignal {
    constructor() {
        this._aborted = false;
        this._reason = undefined;
        this._listeners = [];
    }
    get aborted() { return this._aborted; }
    get reason() { return this._reason; }
    addEventListener(type, listener) {
        if (type !== "abort") return;
        if (this._aborted) {
            try { listener(this._reason); } catch (_) {}
            return;
        }
        this._listeners.push(listener);
    }
    removeEventListener(type, listener) {
        if (type !== "abort") return;
        this._listeners = this._listeners.filter(l => l !== listener);
    }
    throwIfAborted() {
        if (this._aborted) throw this._reason;
    }
    _doAbort(reason) {
        if (this._aborted) return;
        this._aborted = true;
        this._reason = reason !== undefined ? reason : {
            name: __abort.defaultReasonName(),
            code: __abort.defaultReasonCode(),
            message: "The operation was aborted",
        };
        const listeners = this._listeners;
        this._listeners = [];
        for (const l of listeners) {
            try { l(this._reason); } catch (_) {}
        }
    }
    static abort(reason) {
        const s = new AbortSignal();
        s._doAbort(reason);
        return s;
    }
    static any(signals) {
        const result = new AbortSignal();
        for (const s of signals) {
            if (s._aborted) { result._doAbort(s._reason); return result; }
        }
        for (const s of signals) {
            s.addEventListener("abort", (reason) => {
                if (!result._aborted) result._doAbort(reason);
            });
        }
        return result;
    }
    static timeout(ms) {
        const s = new AbortSignal();
        setTimeout(() => {
            const err = new Error("The operation was aborted due to timeout");
            err.name = "TimeoutError";
            s._doAbort(err);
        }, ms);
        return s;
    }
};

globalThis.AbortController = class AbortController {
    constructor() {
        this._signal = new AbortSignal();
    }
    get signal() { return this._signal; }
    abort(reason) {
        this._signal._doAbort(reason);
    }
};
"#;

fn install_abort_controller_classes_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(ABORT_CONTROLLER_CLASSES_JS)?;
    Ok(())
}

// ─────────────────── Headers ─────────────────────────────────────────

fn wire_headers_static<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "validateName",
        Function::new(ctx.clone(), |name: String| -> bool {
            // Validate via the pilot's append (cheapest way to invoke
            // validate_name without exposing private fns).
            let mut h = rusty_fetch_api::Headers::new();
            h.append(&name, "x").is_ok()
        })?,
    )?;
    ns.set(
        "validateValue",
        Function::new(ctx.clone(), |value: String| -> bool {
            let mut h = rusty_fetch_api::Headers::new();
            h.append("x", &value).is_ok()
        })?,
    )?;
    ns.set(
        "lowercaseName",
        Function::new(ctx.clone(), |s: String| -> String {
            s.to_ascii_lowercase()
        })?,
    )?;
    ns.set(
        "stripWhitespace",
        Function::new(ctx.clone(), |s: String| -> String {
            s.trim_matches(|c: char| matches!(c, ' ' | '\t' | '\n' | '\r')).to_string()
        })?,
    )?;
    global.set("__headers", ns)?;
    Ok(())
}

// ─────────────────── Response (static helpers) ───────────────────────

fn wire_response_static<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "validStatus",
        Function::new(ctx.clone(), |s: u16| -> bool {
            (200..=599).contains(&s)
        })?,
    )?;
    ns.set(
        "validRedirectStatus",
        Function::new(ctx.clone(), |s: u16| -> bool {
            matches!(s, 301 | 302 | 303 | 307 | 308)
        })?,
    )?;
    global.set("__response", ns)?;
    Ok(())
}

// ─────────────────── Fetch API JS-side classes ───────────────────────

const FETCH_API_CLASSES_JS: &str = r#"
globalThis.Headers = class Headers {
    constructor(init) {
        this._entries = [];
        if (init === undefined || init === null) return;
        if (init instanceof Headers) {
            for (const [n, v] of init.entries()) this.append(n, v);
        } else if (Array.isArray(init)) {
            for (const pair of init) this.append(pair[0], pair[1]);
        } else if (typeof init === "object") {
            for (const [k, v] of Object.entries(init)) this.append(k, v);
        }
    }
    append(name, value) {
        if (!__headers.validateName(String(name))) {
            throw new TypeError("Invalid header name: " + name);
        }
        const stripped = __headers.stripWhitespace(String(value));
        if (!__headers.validateValue(stripped)) {
            throw new TypeError("Invalid header value: " + value);
        }
        this._entries.push([__headers.lowercaseName(String(name)), stripped]);
    }
    delete(name) {
        const lower = __headers.lowercaseName(String(name));
        this._entries = this._entries.filter(p => p[0] !== lower);
    }
    get(name) {
        const lower = __headers.lowercaseName(String(name));
        const matches = this._entries.filter(p => p[0] === lower);
        if (matches.length === 0) return null;
        return matches.map(p => p[1]).join(", ");
    }
    getSetCookie() {
        return this._entries.filter(p => p[0] === "set-cookie").map(p => p[1]);
    }
    has(name) {
        const lower = __headers.lowercaseName(String(name));
        return this._entries.some(p => p[0] === lower);
    }
    set(name, value) {
        if (!__headers.validateName(String(name))) {
            throw new TypeError("Invalid header name: " + name);
        }
        const stripped = __headers.stripWhitespace(String(value));
        if (!__headers.validateValue(stripped)) {
            throw new TypeError("Invalid header value: " + value);
        }
        const lower = __headers.lowercaseName(String(name));
        const newEntries = [];
        let placed = false;
        for (const p of this._entries) {
            if (p[0] === lower) {
                if (!placed) { newEntries.push([lower, stripped]); placed = true; }
            } else {
                newEntries.push(p);
            }
        }
        if (!placed) newEntries.push([lower, stripped]);
        this._entries = newEntries;
    }
    *entries() {
        const sorted = [...this._entries].sort((a, b) => a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0);
        for (const e of sorted) yield e;
    }
    *keys() { for (const [n, _] of this.entries()) yield n; }
    *values() { for (const [_, v] of this.entries()) yield v; }
    forEach(cb) { for (const [n, v] of this.entries()) cb(v, n, this); }
    [Symbol.iterator]() { return this.entries(); }
};

globalThis.Request = class Request {
    constructor(input, init) {
        if (typeof input !== "string" && !(input instanceof Request)) {
            throw new TypeError("Invalid Request input");
        }
        if (input instanceof Request) {
            this._method = input._method;
            this._url = input._url;
            this._headers = new Headers(input._headers);
            this._body = input._body;
            this._bodyUsed = false;
        } else {
            this._method = (init && init.method) ? String(init.method).toUpperCase() : "GET";
            this._url = String(input);
            this._headers = new Headers(init && init.headers);
            this._body = (init && init.body !== undefined) ? init.body : null;
            this._bodyUsed = false;
        }
        this._mode = (init && init.mode) || "cors";
        this._credentials = (init && init.credentials) || "same-origin";
        this._cache = (init && init.cache) || "default";
        this._redirect = (init && init.redirect) || "follow";
        this._signal = (init && init.signal) || new AbortSignal();
    }
    get method() { return this._method; }
    get url() { return this._url; }
    get headers() { return this._headers; }
    get body() { return this._body; }
    get bodyUsed() { return this._bodyUsed; }
    get mode() { return this._mode; }
    get credentials() { return this._credentials; }
    get cache() { return this._cache; }
    get redirect() { return this._redirect; }
    get signal() { return this._signal; }
    async text() {
        if (this._bodyUsed) throw new TypeError("Body already used");
        this._bodyUsed = true;
        if (this._body === null || this._body === undefined) return "";
        if (typeof this._body === "string") return this._body;
        if (this._body instanceof Uint8Array) return new TextDecoder().decode(this._body);
        if (this._body instanceof ArrayBuffer) return new TextDecoder().decode(new Uint8Array(this._body));
        if (Array.isArray(this._body)) return new TextDecoder().decode(this._body);
        return String(this._body);
    }
    async arrayBuffer() {
        if (this._bodyUsed) throw new TypeError("Body already used");
        this._bodyUsed = true;
        if (this._body === null || this._body === undefined) return [];
        if (typeof this._body === "string") return new TextEncoder().encode(this._body);
        if (this._body instanceof Uint8Array) return Array.from(this._body);
        if (Array.isArray(this._body)) return this._body;
        return [];
    }
    async bytes() {
        // Return Uint8Array per spec. arrayBuffer is consumed by this call;
        // recompute body sourcing inline to avoid double-consume.
        if (this._bodyUsed) throw new TypeError("Body already used");
        this._bodyUsed = true;
        if (this._body === null || this._body === undefined) return new Uint8Array(0);
        if (typeof this._body === "string") {
            const arr = new TextEncoder().encode(this._body);
            // arr may be plain Array in rusty-bun-host runtime; coerce to Uint8Array.
            return arr instanceof Uint8Array ? arr : new Uint8Array(arr);
        }
        if (this._body instanceof Uint8Array) return this._body;
        if (Array.isArray(this._body)) return new Uint8Array(this._body);
        return new Uint8Array(0);
    }
    async json() {
        return JSON.parse(await this.text());
    }
    async blob() {
        const bytes = await this.bytes();
        return new Blob([Array.from(bytes)], { type: this.headers.get("content-type") || "" });
    }
    clone() {
        if (this._bodyUsed) throw new TypeError("Body already used");
        return new Request(this);
    }
};

globalThis.Response = class Response {
    constructor(body, init) {
        const status = (init && init.status !== undefined) ? init.status : 200;
        if (!__response.validStatus(status)) {
            throw new RangeError("Status out of range: " + status);
        }
        this._status = status;
        this._statusText = (init && init.statusText) ? String(init.statusText) : "";
        this._headers = new Headers(init && init.headers);
        this._body = body !== undefined ? body : null;
        this._bodyUsed = false;
        this._type = "default";
        this._url = "";
        this._redirected = false;
    }
    static error() {
        const r = Object.create(Response.prototype);
        r._status = 0;
        r._statusText = "";
        r._headers = new Headers();
        r._body = null;
        r._bodyUsed = false;
        r._type = "error";
        r._url = "";
        r._redirected = false;
        return r;
    }
    static json(data, init) {
        const headers = new Headers(init && init.headers);
        headers.set("Content-Type", "application/json");
        const body = (typeof data === "string") ? data : JSON.stringify(data);
        return new Response(body, { ...init, headers });
    }
    static redirect(url, status) {
        const s = (status === undefined) ? 302 : status;
        if (!__response.validRedirectStatus(s)) {
            throw new RangeError("Invalid redirect status: " + s);
        }
        const headers = new Headers();
        headers.set("Location", String(url));
        return new Response(null, { status: s, headers });
    }
    get status() { return this._status; }
    get statusText() { return this._statusText; }
    get headers() { return this._headers; }
    get body() { return this._body; }
    get bodyUsed() { return this._bodyUsed; }
    get ok() { return this._status >= 200 && this._status <= 299; }
    get type() { return this._type; }
    get url() { return this._url; }
    get redirected() { return this._redirected; }
    async text() {
        if (this._bodyUsed) throw new TypeError("Body already used");
        this._bodyUsed = true;
        if (this._body === null || this._body === undefined) return "";
        if (typeof this._body === "string") return this._body;
        if (this._body instanceof Uint8Array) return new TextDecoder().decode(this._body);
        if (this._body instanceof ArrayBuffer) return new TextDecoder().decode(new Uint8Array(this._body));
        if (Array.isArray(this._body)) return new TextDecoder().decode(this._body);
        return String(this._body);
    }
    async arrayBuffer() {
        if (this._bodyUsed) throw new TypeError("Body already used");
        this._bodyUsed = true;
        if (this._body === null || this._body === undefined) return [];
        if (typeof this._body === "string") return new TextEncoder().encode(this._body);
        if (this._body instanceof Uint8Array) return Array.from(this._body);
        if (Array.isArray(this._body)) return this._body;
        return [];
    }
    async bytes() {
        if (this._bodyUsed) throw new TypeError("Body already used");
        this._bodyUsed = true;
        if (this._body === null || this._body === undefined) return new Uint8Array(0);
        if (typeof this._body === "string") {
            const arr = new TextEncoder().encode(this._body);
            return arr instanceof Uint8Array ? arr : new Uint8Array(arr);
        }
        if (this._body instanceof Uint8Array) return this._body;
        if (Array.isArray(this._body)) return new Uint8Array(this._body);
        return new Uint8Array(0);
    }
    async json() {
        return JSON.parse(await this.text());
    }
    async blob() {
        const bytes = await this.bytes();
        return new Blob([Array.from(bytes)], { type: this.headers.get("content-type") || "" });
    }
    clone() {
        if (this._bodyUsed) throw new TypeError("Body already used");
        const r = new Response(this._body, {
            status: this._status,
            statusText: this._statusText,
            headers: this._headers,
        });
        r._type = this._type;
        r._url = this._url;
        r._redirected = this._redirected;
        return r;
    }
};

// ─── globalThis.fetch (Tier-Π1.1) ─────────────────────────────────
// Real fetch() composing globalThis.HTTP (RFC 7230 codec) + globalThis.TCP
// (sockets pilot, async-listener-capable). HTTP only this round; HTTPS
// (Π1.4 TLS) + DNS (Π1.2) + chunked-streaming response (Π1 polish) come
// in follow-on rounds. The fetch-api classes (Request/Response/Headers)
// installed above remain the data-layer; this wiring lifts them onto
// real wire-level I/O for http:// URLs.
//
// Scope (this round):
//   - http:// only; throws "ENOTLS" for https:// (Π1.4 closes)
//   - IP-literal hosts + "localhost" supported; other hostnames throw
//     "ENODNS" with a pointer to Π1.2
//   - Response body read by looping TCP.read until orderly close
//     (forces Connection: close in the request)
//   - Returns a real Response built from the codec's ParsedResponse
//
// Out of scope this round (deferred to Π1 follow-on rounds):
//   - Keep-alive / connection pooling
//   - Transfer-Encoding: chunked response streaming (parseResponse already
//     handles whole-message chunked; streaming is later)
//   - Compression (Π1.3): Content-Encoding gzip/deflate/brotli decoding

if (typeof globalThis.fetch === "undefined" ||
    !globalThis.fetch.__isRustyBunReal) {
    globalThis.fetch = async function fetch(input, init) {
        if (typeof globalThis.TCP === "undefined" ||
            typeof globalThis.HTTP === "undefined") {
            throw new TypeError("rusty-bun-host: fetch requires globalThis.TCP + globalThis.HTTP (Tier-G substrate)");
        }
        // Accept string URL, URL object, or Request object.
        let url, method, headers, body;
        if (input instanceof Request) {
            url = new URL(input.url);
            method = input.method;
            headers = input.headers;
            body = (init && init.body !== undefined) ? init.body :
                (input._body !== null && input._body !== undefined) ? input._body : null;
        } else {
            url = (input instanceof URL) ? input : new URL(String(input));
            init = init || {};
            method = (init.method || "GET").toUpperCase();
            headers = new Headers(init.headers || {});
            body = init.body != null ? init.body : null;
        }

        // Scheme + host validation. http: + https: supported per Tier-Π1.1-Π1.4.
        const isHttps = (url.protocol === "https:");
        if (url.protocol !== "http:" && !isHttps) {
            throw new TypeError("rusty-bun-host: unsupported scheme '" + url.protocol + "' (only http: and https:)");
        }
        let host = url.hostname;
        if (host === "localhost") host = "127.0.0.1";
        // IPv4 literal check: a.b.c.d with 0-255 octets. If not literal,
        // route through the DNS resolver (Π1.2: std::net::ToSocketAddrs).
        const isIPv4 = /^(\d{1,3}\.){3}\d{1,3}$/.test(host) &&
            host.split(".").every(o => +o >= 0 && +o <= 255);
        if (!isIPv4) {
            // Strip IPv6 bracket form: URL gives "[::1]" as hostname.
            const bracketed = /^\[.*\]$/.test(host);
            if (bracketed) {
                // IPv6 literal — passthrough; sockets layer must accept it.
                host = host.slice(1, -1);
            } else {
                try {
                    host = globalThis.__dns.lookup_sync(host);
                } catch (e) {
                    throw new TypeError("rusty-bun-host: DNS resolution failed for '" + url.hostname + "': " + e.message);
                }
            }
        }
        const port = url.port ? parseInt(url.port, 10) : (isHttps ? 443 : 80);

        // Build headers list as [name, value] pairs.
        const headerList = [];
        let hasHost = false, hasContentLength = false, hasConnection = false;
        headers.forEach((v, n) => {
            const lower = n.toLowerCase();
            if (lower === "host") hasHost = true;
            if (lower === "content-length") hasContentLength = true;
            if (lower === "connection") hasConnection = true;
            headerList.push([n, v]);
        });
        if (!hasHost) headerList.push(["Host", url.host]);
        // Force Connection: close so the server closes the TCP connection
        // when the response is complete; we use orderly-close as the
        // "response complete" signal in this round. Keep-alive is later.
        if (!hasConnection) headerList.push(["Connection", "close"]);

        // Body normalization.
        let bodyBytes;
        if (body == null) {
            bodyBytes = new Uint8Array(0);
        } else if (typeof body === "string") {
            bodyBytes = new TextEncoder().encode(body);
        } else if (body instanceof Uint8Array) {
            bodyBytes = body;
        } else if (body instanceof ArrayBuffer) {
            bodyBytes = new Uint8Array(body);
        } else if (Array.isArray(body)) {
            bodyBytes = new Uint8Array(body);
        } else if (body && typeof body === "object" && typeof body.byteLength === "number") {
            bodyBytes = new Uint8Array(body.buffer || body, body.byteOffset || 0, body.byteLength);
        } else {
            // Best-effort coercion via String — matches fetch's behavior for
            // miscellaneous body values (URLSearchParams, FormData not yet).
            bodyBytes = new TextEncoder().encode(String(body));
        }
        if (!hasContentLength && bodyBytes.length > 0) {
            headerList.push(["Content-Length", String(bodyBytes.length)]);
        }

        // Build request bytes via the codec.
        const reqPath = url.pathname + (url.search || "");
        const reqBytes = globalThis.HTTP.serializeRequest(method, reqPath, headerList, bodyBytes);

        // Connect and send. Π1.4: https:// routes through globalThis.__tls;
        // http:// stays on globalThis.TCP. The TLS path uses a trust store
        // built from either env-supplied NODE_EXTRA_CA_CERTS / RUSTY_BUN_CA
        // (a PEM file path) plus, if absent, the system default location.
        let sid;
        let useTls = isHttps;
        let caPem = "";
        if (useTls) {
            // Read CA bundle: prefer RUSTY_BUN_CA env override (for tests),
            // then NODE_EXTRA_CA_CERTS, then system default paths.
            const env = (globalThis.process && globalThis.process.env) || {};
            const caCandidates = [];
            if (env.RUSTY_BUN_CA) caCandidates.push(env.RUSTY_BUN_CA);
            if (env.NODE_EXTRA_CA_CERTS) caCandidates.push(env.NODE_EXTRA_CA_CERTS);
            const systemPaths = [
                "/etc/ssl/certs/ca-certificates.crt",
                "/etc/pki/tls/certs/ca-bundle.crt",
                "/etc/ssl/cert.pem",
                "/etc/ssl/ca-bundle.pem",
            ];
            for (const p of systemPaths) caCandidates.push(p);
            for (const path of caCandidates) {
                try {
                    caPem = globalThis.fs.readFileSyncUtf8(path);
                    if (caPem.length > 0) break;
                } catch (_) {}
            }
            if (!caPem) {
                throw new TypeError("rusty-bun-host: no CA bundle found for https; tried RUSTY_BUN_CA / NODE_EXTRA_CA_CERTS / system paths");
            }
            sid = globalThis.__tls.connect(host, port, caPem);
        } else {
            sid = globalThis.TCP.connect(host + ":" + port);
        }
        try {
            if (useTls) {
                // __tls.write/read take Vec<u8>. reqBytes is a Uint8Array
                // returned by serializeRequest; convert per F8 pattern.
                const toFfiBytes = (b) => {
                    const arr = new Array(b.length);
                    for (let i = 0; i < b.length; i++) arr[i] = b[i];
                    return arr;
                };
                globalThis.__tls.write(sid, toFfiBytes(reqBytes));
            } else {
                globalThis.TCP.writeAll(sid, reqBytes);
            }
            // Read until orderly close: accumulate chunks.
            // Π2.6.b: for plain TCP (not TLS), use nonblocking reads so a
            // same-process Bun.serve({autoServe:true}) can interleave its
            // __tick + handler. WouldBlock → tick keep-alive registry
            // (lets the server run) + microtask yield + retry.
            const chunks = [];
            let total = 0;
            if (!useTls) {
                globalThis.TCP.setNonblocking(sid, true);
            }
            let idleSpins = 0;
            const maxIdleSpins = 200000; // ~bounded retry budget (no real timer)
            while (true) {
                let chunk;
                if (useTls) {
                    try {
                        chunk = globalThis.__tls.read(sid);
                    } catch (_) {
                        break;
                    }
                } else {
                    chunk = globalThis.TCP.tryRead(sid, 65536);
                    if (chunk === null) {
                        // Π2.6.c.b: replaced the 8-microtask-burst busy-spin
                        // with a reactor-await. waitReadable(sid) registers
                        // the fd with mio and parks on a Promise; the eval
                        // loop's __reactorDrain resolves us when the fd
                        // becomes readable. The in-process server's __tick
                        // continues to run between eval-loop iterations
                        // because reactor.poll's timeout in the eval loop is
                        // capped short (Π2.6.c.c will lift the cap once the
                        // listener also becomes mio-driven).
                        await globalThis.TCP.waitReadable(sid);
                        idleSpins++;
                        if (idleSpins > maxIdleSpins) {
                            throw new Error("rusty-bun-host fetch: read stalled (no data, no progress)");
                        }
                        continue;
                    }
                    idleSpins = 0;
                }
                if (chunk.length === 0) break;  // FIN
                chunks.push(chunk);
                total += chunk.length;
                if (total > 32 * 1024 * 1024) {
                    throw new RangeError("rusty-bun-host fetch: response exceeds 32 MB buffered limit");
                }
                // HTTP/1.1 typically frames by Content-Length or
                // Transfer-Encoding: chunked. If the headers carry one
                // of these and parseResponse succeeds on the buffered
                // bytes, the response is complete — exit without waiting
                // for FIN (Bun.serve keeps the connection open). Without
                // a length-framing header, fall back to read-until-close.
                if (!useTls) {
                    const peekFull = new Uint8Array(total);
                    let off = 0;
                    for (const c of chunks) { peekFull.set(c, off); off += c.length; }
                    // Quick header-only check to see if framing is present.
                    const hdrEnd = (() => {
                        for (let i = 0; i + 3 < peekFull.length; i++) {
                            if (peekFull[i] === 0x0d && peekFull[i+1] === 0x0a &&
                                peekFull[i+2] === 0x0d && peekFull[i+3] === 0x0a) return i + 4;
                        }
                        return -1;
                    })();
                    if (hdrEnd >= 0) {
                        const headerText = new TextDecoder().decode(peekFull.subarray(0, hdrEnd)).toLowerCase();
                        const framed = headerText.includes("content-length:") ||
                                       headerText.includes("transfer-encoding:");
                        if (framed) {
                            try {
                                globalThis.HTTP.parseResponse(peekFull);
                                chunks.length = 0;
                                chunks.push(peekFull);
                                break;
                            } catch (_) { /* not complete yet */ }
                        }
                    }
                }
            }
            // Concatenate.
            const full = new Uint8Array(total);
            let off = 0;
            for (const c of chunks) {
                const view = c instanceof Uint8Array ? c : new Uint8Array(c);
                full.set(view, off);
                off += view.length;
            }
            // Parse via codec.
            const parsed = globalThis.HTTP.parseResponse(full);
            // Build Response.
            const respHeaders = new Headers();
            for (const [n, v] of parsed.headers) respHeaders.append(n, v);
            // Π1.3: Content-Encoding decode. Per RFC 7231 §3.1.2.2, the
            // ordered list of codings was applied; reverse them to decode.
            // We support gzip + deflate + identity in this round (brotli
            // gated to Π1.3.c). After decoding, drop the Content-Encoding
            // header so consumers see only the decoded body, and adjust
            // Content-Length to match the decoded bytes.
            let respBody = parsed.body;
            const ceHeader = respHeaders.get("content-encoding");
            if (ceHeader) {
                // F8 (seed §A8 bug-catcher): rquickjs Vec<u8> FFI binding
                // rejects Uint8Array. Convert to plain Array of numbers.
                const toFfiBytes = (b) => {
                    if (b == null) return [];
                    const arr = new Array(b.length);
                    for (let i = 0; i < b.length; i++) arr[i] = b[i];
                    return arr;
                };
                const codings = ceHeader.split(",").map(s => s.trim().toLowerCase()).filter(c => c.length > 0).reverse();
                for (const c of codings) {
                    if (c === "identity") continue;
                    const inArr = toFfiBytes(respBody);
                    if (c === "gzip" || c === "x-gzip") {
                        respBody = new Uint8Array(globalThis.__compression.gunzip(inArr));
                    } else if (c === "deflate") {
                        respBody = new Uint8Array(globalThis.__compression.http_deflate_inflate(inArr));
                    } else if (c === "br") {
                        respBody = new Uint8Array(globalThis.__compression.brotli_decode(inArr));
                    } else {
                        throw new TypeError("rusty-bun-host: unsupported Content-Encoding '" + c + "'");
                    }
                }
                respHeaders.delete("content-encoding");
                respHeaders.set("content-length", String(respBody.length));
            }
            const resp = new Response(respBody, {
                status: parsed.status,
                statusText: parsed.reason,
                headers: respHeaders,
            });
            // Per spec: Response.url is the final URL after redirects. We
            // don't follow redirects in this round; set to the request URL.
            resp._url = url.href;
            return resp;
        } finally {
            try {
                if (useTls) globalThis.__tls.close(sid);
                else globalThis.TCP.close(sid);
            } catch (_) {}
        }
    };
    globalThis.fetch.__isRustyBunReal = true;
}
"#;

fn install_fetch_api_classes_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(FETCH_API_CLASSES_JS)?;
    Ok(())
}

// ─────────────────── Bun namespace (Bun.file etc.) ───────────────────

fn wire_bun_namespace_static<'js>(
    ctx: &rquickjs::Ctx<'js>, global: &Object<'js>,
) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "fileMimeType",
        Function::new(ctx.clone(), |path: String| -> String {
            // Use rusty-bun-file's extension-to-MIME mapping.
            rusty_bun_file::BunFile::open(&path).mime_type()
        })?,
    )?;
    ns.set(
        "fileExists",
        Function::new(ctx.clone(), |path: String| -> bool {
            rusty_bun_file::BunFile::open(&path).exists()
        })?,
    )?;
    ns.set(
        "fileSize",
        Function::new(ctx.clone(), |path: String| -> JsResult<i64> {
            rusty_bun_file::BunFile::open(&path)
                .size()
                .map(|s| s as i64)
                .map_err(|e| rquickjs::Error::new_from_js_message(
                    "fileSize", "i64", format!("{}", e)))
        })?,
    )?;
    ns.set(
        "fileText",
        Function::new(ctx.clone(), |path: String| -> JsResult<String> {
            rusty_bun_file::BunFile::open(&path).text().map_err(|e| {
                rquickjs::Error::new_from_js_message("fileText", "string", format!("{}", e))
            })
        })?,
    )?;
    ns.set(
        "fileBytes",
        Function::new(ctx.clone(), |path: String| -> JsResult<Vec<u8>> {
            rusty_bun_file::BunFile::open(&path).bytes().map_err(|e| {
                rquickjs::Error::new_from_js_message("fileBytes", "Vec<u8>", format!("{}", e))
            })
        })?,
    )?;
    global.set("__bun", ns)?;
    Ok(())
}

const BUN_NAMESPACE_JS: &str = r#"
globalThis.Bun = {
    file(path, options) {
        const explicitType = (options && typeof options.type === "string") ? options.type : null;
        const handle = {
            _path: String(path),
            _explicitType: explicitType,
            get name() { return this._path; },
            get size() { return __bun.fileSize(this._path); },
            get type() {
                return this._explicitType !== null
                    ? this._explicitType
                    : __bun.fileMimeType(this._path);
            },
            exists() { return __bun.fileExists(this._path); },
            text() { return __bun.fileText(this._path); },
            arrayBuffer() { return __bun.fileBytes(this._path); },
            bytes() { return __bun.fileBytes(this._path); },
            slice(start, end, contentType) {
                const all = __bun.fileBytes(this._path);
                const startN = (typeof start === "number") ? start : 0;
                const blob = new Blob([all]);
                return blob.slice(startN, end, contentType);
            },
        };
        return handle;
    },
};
"#;

fn install_bun_namespace_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(BUN_NAMESPACE_JS)?;
    Ok(())
}

// ─────────────────── Bun.serve (data-layer) ──────────────────────────
//
// The pilot's data-layer dispatch + route matching exposed as Rust helpers;
// JS-side class holds options and returns a server handle. No socket
// binding (data-layer scope per pilot AUDIT). User calls
// server.fetch(request) to invoke the routing pipeline programmatically.

fn wire_bun_serve_static<'js>(
    ctx: &rquickjs::Ctx<'js>, global: &Object<'js>,
) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    // Route pattern matching: returns an object {matched, params} with
    // params as Vec<Vec<String>> (JS-side reads as array of [name, value]).
    ns.set(
        "matchPattern",
        Function::new(ctx.clone(), |pattern: String, url: String| -> JsResult<Object<'js>> {
            // We need a Ctx to construct an object; can't access it here
            // without changing signature. Return a serialized form instead:
            // a Vec<Vec<String>> where empty = no match, else the param pairs.
            let _ = (pattern, url);
            // This branch isn't taken; see matchPatternPairs below.
            unreachable!("use matchPatternPairs")
        })?,
    )?;
    ns.set(
        "matchPatternPairs",
        Function::new(ctx.clone(), |pattern: String, url: String| -> Vec<Vec<String>> {
            // Return pair-list of captures, OR a single pair ["__nomatch__",
            // ""] sentinel when the pattern doesn't match.
            match rusty_bun_serve::match_pattern(&pattern, &url) {
                Some(params) => params
                    .captures
                    .into_iter()
                    .map(|(k, v)| vec![k, v])
                    .collect(),
                None => vec![vec!["__nomatch__".to_string(), String::new()]],
            }
        })?,
    )?;
    global.set("__serve", ns)?;
    Ok(())
}

const BUN_SERVE_JS: &str = r#"
// Extends globalThis.Bun (already installed by install_bun_namespace_js).
(function() {
    function matchRoute(pattern, urlOrPath) {
        const result = __serve.matchPatternPairs(pattern, urlOrPath);
        if (result.length === 1 && result[0][0] === "__nomatch__") return null;
        const params = {};
        for (const [k, v] of result) params[k] = v;
        return params;
    }

    function dispatch(server, request) {
        if (server._stopped) return Response.error();
        const method = (request && request.method) || "GET";
        const url = (request && request.url) || "/";

        // Route matching pass.
        if (Array.isArray(server._routes)) {
            for (const route of server._routes) {
                const params = matchRoute(route.pattern, url);
                if (params === null) continue;
                // Method-keyed dispatch.
                if (route.methods && route.methods[method]) {
                    return route.methods[method](request, params);
                }
                if (route.methods && route.methods[""]) {
                    return route.methods[""](request, params);
                }
                // Pattern matched, no handler for this method → 405.
                return new Response(null, {status: 405});
            }
        }
        // Fall through to fetch handler.
        if (typeof server._fetch === "function") {
            return server._fetch(request);
        }
        // Error handler.
        if (typeof server._error === "function") {
            return server._error(new Error("no route matched"));
        }
        return new Response(null, {status: 404});
    }

    Bun.serve = function serve(options) {
        const opts = options || {};
        const port = (typeof opts.port === "number") ? opts.port : 3000;
        const hostname = (typeof opts.hostname === "string") ? opts.hostname : "localhost";

        // Routes: convert object form ({"/path": handler-or-method-map}) to
        // array of {pattern, methods}.
        let routes = [];
        if (opts.routes && typeof opts.routes === "object") {
            for (const [pattern, handler] of Object.entries(opts.routes)) {
                if (typeof handler === "function") {
                    routes.push({pattern, methods: {"": handler}});
                } else if (handler && typeof handler === "object") {
                    routes.push({pattern, methods: handler});
                }
            }
        }

        const server = {
            _port: port,
            _hostname: hostname,
            _development: !!opts.development,
            _routes: routes,
            _fetch: opts.fetch || null,
            _error: opts.error || null,
            _stopped: false,
            _pendingRequests: 0,
            get port() { return this._port; },
            get hostname() { return this._hostname; },
            get development() { return this._development; },
            get url() { return "http://" + this._hostname + ":" + this._port + "/"; },
            get pendingRequests() { return this._pendingRequests; },
            get listening() { return !this._stopped; },
            fetch(request) {
                this._pendingRequests++;
                try {
                    return dispatch(this, request);
                } finally {
                    this._pendingRequests--;
                }
            },
            stop() {
                this._stopped = true;
                if (this._tcpListenerId != null && typeof globalThis.TCP !== "undefined") {
                    try { globalThis.TCP.stopAsync(this._tcpListenerId); } catch (_) {}
                    this._tcpListenerId = null;
                }
                // Π2.6: deregister from keep-alive registry.
                if (globalThis.__keepAlive) globalThis.__keepAlive.delete(this);
            },
            // Π2.6: __tick is the sync entry point the host's eval loop
            // calls between microtask drains. Polls the listener with
            // timeout 0 (non-blocking). If a connection is queued,
            // schedules the async handle-and-respond as a Promise (fire-
            // and-forget) and returns true. If no connection, returns
            // false. Composes on the existing tick() body without
            // requiring eval-side await.
            __tick(_unused) {
                if (this._stopped) return false;
                if (this._tcpListenerId == null) return false;
                const ev = globalThis.TCP.poll(this._tcpListenerId, 0);
                if (!ev || ev.type !== "connection") return false;
                // Fire-and-forget the async handler; microtask drain
                // between ticks completes it.
                (async () => {
                    const streamId = ev.streamId;
                    try {
                        const bytes = globalThis.TCP.read(streamId, 65536);
                        if (bytes.length === 0) return;
                        const parsed = globalThis.HTTP.parseRequest(bytes);
                        const hostHeader = parsed.headers.find(h => h[0].toLowerCase() === "host");
                        const host = hostHeader ? hostHeader[1] : "localhost";
                        const url = "http://" + host + parsed.target;
                        const headers = new Headers();
                        for (const [n, v] of parsed.headers) headers.append(n, v);
                        const init = { method: parsed.method, headers };
                        if (parsed.method !== "GET" && parsed.method !== "HEAD" &&
                            parsed.body.length > 0) {
                            init.body = parsed.body;
                        }
                        const req = new Request(url, init);
                        this._pendingRequests++;
                        let resp;
                        try { resp = await this.fetch(req); }
                        finally { this._pendingRequests--; }
                        const respHeaders = [];
                        resp.headers.forEach((v, n) => respHeaders.push([n, v]));
                        const respBody = await resp.bytes();
                        const respBytes = globalThis.HTTP.serializeResponse(
                            resp.status, resp.statusText || "", respHeaders, respBody);
                        globalThis.TCP.writeAll(streamId, respBytes);
                    } catch (e) {
                        globalThis.__stderrBuf += "Bun.serve __tick error: " +
                            (e && e.message ? e.message : String(e)) + "\n";
                    } finally {
                        try { globalThis.TCP.close(streamId); } catch (_) {}
                    }
                })();
                return true;
            },
            reload(newOptions) {
                // Per spec: port + hostname preserved across reload.
                const port = this._port;
                const hostname = this._hostname;
                Object.assign(this, Bun.serve(newOptions));
                this._port = port;
                this._hostname = hostname;
            },
            // ─── Engagement Tier-G extension (option A) ───────────────
            // listen(): bind a real TCP listener via TCP.bindAsync and start
            // the accept loop. Optional — existing fixtures using Bun.serve
            // for in-process routing continue to work without calling this.
            //
            // The full Bun.serve semantics (server auto-listens at construction)
            // can be approximated by calling `await server.listen()` after
            // construction.
            _tcpListenerId: null,
            // listen() is synchronous (returns `this`) — there is no async
            // bind operation to await. Matches Bun's "server is listening
            // when serve() returns" semantics more closely than an async
            // listen() would. Consumers can call `server.listen()` directly
            // or wrap in `await Promise.resolve(server.listen())` for
            // consistency with the previous async signature.
            listen() {
                if (typeof globalThis.TCP === "undefined" ||
                    typeof globalThis.HTTP === "undefined") {
                    throw new Error("Bun.serve.listen: globalThis.TCP + HTTP required");
                }
                const result = globalThis.TCP.bindAsync(
                    this._hostname + ":" + (this._port || 0));
                this._tcpListenerId = result.id;
                this._port = parseInt(result.addr.split(":").pop(), 10);
                return this;
            },
            // tick(maxWaitMs): process at most one accepted connection.
            // Returns true if work was done, false if poll timed out.
            async tick(maxWaitMs = 50) {
                if (this._stopped) return false;
                if (this._tcpListenerId == null) {
                    throw new Error("Bun.serve.tick: not listening (call await server.listen() first)");
                }
                const ev = globalThis.TCP.poll(this._tcpListenerId, maxWaitMs);
                if (!ev || ev.type !== "connection") return false;
                const streamId = ev.streamId;
                try {
                    const bytes = globalThis.TCP.read(streamId, 65536);
                    if (bytes.length === 0) return true;
                    const parsed = globalThis.HTTP.parseRequest(bytes);
                    const hostHeader = parsed.headers.find(h => h[0].toLowerCase() === "host");
                    const host = hostHeader ? hostHeader[1] : "localhost";
                    const url = "http://" + host + parsed.target;
                    const headers = new Headers();
                    for (const [n, v] of parsed.headers) headers.append(n, v);
                    const init = { method: parsed.method, headers };
                    if (parsed.method !== "GET" && parsed.method !== "HEAD" &&
                        parsed.body.length > 0) {
                        init.body = parsed.body;
                    }
                    const req = new Request(url, init);
                    this._pendingRequests++;
                    let resp;
                    try { resp = await this.fetch(req); }
                    finally { this._pendingRequests--; }
                    const respHeaders = [];
                    resp.headers.forEach((v, n) => respHeaders.push([n, v]));
                    const respBody = await resp.bytes();
                    const respBytes = globalThis.HTTP.serializeResponse(
                        resp.status, resp.statusText || "", respHeaders, respBody);
                    globalThis.TCP.writeAll(streamId, respBytes);
                } finally {
                    try { globalThis.TCP.close(streamId); } catch (_) {}
                }
                return true;
            },
            // serve(): run tick() in a loop until stop() is called. Returns
            // when stopped. This is the engagement-bridge equivalent of
            // Bun.serve's implicit background loop — explicit here because
            // rusty-bun-host's single-threaded JS model can't run the loop
            // in the background while other JS code runs synchronously.
            async serve() {
                if (this._tcpListenerId == null) await this.listen();
                while (!this._stopped) {
                    await this.tick(50);
                }
            },
        };
        // Π2.6: opts.autoServe = true makes Bun.serve auto-listen on
        // construction AND register the server in the host's
        // __keepAlive registry. The eval loop continues ticking the
        // server until server.stop() removes it. Unlocks canonical
        // real-Bun shape `Bun.serve({fetch, port, autoServe: true})`
        // without explicit listen()/serve() calls in consumer code.
        if (opts.autoServe === true) {
            server.listen();
            if (globalThis.__keepAlive) globalThis.__keepAlive.add(server);
        }
        return server;
    };
})();
"#;

fn install_bun_serve_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(BUN_SERVE_JS)?;
    Ok(())
}

// ─────────────────── Bun.spawn ───────────────────────────────────────
//
// The pilot wraps std::process::Command. JS-side exposes spawnSync
// returning {stdout, stderr, exitCode, success} for the most common
// shell-out pattern. spawn (async-shaped) returns a handle the JS user
// can call .wait() on; per the host's synchronous-poll model.

fn wire_bun_spawn_static<'js>(
    ctx: &rquickjs::Ctx<'js>, global: &Object<'js>,
) -> JsResult<()> {
    use rusty_bun_spawn::{SpawnOptions, StdinInput, StdioMode};
    use std::path::PathBuf;

    let ns = Object::new(ctx.clone())?;
    ns.set(
        "spawnSync",
        Function::new(ctx.clone(), |args: Vec<String>, stdin_text: Opt<String>, cwd: Opt<String>|
                -> JsResult<Object<'js>> {
            let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let opts = SpawnOptions {
                cwd: cwd.0.map(PathBuf::from),
                env: None,
                stdin: match stdin_text.0 {
                    Some(s) => StdinInput::Text(s),
                    None => StdinInput::Null,
                },
                stdout: StdioMode::Pipe,
                stderr: StdioMode::Pipe,
            };
            let _ = (args_refs.clone(), opts.clone());
            // We need a Ctx<'js> to build an Object; we don't have it here.
            // Fall through to spawnSyncResult below which returns a flat
            // pair-list the JS side rebuilds into an object.
            unreachable!("use spawnSyncResult")
        })?,
    )?;
    ns.set(
        "spawnSyncResult",
        Function::new(ctx.clone(), |args: Vec<String>, stdin_text: Opt<String>, cwd: Opt<String>|
                -> JsResult<Vec<Vec<String>>> {
            let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let opts = SpawnOptions {
                cwd: cwd.0.map(PathBuf::from),
                env: None,
                stdin: match stdin_text.0 {
                    Some(s) => StdinInput::Text(s),
                    None => StdinInput::Null,
                },
                stdout: StdioMode::Pipe,
                stderr: StdioMode::Pipe,
            };
            match rusty_bun_spawn::spawn_sync(&args_refs, opts) {
                Ok(r) => Ok(vec![
                    vec!["stdout".into(), String::from_utf8_lossy(&r.stdout).into_owned()],
                    vec!["stderr".into(), String::from_utf8_lossy(&r.stderr).into_owned()],
                    vec!["exitCode".into(), r.exit_code.to_string()],
                    vec!["success".into(), if r.success { "1".into() } else { "0".into() }],
                ]),
                Err(e) => Err(rquickjs::Error::new_from_js_message(
                    "spawnSync", "object", format!("{:?}", e))),
            }
        })?,
    )?;
    global.set("__spawn", ns)?;
    Ok(())
}

const BUN_SPAWN_JS: &str = r#"
(function() {
    // Async Bun.spawn — wraps spawnSync for now. Returns a subprocess-shaped
    // object with .exited (Promise<exitCode>), .stdout / .stderr as strings,
    // .pid, .kill(). Real async pipe streaming is deferred; spawnSync is
    // synchronous and the caller's await on .exited resolves immediately.
    Bun.spawn = function spawn(args, options) {
        const r = Bun.spawnSync(args, options || {});
        return {
            pid: r.pid || -1,
            exitCode: r.exitCode,
            exited: Promise.resolve(r.exitCode),
            stdout: r.stdout,
            stderr: r.stderr,
            success: r.success,
            kill() {},
        };
    };
    Bun.spawnSync = function spawnSync(args, options) {
        const stdinOpt = (options && options.stdin && typeof options.stdin === "string")
            ? options.stdin : undefined;
        const cwd = (options && typeof options.cwd === "string") ? options.cwd : undefined;
        const pairs = (stdinOpt !== undefined && cwd !== undefined)
            ? __spawn.spawnSyncResult(args, stdinOpt, cwd)
            : (stdinOpt !== undefined)
                ? __spawn.spawnSyncResult(args, stdinOpt)
                : (cwd !== undefined)
                    ? __spawn.spawnSyncResult(args, undefined, cwd)
                    : __spawn.spawnSyncResult(args);
        const result = {};
        for (const [k, v] of pairs) result[k] = v;
        // Convert string fields back to expected types.
        return {
            stdout: result.stdout || "",
            stderr: result.stderr || "",
            exitCode: parseInt(result.exitCode || "0", 10),
            success: result.success === "1",
        };
    };
})();
"#;

fn install_bun_spawn_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(BUN_SPAWN_JS)?;
    Ok(())
}

// ════════════════════════════════════════════════════════════════════════
// structuredClone — JS-side wiring (Pattern: pure-JS algorithm pilot)
// ════════════════════════════════════════════════════════════════════════
//
// The structured-clone pilot's Rust crate models the algorithm against a
// custom Heap/Value representation for verifier-test purposes. Routing
// JS-side values through that Heap representation would require a
// round-tripping bridge that adds no value: the algorithm is pure
// recursion plus a memo for cycle handling, and JS already has all the
// primitives it needs (Date, RegExp, Map, Set, ArrayBuffer, TypedArrays,
// Blob, File). The pilot's Rust crate stays the canonical algorithmic
// reference; the host wires structuredClone as a JS-side reimplementation
// against the same constraint set the pilot was derived from.
//
// This is a third pattern alongside Pattern 1 (pure-value Rust) and
// Pattern 3 (stateless Rust + JS class): Pattern 4, "spec-formalization
// pilot, JS-side instantiation". Folded back into seed §III.A8 in the
// resolution-increase pass that ships with this round.

const STRUCTURED_CLONE_JS: &str = r#"
(function() {
    function clone(value, memo) {
        // Primitives: cloned by value automatically.
        if (value === null || value === undefined) return value;
        const t = typeof value;
        if (t === "number" || t === "string" || t === "boolean" || t === "bigint") return value;
        if (t === "symbol") throw new DOMExceptionLike("Symbols are not cloneable", "DataCloneError");
        if (t === "function") throw new DOMExceptionLike("Functions are not cloneable", "DataCloneError");

        // Cycle handling.
        if (memo.has(value)) return memo.get(value);

        // Date.
        if (value instanceof Date) {
            const out = new Date(value.getTime());
            memo.set(value, out);
            return out;
        }

        // RegExp.
        if (value instanceof RegExp) {
            const out = new RegExp(value.source, value.flags);
            memo.set(value, out);
            return out;
        }

        // Map.
        if (value instanceof Map) {
            const out = new Map();
            memo.set(value, out);
            for (const [k, v] of value) {
                out.set(clone(k, memo), clone(v, memo));
            }
            return out;
        }

        // Set.
        if (value instanceof Set) {
            const out = new Set();
            memo.set(value, out);
            for (const v of value) {
                out.add(clone(v, memo));
            }
            return out;
        }

        // ArrayBuffer.
        if (value instanceof ArrayBuffer) {
            const out = value.slice(0);
            memo.set(value, out);
            return out;
        }

        // Typed arrays / DataView.
        if (ArrayBuffer.isView(value)) {
            const buf = clone(value.buffer, memo);
            const ctor = value.constructor;
            const out = value instanceof DataView
                ? new DataView(buf, value.byteOffset, value.byteLength)
                : new ctor(buf, value.byteOffset, value.length);
            memo.set(value, out);
            return out;
        }

        // Blob / File: rely on the host's Blob.slice for byte-copy.
        if (typeof Blob !== "undefined" && value instanceof Blob) {
            // Re-assemble via the bytes-getter the wired Blob exposes.
            const bytes = value._bytes ? value._bytes.slice() : [];
            if (typeof File !== "undefined" && value instanceof File) {
                const out = new File([new Uint8Array(bytes)], value.name, {
                    type: value.type,
                    lastModified: value.lastModified,
                });
                memo.set(value, out);
                return out;
            }
            const out = new Blob([new Uint8Array(bytes)], { type: value.type });
            memo.set(value, out);
            return out;
        }

        // Array.
        if (Array.isArray(value)) {
            const out = [];
            memo.set(value, out);
            for (let i = 0; i < value.length; i++) {
                out[i] = clone(value[i], memo);
            }
            return out;
        }

        // Plain object (own enumerable string keys; Symbol keys excluded per spec).
        if (t === "object") {
            const proto = Object.getPrototypeOf(value);
            if (proto !== null && proto !== Object.prototype) {
                throw new DOMExceptionLike(
                    "Object with non-plain prototype is not cloneable",
                    "DataCloneError"
                );
            }
            const out = {};
            memo.set(value, out);
            for (const k of Object.keys(value)) {
                out[k] = clone(value[k], memo);
            }
            return out;
        }

        throw new DOMExceptionLike("Value is not cloneable", "DataCloneError");
    }

    // Lightweight DOMException stand-in for the DataCloneError surface.
    function DOMExceptionLike(message, name) {
        const err = new Error(message);
        err.name = name || "Error";
        return err;
    }

    globalThis.structuredClone = function structuredClone(value, options) {
        // options.transfer is part of the spec but per pilot scope (Doc 708:
        // "structured-clone pilot, ecosystem-only"), transfer-list semantics
        // are deferred. The arg is accepted for API compatibility.
        const memo = new Map();
        return clone(value, memo);
    };
})();
"#;

fn install_structured_clone_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(STRUCTURED_CLONE_JS)?;
    Ok(())
}

// ════════════════════════════════════════════════════════════════════════
// Streams — JS-side wiring (Pattern 4: spec-formalization pilot)
// ════════════════════════════════════════════════════════════════════════
//
// The streams pilot's Rust crate models ReadableStream<T>/WritableStream<T>/
// TransformStream<I,O> generically with Rc<RefCell> state machines. The
// types are not naturally bindable to JS (genericity, internal mutability,
// promises). Per seed §III.A8.2bis, this wires as a JS-side reimplementation
// against the same WHATWG Streams Standard constraint set the pilot was
// derived from. The Rust pilot remains the canonical algorithmic reference
// and the ratio anchor; this is a sibling instantiation.
//
// Scope: minimal-yet-spec-shaped subset sufficient for canonical patterns:
//   - new ReadableStream({start, pull, cancel}) with controller.enqueue/
//     close/error; getReader().read() → Promise<{value, done}>; async
//     iteration via Symbol.asyncIterator.
//   - new WritableStream({start, write, close, abort}); getWriter().write/
//     close.
//   - new TransformStream({transform, flush}) with .readable / .writable.
// Backpressure (highWaterMark, desiredSize, queuingStrategy) is API-stub
// only — the spec algorithm here is queue + pull-loop, sufficient for the
// pull-driven canonical patterns. BYOB / TeeStream / pipeTo are deferred.

const STREAMS_JS: &str = r#"
(function() {
    // ─── ReadableStream ────────────────────────────────────────────────
    class ReadableStreamDefaultController {
        constructor(stream) {
            this._stream = stream;
        }
        enqueue(chunk) {
            const s = this._stream;
            if (s._state !== "readable") {
                throw new TypeError("Cannot enqueue on " + s._state + " stream");
            }
            // If a pending read exists, satisfy it directly.
            if (s._pendingReads.length > 0) {
                const p = s._pendingReads.shift();
                p.resolve({ value: chunk, done: false });
            } else {
                s._queue.push(chunk);
            }
        }
        close() {
            const s = this._stream;
            if (s._state !== "readable") return;
            s._state = "closed";
            while (s._pendingReads.length > 0) {
                const p = s._pendingReads.shift();
                p.resolve({ value: undefined, done: true });
            }
        }
        error(e) {
            const s = this._stream;
            if (s._state === "errored") return;
            s._state = "errored";
            s._storedError = e;
            while (s._pendingReads.length > 0) {
                const p = s._pendingReads.shift();
                p.reject(e);
            }
        }
        get desiredSize() {
            const s = this._stream;
            if (s._state === "errored") return null;
            if (s._state === "closed") return 0;
            return 1;  // stub backpressure
        }
    }

    class ReadableStreamDefaultReader {
        constructor(stream) {
            if (stream._reader) {
                throw new TypeError("Stream already has a reader");
            }
            this._stream = stream;
            stream._reader = this;
        }
        read() {
            const s = this._stream;
            if (!s) return Promise.reject(new TypeError("Reader released"));
            if (s._state === "errored") return Promise.reject(s._storedError);
            if (s._queue.length > 0) {
                return Promise.resolve({ value: s._queue.shift(), done: false });
            }
            if (s._state === "closed") {
                return Promise.resolve({ value: undefined, done: true });
            }
            return new Promise((resolve, reject) => {
                s._pendingReads.push({ resolve, reject });
                // Trigger pull if source is pull-driven.
                if (s._source.pull && !s._pulling) {
                    s._pulling = true;
                    Promise.resolve().then(() => {
                        try {
                            const r = s._source.pull(s._controller);
                            if (r && typeof r.then === "function") {
                                r.then(
                                    () => { s._pulling = false; },
                                    (e) => { s._controller.error(e); s._pulling = false; }
                                );
                            } else {
                                s._pulling = false;
                            }
                        } catch (e) {
                            s._controller.error(e);
                            s._pulling = false;
                        }
                    });
                }
            });
        }
        cancel(reason) {
            const s = this._stream;
            if (!s) return Promise.resolve();
            if (s._state === "errored") return Promise.reject(s._storedError);
            s._state = "closed";
            s._queue = [];
            const r = s._source.cancel ? s._source.cancel(reason) : undefined;
            return Promise.resolve(r).then(() => undefined);
        }
        releaseLock() {
            if (this._stream) {
                this._stream._reader = null;
                this._stream = null;
            }
        }
        get closed() {
            const s = this._stream;
            if (!s) return Promise.resolve();
            if (s._state === "closed") return Promise.resolve();
            if (s._state === "errored") return Promise.reject(s._storedError);
            return new Promise((resolve, reject) => {
                s._closedPromises.push({ resolve, reject });
            });
        }
    }

    class ReadableStream {
        constructor(source = {}) {
            this._source = source;
            this._state = "readable";
            this._queue = [];
            this._pendingReads = [];
            this._closedPromises = [];
            this._reader = null;
            this._pulling = false;
            this._controller = new ReadableStreamDefaultController(this);
            // Run start() synchronously per spec.
            if (source.start) {
                try {
                    const r = source.start(this._controller);
                    if (r && typeof r.then === "function") {
                        r.catch((e) => this._controller.error(e));
                    }
                } catch (e) {
                    this._controller.error(e);
                }
            }
        }
        getReader() {
            return new ReadableStreamDefaultReader(this);
        }
        cancel(reason) {
            const r = new ReadableStreamDefaultReader(this);
            const p = r.cancel(reason);
            r.releaseLock();
            return p;
        }
        get locked() {
            return this._reader !== null;
        }
        [Symbol.asyncIterator]() {
            const reader = this.getReader();
            return {
                next() { return reader.read(); },
                return(value) {
                    reader.releaseLock();
                    return Promise.resolve({ value, done: true });
                },
                [Symbol.asyncIterator]() { return this; },
            };
        }
        // pipeTo(destWritable) — read chunks and forward to the
        // destination's writer; closes the writer when source ends.
        // Returns a promise that resolves on completion.
        async pipeTo(dest, opts) {
            opts = opts || {};
            const reader = this.getReader();
            const writer = dest.getWriter();
            try {
                while (true) {
                    const { value, done } = await reader.read();
                    if (done) break;
                    await writer.write(value);
                }
                if (!opts.preventClose) await writer.close();
            } catch (e) {
                if (!opts.preventAbort) {
                    try { await writer.abort(e); } catch (_) {}
                }
                throw e;
            } finally {
                try { reader.releaseLock(); } catch (_) {}
                try { writer.releaseLock(); } catch (_) {}
            }
        }
        // pipeThrough(transformer, opts) — pipe this → transformer.writable
        // and return transformer.readable. Composable for chaining.
        pipeThrough(transformer, opts) {
            // Accept either a TransformStream (has .readable/.writable)
            // or a { writable, readable } object.
            const writable = transformer.writable;
            const readable = transformer.readable;
            // Fire-and-forget pipeTo; errors propagate via the readable.
            this.pipeTo(writable, opts).catch((e) => {
                try {
                    // Best-effort error propagation into the readable.
                    if (readable._controller && typeof readable._controller.error === "function") {
                        readable._controller.error(e);
                    }
                } catch (_) {}
            });
            return readable;
        }
        // tee() returns two ReadableStreams that emit the same chunks.
        tee() {
            const queues = [[], []];
            const readers = [null, null];
            const reader = this.getReader();
            let done = false;
            const drainTo = async (which) => {
                while (true) {
                    if (queues[which].length > 0) return queues[which].shift();
                    if (done) return null;
                    const { value, done: d } = await reader.read();
                    if (d) { done = true; return null; }
                    queues[1 - which].push(value);
                    return value;
                }
            };
            const make = (which) => new ReadableStream({
                async pull(c) {
                    const v = await drainTo(which);
                    if (v === null) c.close();
                    else c.enqueue(v);
                },
            });
            return [make(0), make(1)];
        }
    }

    // ─── WritableStream ────────────────────────────────────────────────
    class WritableStreamDefaultController {
        constructor(stream) {
            this._stream = stream;
        }
        error(e) {
            const s = this._stream;
            if (s._state !== "writable") return;
            s._state = "errored";
            s._storedError = e;
        }
    }

    class WritableStreamDefaultWriter {
        constructor(stream) {
            if (stream._writer) {
                throw new TypeError("Stream already has a writer");
            }
            this._stream = stream;
            stream._writer = this;
        }
        write(chunk) {
            const s = this._stream;
            if (!s) return Promise.reject(new TypeError("Writer released"));
            if (s._state === "errored") return Promise.reject(s._storedError);
            if (s._state !== "writable") return Promise.reject(new TypeError("Stream is " + s._state));
            if (!s._sink.write) return Promise.resolve();
            try {
                const r = s._sink.write(chunk, s._controller);
                return Promise.resolve(r);
            } catch (e) {
                s._controller.error(e);
                return Promise.reject(e);
            }
        }
        close() {
            const s = this._stream;
            if (!s) return Promise.reject(new TypeError("Writer released"));
            if (s._state === "errored") return Promise.reject(s._storedError);
            s._state = "closed";
            if (s._sink.close) {
                try { return Promise.resolve(s._sink.close()); }
                catch (e) { return Promise.reject(e); }
            }
            return Promise.resolve();
        }
        abort(reason) {
            const s = this._stream;
            if (!s) return Promise.resolve();
            s._state = "errored";
            s._storedError = reason;
            if (s._sink.abort) {
                try { return Promise.resolve(s._sink.abort(reason)); }
                catch (e) { return Promise.reject(e); }
            }
            return Promise.resolve();
        }
        releaseLock() {
            if (this._stream) {
                this._stream._writer = null;
                this._stream = null;
            }
        }
        get desiredSize() {
            const s = this._stream;
            if (!s) return null;
            if (s._state === "errored") return null;
            if (s._state === "closed") return 0;
            return 1;
        }
    }

    class WritableStream {
        constructor(sink = {}) {
            this._sink = sink;
            this._state = "writable";
            this._writer = null;
            this._controller = new WritableStreamDefaultController(this);
            if (sink.start) {
                try {
                    const r = sink.start(this._controller);
                    if (r && typeof r.then === "function") {
                        r.catch((e) => this._controller.error(e));
                    }
                } catch (e) {
                    this._controller.error(e);
                }
            }
        }
        getWriter() {
            return new WritableStreamDefaultWriter(this);
        }
        abort(reason) {
            const w = new WritableStreamDefaultWriter(this);
            const p = w.abort(reason);
            w.releaseLock();
            return p;
        }
        close() {
            const w = new WritableStreamDefaultWriter(this);
            const p = w.close();
            w.releaseLock();
            return p;
        }
        get locked() {
            return this._writer !== null;
        }
    }

    // ─── TransformStream ───────────────────────────────────────────────
    class TransformStreamDefaultController {
        constructor() {
            this._readableController = null;
        }
        enqueue(chunk) { this._readableController.enqueue(chunk); }
        terminate() { this._readableController.close(); }
        error(e) { this._readableController.error(e); }
    }

    class TransformStream {
        constructor(transformer = {}) {
            const tController = new TransformStreamDefaultController();
            const transformFn = transformer.transform || ((chunk, controller) => controller.enqueue(chunk));
            const flushFn = transformer.flush;

            this.readable = new ReadableStream({
                start(controller) {
                    tController._readableController = controller;
                },
            });

            this.writable = new WritableStream({
                start() {
                    if (transformer.start) return transformer.start(tController);
                },
                write(chunk) {
                    return Promise.resolve(transformFn(chunk, tController));
                },
                close() {
                    const r = flushFn ? flushFn(tController) : undefined;
                    return Promise.resolve(r).then(() => {
                        tController._readableController.close();
                    });
                },
                abort(reason) {
                    tController._readableController.error(reason);
                },
            });
        }
    }

    globalThis.ReadableStream = ReadableStream;
    globalThis.WritableStream = WritableStream;
    globalThis.TransformStream = TransformStream;
})();
"#;

fn install_streams_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(STREAMS_JS)?;
    Ok(())
}

// ════════════════════════════════════════════════════════════════════════
// node-http data-layer — JS-side wiring (Pattern 4)
// ════════════════════════════════════════════════════════════════════════
//
// node-http's pilot is data-only: NodeHeaders + IncomingMessage +
// ServerResponse + ClientRequest + Server with no transport. All state
// is plain values; the only algorithm is case-insensitive header
// normalization (lowercased keys per Node API). Per seed §III.A8.2bis,
// wires as JS-side reimplementation against the same constraint set.
//
// Provides node:http data-layer surface accessible as `nodeHttp.*` on
// globalThis. Real consumers would import from "node:http"; module
// resolution is Tier-H.3 (deferred).

const NODE_HTTP_JS: &str = r#"
(function() {
    function makeHeaders() {
        // Node represents headers as plain object with lowercased keys.
        // Multi-value headers stored as arrays per Node convention for
        // set-cookie etc.; pilot scope keeps single-value semantics.
        return Object.create(null);
    }

    function normalizeName(name) {
        return String(name).toLowerCase();
    }

    function setHeader(headers, name, value) {
        headers[normalizeName(name)] = String(value);
    }

    function getHeader(headers, name) {
        return headers[normalizeName(name)];
    }

    function removeHeader(headers, name) {
        delete headers[normalizeName(name)];
    }

    // IncomingMessage + ServerResponse as function-style constructors
    // (NOT ES6 classes) so consumer code that does the Node-inheritance
    // pattern — `IncomingMessage.call(this, init)` from a child function
    // — works without "class constructors must be invoked with 'new'".
    // light-my-request, follow-redirects, fastify's plugin chain all
    // use this idiom.
    function IncomingMessage(init) {
        init = init || {};
        this.method = init.method || "GET";
        this.url = init.url || "/";
        this.httpVersion = init.httpVersion || "1.1";
        this.headers = makeHeaders();
        if (init.headers) {
            for (const k of Object.keys(init.headers)) {
                setHeader(this.headers, k, init.headers[k]);
            }
        }
        this.statusCode = init.statusCode || 0;
        this.statusMessage = init.statusMessage || "";
        this._body = init.body || "";
        this.complete = init.complete !== undefined ? init.complete : true;
    }

    function ServerResponse() {
        this.statusCode = 200;
        this.statusMessage = "OK";
        this._headers = makeHeaders();
        this._body = [];
        this.headersSent = false;
        this.ended = false;
    }
    ServerResponse.prototype.writeHead = function (statusCode, statusMessage, headers) {
        if (this.headersSent) return this;
        this.statusCode = statusCode;
        if (typeof statusMessage === "object" && statusMessage !== null) {
            headers = statusMessage;
            statusMessage = undefined;
        }
        if (statusMessage !== undefined) this.statusMessage = String(statusMessage);
        if (headers) {
            for (const k of Object.keys(headers)) {
                setHeader(this._headers, k, headers[k]);
            }
        }
        this.headersSent = true;
        return this;
    };
    ServerResponse.prototype.setHeader = function (name, value) {
        setHeader(this._headers, name, value); return this;
    };
    ServerResponse.prototype.getHeader = function (name) {
        return getHeader(this._headers, name);
    };
    ServerResponse.prototype.removeHeader = function (name) {
        removeHeader(this._headers, name); return this;
    };
    ServerResponse.prototype.getHeaders = function () {
        return Object.assign({}, this._headers);
    };
    ServerResponse.prototype.write = function (chunk) {
        if (this.ended) return false;
        this.headersSent = true;
        this._body.push(String(chunk));
        return true;
    };
    ServerResponse.prototype.end = function (chunk) {
        if (this.ended) return this;
        if (chunk !== undefined) this._body.push(String(chunk));
        this.headersSent = true;
        this.ended = true;
        if (typeof this._resolve === "function") {
            try {
                const r = new Response(this.body(), {
                    status: this.statusCode,
                    statusText: this.statusMessage || undefined,
                    headers: Object.assign({}, this._headers),
                });
                this._resolve(r);
            } catch (e) {
                if (typeof this._reject === "function") this._reject(e);
            }
        }
        return this;
    };
    ServerResponse.prototype.body = function () { return this._body.join(""); };

    // EventEmitter-shape stubs on IncomingMessage and ServerResponse.
    // light-my-request, fastify, many Node http consumers subscribe
    // to 'data', 'end', 'close', 'finish', 'drain' events. Inject mode
    // doesn't run a real socket loop, but listener wiring must accept
    // registration and emit without throwing.
    function _eeInit(self) {
        if (!self._events) self._events = Object.create(null);
    }
    function _eeOn(event, fn) {
        _eeInit(this);
        if (!this._events[event]) this._events[event] = [];
        this._events[event].push(fn);
        return this;
    }
    function _eeOff(event, fn) {
        _eeInit(this);
        const list = this._events[event];
        if (list) {
            const idx = list.indexOf(fn);
            if (idx >= 0) list.splice(idx, 1);
        }
        return this;
    }
    function _eeEmit(event) {
        _eeInit(this);
        const list = this._events[event];
        if (!list || list.length === 0) return false;
        const args = Array.prototype.slice.call(arguments, 1);
        for (const fn of list.slice()) {
            try { fn.apply(this, args); }
            catch (e) {
                if (typeof console !== "undefined" && console.error) {
                    console.error("uncaught in EE listener:", e);
                }
            }
        }
        return true;
    }
    function _eeOnce(event, fn) {
        _eeInit(this);
        const self = this;
        const wrap = function () {
            self.removeListener(event, wrap);
            fn.apply(self, arguments);
        };
        return self.on(event, wrap);
    }
    function _eeListeners(event) {
        _eeInit(this);
        return (this._events[event] || []).slice();
    }
    function _eeListenerCount(event) {
        _eeInit(this);
        return (this._events[event] || []).length;
    }
    function _eeRemoveAllListeners(event) {
        _eeInit(this);
        if (event === undefined) this._events = Object.create(null);
        else delete this._events[event];
        return this;
    }
    function _eeSetMaxListeners() { return this; }

    for (const Klass of [IncomingMessage, ServerResponse]) {
        Klass.prototype.on = _eeOn;
        Klass.prototype.addListener = _eeOn;
        Klass.prototype.off = _eeOff;
        Klass.prototype.removeListener = _eeOff;
        Klass.prototype.emit = _eeEmit;
        Klass.prototype.once = _eeOnce;
        Klass.prototype.listeners = _eeListeners;
        Klass.prototype.listenerCount = _eeListenerCount;
        Klass.prototype.removeAllListeners = _eeRemoveAllListeners;
        Klass.prototype.setMaxListeners = _eeSetMaxListeners;
        Klass.prototype.ref = function () { return this; };
        Klass.prototype.unref = function () { return this; };
    }

    // Socket attachment stubs. light-my-request calls assignSocket
    // with a null-socket adapter; we accept and stash without rejecting.
    ServerResponse.prototype.assignSocket = function (socket) {
        this.socket = socket;
        this.connection = socket;
        return this;
    };
    ServerResponse.prototype.detachSocket = function () {
        this.socket = null;
        this.connection = null;
        return this;
    };
    ServerResponse.prototype.flushHeaders = function () {
        this.headersSent = true;
        return this;
    };
    ServerResponse.prototype.hasHeader = function (name) {
        return getHeader(this._headers, name) !== undefined;
    };
    ServerResponse.prototype.appendHeader = ServerResponse.prototype.setHeader;
    ServerResponse.prototype.pipe = function (dest) { return dest; };

    class ClientRequest {
        constructor(method, url) {
            this.method = method;
            this.url = url;
            this._headers = makeHeaders();
            this._body = [];
            this.aborted = false;
            this.ended = false;
        }
        setHeader(name, value) { setHeader(this._headers, name, value); return this; }
        getHeader(name) { return getHeader(this._headers, name); }
        write(chunk) {
            if (this.aborted || this.ended) return false;
            this._body.push(String(chunk));
            return true;
        }
        end(chunk) {
            if (this.aborted || this.ended) return this;
            if (chunk !== undefined) this._body.push(String(chunk));
            this.ended = true;
            return this;
        }
        abort() { this.aborted = true; return this; }
        getHeaders() { return Object.assign({}, this._headers); }
        body() { return this._body.join(""); }
    }

    class Server {
        constructor(handler) {
            this._handler = handler || null;
            this._port = 0;
            this._listening = false;
            this._closed = false;
            this._bunServer = null;
        }
        on(event, handler) {
            if (event === "request") this._handler = handler;
            return this;
        }
        // listen(port?, host?, cb?) — Bun.serve bridge. The Node-style
        // request handler (req, res) is wrapped in a Web-fetch handler:
        // build IncomingMessage from Request, hand it + a ServerResponse
        // to the user's handler, await res.end() resolving to a Response.
        // Enables express/koa/fastify-style code that calls .listen() to
        // run end-to-end via Π2.6.b cooperative-yield self-fetch.
        listen(...args) {
            // Parse args: listen(port?, host?, cb?) or listen(opts, cb)
            let port = 0, host = "127.0.0.1", cb = null;
            for (const a of args) {
                if (typeof a === "number") { port = a; }
                else if (typeof a === "string") { host = a; }
                else if (typeof a === "function") { cb = a; }
                else if (a && typeof a === "object") {
                    if (typeof a.port === "number") port = a.port;
                    if (typeof a.host === "string") host = a.host;
                }
            }
            if (this._listening) {
                if (cb) Promise.resolve().then(cb);
                return this;
            }
            // Gate: the Bun.serve bridge needs the async eval loop. In
            // sync-eval contexts (eval_i64/eval_bool/eval_string from
            // unit tests of the data-layer surface), keep the old
            // flag-only behavior — listen() is non-binding.
            if (!globalThis.__asyncEvalActive || !this._handler) {
                this._port = port || this._port;
                this._listening = true;
                if (cb) Promise.resolve().then(cb);
                return this;
            }
            const handler = this._handler;
            this._bunServer = Bun.serve({
                port, hostname: host, autoServe: true,
                fetch: (req) => new Promise((resolve, reject) => {
                    const u = new URL(req.url);
                    const headerObj = {};
                    req.headers.forEach((v, n) => { headerObj[n] = v; });
                    // Buffer the request body, then build the Node-style req+res.
                    req.text().then((body) => {
                        const incoming = new IncomingMessage({
                            method: req.method,
                            url: u.pathname + u.search,
                            httpVersion: "1.1",
                            headers: headerObj,
                            body,
                            complete: true,
                        });
                        const res = new ServerResponse();
                        res._resolve = resolve;
                        res._reject = reject;
                        try { handler(incoming, res); }
                        catch (e) { reject(e); }
                    }).catch(reject);
                }),
            });
            this._port = this._bunServer.port;
            this._listening = true;
            if (cb) Promise.resolve().then(cb);
            return this;
        }
        close(cb) {
            if (this._bunServer && this._bunServer.stop) {
                try { this._bunServer.stop(); } catch (_) {}
            }
            this._listening = false;
            this._closed = true;
            if (typeof cb === "function") Promise.resolve().then(cb);
            return this;
        }
        get listening() { return this._listening; }
        get port() { return this._port; }
        address() {
            return this._listening
                ? { address: "127.0.0.1", family: "IPv4", port: this._port }
                : null;
        }
        // setTimeout(ms, cb?) — fastify calls server.setTimeout(connectionTimeout)
        // at construction. We accept and store but don't enforce a real
        // socket timeout (no inbound socket in inject() mode).
        setTimeout(ms, cb) {
            this._timeoutMs = ms;
            if (typeof cb === "function") this.on("timeout", cb);
            return this;
        }
        // Other server setters fastify pokes (no-op except recording):
        get maxConnections() { return this._maxConnections || 0; }
        set maxConnections(v) { this._maxConnections = v; }
        // Event-emitter-shape addListener/removeListener stubs so libraries
        // that subscribe to 'clientError', 'connection', etc. don't throw.
        addListener(event, fn) { return this.on(event, fn); }
        removeListener(_event, _fn) { return this; }
        removeAllListeners(_event) { return this; }
        emit(_event) { return false; }
        once(event, fn) { return this.on(event, fn); }
        off(event, fn) { return this.removeListener(event, fn); }
        ref() { return this; }
        unref() { return this; }
        // Pilot helper preserved: synchronous data-layer dispatch.
        dispatch(req) {
            const incoming = req instanceof IncomingMessage ? req : new IncomingMessage(req);
            const res = new ServerResponse();
            if (this._handler) this._handler(incoming, res);
            return res;
        }
    }

    function createServer(opts, handler) {
        // Two-arg form: (options, handler). fastify passes (options.http, httpHandler).
        // One-arg form: (handler) or (options).
        if (typeof opts === "function") return new Server(opts);
        return new Server(handler || null);
    }

    function request(options, cb) {
        // Accept both string-url and options-object forms per Node.
        const opts = typeof options === "string"
            ? { method: "GET", url: options }
            : options;
        const req = new ClientRequest(opts.method || "GET", opts.url || opts.path || "/");
        if (opts.headers) {
            for (const k of Object.keys(opts.headers)) {
                setHeader(req._headers, k, opts.headers[k]);
            }
        }
        // Per Node API, the response callback is invoked when the response
        // arrives. Pilot data-layer cannot actually send; if a cb is given,
        // it gets a stub IncomingMessage with status 0 to indicate no
        // transport occurred. Real wiring requires Tier-G.
        if (typeof cb === "function") {
            Promise.resolve().then(() =>
                cb(new IncomingMessage({ statusCode: 0, statusMessage: "no-transport" })));
        }
        return req;
    }

    globalThis.nodeHttp = {
        createServer,
        request,
        get: request,
        IncomingMessage,
        ServerResponse,
        ClientRequest,
        Server,
        METHODS: ["ACL", "BIND", "CHECKOUT", "CONNECT", "COPY", "DELETE",
            "GET", "HEAD", "LINK", "LOCK", "M-SEARCH", "MERGE", "MKACTIVITY",
            "MKCALENDAR", "MKCOL", "MOVE", "NOTIFY", "OPTIONS", "PATCH",
            "POST", "PROPFIND", "PROPPATCH", "PURGE", "PUT", "REBIND",
            "REPORT", "SEARCH", "SOURCE", "SUBSCRIBE", "TRACE", "UNBIND",
            "UNLINK", "UNLOCK", "UNSUBSCRIBE"],
        STATUS_CODES: {
            100: "Continue", 101: "Switching Protocols", 102: "Processing",
            200: "OK", 201: "Created", 202: "Accepted", 204: "No Content",
            301: "Moved Permanently", 302: "Found", 304: "Not Modified",
            307: "Temporary Redirect", 308: "Permanent Redirect",
            400: "Bad Request", 401: "Unauthorized", 403: "Forbidden",
            404: "Not Found", 405: "Method Not Allowed", 408: "Request Timeout",
            409: "Conflict", 410: "Gone", 413: "Payload Too Large",
            415: "Unsupported Media Type", 418: "I'm a Teapot", 422: "Unprocessable Entity",
            429: "Too Many Requests", 500: "Internal Server Error",
            501: "Not Implemented", 502: "Bad Gateway", 503: "Service Unavailable",
            504: "Gateway Timeout",
        },
        globalAgent: { maxSockets: Infinity, keepAlive: false },
        Agent: class Agent {
            constructor(opts) { Object.assign(this, opts || {}); }
        },
    };
})();
"#;

fn install_node_http_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(NODE_HTTP_JS)?;
    Ok(())
}

// ════════════════════════════════════════════════════════════════════════
// CommonJS module loader (Tier-H.3, partial)
// ════════════════════════════════════════════════════════════════════════
//
// Implements Node-style synchronous require() over the wired fs pilot.
// First subitem of H.3 (module loader/resolver). ESM (import/import.meta)
// is deferred to a follow-on round; rquickjs has a built-in FileResolver
// + ScriptLoader that handles the ESM side, but composing them with
// node_modules-walking resolution requires its own pass.
//
// Scope of THIS round (Node-spec CommonJS, sufficient for typical npm
// packages whose entry is a CJS module):
//   - require(specifier) resolves relative paths (./foo, ../bar) and
//     bare specifiers (pkg, pkg/sub) walking node_modules upward.
//   - Extensions tried in order: as-is, .js, .json, /index.js, /index.json.
//   - package.json `main` field honored. `exports` (Node 12+ subpath
//     exports) is partially supported (string + "." key only).
//   - Module cache by absolute resolved path.
//   - Loaded sources wrapped in (function(module, exports, require,
//     __filename, __dirname) { ... }).
//   - Cycle handling: returns the partial exports per Node semantics.
//
// Bootstrapping require: the loader exposes a global `bootRequire(absPath)`
// that loads `absPath` as the entry module. From there, that module's
// require() resolves everything relative to its own __dirname.

const COMMONJS_LOADER_JS: &str = r#"
(function() {
    if (typeof fs === "undefined") {
        throw new Error("fs must be wired before commonjs-loader");
    }

    // S6 closure: rewrite `\-` inside character classes within regex
    // literals that carry the /u flag. QuickJS rejects \- under /u; the
    // ECMAScript spec allows it (though discouraged). Replace `\-` with
    // `-` which is unambiguously a literal hyphen under /u and
    // semantically identical. fast-uri/lib/utils.js, ajv-formats, and
    // similar libs use this pattern.
    function fixRegexUEscapes(source) {
        // Fast skip: if source contains no `\-`, nothing to rewrite.
        if (source.indexOf("\\-") < 0 && source.indexOf("/v") < 0) return source;
        // State-machine walk over source. A backtracking regex on the
        // same task catastrophically blows up on 10KB+ inputs with many
        // backslash sequences; the linear walk is bounded O(n).
        // Tokens we skip safely as a single unit:
        //   - // line comments
        //   - /* block comments */
        //   - "..." '...' `...` string templates (no template-expression
        //     awareness — close on matching quote; sufficient for our
        //     ASCII-only regex-literal rewriting goal).
        // For each `/` in regex-start context, scan body until unescaped
        // closing `/` outside char class, read flags, rewrite if /u/v.
        let out = "";
        let emit = 0;
        let i = 0;
        const n = source.length;
        while (i < n) {
            const c = source.charCodeAt(i);
            // String literal
            if (c === 0x22 /* " */ || c === 0x27 /* ' */ || c === 0x60 /* ` */) {
                const q = c;
                i++;
                while (i < n) {
                    const d = source.charCodeAt(i);
                    if (d === 0x5C /* \ */ && i + 1 < n) { i += 2; continue; }
                    i++;
                    if (d === q) break;
                }
                continue;
            }
            // Line comment
            if (c === 0x2F /* / */ && i + 1 < n && source.charCodeAt(i + 1) === 0x2F) {
                while (i < n && source.charCodeAt(i) !== 0x0A) i++;
                continue;
            }
            // Block comment
            if (c === 0x2F && i + 1 < n && source.charCodeAt(i + 1) === 0x2A) {
                i += 2;
                while (i + 1 < n && !(source.charCodeAt(i) === 0x2A && source.charCodeAt(i + 1) === 0x2F)) i++;
                if (i + 1 < n) i += 2;
                continue;
            }
            // Possible regex literal
            if (c === 0x2F) {
                let j = i;
                while (j > emit && (source.charCodeAt(j - 1) === 0x20 || source.charCodeAt(j - 1) === 0x09)) j--;
                const prev = j === 0 ? 0x0A : source.charCodeAt(j - 1);
                // regex-context preceding chars
                const ok_prev =
                    prev === 0x28 || prev === 0x2C || prev === 0x3D || prev === 0x3A ||
                    prev === 0x5B || prev === 0x21 || prev === 0x26 || prev === 0x7C ||
                    prev === 0x3F || prev === 0x7B || prev === 0x3B || prev === 0x2B ||
                    prev === 0x2D || prev === 0x2A || prev === 0x25 || prev === 0x5E ||
                    prev === 0x7E || prev === 0x3C || prev === 0x3E ||
                    prev === 0x0A || prev === 0x0D || prev === 0x2F || prev === 0x5C;
                if (ok_prev) {
                    const bodyStart = i + 1;
                    let k = bodyStart;
                    let inCls = false;
                    let okScan = false;
                    while (k < n) {
                        const b = source.charCodeAt(k);
                        if (b === 0x5C && k + 1 < n) { k += 2; continue; }
                        if (b === 0x0A) break;
                        if (b === 0x5B) { inCls = true; k++; continue; }
                        if (b === 0x5D) { inCls = false; k++; continue; }
                        if (b === 0x2F && !inCls) { okScan = true; break; }
                        k++;
                    }
                    if (okScan) {
                        const bodyEnd = k;
                        let f = bodyEnd + 1;
                        const FLAGS = "gimsuyvd";
                        while (f < n && FLAGS.indexOf(source[f]) >= 0) f++;
                        const flags = source.substring(bodyEnd + 1, f);
                        if (flags.indexOf("u") >= 0 || flags.indexOf("v") >= 0) {
                            // Rewrite body.
                            const body = source.substring(bodyStart, bodyEnd);
                            let rew = "";
                            let sl = 0;
                            let inCls2 = false;
                            let bi = 0;
                            while (bi < body.length) {
                                const bc = body.charCodeAt(bi);
                                if (bc === 0x5C && bi + 1 < body.length) {
                                    const nx = body.charCodeAt(bi + 1);
                                    if (inCls2 && nx === 0x2D) {
                                        rew += body.substring(sl, bi);
                                        rew += "\\u002D";
                                        bi += 2;
                                        sl = bi;
                                        continue;
                                    }
                                    bi += 2;
                                    continue;
                                }
                                if (bc === 0x5B && !inCls2) inCls2 = true;
                                else if (bc === 0x5D && inCls2) inCls2 = false;
                                bi++;
                            }
                            rew += body.substring(sl);
                            // /v → /u when body has no set notation (-- or &&).
                            // QuickJS doesn't support /v but the in-basin
                            // regex patterns rarely use set operators.
                            let outFlags = flags;
                            if (flags.indexOf("v") >= 0
                                && body.indexOf("--") < 0
                                && body.indexOf("&&") < 0) {
                                outFlags = flags.replace("v", "u");
                            }
                            out += source.substring(emit, i) + "/" + rew + "/" + outFlags;
                            emit = f;
                            i = f;
                            continue;
                        }
                        // No /u/v — keep verbatim, advance past it.
                        i = f;
                        continue;
                    }
                }
            }
            i++;
        }
        out += source.substring(emit);
        return out;
    }

    // Rename reserved-keyword class fields in CJS sources before eval.
    // Mirrors the FsLoader-side Rust preprocessor for the ESM path.
    function fixReservedClassFields(source) {
        if (!/[\s;{](?:static|set|get|delete)[\s(=;]/.test(source)) return source;
        const lines = source.split("\n");
        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            let p = 0;
            while (p < line.length && (line[p] === " " || line[p] === "\t")) p++;
            if (p === 0) continue;
            const rest = line.substring(p);
            if (rest === "set;" || rest === "get;" || rest === "delete;") {
                lines[i] = line.substring(0, p) + "_" + rest;
                continue;
            }
            // Arrow-init class-field with reserved name. Heuristic guard
            // against vitest-class non-class-body assignments (`var set;
            // set = copyPrototype(...);` inside requireSet()): require
            // the indent to be small (typical class-body indent ≤ 4
            // spaces / 1 tab), AND require the rest to look like an
            // arrow function or value-init (`= (` or `= function` or
            // a non-reference RHS). vitest's pattern is `set = ` with
            // a single-token RHS continuing on the same line, which
            // does match — so add: skip rename if there is a top-level
            // `var <kw>;` declaration anywhere in the source. That
            // declares <kw> as a captured binding, signaling non-class.
            if (rest.startsWith("set =") || rest.startsWith("set=")
                || rest.startsWith("get =") || rest.startsWith("get=")
                || rest.startsWith("delete =") || rest.startsWith("delete=")) {
                const kw = rest.startsWith("set") ? "set"
                         : rest.startsWith("get") ? "get" : "delete";
                // If the source has a top-level `var <kw>;` or `let <kw>;`
                // or `const <kw>;` declaration, the assignment is a
                // captured-binding write, not a class field. Skip rename.
                const declRe = new RegExp("(?:^|[\\s;])(?:var|let|const)\\s+" + kw + "(?:\\s*[;,=]|\\s*$)", "m");
                if (declRe.test(source)) continue;
                lines[i] = line.substring(0, p) + "_" + rest;
                continue;
            }
            if (rest.startsWith("static(")) {
                lines[i] = line.substring(0, p) + "_" + rest;
                continue;
            }
        }
        return lines.join("\n");
    }
    function readSourceUtf8(absPath) {
        return fixReservedClassFields(fixRegexUEscapes(fs.readFileSyncUtf8(absPath)));
    }

    function pathExists(absPath) {
        return fs.existsSync(absPath);
    }

    // Pure-JS path utilities (we don't want a dependency on the `path`
    // wiring exhibiting a circular load order).
    function dirname(p) {
        const i = p.lastIndexOf("/");
        if (i < 0) return ".";
        if (i === 0) return "/";
        return p.substring(0, i);
    }

    function basename(p) {
        const i = p.lastIndexOf("/");
        return i < 0 ? p : p.substring(i + 1);
    }

    function joinPath(a, b) {
        if (b.startsWith("/")) return b;
        if (a.endsWith("/")) return a + b;
        return a + "/" + b;
    }

    function normalizePath(p) {
        const isAbsolute = p.startsWith("/");
        const segments = p.split("/").filter((s) => s.length > 0);
        const out = [];
        for (const seg of segments) {
            if (seg === ".") continue;
            if (seg === "..") {
                if (out.length > 0 && out[out.length - 1] !== "..") out.pop();
                else if (!isAbsolute) out.push("..");
            } else {
                out.push(seg);
            }
        }
        return (isAbsolute ? "/" : "") + out.join("/");
    }

    const EXTENSIONS = ["", ".js", ".json", ".cjs"];

    function tryExtensions(absPath) {
        for (const ext of EXTENSIONS) {
            const candidate = absPath + ext;
            if (fs.isFileSync(candidate)) return candidate;
        }
        return null;
    }

    // Resolve a conditional-exports value (string or recursive object)
    // against a priority-ordered conditions list. Mirrors the Rust-side
    // resolve_exports_value so ESM and CJS pick the same entry.
    function resolveExportsValue(value, conditions) {
        if (typeof value === "string") return value;
        if (value && typeof value === "object") {
            for (const cond of conditions) {
                if (Object.prototype.hasOwnProperty.call(value, cond)) {
                    const r = resolveExportsValue(value[cond], conditions);
                    if (r) return r;
                }
            }
            if (Object.prototype.hasOwnProperty.call(value, "default")) {
                return resolveExportsValue(value["default"], conditions);
            }
        }
        return null;
    }

    function tryDirectoryWithIndex(absDir) {
        const pkgJson = absDir + "/package.json";
        if (pathExists(pkgJson)) {
            try {
                const pkg = JSON.parse(readSourceUtf8(pkgJson));
                // exports field — string or object. Recursively walk
                // conditional keys: bun → require → node → default
                // matches Bun's CJS resolution order.
                if (pkg.exports) {
                    const conditions = ["bun", "require", "node", "module", "default"];
                    let root = null;
                    if (typeof pkg.exports === "string") {
                        root = pkg.exports;
                    } else if (typeof pkg.exports === "object") {
                        if (Object.prototype.hasOwnProperty.call(pkg.exports, ".")) {
                            root = pkg.exports["."];
                        } else {
                            // No subpath keys (no leading "." entries) →
                            // treat the whole object as a conditional set.
                            const hasSubpath = Object.keys(pkg.exports).some(k => k.startsWith("."));
                            if (!hasSubpath) root = pkg.exports;
                        }
                    }
                    if (root != null) {
                        const rel = resolveExportsValue(root, conditions);
                        if (rel) {
                            const target = normalizePath(joinPath(absDir, rel.replace(/^\.\//, "")));
                            const asFile = tryExtensions(target);
                            if (asFile) return asFile;
                            if (pathExists(target)) return target;
                        }
                    }
                }
                // main / module fields.
                const mainStr = pkg.main || pkg.module;
                if (typeof mainStr === "string") {
                    const target = normalizePath(joinPath(absDir, mainStr));
                    const resolved = tryExtensions(target);
                    if (resolved) return resolved;
                    // main might point at a directory — walk into it for
                    // its index.js (Node's "./node" → "./node/index.js"
                    // resolution; @dabh/diagnostics uses this pattern).
                    if (fs.isDirectorySync(target)) {
                        for (const idx of ["/index.js", "/index.json", "/index.cjs"]) {
                            const candidate = target + idx;
                            if (pathExists(candidate)) return candidate;
                        }
                    }
                }
            } catch (e) {
                // Ignore malformed package.json; fall through to index.
            }
        }
        // Default index files.
        for (const idx of ["/index.js", "/index.json", "/index.cjs"]) {
            const candidate = absDir + idx;
            if (pathExists(candidate)) return candidate;
        }
        return null;
    }

    function resolvePath(specifier, fromDir) {
        // node:* and bare builtin names short-circuit to the builtin path.
        if (Object.prototype.hasOwnProperty.call(NODE_BUILTINS, specifier)) {
            return specifier;  // returned literally; loadModule recognizes it
        }
        // Relative specifier.
        if (specifier.startsWith("./") || specifier.startsWith("../") || specifier === "." || specifier === "..") {
            const joined = normalizePath(joinPath(fromDir, specifier));
            // Try as file first.
            const asFile = tryExtensions(joined);
            if (asFile) return asFile;
            // Then as directory.
            const asDir = tryDirectoryWithIndex(joined);
            if (asDir) return asDir;
            throw new Error("Cannot find module '" + specifier + "' from " + fromDir);
        }
        // Absolute specifier.
        if (specifier.startsWith("/")) {
            const asFile = tryExtensions(specifier);
            if (asFile) return asFile;
            const asDir = tryDirectoryWithIndex(specifier);
            if (asDir) return asDir;
            throw new Error("Cannot find module '" + specifier + "'");
        }
        // Package-internal subpath imports (charCode 35 prefix per Node
        // spec). Chalk and similar libs use this for bundled vendor copies.
        if (specifier.charCodeAt(0) === 35) {
            let dir = fromDir;
            while (true) {
                const pkgJsonPath = dir + '/package.json';
                if (pathExists(pkgJsonPath)) {
                    try {
                        const pkg = JSON.parse(readSourceUtf8(pkgJsonPath));
                        if (pkg.imports) {
                            const conditions = ['bun', 'require', 'node', 'module', 'default'];
                            if (Object.prototype.hasOwnProperty.call(pkg.imports, specifier)) {
                                const rel = resolveExportsValue(pkg.imports[specifier], conditions);
                                if (rel) {
                                    const target = normalizePath(joinPath(dir, rel.replace(/^\.\//, '')));
                                    const asFile = tryExtensions(target);
                                    if (asFile) return asFile;
                                    if (pathExists(target)) return target;
                                }
                            }
                            for (const k of Object.keys(pkg.imports)) {
                                if (k.endsWith('/*')) {
                                    const prefix = k.substring(0, k.length - 2);
                                    if (specifier.startsWith(prefix) && specifier.length > prefix.length) {
                                        const suffix = specifier.substring(prefix.length + 1);
                                        const rel = resolveExportsValue(pkg.imports[k], conditions);
                                        if (rel) {
                                            const target = normalizePath(joinPath(dir, rel.replace('*', suffix).replace(/^\.\//, '')));
                                            const asFile = tryExtensions(target);
                                            if (asFile) return asFile;
                                            if (pathExists(target)) return target;
                                        }
                                    }
                                }
                            }
                            break;
                        }
                    } catch (e) { /* fall through */ }
                }
                if (dir === '/' || dir === '' || dir === '.') break;
                const parent = dirname(dir);
                if (parent === dir) break;
                dir = parent;
            }
            throw new Error("Cannot find module '" + specifier + "' from " + fromDir);
        }
        // Bare specifier — walk up node_modules.
        // Split into pkg + subpath: "pkg" or "pkg/sub" or "@scope/pkg/sub".
        let pkgEnd;
        if (specifier.startsWith("@")) {
            const firstSlash = specifier.indexOf("/");
            if (firstSlash < 0) throw new Error("Invalid scoped specifier: " + specifier);
            const secondSlash = specifier.indexOf("/", firstSlash + 1);
            pkgEnd = secondSlash < 0 ? specifier.length : secondSlash;
        } else {
            const firstSlash = specifier.indexOf("/");
            pkgEnd = firstSlash < 0 ? specifier.length : firstSlash;
        }
        const pkgName = specifier.substring(0, pkgEnd);
        const subPath = specifier.substring(pkgEnd);  // includes leading / or ""

        let dir = fromDir;
        while (true) {
            const pkgRoot = joinPath(dir, "node_modules/" + pkgName);
            if (pathExists(pkgRoot)) {
                if (subPath.length > 0) {
                    // First: check pkg.exports for an explicit subpath
                    // or `./*` pattern (Node spec). stream-chain uses
                    // `./*: ./src/*` to remap utils/X to src/utils/X.
                    const pkgJsonPath = pkgRoot + "/package.json";
                    if (pathExists(pkgJsonPath)) {
                        try {
                            const pkg = JSON.parse(readSourceUtf8(pkgJsonPath));
                            if (pkg.exports && typeof pkg.exports === "object") {
                                const conditions = ["bun", "require", "node", "module", "default"];
                                const subKey = "." + subPath;  // subPath has leading /
                                if (Object.prototype.hasOwnProperty.call(pkg.exports, subKey)) {
                                    const rel = resolveExportsValue(pkg.exports[subKey], conditions);
                                    if (rel) {
                                        const target = normalizePath(joinPath(pkgRoot, rel.replace(/^\.\//, "")));
                                        const asFile = tryExtensions(target);
                                        if (asFile) return asFile;
                                        if (pathExists(target)) return target;
                                    }
                                }
                                for (const k of Object.keys(pkg.exports)) {
                                    if (k.endsWith("/*")) {
                                        const prefix = k.substring(0, k.length - 2);
                                        if (subKey.startsWith(prefix) && subKey.length > prefix.length) {
                                            const suffix = subKey.substring(prefix.length + 1);
                                            const rel = resolveExportsValue(pkg.exports[k], conditions);
                                            if (rel) {
                                                const target = normalizePath(joinPath(pkgRoot, rel.replace("*", suffix).replace(/^\.\//, "")));
                                                const asFile = tryExtensions(target);
                                                if (asFile) return asFile;
                                                if (pathExists(target)) return target;
                                            }
                                        }
                                    }
                                }
                            }
                        } catch (_) { /* fall through */ }
                    }
                    const target = normalizePath(pkgRoot + subPath);
                    const asFile = tryExtensions(target);
                    if (asFile) return asFile;
                    const asDir = tryDirectoryWithIndex(target);
                    if (asDir) return asDir;
                } else {
                    const asDir = tryDirectoryWithIndex(pkgRoot);
                    if (asDir) return asDir;
                }
            }
            if (dir === "/" || dir === "" || dir === ".") break;
            const parent = dirname(dir);
            if (parent === dir) break;
            dir = parent;
        }
        throw new Error("Cannot find module '" + specifier + "' from " + fromDir);
    }

    const moduleCache = Object.create(null);

    // node: scheme builtins. Real Bun resolves these to native modules; we
    // map to the wired host globals. Reachable via require("node:fs") etc.
    const NODE_BUILTINS = {
        "node:fs": () => globalThis.fs,
        "fs": () => globalThis.fs,
        "node:path": () => globalThis.path,
        "path": () => globalThis.path,
        "node:http": () => globalThis.nodeHttp,
        "http": () => globalThis.nodeHttp,
        "node:crypto": () => globalThis.crypto,
        "crypto": () => globalThis.crypto,
        "node:buffer": () => ({ Buffer: globalThis.Buffer }),
        "buffer": () => ({ Buffer: globalThis.Buffer }),
        "node:os": () => globalThis.os,
        "os": () => globalThis.os,
        "node:process": () => globalThis.process,
        "process": () => globalThis.process,
        "node:dns": () => globalThis.nodeDns,
        "dns": () => globalThis.nodeDns,
        "node:dns/promises": () => globalThis.nodeDnsPromises,
        "dns/promises": () => globalThis.nodeDnsPromises,
        "node:events": () => globalThis.nodeEvents,
        "events": () => globalThis.nodeEvents,
        "node:util": () => globalThis.nodeUtil,
        "util": () => globalThis.nodeUtil,
        "node:util/types": () => globalThis.nodeUtilTypes,
        "util/types": () => globalThis.nodeUtilTypes,
        "node:stream": () => globalThis.nodeStream,
        "stream": () => globalThis.nodeStream,
        "node:stream/promises": () => globalThis.nodeStreamPromises,
        "stream/promises": () => globalThis.nodeStreamPromises,
        "node:querystring": () => globalThis.nodeQuerystring,
        "querystring": () => globalThis.nodeQuerystring,
        "node:assert": () => globalThis.nodeAssert,
        "assert": () => globalThis.nodeAssert,
        "node:assert/strict": () => globalThis.nodeAssertStrict,
        "assert/strict": () => globalThis.nodeAssertStrict,
        "node:url": () => globalThis.nodeUrl,
        "url": () => globalThis.nodeUrl,
        // node:child_process — minimal shim. Many libs (commander, others)
        // top-level-require this even when they don't use it. spawnSync
        // composes on Bun.spawnSync; spawn/exec/execSync/fork throw if
        // called (not yet wired). Modules that only top-level-import for
        // optional features (commander's executable subcommands) work.
        "node:child_process": () => globalThis.nodeChildProcess,
        "child_process": () => globalThis.nodeChildProcess,
        "node:net": () => globalThis.nodeNet,
        "net": () => globalThis.nodeNet,
        "node:tty": () => globalThis.nodeTty,
        "tty": () => globalThis.nodeTty,
        "node:zlib": () => globalThis.nodeZlib,
        "zlib": () => globalThis.nodeZlib,
        "node:diagnostics_channel": () => globalThis.nodeDiagnosticsChannel,
        "diagnostics_channel": () => globalThis.nodeDiagnosticsChannel,
        "node:https": () => globalThis.nodeHttps,
        "https": () => globalThis.nodeHttps,
        "node:perf_hooks": () => globalThis.nodePerfHooks,
        "perf_hooks": () => globalThis.nodePerfHooks,
        "node:async_hooks": () => globalThis.nodeAsyncHooks,
        "async_hooks": () => globalThis.nodeAsyncHooks,
        "node:timers": () => globalThis.nodeTimers,
        "timers": () => globalThis.nodeTimers,
        "node:timers/promises": () => globalThis.nodeTimersPromises,
        "timers/promises": () => globalThis.nodeTimersPromises,
        "node:console": () => globalThis.nodeConsoleModule,
        "console": () => globalThis.nodeConsoleModule,
        "node:fs/promises": () => globalThis.nodeFsPromises,
        "fs/promises": () => globalThis.nodeFsPromises,
        "node:stream/web": () => globalThis.nodeStreamWeb,
        "stream/web": () => globalThis.nodeStreamWeb,
        "node:test": () => globalThis.nodeTest,
        "test": () => globalThis.nodeTest,
        "node:worker_threads": () => globalThis.nodeWorkerThreads,
        "worker_threads": () => globalThis.nodeWorkerThreads,
        "node:http2": () => globalThis.nodeHttp2,
        "http2": () => globalThis.nodeHttp2,
        "node:vm": () => globalThis.nodeVm,
        "vm": () => globalThis.nodeVm,
        "node:string_decoder": () => globalThis.nodeStringDecoder,
        "string_decoder": () => globalThis.nodeStringDecoder,
        "node:readline": () => globalThis.nodeReadline,
        "readline": () => globalThis.nodeReadline,
        "node:readline/promises": () => globalThis.nodeReadlinePromises,
        "readline/promises": () => globalThis.nodeReadlinePromises,
        "node:module": () => globalThis.nodeModule,
        "module": () => globalThis.nodeModule,
        "node:cluster": () => globalThis.nodeCluster,
        "cluster": () => globalThis.nodeCluster,
        "node:tls": () => globalThis.nodeTls,
        "tls": () => globalThis.nodeTls,
        "node:v8": () => globalThis.nodeV8,
        "v8": () => globalThis.nodeV8,
        "node:constants": () => globalThis.nodeConstants,
        "constants": () => globalThis.nodeConstants,
    };

    function loadModule(absPath) {
        if (Object.prototype.hasOwnProperty.call(NODE_BUILTINS, absPath)) {
            return NODE_BUILTINS[absPath]();
        }
        if (moduleCache[absPath]) return moduleCache[absPath].exports;

        const moduleObj = {
            exports: {},
            id: absPath,
            filename: absPath,
            loaded: false,
            children: [],
        };
        // Cache BEFORE evaluating, so cycles see partial exports.
        moduleCache[absPath] = moduleObj;

        const source = readSourceUtf8(absPath);

        // .json modules: parse and assign.
        if (absPath.endsWith(".json")) {
            moduleObj.exports = JSON.parse(source);
            moduleObj.loaded = true;
            return moduleObj.exports;
        }

        const dir = dirname(absPath);
        const requireFn = function require(spec) {
            const resolved = resolvePath(spec, dir);
            return loadModule(resolved);
        };
        requireFn.cache = moduleCache;
        requireFn.resolve = function (spec) {
            return resolvePath(spec, dir);
        };

        // Wrap source per Node's module wrapper.
        const wrapper = "(function (exports, require, module, __filename, __dirname) { " +
            source +
            "\n})";
        try {
            const fn = (0, eval)(wrapper);
            fn(moduleObj.exports, requireFn, moduleObj, absPath, dir);
            moduleObj.loaded = true;
        } catch (e) {
            // Remove from cache so that a retry isn't poisoned.
            delete moduleCache[absPath];
            if (globalThis.__cjsLoadTrace) globalThis.__cjsLoadTrace(absPath, e);
            throw e;
        }
        return moduleObj.exports;
    }

    globalThis.bootRequire = function bootRequire(absPath) {
        return loadModule(absPath);
    };
    // Expose resolution + cache for tests/diagnostics.
    globalThis.__cjs = { resolvePath, moduleCache, loadModule, NODE_BUILTINS };

    // Top-level globalThis.require — Bun exposes require() in ESM modules
    // for compatibility with libraries that conditionally use either
    // import or require (e.g., ulid checks `typeof window` then falls
    // through to `require("crypto")`). Bare specifiers matching node:
    // builtins resolve via NODE_BUILTINS; other paths route through
    // loadModule from the cwd.
    globalThis.require = function require(spec) {
        if (typeof spec !== "string") {
            throw new TypeError("require: spec must be a string");
        }
        // Node-builtin shortcut.
        if (Object.prototype.hasOwnProperty.call(NODE_BUILTINS, spec)) {
            return NODE_BUILTINS[spec]();
        }
        // Resolve from cwd. ESM modules have no caller-dir context; cwd
        // is the only sane default.
        const cwd = (globalThis.process && globalThis.process.cwd && globalThis.process.cwd()) || "/";
        const resolved = resolvePath(spec, cwd);
        return loadModule(resolved);
    };
    globalThis.require.cache = moduleCache;
    globalThis.require.resolve = function (spec) {
        if (Object.prototype.hasOwnProperty.call(NODE_BUILTINS, spec)) return spec;
        const cwd = (globalThis.process && globalThis.process.cwd && globalThis.process.cwd()) || "/";
        return resolvePath(spec, cwd);
    };
})();
"#;

fn install_websocket_class_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    // Π1.5.c: JS-side WebSocket class.
    //
    // WHATWG WebSocket interface
    // (https://html.spec.whatwg.org/multipage/web-sockets.html) shape:
    //   new WebSocket(url, protocols?) -> ws
    //   ws.readyState ∈ {CONNECTING=0, OPEN=1, CLOSING=2, CLOSED=3}
    //   ws.send(data); ws.close(code?, reason?)
    //   ws.onopen / onmessage / onclose / onerror; addEventListener
    //   ws.url, ws.protocol, ws.extensions, ws.binaryType, ws.bufferedAmount
    //
    // Tier-3 implementation-contingent divergence from real Bun per
    // seed C1: the connection driver runs synchronously during
    // construction (open TCP + Upgrade handshake + verify Accept),
    // since rusty-bun-host has no real event loop. The result is that
    // by the time `new WebSocket(url)` returns, readyState is either
    // OPEN (success) or CLOSED (failure). onopen fires via microtask
    // after construction returns. Real Bun async-emits 'open' from
    // its event loop; consumer code awaiting `await new Promise(r =>
    // ws.onopen = r)` works under both implementations.
    //
    // Per seed §A8.16: WebSocket reuses globalThis.TCP / __tls — the
    // process-global state is already guarded by those layers.
    ctx.eval::<(), _>(r#"
        (function() {
            const ws_ns = globalThis.__ws;
            if (!ws_ns) return;  // Π1.5.b not yet wired; class unavailable.

            class WebSocket {
                static get CONNECTING() { return 0; }
                static get OPEN() { return 1; }
                static get CLOSING() { return 2; }
                static get CLOSED() { return 3; }

                constructor(url, protocols) {
                    const u = new URL(url);
                    // Per Bun-shape (lax): http:// and https:// are silently
                    // mapped to ws:// and wss:// respectively. Real WHATWG
                    // says SyntaxError but Bun is permissive.
                    let isSecure;
                    if (u.protocol === "ws:") isSecure = false;
                    else if (u.protocol === "wss:") isSecure = true;
                    else if (u.protocol === "http:") isSecure = false;
                    else if (u.protocol === "https:") isSecure = true;
                    else {
                        throw new SyntaxError("WebSocket: unsupported scheme " + u.protocol);
                    }
                    this._url = u.href;
                    this._isSecure = isSecure;
                    this._readyState = 0;  // CONNECTING
                    this._protocol = "";
                    this._extensions = "";
                    this._binaryType = "blob";
                    this._bufferedAmount = 0;
                    this._listeners = Object.create(null);
                    this._sid = null;
                    this._readBuf = [];   // accumulator of bytes from the transport
                    this._isTls = this._isSecure;

                    // Synchronous connect + handshake (Tier-3 divergence
                    // documented above).
                    try {
                        this._open(u, protocols);
                        this._readyState = 1;  // OPEN
                        // Π1.5.e: register in __keepAlive so the eval loop
                        // auto-pumps the receive path between microtasks.
                        // Real Bun runs this in a background task; we
                        // simulate by hooking into the existing keep-alive
                        // registry (Π2.6 infrastructure). The consumer
                        // pattern `ws.onmessage = cb; ...` now works without
                        // explicit pump() calls.
                        if (globalThis.__keepAlive) globalThis.__keepAlive.add(this);
                        queueMicrotask(() => this._dispatch("open", { type: "open" }));
                    } catch (e) {
                        this._readyState = 3;  // CLOSED
                        const err = e instanceof Error ? e : new Error(String(e));
                        queueMicrotask(() => {
                            this._dispatch("error", { type: "error", message: err.message });
                            this._dispatch("close", { type: "close", code: 1006, reason: err.message, wasClean: false });
                        });
                    }
                }

                _open(u, protocols) {
                    let host = u.hostname;
                    if (host === "localhost") host = "127.0.0.1";
                    const port = u.port ? parseInt(u.port, 10) : (this._isSecure ? 443 : 80);
                    const path = u.pathname + (u.search || "");

                    // Generate Sec-WebSocket-Key + expected Accept.
                    const key = ws_ns.generate_key();
                    const expectedAccept = ws_ns.derive_accept(key);

                    // Build the Upgrade request.
                    const headers = [
                        ["Host", host + ":" + port],
                        ["Upgrade", "websocket"],
                        ["Connection", "Upgrade"],
                        ["Sec-WebSocket-Key", key],
                        ["Sec-WebSocket-Version", "13"],
                    ];
                    if (protocols) {
                        const pList = Array.isArray(protocols) ? protocols.join(", ") : String(protocols);
                        headers.push(["Sec-WebSocket-Protocol", pList]);
                    }
                    const reqBytes = globalThis.HTTP.serializeRequest("GET", path, headers, []);

                    // Open the transport.
                    if (this._isTls) {
                        // Read CA bundle the same way fetch() does.
                        const env = (globalThis.process && globalThis.process.env) || {};
                        const caCandidates = [];
                        if (env.RUSTY_BUN_CA) caCandidates.push(env.RUSTY_BUN_CA);
                        if (env.NODE_EXTRA_CA_CERTS) caCandidates.push(env.NODE_EXTRA_CA_CERTS);
                        caCandidates.push("/etc/ssl/certs/ca-certificates.crt",
                                          "/etc/pki/tls/certs/ca-bundle.crt",
                                          "/etc/ssl/cert.pem",
                                          "/etc/ssl/ca-bundle.pem");
                        let caPem = "";
                        for (const p of caCandidates) {
                            try {
                                caPem = globalThis.fs.readFileSyncUtf8(p);
                                if (caPem.length > 0) break;
                            } catch (_) {}
                        }
                        if (!caPem) {
                            throw new TypeError("WebSocket: no CA bundle found for wss://");
                        }
                        this._sid = globalThis.__tls.connect(host, port, caPem);
                        const toFfiBytes = (b) => {
                            const arr = new Array(b.length);
                            for (let i = 0; i < b.length; i++) arr[i] = b[i];
                            return arr;
                        };
                        globalThis.__tls.write(this._sid, toFfiBytes(reqBytes));
                    } else {
                        this._sid = globalThis.TCP.connect(host + ":" + port);
                        globalThis.TCP.writeAll(this._sid, reqBytes);
                    }

                    // Read the upgrade response. Server sends a single HTTP/1.1
                    // 101 Switching Protocols followed by the empty body. The
                    // body may contain post-handshake frame bytes; we accumulate
                    // everything past the header into _readBuf for the frame
                    // pump.
                    const acc = [];
                    let bodyStart = -1;
                    while (bodyStart < 0) {
                        let chunk;
                        if (this._isTls) {
                            try { chunk = globalThis.__tls.read(this._sid); } catch (_) { chunk = new Uint8Array(0); }
                        } else {
                            chunk = globalThis.TCP.read(this._sid, 65536);
                        }
                        if (chunk.length === 0) {
                            throw new Error("WebSocket: connection closed during handshake");
                        }
                        for (let i = 0; i < chunk.length; i++) acc.push(chunk[i]);
                        // Look for \r\n\r\n which terminates HTTP headers.
                        for (let i = 3; i < acc.length; i++) {
                            if (acc[i - 3] === 13 && acc[i - 2] === 10 &&
                                acc[i - 1] === 13 && acc[i] === 10) {
                                bodyStart = i + 1;
                                break;
                            }
                        }
                    }
                    const headerBytes = new Uint8Array(acc.slice(0, bodyStart));
                    const parsed = globalThis.HTTP.parseResponse(headerBytes);
                    if (parsed.status !== 101) {
                        throw new Error("WebSocket: expected 101 Switching Protocols, got " + parsed.status);
                    }
                    const accept = parsed.headers.find(h => h[0].toLowerCase() === "sec-websocket-accept");
                    if (!accept || !ws_ns.verify_accept(key, accept[1])) {
                        throw new Error("WebSocket: Sec-WebSocket-Accept mismatch");
                    }
                    const proto = parsed.headers.find(h => h[0].toLowerCase() === "sec-websocket-protocol");
                    if (proto) this._protocol = proto[1];

                    // Any bytes past the headers belong to subsequent frames.
                    for (let i = bodyStart; i < acc.length; i++) this._readBuf.push(acc[i]);
                }

                get url() { return this._url; }
                get readyState() { return this._readyState; }
                get protocol() { return this._protocol; }
                get extensions() { return this._extensions; }
                get bufferedAmount() { return this._bufferedAmount; }
                get binaryType() { return this._binaryType; }
                set binaryType(v) {
                    if (v !== "blob" && v !== "arraybuffer") {
                        throw new SyntaxError("WebSocket.binaryType must be 'blob' or 'arraybuffer'");
                    }
                    this._binaryType = v;
                }

                addEventListener(event, listener) {
                    if (!this._listeners[event]) this._listeners[event] = [];
                    this._listeners[event].push(listener);
                }
                removeEventListener(event, listener) {
                    if (!this._listeners[event]) return;
                    const i = this._listeners[event].indexOf(listener);
                    if (i >= 0) this._listeners[event].splice(i, 1);
                }
                _dispatch(event, ev) {
                    const arr = this._listeners[event] || [];
                    for (const l of arr) { try { l(ev); } catch (_) {} }
                    const propName = "on" + event;
                    if (typeof this[propName] === "function") {
                        try { this[propName](ev); } catch (_) {}
                    }
                }

                send(data) {
                    if (this._readyState !== 1) {
                        throw new Error("WebSocket: not OPEN (readyState=" + this._readyState + ")");
                    }
                    let payload;
                    let opcode;
                    if (typeof data === "string") {
                        payload = new TextEncoder().encode(data);
                        opcode = 0x1;
                    } else if (data instanceof Uint8Array) {
                        payload = data;
                        opcode = 0x2;
                    } else if (data instanceof ArrayBuffer) {
                        payload = new Uint8Array(data);
                        opcode = 0x2;
                    } else {
                        throw new TypeError("WebSocket.send: unsupported data type");
                    }
                    // Client → server: must mask. Generate a fresh mask per frame.
                    const maskBytes = new Uint8Array(4);
                    crypto.getRandomValues(maskBytes);
                    const payloadArr = new Array(payload.length);
                    for (let i = 0; i < payload.length; i++) payloadArr[i] = payload[i];
                    const maskArr = [maskBytes[0], maskBytes[1], maskBytes[2], maskBytes[3]];
                    const frameBytes = ws_ns.encode_frame(true, opcode, maskArr, payloadArr);
                    this._writeBytes(frameBytes);
                }

                close(code, reason) {
                    if (this._readyState === 2 || this._readyState === 3) return;
                    this._readyState = 2;  // CLOSING
                    try {
                        const closePayload = ws_ns.encode_close(code || 1000, reason || "");
                        const payloadArr = [];
                        for (let i = 0; i < closePayload.length; i++) payloadArr.push(closePayload[i]);
                        const maskBytes = new Uint8Array(4);
                        crypto.getRandomValues(maskBytes);
                        const maskArr = [maskBytes[0], maskBytes[1], maskBytes[2], maskBytes[3]];
                        const frameBytes = ws_ns.encode_frame(true, 0x8 /* Close */, maskArr, payloadArr);
                        this._writeBytes(frameBytes);
                    } catch (_) {}
                    this._teardownTransport();
                    this._readyState = 3;  // CLOSED
                    if (globalThis.__keepAlive) globalThis.__keepAlive.delete(this);
                    queueMicrotask(() => {
                        this._dispatch("close", {
                            type: "close", code: code || 1000, reason: reason || "", wasClean: true,
                        });
                    });
                }

                _writeBytes(bytes) {
                    const toFfiBytes = (b) => {
                        const arr = new Array(b.length);
                        for (let i = 0; i < b.length; i++) arr[i] = b[i];
                        return arr;
                    };
                    if (this._isTls) {
                        globalThis.__tls.write(this._sid, toFfiBytes(bytes));
                    } else {
                        globalThis.TCP.writeAll(this._sid, bytes);
                    }
                }

                _teardownTransport() {
                    try {
                        if (this._isTls) globalThis.__tls.close(this._sid);
                        else globalThis.TCP.close(this._sid);
                    } catch (_) {}
                    this._sid = null;
                }

                // pump(): synchronous frame pump. Reads available bytes from
                // the transport (one chunk), accumulates, decodes complete
                // frames, and dispatches events. Returns the number of frames
                // dispatched. Real Bun runs this loop in the background; the
                // host's single-threaded model has the consumer call this
                // explicitly (or the eval loop's keep-alive registry can pump
                // it; future Π1.5.d wires that).
                // __tick(): called by the eval loop's __keepAlive pump
                // between microtasks. Returns true if a frame was dispatched
                // (signals "did work" to the keep-alive scheduler).
                __tick(_maxWaitMs) {
                    return this.pump() > 0;
                }
                pump() {
                    if (this._readyState !== 1 && this._readyState !== 2) return 0;
                    let dispatched = 0;
                    // Read one chunk if available.
                    let chunk = null;
                    try {
                        if (this._isTls) chunk = globalThis.__tls.read(this._sid);
                        else chunk = globalThis.TCP.read(this._sid, 65536);
                    } catch (_) { chunk = new Uint8Array(0); }
                    if (chunk && chunk.length > 0) {
                        for (let i = 0; i < chunk.length; i++) this._readBuf.push(chunk[i]);
                    }
                    // Drain complete frames.
                    while (this._readBuf.length >= 2) {
                        const view = this._readBuf.slice(0, Math.min(this._readBuf.length, 65536));
                        let frameJson;
                        try { frameJson = ws_ns.decode_frame_json(view); }
                        catch (_) { break; }  // need more bytes
                        const f = JSON.parse(frameJson);
                        this._readBuf.splice(0, f.consumed);
                        if (f.opcode === 0x1 /* Text */) {
                            const text = new TextDecoder().decode(new Uint8Array(f.payload));
                            this._dispatch("message", { type: "message", data: text });
                        } else if (f.opcode === 0x2 /* Binary */) {
                            const data = this._binaryType === "arraybuffer"
                                ? new Uint8Array(f.payload).buffer
                                : new Blob([new Uint8Array(f.payload)]);
                            this._dispatch("message", { type: "message", data });
                        } else if (f.opcode === 0x8 /* Close */) {
                            this._teardownTransport();
                            this._readyState = 3;
                            if (globalThis.__keepAlive) globalThis.__keepAlive.delete(this);
                            this._dispatch("close", { type: "close", code: 1000, reason: "", wasClean: true });
                            break;
                        } else if (f.opcode === 0x9 /* Ping */) {
                            // Respond with Pong of same payload (RFC §5.5.3).
                            const maskBytes = new Uint8Array(4);
                            crypto.getRandomValues(maskBytes);
                            const pongBytes = ws_ns.encode_frame(true, 0xA, [maskBytes[0], maskBytes[1], maskBytes[2], maskBytes[3]], f.payload);
                            this._writeBytes(pongBytes);
                        }
                        // Pong ignored.
                        dispatched += 1;
                    }
                    return dispatched;
                }
            }

            globalThis.WebSocket = WebSocket;
        })();
    "#)?;
    Ok(())
}

fn install_node_events_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    // Π3.8: node:events. EventEmitter class is the canonical
    // npm-dependency primitive (most-imported builtin across the
    // ecosystem). Real Node + Bun export `EventEmitter` as the default
    // AND as a named export, plus module-level helpers `once` and `on`
    // for promise-bridging.
    //
    // Per M9 (spec-first): targeted against Bun's actual shape via test
    // probes; per §III.A8.2 the class is JS-side stateless-Rust-free.
    ctx.eval::<(), _>(r#"
        (function() {
            const DEFAULT_MAX = 10;

            function EventEmitter() {
                this._events = Object.create(null);
                this._maxListeners = undefined;
            }
            EventEmitter.defaultMaxListeners = DEFAULT_MAX;

            function _ensure(self) {
                if (!self._events) self._events = Object.create(null);
                return self._events;
            }

            EventEmitter.prototype.on = function on(event, listener) {
                if (typeof listener !== "function") {
                    throw new TypeError('The "listener" argument must be of type Function. Received type ' + typeof listener);
                }
                const events = _ensure(this);
                if (!events[event]) events[event] = [];
                // Fire 'newListener' before adding, per Node convention.
                if (events.newListener && event !== "newListener") {
                    this.emit("newListener", event, listener._original || listener);
                }
                events[event].push(listener);
                return this;
            };
            EventEmitter.prototype.addListener = EventEmitter.prototype.on;

            EventEmitter.prototype.once = function once(event, listener) {
                if (typeof listener !== "function") {
                    throw new TypeError('The "listener" argument must be of type Function.');
                }
                const self = this;
                const wrapped = function(...args) {
                    self.off(event, wrapped);
                    listener.apply(self, args);
                };
                wrapped._original = listener;
                return self.on(event, wrapped);
            };

            EventEmitter.prototype.prependListener = function prependListener(event, listener) {
                if (typeof listener !== "function") throw new TypeError("listener must be function");
                const events = _ensure(this);
                if (!events[event]) events[event] = [];
                if (events.newListener && event !== "newListener") {
                    this.emit("newListener", event, listener._original || listener);
                }
                events[event].unshift(listener);
                return this;
            };

            EventEmitter.prototype.prependOnceListener = function prependOnceListener(event, listener) {
                const self = this;
                const wrapped = function(...args) {
                    self.off(event, wrapped);
                    listener.apply(self, args);
                };
                wrapped._original = listener;
                return self.prependListener(event, wrapped);
            };

            EventEmitter.prototype.off = function off(event, listener) {
                const events = _ensure(this);
                const arr = events[event];
                if (!arr) return this;
                for (let i = arr.length - 1; i >= 0; i--) {
                    if (arr[i] === listener || arr[i]._original === listener) {
                        arr.splice(i, 1);
                        if (events.removeListener) {
                            this.emit("removeListener", event, listener);
                        }
                        break;
                    }
                }
                if (arr.length === 0) delete events[event];
                return this;
            };
            EventEmitter.prototype.removeListener = EventEmitter.prototype.off;

            EventEmitter.prototype.removeAllListeners = function removeAllListeners(event) {
                const events = _ensure(this);
                if (event === undefined) {
                    this._events = Object.create(null);
                } else {
                    delete events[event];
                }
                return this;
            };

            EventEmitter.prototype.emit = function emit(event, ...args) {
                const events = _ensure(this);
                const arr = events[event];
                // 'error' event with no listener throws per Node convention.
                if (event === "error" && (!arr || arr.length === 0)) {
                    const err = args[0];
                    if (err instanceof Error) throw err;
                    const e = new Error("Unhandled error.");
                    e.context = err;
                    throw e;
                }
                if (!arr || arr.length === 0) return false;
                const copy = arr.slice();
                for (const l of copy) {
                    try { l.apply(this, args); } catch (e) {
                        // Re-emit as 'error' if a listener throws and 'error' has handlers.
                        const errArr = events.error;
                        if (errArr && errArr.length > 0 && event !== "error") {
                            this.emit("error", e);
                        } else {
                            throw e;
                        }
                    }
                }
                return true;
            };

            EventEmitter.prototype.listenerCount = function listenerCount(event) {
                const events = _ensure(this);
                return events[event] ? events[event].length : 0;
            };

            EventEmitter.prototype.listeners = function listeners(event) {
                const events = _ensure(this);
                if (!events[event]) return [];
                return events[event].map(l => l._original || l);
            };

            EventEmitter.prototype.rawListeners = function rawListeners(event) {
                const events = _ensure(this);
                return events[event] ? events[event].slice() : [];
            };

            EventEmitter.prototype.eventNames = function eventNames() {
                const events = _ensure(this);
                return Object.keys(events);
            };

            EventEmitter.prototype.setMaxListeners = function setMaxListeners(n) {
                if (typeof n !== "number" || n < 0 || Number.isNaN(n)) {
                    throw new RangeError("n must be a non-negative number");
                }
                this._maxListeners = n;
                return this;
            };

            EventEmitter.prototype.getMaxListeners = function getMaxListeners() {
                return this._maxListeners === undefined
                    ? EventEmitter.defaultMaxListeners
                    : this._maxListeners;
            };

            // Module-level helpers per Node v15+ (Promise-bridging).
            // once(emitter, event) -> Promise<args[]>
            function onceHelper(emitter, eventName, options) {
                return new Promise((resolve, reject) => {
                    const errHandler = (err) => {
                        emitter.off(eventName, valHandler);
                        reject(err);
                    };
                    const valHandler = (...args) => {
                        if (eventName !== "error") emitter.off("error", errHandler);
                        resolve(args);
                    };
                    emitter.once(eventName, valHandler);
                    if (eventName !== "error") emitter.once("error", errHandler);
                });
            }

            // on(emitter, event) -> AsyncIterable<args[]>
            function onHelper(emitter, eventName) {
                const buffer = [];
                let resolveNext = null;
                let done = false;
                emitter.on(eventName, (...args) => {
                    if (resolveNext) {
                        const r = resolveNext;
                        resolveNext = null;
                        r({ value: args, done: false });
                    } else {
                        buffer.push(args);
                    }
                });
                return {
                    [Symbol.asyncIterator]() { return this; },
                    next() {
                        if (buffer.length > 0) {
                            return Promise.resolve({ value: buffer.shift(), done: false });
                        }
                        if (done) return Promise.resolve({ value: undefined, done: true });
                        return new Promise(r => { resolveNext = r; });
                    },
                    return() { done = true; return Promise.resolve({ value: undefined, done: true }); },
                };
            }

            // Self-reference: node:events default export is the class
            // AND the class has .EventEmitter pointing to itself.
            EventEmitter.EventEmitter = EventEmitter;
            EventEmitter.once = onceHelper;
            EventEmitter.on = onHelper;
            EventEmitter.captureRejectionSymbol = Symbol("nodejs.rejection");
            EventEmitter.errorMonitor = Symbol("events.errorMonitor");
            EventEmitter.setMaxListeners = function(n, ...emitters) {
                for (const e of emitters) e.setMaxListeners(n);
            };
            EventEmitter.getEventListeners = function(emitter, eventName) {
                return emitter.listeners ? emitter.listeners(eventName) : [];
            };

            // Module-level helpers (Node 18+) on EventEmitter as namespace.
            EventEmitter.addAbortListener = function(signal, listener) {
                if (signal && typeof signal.addEventListener === "function") {
                    signal.addEventListener("abort", listener, { once: true });
                }
                return { [Symbol.dispose]() {} };
            };
            EventEmitter.captureRejections = false;
            globalThis.nodeEvents = EventEmitter;
            globalThis.EventEmitter = EventEmitter;
        })();
    "#)?;
    Ok(())
}

fn install_node_util_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    // Π3.10: node:util. Most-used pieces: promisify, callbackify,
    // format (sprintf-style), inspect (debug repr), types.isXxx
    // predicates, isDeepStrictEqual.
    //
    // Per M9 (spec-first against Bun): targeted against Bun's actual
    // shape. inspect is minimal but functional; consumers needing
    // deep customization (showHidden, colors) get reasonable defaults.
    ctx.eval::<(), _>(r#"
        (function() {
            // promisify: callback-style fn(args..., (err, value) => ...) → Promise.
            function promisify(original) {
                if (typeof original !== "function") {
                    throw new TypeError("util.promisify: original must be a function");
                }
                if (original[promisify.custom]) {
                    return original[promisify.custom];
                }
                function promisified(...args) {
                    return new Promise((resolve, reject) => {
                        original.call(this, ...args, (err, value) => {
                            if (err) reject(err);
                            else resolve(value);
                        });
                    });
                }
                Object.setPrototypeOf(promisified, Object.getPrototypeOf(original));
                Object.defineProperties(promisified, Object.getOwnPropertyDescriptors(original));
                return promisified;
            }
            promisify.custom = Symbol.for("nodejs.util.promisify.custom");

            // callbackify: Promise-returning fn → callback-style.
            function callbackify(original) {
                if (typeof original !== "function") {
                    throw new TypeError("util.callbackify: original must be a function");
                }
                function callbackified(...args) {
                    const cb = args.pop();
                    if (typeof cb !== "function") {
                        throw new TypeError("util.callbackify: last argument must be callback");
                    }
                    original.call(this, ...args).then(
                        (value) => queueMicrotask(() => cb(null, value)),
                        (err) => queueMicrotask(() => cb(err))
                    );
                }
                Object.setPrototypeOf(callbackified, Object.getPrototypeOf(original));
                Object.defineProperties(callbackified, Object.getOwnPropertyDescriptors(original));
                return callbackified;
            }

            // format: sprintf-style. Supports %s, %d, %i, %f, %j, %o, %O, %%.
            function format(...args) {
                return formatWithOptions({}, ...args);
            }
            function formatWithOptions(_opts, ...args) {
                if (args.length === 0) return "";
                const first = args[0];
                let i = 1;
                if (typeof first !== "string") {
                    return args.map(a => inspect(a)).join(" ");
                }
                let s = "";
                let lastPos = 0;
                for (let p = 0; p < first.length - 1; p++) {
                    if (first.charCodeAt(p) === 37) {  // '%'
                        const c = first[p + 1];
                        let replacement;
                        switch (c) {
                            case "s": replacement = String(args[i++]); break;
                            case "d": replacement = Number(args[i++]).toString(); break;
                            case "i": replacement = Math.trunc(Number(args[i++])).toString(); break;
                            case "f": replacement = Number(args[i++]).toString(); break;
                            case "j": try { replacement = JSON.stringify(args[i++]); } catch (_) { replacement = "[Circular]"; } break;
                            case "o":
                            case "O": replacement = inspect(args[i++]); break;
                            case "%": replacement = "%"; break;
                            default: continue;
                        }
                        s += first.slice(lastPos, p) + replacement;
                        p++;  // skip the format char
                        lastPos = p + 1;
                    }
                }
                s += first.slice(lastPos);
                if (i < args.length) {
                    const rest = args.slice(i).map(a => typeof a === "string" ? a : inspect(a)).join(" ");
                    s += " " + rest;
                }
                return s;
            }

            // inspect: minimal but functional debug representation.
            function inspect(value, opts) {
                const depth = (opts && opts.depth !== undefined) ? opts.depth : 2;
                return _inspect(value, depth, new WeakSet());
            }
            function _inspect(value, depth, seen) {
                if (value === null) return "null";
                if (value === undefined) return "undefined";
                const t = typeof value;
                if (t === "string") return JSON.stringify(value);
                if (t === "number" || t === "boolean") return String(value);
                if (t === "bigint") return String(value) + "n";
                if (t === "symbol") return value.toString();
                if (t === "function") {
                    const name = value.name || "(anonymous)";
                    return "[Function: " + name + "]";
                }
                if (depth < 0) {
                    if (Array.isArray(value)) return "[Array]";
                    if (value instanceof Map) return "[Map]";
                    if (value instanceof Set) return "[Set]";
                    return "[Object]";
                }
                if (seen.has(value)) return "[Circular]";
                seen.add(value);
                if (Array.isArray(value)) {
                    if (value.length === 0) return "[]";
                    const items = value.map(v => _inspect(v, depth - 1, seen));
                    seen.delete(value);
                    return "[ " + items.join(", ") + " ]";
                }
                if (value instanceof Map) {
                    if (value.size === 0) return "Map(0) {}";
                    const items = [];
                    for (const [k, v] of value) {
                        items.push(_inspect(k, depth - 1, seen) + " => " + _inspect(v, depth - 1, seen));
                    }
                    seen.delete(value);
                    return "Map(" + value.size + ") { " + items.join(", ") + " }";
                }
                if (value instanceof Set) {
                    if (value.size === 0) return "Set(0) {}";
                    const items = [];
                    for (const v of value) items.push(_inspect(v, depth - 1, seen));
                    seen.delete(value);
                    return "Set(" + value.size + ") { " + items.join(", ") + " }";
                }
                if (value instanceof Date) return value.toISOString();
                if (value instanceof RegExp) return value.toString();
                if (value instanceof Error) {
                    return value.stack || (value.name + ": " + value.message);
                }
                // Plain object.
                const keys = Object.keys(value);
                if (keys.length === 0) return "{}";
                const items = keys.map(k => {
                    const kr = /^[A-Za-z_$][A-Za-z0-9_$]*$/.test(k) ? k : JSON.stringify(k);
                    return kr + ": " + _inspect(value[k], depth - 1, seen);
                });
                seen.delete(value);
                return "{ " + items.join(", ") + " }";
            }

            // isDeepStrictEqual: structural deep equality.
            function isDeepStrictEqual(a, b) {
                return _deepEq(a, b, new WeakMap());
            }
            function _deepEq(a, b, seen) {
                if (Object.is(a, b)) return true;
                if (a === null || b === null || typeof a !== "object" || typeof b !== "object") return false;
                if (Object.getPrototypeOf(a) !== Object.getPrototypeOf(b)) return false;
                if (seen.get(a) === b) return true;
                seen.set(a, b);
                if (Array.isArray(a)) {
                    if (!Array.isArray(b) || a.length !== b.length) return false;
                    for (let i = 0; i < a.length; i++) if (!_deepEq(a[i], b[i], seen)) return false;
                    return true;
                }
                if (a instanceof Map) {
                    if (!(b instanceof Map) || a.size !== b.size) return false;
                    for (const [k, v] of a) {
                        if (!b.has(k)) return false;
                        if (!_deepEq(v, b.get(k), seen)) return false;
                    }
                    return true;
                }
                if (a instanceof Set) {
                    if (!(b instanceof Set) || a.size !== b.size) return false;
                    for (const v of a) if (!b.has(v)) return false;
                    return true;
                }
                if (a instanceof Date) return b instanceof Date && a.getTime() === b.getTime();
                if (a instanceof RegExp) return b instanceof RegExp && a.source === b.source && a.flags === b.flags;
                const ak = Object.keys(a), bk = Object.keys(b);
                if (ak.length !== bk.length) return false;
                for (const k of ak) {
                    if (!Object.prototype.hasOwnProperty.call(b, k)) return false;
                    if (!_deepEq(a[k], b[k], seen)) return false;
                }
                return true;
            }

            // deprecate: wraps a function; logs once to stderr on first call.
            function deprecate(fn, msg, code) {
                let warned = false;
                return function deprecated(...args) {
                    if (!warned) {
                        warned = true;
                        globalThis.__stderrBuf += "(node:deprecate) " + msg +
                            (code ? " [" + code + "]" : "") + "\n";
                    }
                    return fn.apply(this, args);
                };
            }

            // debuglog: returns a logger; active only if NODE_DEBUG env names this section.
            function debuglog(section) {
                const env = (globalThis.process && globalThis.process.env) || {};
                const active = (env.NODE_DEBUG || "").split(",").some(s => s.trim() === section || s.trim() === "*");
                if (active) {
                    return function(...args) {
                        globalThis.__stderrBuf += section.toUpperCase() + " " + format(...args) + "\n";
                    };
                }
                return function() {};
            }

            // types — type-predicate namespace.
            const types = {
                isPromise: (v) => v != null && typeof v.then === "function",
                isDate: (v) => v instanceof Date,
                isRegExp: (v) => v instanceof RegExp,
                isMap: (v) => v instanceof Map,
                isSet: (v) => v instanceof Set,
                isWeakMap: (v) => v instanceof WeakMap,
                isWeakSet: (v) => v instanceof WeakSet,
                isArrayBuffer: (v) => v instanceof ArrayBuffer,
                isSharedArrayBuffer: (v) => typeof SharedArrayBuffer !== "undefined" && v instanceof SharedArrayBuffer,
                isTypedArray: (v) => ArrayBuffer.isView(v) && !(v instanceof DataView),
                isUint8Array: (v) => v instanceof Uint8Array,
                isUint8ClampedArray: (v) => v instanceof Uint8ClampedArray,
                isInt8Array: (v) => v instanceof Int8Array,
                isUint16Array: (v) => v instanceof Uint16Array,
                isInt16Array: (v) => v instanceof Int16Array,
                isUint32Array: (v) => v instanceof Uint32Array,
                isInt32Array: (v) => v instanceof Int32Array,
                isFloat32Array: (v) => v instanceof Float32Array,
                isFloat64Array: (v) => v instanceof Float64Array,
                isBigInt64Array: (v) => v instanceof BigInt64Array,
                isBigUint64Array: (v) => v instanceof BigUint64Array,
                isDataView: (v) => v instanceof DataView,
                isAsyncFunction: (v) => typeof v === "function" && v.constructor && v.constructor.name === "AsyncFunction",
                isGeneratorFunction: (v) => typeof v === "function" && v.constructor && v.constructor.name === "GeneratorFunction",
            };

            // util.inherits — Node legacy ES5-style prototypal inheritance.
            // ctor.prototype = Object.create(superCtor.prototype, {
            //   constructor: { value: ctor, enumerable: false, writable: true, configurable: true }
            // }); + ctor.super_ = superCtor.
            // Used pervasively in older npm: send, http-errors, every Stream subclass.
            const inherits = function inherits(ctor, superCtor) {
                if (typeof ctor !== "function" || ctor === null) {
                    throw new TypeError("util.inherits: constructor must be a function");
                }
                // Permissive: when superCtor is missing/non-function (qrcode's
                // build-time bundling passes an undefined stream parent in
                // some configurations), no-op the prototype chain wiring so
                // the consumer can complete its top-level class evaluation.
                if (typeof superCtor !== "function" || superCtor === null) {
                    Object.defineProperty(ctor, "super_", {
                        value: undefined, writable: true, configurable: true,
                    });
                    return;
                }
                Object.defineProperty(ctor, "super_", {
                    value: superCtor, writable: true, configurable: true,
                });
                Object.setPrototypeOf(ctor.prototype, superCtor.prototype);
            };

            // util.styleText (Node 22+) — ANSI styling. Map common format
            // names; fall through identity for unknown.
            const __ANSI = {
                reset: "\x1b[0m", bold: "\x1b[1m", italic: "\x1b[3m", underline: "\x1b[4m",
                inverse: "\x1b[7m", dim: "\x1b[2m", strikethrough: "\x1b[9m",
                black: "\x1b[30m", red: "\x1b[31m", green: "\x1b[32m", yellow: "\x1b[33m",
                blue: "\x1b[34m", magenta: "\x1b[35m", cyan: "\x1b[36m", white: "\x1b[37m",
                gray: "\x1b[90m", grey: "\x1b[90m",
                bgBlack: "\x1b[40m", bgRed: "\x1b[41m", bgGreen: "\x1b[42m",
                bgYellow: "\x1b[43m", bgBlue: "\x1b[44m", bgMagenta: "\x1b[45m",
                bgCyan: "\x1b[46m", bgWhite: "\x1b[47m",
            };
            function styleText(format, text) {
                const formats = Array.isArray(format) ? format : [format];
                let prefix = "";
                for (const f of formats) prefix += __ANSI[f] || "";
                return prefix + String(text) + __ANSI.reset;
            }
            // Default parseArgs: minimal Node parseArgs (Node 18+).
            function parseArgs(config) {
                const args = config.args || [];
                const options = config.options || {};
                const values = {};
                const positionals = [];
                for (let i = 0; i < args.length; i++) {
                    const a = args[i];
                    if (a.startsWith("--")) {
                        const eq = a.indexOf("=");
                        const name = eq < 0 ? a.slice(2) : a.slice(2, eq);
                        const inline = eq < 0 ? undefined : a.slice(eq + 1);
                        const opt = options[name] || {};
                        if (opt.type === "boolean") {
                            values[name] = inline === undefined ? true : inline !== "false";
                        } else {
                            values[name] = inline !== undefined ? inline : args[++i];
                        }
                    } else if (a.startsWith("-") && a.length > 1) {
                        const name = a.slice(1);
                        const opt = options[name] || {};
                        if (opt.type === "boolean") values[name] = true;
                        else values[name] = args[++i];
                    } else {
                        positionals.push(a);
                    }
                }
                return { values, positionals };
            }
            globalThis.nodeUtil = {
                promisify,
                callbackify,
                format,
                formatWithOptions,
                inspect,
                isDeepStrictEqual,
                deprecate,
                debuglog,
                inherits,
                types,
                styleText,
                parseArgs,
                parseEnv: (text) => {
                    // Minimal .env parser: KEY=VALUE per line, hash-prefixed
                    // comments stripped. (Avoid hash char in raw-string content.)
                    const out = {};
                    const COMMENT = String.fromCharCode(35);
                    for (const line of String(text).split(/\r?\n/)) {
                        const trimmed = line.replace(/^\s+|\s+$/g, "");
                        if (!trimmed || trimmed.charAt(0) === COMMENT) continue;
                        const eq = trimmed.indexOf("=");
                        if (eq < 0) continue;
                        const k = trimmed.substring(0, eq).trim();
                        let v = trimmed.substring(eq + 1).trim();
                        if ((v.startsWith('"') && v.endsWith('"'))
                            || (v.startsWith("'") && v.endsWith("'"))) {
                            v = v.slice(1, -1);
                        }
                        out[k] = v;
                    }
                    return out;
                },
                _extend: (target, source) => Object.assign(target, source),
                stripVTControlCharacters: (s) => String(s).replace(/\x1b\[[\d;]*m/g, ""),
                getSystemErrorName: (errno) => "UNKNOWN",
                getSystemErrorMap: () => new Map(),
                aborted: (signal, resource) => {
                    return new Promise((resolve, reject) => {
                        if (signal.aborted) return reject(signal.reason);
                        signal.addEventListener("abort", () => reject(signal.reason), { once: true });
                    });
                },
                MIMEType: class MIMEType {
                    constructor(input) { this.input = String(input); }
                    toString() { return this.input; }
                },
                MIMEParams: class MIMEParams {
                    constructor() { this._params = new Map(); }
                    get(name) { return this._params.get(name) || null; }
                    set(name, value) { this._params.set(name, value); }
                    delete(name) { this._params.delete(name); }
                    has(name) { return this._params.has(name); }
                    [Symbol.iterator]() { return this._params.entries(); }
                },
                deprecationWarned: new Map(),
                getCallSite: () => [],
                transferableAbortController: (controller) => controller,
                transferableAbortSignal: (signal) => signal,
                TextEncoder: globalThis.TextEncoder,
                TextDecoder: globalThis.TextDecoder,
            };
            globalThis.nodeUtilTypes = types;
        })();
    "#)?;
    Ok(())
}

fn install_node_stream_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    // Π3.9: node:stream. Distinct from web-streams (already installed
    // by install_streams_js as ReadableStream/WritableStream/
    // TransformStream); node:stream is the Node convention with
    // EventEmitter-driven on('data')/on('end')/on('error') + push/
    // write/end internal API.
    //
    // Per M9 (spec-first): minimum viable load-bearing subset for real
    // npm consumers. High-water-mark precision, encoding parsing,
    // objectMode subtleties deferred to follow-on if a consumer
    // surfaces them.
    //
    // Per seed §A8.2: pure JS-side classes, no closure-captured Rust state.
    // Composes on globalThis.EventEmitter from Π3.8.
    ctx.eval::<(), _>(r#"
        (function() {
            const EE = globalThis.EventEmitter;

            // ─── Readable ─────────────────────────────────────────────
            function Readable(options) {
                EE.call(this);
                options = options || {};
                this._readableBuffer = [];
                this._readableEnded = false;
                this._readableDestroyed = false;
                this._readableFlowing = null;  // null/true/false per Node
                this._readableHighWaterMark = options.highWaterMark || 16 * 1024;
                this.readable = true;
                if (typeof options.read === "function") this._read = options.read;
                if (typeof options.destroy === "function") this._destroy = options.destroy;
            }
            Readable.prototype = Object.create(EE.prototype);
            Readable.prototype.constructor = Readable;
            Readable.prototype._read = function _read(_size) {};
            Readable.prototype._emitEndOnce = function _emitEndOnce() {
                if (this._endEmitted) return;
                this._endEmitted = true;
                try { this.emit("end"); } catch (_) {}
            };
            Readable.prototype.push = function push(chunk) {
                if (this._readableEnded) return false;
                if (chunk === null) {
                    this._readableEnded = true;
                    // Emit a final 'readable' so pull-style consumers
                    // see read() return null and can finalize, then 'end'.
                    if (this._readableBuffer.length === 0) {
                        queueMicrotask(() => {
                            try { this.emit("readable"); } catch (_) {}
                            this._emitEndOnce();
                        });
                    } else {
                        // Still buffered chunks; emit readable to drain,
                        // then end fires after read() returns null.
                        queueMicrotask(() => { try { this.emit("readable"); } catch (_) {} });
                    }
                    return false;
                }
                this._readableBuffer.push(chunk);
                if (this._readableFlowing === true) {
                    queueMicrotask(() => this._drainFlowing());
                } else if (this._readableFlowing !== false) {
                    // Pull-style: schedule 'readable' so on('readable')
                    // listeners fire and can call read(). csv-parse, ndjson
                    // and similar consumers depend on this.
                    if (!this._readableScheduled) {
                        this._readableScheduled = true;
                        queueMicrotask(() => {
                            this._readableScheduled = false;
                            try { this.emit("readable"); } catch (_) {}
                        });
                    }
                }
                return true;
            };
            Readable.prototype._drainFlowing = function _drainFlowing() {
                while (this._readableFlowing === true && this._readableBuffer.length > 0) {
                    const chunk = this._readableBuffer.shift();
                    this.emit("data", chunk);
                }
                if (this._readableFlowing === true && this._readableEnded) {
                    this._emitEndOnce();
                    return;
                }
                // Pull-driven: after the buffer drains, ask _read for more.
                // The subclass-via-options pattern depends on this; without it,
                // _read fires once and the stream stalls. Scheduled rather than
                // called directly to keep microtask ordering predictable.
                if (this._readableFlowing === true && !this._readableEnded) {
                    queueMicrotask(() => {
                        if (!this._readableEnded && this._readableFlowing === true) {
                            try { this._read(this._readableHighWaterMark); } catch (e) { this.emit("error", e); }
                        }
                    });
                }
            };
            Readable.prototype.read = function read(_n) {
                if (this._readableBuffer.length > 0) return this._readableBuffer.shift();
                return null;
            };
            Readable.prototype.on = function on(event, listener) {
                EE.prototype.on.call(this, event, listener);
                if (event === "data" && this._readableFlowing === null) {
                    this._readableFlowing = true;
                    queueMicrotask(() => {
                        try { this._read(this._readableHighWaterMark); } catch (e) { this.emit("error", e); }
                        this._drainFlowing();
                    });
                } else if (event === "readable" && this._readableFlowing === null) {
                    // Pull-style: registration forces flowing=false so
                    // push() emits 'readable' instead of dispatching 'data'.
                    this._readableFlowing = false;
                    // If chunks were already buffered before listener attach,
                    // schedule one 'readable' so the consumer can drain.
                    if (this._readableBuffer.length > 0 && !this._readableScheduled) {
                        this._readableScheduled = true;
                        queueMicrotask(() => {
                            this._readableScheduled = false;
                            try { this.emit("readable"); } catch (_) {}
                        });
                    }
                }
                return this;
            };
            Readable.prototype.pause = function pause() {
                this._readableFlowing = false;
                return this;
            };
            Readable.prototype.resume = function resume() {
                if (!this._readableFlowing) {
                    this._readableFlowing = true;
                    queueMicrotask(() => this._drainFlowing());
                }
                return this;
            };
            Readable.prototype.destroy = function destroy(err) {
                this._readableDestroyed = true;
                this.readable = false;
                try { this._destroy && this._destroy(err, () => {}); } catch (_) {}
                if (err) queueMicrotask(() => this.emit("error", err));
                queueMicrotask(() => this.emit("close"));
                return this;
            };
            Readable.prototype[Symbol.asyncIterator] = function() {
                const self = this;
                return {
                    next() {
                        return new Promise((resolve, reject) => {
                            if (self._readableBuffer.length > 0) {
                                resolve({ value: self._readableBuffer.shift(), done: false });
                                return;
                            }
                            if (self._readableEnded) {
                                resolve({ value: undefined, done: true });
                                return;
                            }
                            const onData = (chunk) => {
                                self.off("end", onEnd);
                                self.off("error", onErr);
                                resolve({ value: chunk, done: false });
                            };
                            const onEnd = () => {
                                self.off("data", onData);
                                self.off("error", onErr);
                                resolve({ value: undefined, done: true });
                            };
                            const onErr = (e) => {
                                self.off("data", onData);
                                self.off("end", onEnd);
                                reject(e);
                            };
                            self.once("data", onData);
                            self.once("end", onEnd);
                            self.once("error", onErr);
                            self.resume();
                        });
                    },
                    return() {
                        self.destroy();
                        return Promise.resolve({ value: undefined, done: true });
                    },
                };
            };
            Readable.prototype.pipe = function pipe(dst, _opts) {
                const src = this;
                src.on("data", (chunk) => {
                    const more = dst.write(chunk);
                    if (more === false) src.pause();
                });
                src.on("end", () => dst.end());
                src.on("error", (err) => dst.destroy(err));
                dst.on("drain", () => src.resume());
                return dst;
            };
            Readable.from = function from(iter, _opts) {
                const r = new Readable();
                r._read = function() {};
                (async () => {
                    try {
                        for await (const chunk of iter) {
                            r.push(chunk);
                        }
                        r.push(null);
                    } catch (e) {
                        r.destroy(e);
                    }
                })();
                return r;
            };

            // ─── Writable ─────────────────────────────────────────────
            function Writable(options) {
                EE.call(this);
                options = options || {};
                this._writableEnded = false;
                this._writableDestroyed = false;
                this._writableHighWaterMark = options.highWaterMark || 16 * 1024;
                this._writableQueue = [];
                this._writing = false;
                this.writable = true;
                if (typeof options.write === "function") this._write = options.write;
                if (typeof options.final === "function") this._final = options.final;
                if (typeof options.destroy === "function") this._destroy = options.destroy;
            }
            Writable.prototype = Object.create(EE.prototype);
            Writable.prototype.constructor = Writable;
            Writable.prototype._write = function _write(_chunk, _enc, cb) { cb(); };
            Writable.prototype.write = function write(chunk, encOrCb, maybeCb) {
                if (this._writableEnded) {
                    const err = new Error("write after end");
                    queueMicrotask(() => this.emit("error", err));
                    return false;
                }
                let encoding, cb;
                if (typeof encOrCb === "function") { cb = encOrCb; encoding = "utf8"; }
                else { encoding = encOrCb || "utf8"; cb = maybeCb; }
                this._writableQueue.push({ chunk, encoding, cb });
                this._drainWritable();
                return this._writableQueue.length < (this._writableHighWaterMark / 16);
            };
            Writable.prototype._drainWritable = function _drainWritable() {
                if (this._writing) return;
                const item = this._writableQueue.shift();
                if (!item) {
                    // Empty queue. If we were under high-water-mark and now drained, emit 'drain'.
                    queueMicrotask(() => this.emit("drain"));
                    return;
                }
                this._writing = true;
                try {
                    this._write(item.chunk, item.encoding, (err) => {
                        this._writing = false;
                        if (item.cb) try { item.cb(err); } catch (_) {}
                        if (err) {
                            queueMicrotask(() => this.emit("error", err));
                            return;
                        }
                        this._drainWritable();
                    });
                } catch (e) {
                    this._writing = false;
                    if (item.cb) try { item.cb(e); } catch (_) {}
                    queueMicrotask(() => this.emit("error", e));
                }
            };
            Writable.prototype.end = function end(chunkOrCb, encOrCb, maybeCb) {
                let chunk, encoding, cb;
                if (typeof chunkOrCb === "function") { cb = chunkOrCb; }
                else { chunk = chunkOrCb; if (typeof encOrCb === "function") cb = encOrCb; else { encoding = encOrCb; cb = maybeCb; } }
                if (chunk !== undefined && chunk !== null) this.write(chunk, encoding);
                this._writableEnded = true;
                const finish = () => {
                    const doFinish = () => {
                        if (cb) try { cb(); } catch (_) {}
                        this.emit("finish");
                    };
                    if (this._final) {
                        this._final((err) => {
                            if (err) queueMicrotask(() => this.emit("error", err));
                            else queueMicrotask(doFinish);
                        });
                    } else {
                        queueMicrotask(doFinish);
                    }
                };
                // Wait for queue to drain.
                const waitDrain = () => {
                    if (this._writableQueue.length === 0 && !this._writing) finish();
                    else queueMicrotask(waitDrain);
                };
                waitDrain();
                return this;
            };
            Writable.prototype.destroy = function destroy(err) {
                this._writableDestroyed = true;
                this.writable = false;
                try { this._destroy && this._destroy(err, () => {}); } catch (_) {}
                if (err) queueMicrotask(() => this.emit("error", err));
                queueMicrotask(() => this.emit("close"));
                return this;
            };
            Writable.prototype.cork = function cork() { return this; };
            Writable.prototype.uncork = function uncork() { return this; };

            // ─── Duplex ───────────────────────────────────────────────
            function Duplex(options) {
                Readable.call(this, options);
                Writable.call(this, options);
            }
            Duplex.prototype = Object.create(Readable.prototype);
            Object.assign(Duplex.prototype, Writable.prototype);
            Duplex.prototype.constructor = Duplex;

            // ─── Transform ────────────────────────────────────────────
            function Transform(options) {
                Duplex.call(this, options);
                options = options || {};
                if (typeof options.transform === "function") this._transform = options.transform;
                if (typeof options.flush === "function") this._flush = options.flush;
            }
            Transform.prototype = Object.create(Duplex.prototype);
            Transform.prototype.constructor = Transform;
            Transform.prototype._transform = function _transform(chunk, _enc, cb) { cb(null, chunk); };
            // Override _write to route through _transform.
            Transform.prototype._write = function _write(chunk, enc, cb) {
                const self = this;
                self._transform(chunk, enc, (err, transformed) => {
                    if (err) { cb(err); return; }
                    if (transformed !== undefined && transformed !== null) self.push(transformed);
                    cb();
                });
            };
            Transform.prototype._final = function _final(cb) {
                if (this._flush) {
                    this._flush((err, tail) => {
                        if (err) { cb(err); return; }
                        if (tail !== undefined && tail !== null) this.push(tail);
                        this.push(null);
                        cb();
                    });
                } else {
                    this.push(null);
                    cb();
                }
            };

            // ─── PassThrough ──────────────────────────────────────────
            function PassThrough(options) {
                Transform.call(this, options);
            }
            PassThrough.prototype = Object.create(Transform.prototype);
            PassThrough.prototype.constructor = PassThrough;
            PassThrough.prototype._transform = function(chunk, _enc, cb) { cb(null, chunk); };

            // ─── pipeline + finished ──────────────────────────────────
            function pipeline(...args) {
                let cb = typeof args[args.length - 1] === "function" ? args.pop() : null;
                const streams = args;
                if (streams.length < 2) {
                    const err = new TypeError("pipeline: need at least 2 streams");
                    if (cb) queueMicrotask(() => cb(err));
                    return;
                }
                let done = false;
                const finish = (err) => {
                    if (done) return;
                    done = true;
                    if (cb) queueMicrotask(() => cb(err || null));
                };
                for (let i = 0; i < streams.length - 1; i++) {
                    streams[i].on("error", finish);
                    streams[i].pipe(streams[i + 1]);
                }
                const last = streams[streams.length - 1];
                last.on("error", finish);
                last.on("finish", () => finish());
                last.on("end", () => finish());
                return last;
            }
            function finished(stream, cb) {
                let done = false;
                const fire = (err) => {
                    if (done) return;
                    done = true;
                    queueMicrotask(() => cb(err || null));
                };
                stream.on("end", () => fire());
                stream.on("finish", () => fire());
                stream.on("error", (e) => fire(e));
                stream.on("close", () => fire());
            }

            // Promise variants for node:stream/promises.
            function pipelinePromise(...args) {
                return new Promise((resolve, reject) => {
                    pipeline(...args, (err) => err ? reject(err) : resolve());
                });
            }
            function finishedPromise(stream) {
                return new Promise((resolve, reject) => {
                    finished(stream, (err) => err ? reject(err) : resolve());
                });
            }

            // node:stream module shape: require('stream') in Node returns
            // the Stream class itself (legacy ancestor), with Readable/
            // Writable/Duplex/Transform/PassThrough as static properties.
            // npm packages do `var Stream = require('stream'); Stream.call(this)`
            // and `util.inherits(MyClass, Stream)`. Need a class, not an object.
            //
            // Readable plays the legacy Stream role here — it's the most-derived
            // base that already exists, and Stream.call(this) just needs a
            // callable constructor.
            Readable.Readable = Readable;
            Readable.Writable = Writable;
            Readable.Duplex = Duplex;
            Readable.Transform = Transform;
            Readable.PassThrough = PassThrough;
            Readable.pipeline = pipeline;
            Readable.finished = finished;
            Readable.Stream = Readable;
            Readable.getDefaultHighWaterMark = (objectMode) => objectMode ? 16 : 16 * 1024;
            Readable.setDefaultHighWaterMark = () => {};
            Readable.isReadable = (s) => s instanceof Readable || (s && typeof s.read === "function");
            Readable.isWritable = (s) => s instanceof Writable || (s && typeof s.write === "function");
            Readable.compose = (...args) => { throw new Error("stream.compose not implemented"); };
            globalThis.nodeStream = Readable;
            globalThis.nodeStreamPromises = {
                pipeline: pipelinePromise,
                finished: finishedPromise,
            };
        })();
    "#)?;
    Ok(())
}

fn install_node_querystring_and_url_full_js<'js>(ctx: &Ctx<'js>) -> JsResult<()> {
    // Π3.11: node:querystring (legacy URL-query parser/serializer) +
    // node:url full (legacy parse/format + WHATWG fileURLToPath /
    // pathToFileURL helpers). URL + URLSearchParams classes are already
    // wired by install_url_class_js + install_url_search_params_class_js;
    // this round attaches the function-style legacy and WHATWG helpers
    // under a `nodeUrl` namespace.
    ctx.eval::<(), _>(r#"
        (function() {
            // ─── querystring ────────────────────────────────────────
            // Differs from URLSearchParams: returns plain {key: value} or
            // {key: [v1, v2]} objects for multi-valued keys. encode/decode
            // use plus-sign for space (form-urlencoded), not %20.
            function qsParse(str, sep, eq, options) {
                sep = sep || "&";
                eq = eq || "=";
                const max = options && typeof options.maxKeys === "number" ? options.maxKeys : 1000;
                const decode = (options && options.decodeURIComponent) || qsUnescape;
                const out = Object.create(null);
                if (typeof str !== "string" || str.length === 0) return out;
                const pairs = str.split(sep);
                const limit = Math.min(pairs.length, max);
                for (let i = 0; i < limit; i++) {
                    const pair = pairs[i];
                    if (!pair) continue;
                    const idx = pair.indexOf(eq);
                    let k, v;
                    if (idx >= 0) {
                        k = pair.slice(0, idx);
                        v = pair.slice(idx + eq.length);
                    } else {
                        k = pair;
                        v = "";
                    }
                    k = decode(k.replace(/\+/g, " "));
                    v = decode(v.replace(/\+/g, " "));
                    if (Object.prototype.hasOwnProperty.call(out, k)) {
                        if (Array.isArray(out[k])) out[k].push(v);
                        else out[k] = [out[k], v];
                    } else {
                        out[k] = v;
                    }
                }
                return out;
            }
            function qsStringify(obj, sep, eq, options) {
                sep = sep || "&";
                eq = eq || "=";
                const encode = (options && options.encodeURIComponent) || qsEscape;
                if (obj == null || typeof obj !== "object") return "";
                const parts = [];
                for (const k of Object.keys(obj)) {
                    const ek = encode(String(k));
                    const v = obj[k];
                    if (Array.isArray(v)) {
                        for (const vv of v) parts.push(ek + eq + encode(String(vv)));
                    } else if (v === null || v === undefined) {
                        parts.push(ek + eq);
                    } else {
                        parts.push(ek + eq + encode(String(v)));
                    }
                }
                return parts.join(sep);
            }
            function qsEscape(s) {
                // form-urlencoded: spaces → +, then percent-encode the rest.
                return encodeURIComponent(String(s)).replace(/%20/g, "+");
            }
            function qsUnescape(s) {
                try { return decodeURIComponent(String(s)); } catch (_) { return String(s); }
            }
            const nodeQuerystring = {
                parse: qsParse,
                stringify: qsStringify,
                escape: qsEscape,
                unescape: qsUnescape,
                decode: qsParse,
                encode: qsStringify,
            };
            globalThis.nodeQuerystring = nodeQuerystring;

            // ─── node:url full ───────────────────────────────────────
            // Legacy url.parse / url.format / url.resolve + WHATWG
            // fileURLToPath / pathToFileURL. URL + URLSearchParams come
            // from existing global wirings.
            function urlParse(urlStr, parseQueryString, slashesDenoteHost) {
                let u;
                try { u = new URL(urlStr); }
                catch (_) {
                    // Some legacy callers pass paths without scheme.
                    try { u = new URL(urlStr, "file:///"); }
                    catch (_e) { return { href: urlStr, protocol: null, host: null, hostname: null,
                        port: null, pathname: null, search: null, query: null, hash: null }; }
                }
                const search = u.search || "";
                const query = parseQueryString ? qsParse(search.slice(1)) : (search ? search.slice(1) : null);
                const out = {
                    protocol: u.protocol || null,
                    slashes: !!u.host,
                    auth: (u.username || u.password) ? (u.username + (u.password ? ":" + u.password : "")) : null,
                    host: u.host || null,
                    port: u.port || null,
                    hostname: u.hostname || null,
                    hash: u.hash || null,
                    search: search || null,
                    query: query,
                    pathname: u.pathname || null,
                    path: (u.pathname || "") + search,
                    href: u.href,
                };
                return out;
            }
            function urlFormat(obj) {
                if (typeof obj === "string") return obj;
                if (obj instanceof URL) return obj.href;
                let s = "";
                if (obj.protocol) {
                    s += obj.protocol;
                    if (!obj.protocol.endsWith(":")) s += ":";
                }
                if (obj.slashes !== false && (obj.host || obj.hostname)) s += "//";
                if (obj.auth) s += obj.auth + "@";
                if (obj.host) s += obj.host;
                else if (obj.hostname) {
                    s += obj.hostname;
                    if (obj.port) s += ":" + obj.port;
                }
                if (obj.pathname) s += obj.pathname;
                if (obj.search) s += (obj.search.startsWith("?") ? obj.search : "?" + obj.search);
                else if (obj.query) {
                    const qstr = typeof obj.query === "string" ? obj.query : qsStringify(obj.query);
                    if (qstr) s += "?" + qstr;
                }
                if (obj.hash) s += (obj.hash.startsWith('#') ? obj.hash : '#' + obj.hash);
                return s;
            }
            function urlResolve(from, to) {
                try { return new URL(to, from).href; }
                catch (_) { return to; }
            }
            function fileURLToPath(input) {
                const href = typeof input === "string" ? input : (input && input.href);
                if (!href || !href.startsWith("file:")) {
                    throw new TypeError("The argument 'url' must be a file URL. Received: " + href);
                }
                let p = href.slice(href.indexOf("file:") + 5);
                if (p.startsWith("//")) p = p.slice(2);
                // Strip leading host (e.g. "localhost") if present:
                if (p.length > 0 && p[0] !== "/") {
                    const slash = p.indexOf("/");
                    if (slash >= 0) p = p.slice(slash);
                }
                return decodeURIComponent(p);
            }
            function pathToFileURL(p) {
                let path = String(p);
                if (!path.startsWith("/")) {
                    // Resolve against cwd.
                    const cwd = (globalThis.process && globalThis.process.cwd) ? globalThis.process.cwd() : "/";
                    path = cwd.replace(/\/+$/, "") + "/" + path;
                }
                return new URL("file://" + encodeURI(path).replace(/#/g, "%23").replace(/\?/g, "%3F"));
            }
            function domainToASCII(d) { return String(d); }
            function domainToUnicode(d) { return String(d); }

            globalThis.nodeUrl = {
                // URL / URLSearchParams via getters: globalThis.URL may
                // not be installed yet at the time this initialization
                // runs (rquickjs sets globals like URL after the
                // host's eval-time install order). Lazy-bind so that
                // `require('node:url').URL` resolves correctly at
                // call time rather than at module-init time.
                get URL() { return globalThis.URL; },
                get URLSearchParams() { return globalThis.URLSearchParams; },
                parse: urlParse,
                format: urlFormat,
                resolve: urlResolve,
                fileURLToPath,
                pathToFileURL,
                domainToASCII,
                domainToUnicode,
            };
        })();
    "#)?;
    Ok(())
}

fn install_node_zlib_js<'js>(ctx: &Ctx<'js>) -> JsResult<()> {
    // node:zlib — sync wrappers compose on __compression (Π1.3 decode +
    // Π1.3.b stored-block encode). createGzip/createDeflate stream APIs
    // are stub Transform-shape classes; many libs require zlib only for
    // optional features (destroy + body-parser top-level-require it) and
    // tolerate non-functional stream constructors.
    ctx.eval::<(), _>(r#"
        (function() {
            const gzipSync = (input) => {
                const bytes = (typeof input === "string")
                    ? new TextEncoder().encode(input)
                    : (input instanceof Uint8Array ? input : new Uint8Array(input));
                const out = globalThis.__compression.gzip_deflate_stored(Array.from(bytes));
                return typeof Buffer !== "undefined" ? Buffer.from(out) : new Uint8Array(out);
            };
            const gunzipSync = (input) => {
                const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
                const out = globalThis.__compression.gunzip(Array.from(bytes));
                return typeof Buffer !== "undefined" ? Buffer.from(out) : new Uint8Array(out);
            };
            const deflateSync = (input) => {
                const bytes = (typeof input === "string")
                    ? new TextEncoder().encode(input)
                    : (input instanceof Uint8Array ? input : new Uint8Array(input));
                const out = globalThis.__compression.zlib_deflate_stored(Array.from(bytes));
                return typeof Buffer !== "undefined" ? Buffer.from(out) : new Uint8Array(out);
            };
            const inflateSync = (input) => {
                const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
                const out = globalThis.__compression.http_deflate_inflate(Array.from(bytes));
                return typeof Buffer !== "undefined" ? Buffer.from(out) : new Uint8Array(out);
            };
            const deflateRawSync = (input) => {
                const bytes = (typeof input === "string")
                    ? new TextEncoder().encode(input)
                    : (input instanceof Uint8Array ? input : new Uint8Array(input));
                const out = globalThis.__compression.deflate_stored(Array.from(bytes));
                return typeof Buffer !== "undefined" ? Buffer.from(out) : new Uint8Array(out);
            };
            const inflateRawSync = (input) => {
                const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
                const out = globalThis.__compression.inflate(Array.from(bytes));
                return typeof Buffer !== "undefined" ? Buffer.from(out) : new Uint8Array(out);
            };
            const stub = (name) => () => {
                throw new Error("rusty-bun-host: zlib." + name +
                    " (stream variant) not implemented — use *Sync");
            };
            globalThis.nodeZlib = {
                gzipSync, gunzipSync, deflateSync, inflateSync,
                deflateRawSync, inflateRawSync,
                createGzip: stub("createGzip"),
                createGunzip: stub("createGunzip"),
                createDeflate: stub("createDeflate"),
                createInflate: stub("createInflate"),
                constants: {
                    Z_NO_FLUSH: 0, Z_PARTIAL_FLUSH: 1, Z_SYNC_FLUSH: 2,
                    Z_FULL_FLUSH: 3, Z_FINISH: 4, Z_BLOCK: 5, Z_TREES: 6,
                    Z_OK: 0, Z_STREAM_END: 1, Z_NEED_DICT: 2,
                    Z_DEFAULT_COMPRESSION: -1, Z_BEST_SPEED: 1, Z_BEST_COMPRESSION: 9,
                },
            };
        })();
    "#)?;
    Ok(())
}

fn install_node_tty_js<'js>(ctx: &Ctx<'js>) -> JsResult<()> {
    // node:tty — minimal stub. debug + many libs top-level-require this.
    // isatty(fd) returns false in non-TTY contexts (which we always are
    // in the rusty-bun-host eval-loop runtime since stdout is piped to
    // tests). ReadStream/WriteStream are placeholder classes for shape.
    ctx.eval::<(), _>(r#"
        (function() {
            class WriteStream { constructor() { this.isTTY = false; } }
            class ReadStream { constructor() { this.isTTY = false; } }
            globalThis.nodeTty = {
                isatty: (_fd) => false,
                ReadStream,
                WriteStream,
            };
        })();
    "#)?;
    Ok(())
}

fn install_node_diagnostics_channel_js<'js>(ctx: &Ctx<'js>) -> JsResult<()> {
    // node:diagnostics_channel — observability hooks (Node 15+, fastify
    // top-level-imports it). Real implementation publishes events to
    // subscribers; no instrumentation lives in rusty-bun-host yet, so this
    // is a faithful no-op surface: Channel objects with subscribe/publish
    // exist, hasSubscribers always false, publish drops messages on the
    // floor. Allows libraries that gate optional tracing on the API's
    // presence to load and proceed.
    ctx.eval::<(), _>(r#"
        (function() {
            class Channel {
                constructor(name) {
                    this.name = name;
                    this._subs = new Set();
                }
                get hasSubscribers() { return this._subs.size > 0; }
                subscribe(fn) { this._subs.add(fn); }
                unsubscribe(fn) {
                    const had = this._subs.has(fn);
                    this._subs.delete(fn);
                    return had;
                }
                publish(message) {
                    for (const fn of this._subs) {
                        try { fn(message, this.name); }
                        catch (e) {
                            if (typeof console !== "undefined" && console.error) {
                                console.error("diagnostics_channel publish:", e);
                            }
                        }
                    }
                }
                bindStore(_store, _transform) { /* no-op */ }
                unbindStore(_store) { return false; }
                runStores(_data, fn, _thisArg, ...args) {
                    return fn(...args);
                }
            }
            class TracingChannel {
                constructor(nameOrChannels) {
                    if (typeof nameOrChannels === "string") {
                        const n = nameOrChannels;
                        this.start = new Channel("tracing:" + n + ":start");
                        this.end = new Channel("tracing:" + n + ":end");
                        this.asyncStart = new Channel("tracing:" + n + ":asyncStart");
                        this.asyncEnd = new Channel("tracing:" + n + ":asyncEnd");
                        this.error = new Channel("tracing:" + n + ":error");
                    } else {
                        Object.assign(this, nameOrChannels || {});
                    }
                }
                get hasSubscribers() {
                    return [this.start, this.end, this.asyncStart, this.asyncEnd, this.error]
                        .some(c => c && c.hasSubscribers);
                }
                subscribe(handlers) {
                    for (const k of Object.keys(handlers)) {
                        if (this[k] && typeof this[k].subscribe === "function") {
                            this[k].subscribe(handlers[k]);
                        }
                    }
                }
                unsubscribe(handlers) {
                    for (const k of Object.keys(handlers)) {
                        if (this[k] && typeof this[k].unsubscribe === "function") {
                            this[k].unsubscribe(handlers[k]);
                        }
                    }
                }
                traceSync(fn, _ctx, _thisArg, ...args) {
                    try { return fn(...args); } catch (e) { throw e; }
                }
                async tracePromise(fn, _ctx, _thisArg, ...args) {
                    return await fn(...args);
                }
                traceCallback(fn, _pos, _ctx, _thisArg, ...args) {
                    return fn(...args);
                }
            }
            const channels = new Map();
            function channel(name) {
                let c = channels.get(name);
                if (!c) { c = new Channel(name); channels.set(name, c); }
                return c;
            }
            function hasSubscribers(name) {
                const c = channels.get(name);
                return !!(c && c.hasSubscribers);
            }
            function subscribe(name, fn) { channel(name).subscribe(fn); }
            function unsubscribe(name, fn) { return channel(name).unsubscribe(fn); }
            function tracingChannel(nameOrChannels) {
                return new TracingChannel(nameOrChannels);
            }
            globalThis.nodeDiagnosticsChannel = {
                channel, hasSubscribers, subscribe, unsubscribe,
                tracingChannel,
                Channel, TracingChannel,
            };
        })();
    "#)?;
    Ok(())
}

fn install_node_https_perf_async_hooks_js<'js>(ctx: &Ctx<'js>) -> JsResult<()> {
    // E.17 chain: fastify and friends top-level-require these Node core
    // modules even when their consumer never exercises the HTTPS/perf
    // path. Stubs:
    //   - node:https → re-export node:http (no TLS handling yet; plain
    //     http server is what the consumer gets if they call .createServer
    //     with cert options. Acceptable for read-only/observability paths.)
    //   - node:perf_hooks → globalThis.performance + minimal observer stub.
    //   - node:async_hooks → AsyncLocalStorage with same-stack run/getStore
    //     semantics (no real async-id propagation, but the API shape sticks
    //     for the synchronous-callback usage that 90% of libs hit).
    ctx.eval::<(), _>(r#"
        (function() {
            // node:https → identical surface to node:http for the
            // create/request/get triad. TLS opts are accepted and ignored.
            globalThis.nodeHttps = {
                createServer: function(...args) {
                    // (opts, handler) or (handler) — drop opts if present.
                    const handler = typeof args[0] === "function"
                        ? args[0]
                        : (typeof args[1] === "function" ? args[1] : undefined);
                    return globalThis.nodeHttp.createServer(handler);
                },
                request: globalThis.nodeHttp.request,
                get: globalThis.nodeHttp.request,
                Server: globalThis.nodeHttp.Server,
                Agent: function Agent() {},
                globalAgent: {},
            };

            // node:perf_hooks → wrap globalThis.performance.
            class PerformanceObserver {
                constructor(_callback) {}
                observe(_opts) {}
                disconnect() {}
                takeRecords() { return []; }
                static get supportedEntryTypes() { return ["mark", "measure"]; }
            }
            globalThis.nodePerfHooks = {
                performance: globalThis.performance,
                PerformanceObserver,
                monitorEventLoopDelay: () => ({
                    enable() {}, disable() {}, reset() {},
                    min: 0, max: 0, mean: 0, stddev: 0, percentile() { return 0; },
                    percentiles: new Map(), exceeds: 0,
                }),
                constants: {
                    NODE_PERFORMANCE_GC_MAJOR: 2,
                    NODE_PERFORMANCE_GC_MINOR: 1,
                    NODE_PERFORMANCE_GC_INCREMENTAL: 4,
                    NODE_PERFORMANCE_GC_WEAKCB: 8,
                },
            };

            // node:async_hooks — AsyncLocalStorage with same-stack semantics.
            // run(store, fn, ...args) sets current store, calls fn, restores.
            // getStore() returns the active store. Real async-id linking is
            // omitted; works for the dominant pattern (sync handler chains +
            // direct awaits where the JS engine preserves variable scope).
            const _alsStack = [];
            class AsyncLocalStorage {
                constructor() { this._stack = []; }
                run(store, fn, ...args) {
                    this._stack.push(store);
                    try { return fn(...args); }
                    finally { this._stack.pop(); }
                }
                getStore() {
                    return this._stack.length > 0
                        ? this._stack[this._stack.length - 1]
                        : undefined;
                }
                enterWith(store) { this._stack.push(store); }
                disable() { this._stack.length = 0; }
                exit(fn, ...args) {
                    const saved = this._stack.slice();
                    this._stack.length = 0;
                    try { return fn(...args); }
                    finally { this._stack.push(...saved); }
                }
            }
            class AsyncResource {
                constructor(_type, _opts) {}
                runInAsyncScope(fn, thisArg, ...args) {
                    return fn.apply(thisArg, args);
                }
                bind(fn, thisArg) {
                    return fn.bind(thisArg);
                }
                emitDestroy() { return this; }
                static bind(fn, _type, thisArg) {
                    return fn.bind(thisArg);
                }
            }
            globalThis.nodeAsyncHooks = {
                AsyncLocalStorage,
                AsyncResource,
                createHook: () => ({
                    enable() { return this; },
                    disable() { return this; },
                }),
                executionAsyncId: () => 0,
                executionAsyncResource: () => ({}),
                triggerAsyncId: () => 0,
            };
        })();
    "#)?;
    Ok(())
}

fn install_intl_js<'js>(ctx: &Ctx<'js>) -> JsResult<()> {
    // S11 closure (partial): a minimal Intl.NumberFormat sufficient for
    // pretty-bytes (E.35), the date-fns/luxon (E.22) common path, and
    // any consumer using maximumFractionDigits / useGrouping. Locale
    // data is "en"-only for now — full CLDR-driven locale-data is a
    // separate substrate (likely a generated table per locale).
    //
    // L2 namespace: install globalThis.Intl.
    // L3 surface: NumberFormat, DateTimeFormat, Collator (constructors).
    // L5 semantics: en-locale number formatting (decimal, percent,
    //   currency placeholder, scientific via toExponential, grouping).
    ctx.eval::<(), _>(r#"
        (function() {
            if (typeof globalThis.Intl !== "undefined" && Intl.NumberFormat
                && (new Intl.NumberFormat("en")).format(0)) {
                return; // engine already provides Intl
            }
            const Intl = globalThis.Intl || (globalThis.Intl = {});

            function _formatNumber(n, opts) {
                if (!isFinite(n)) {
                    if (isNaN(n)) return "NaN";
                    return n > 0 ? "∞" : "-∞";
                }
                const sign = n < 0 ? "-" : (opts.signDisplay === "always" || opts.signDisplay === "exceptZero" && n !== 0 ? "+" : "");
                n = Math.abs(n);

                const minI = opts.minimumIntegerDigits ?? 1;
                const minF = opts.minimumFractionDigits;
                const maxF = opts.maximumFractionDigits;
                const minS = opts.minimumSignificantDigits;
                const maxS = opts.maximumSignificantDigits;

                let s;
                if (minS !== undefined || maxS !== undefined) {
                    const ms = maxS ?? 21;
                    s = n.toPrecision(ms);
                    // toPrecision may emit scientific notation for very
                    // large/small values; expand to fixed if so.
                    if (s.indexOf("e") >= 0) s = Number(s).toFixed(20).replace(/0+$/, "").replace(/\.$/, "");
                    // Strip trailing zeros below minS.
                    const minsN = minS ?? 1;
                    if (s.indexOf(".") >= 0) {
                        // Keep only minS significant digits (already produced) but
                        // trim trailing zero pads beyond maxS.
                    }
                } else {
                    if (maxF !== undefined) {
                        const m = Math.pow(10, maxF);
                        const mode = opts.roundingMode || "halfExpand";
                        const x = n * m;
                        let rounded;
                        switch (mode) {
                            case "ceil":        rounded = Math.ceil(x); break;
                            case "floor":       rounded = Math.floor(x); break;
                            case "expand":      rounded = x >= 0 ? Math.ceil(x) : Math.floor(x); break;
                            case "trunc":       rounded = Math.trunc(x); break;
                            case "halfCeil":    rounded = Math.round(x); /* close enough */ break;
                            case "halfFloor":   rounded = -Math.round(-x); break;
                            case "halfExpand":  rounded = x >= 0 ? Math.floor(x + 0.5) : -Math.floor(-x + 0.5); break;
                            case "halfTrunc":   rounded = x >= 0 ? Math.ceil(x - 0.5) : -Math.ceil(-x - 0.5); break;
                            case "halfEven":    {
                                const f = Math.floor(x);
                                const diff = x - f;
                                if (diff < 0.5) rounded = f;
                                else if (diff > 0.5) rounded = f + 1;
                                else rounded = (f % 2 === 0) ? f : f + 1;
                                break;
                            }
                            default:            rounded = Math.round(x);
                        }
                        n = rounded / m;
                    }
                    s = n.toString();
                    if (s.indexOf("e") >= 0) {
                        s = n.toFixed(maxF ?? 20);
                        if (maxF === undefined) s = s.replace(/0+$/, "").replace(/\.$/, "");
                    }
                    if (minF !== undefined) {
                        const dot = s.indexOf(".");
                        const fracLen = dot < 0 ? 0 : s.length - dot - 1;
                        if (fracLen < minF) {
                            s = (dot < 0 ? s + "." : s) + "0".repeat(minF - fracLen);
                        }
                    }
                }

                let intPart = s, fracPart = "";
                const dotIdx = s.indexOf(".");
                if (dotIdx >= 0) {
                    intPart = s.substring(0, dotIdx);
                    fracPart = s.substring(dotIdx + 1);
                }

                if (intPart.length < minI) {
                    intPart = "0".repeat(minI - intPart.length) + intPart;
                }

                if (opts.useGrouping !== false && opts.useGrouping !== "false") {
                    intPart = intPart.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
                }

                let out = fracPart ? intPart + "." + fracPart : intPart;
                out = sign + out;

                if (opts.style === "percent") out = out + "%";
                else if (opts.style === "currency" && opts.currency) {
                    // Naive prefix; real CLDR-driven layout deferred.
                    const sym = opts.currency === "USD" ? "$"
                        : opts.currency === "EUR" ? "€"
                        : opts.currency === "GBP" ? "£"
                        : opts.currency === "JPY" ? "¥"
                        : opts.currency + " ";
                    out = sym + out;
                }
                return out;
            }

            class NumberFormat {
                constructor(locales, options) {
                    options = options || {};
                    this._locale = Array.isArray(locales) ? (locales[0] || "en") : (locales || "en");
                    if (options.style === "percent" && options.maximumFractionDigits === undefined) {
                        options = Object.assign({}, options, { maximumFractionDigits: 0 });
                    }
                    this._opts = options;
                }
                format(value) {
                    return _formatNumber(Number(value), this._opts);
                }
                formatToParts(value) {
                    const s = this.format(value);
                    // Minimal single-part — consumers that branch on
                    // formatToParts get one "integer" + one "decimal" +
                    // one "fraction"; not full CLDR parts.
                    const dot = s.indexOf(".");
                    if (dot < 0) return [{ type: "integer", value: s }];
                    return [
                        { type: "integer", value: s.substring(0, dot) },
                        { type: "decimal", value: "." },
                        { type: "fraction", value: s.substring(dot + 1) },
                    ];
                }
                resolvedOptions() {
                    return Object.assign({ locale: this._locale, numberingSystem: "latn" }, this._opts);
                }
            }
            NumberFormat.supportedLocalesOf = function(locales) {
                return Array.isArray(locales) ? locales.slice() : [locales];
            };
            Intl.NumberFormat = NumberFormat;

            // Minimal Intl.DateTimeFormat — locale "en" en-US shape.
            // Many consumer libs call .format(date) expecting a string;
            // a small subset use formatToParts. CLDR locale data full
            // closure is deferred.
            class DateTimeFormat {
                constructor(locales, options) {
                    options = options || {};
                    this._locale = Array.isArray(locales) ? (locales[0] || "en") : (locales || "en");
                    this._opts = options;
                }
                format(date) {
                    date = date instanceof Date ? date : new Date(date);
                    const o = this._opts;
                    const pad = (n, w = 2) => String(n).padStart(w, "0");
                    const y = date.getFullYear();
                    const mo = date.getMonth() + 1;
                    const d = date.getDate();
                    const h = date.getHours();
                    const mi = date.getMinutes();
                    const s = date.getSeconds();
                    // Heuristic: dateStyle: "short" → M/D/YYYY; long/full add weekday + month name
                    if (o.dateStyle === "short" && !o.timeStyle) return mo + "/" + d + "/" + y;
                    if (o.dateStyle === "long" && !o.timeStyle) {
                        const months = ["January","February","March","April","May","June",
                                        "July","August","September","October","November","December"];
                        return months[mo-1] + " " + d + ", " + y;
                    }
                    // Default ISO-like fallback.
                    return mo + "/" + d + "/" + y + ", " + h + ":" + pad(mi) + ":" + pad(s);
                }
                formatToParts(date) {
                    return [{ type: "literal", value: this.format(date) }];
                }
                resolvedOptions() {
                    return Object.assign({ locale: this._locale, timeZone: "UTC" }, this._opts);
                }
            }
            DateTimeFormat.supportedLocalesOf = function(locales) {
                return Array.isArray(locales) ? locales.slice() : [locales];
            };
            Intl.DateTimeFormat = DateTimeFormat;

            // Intl.Collator — minimal localeCompare wrapper.
            class Collator {
                constructor(locales, options) {
                    this._locale = Array.isArray(locales) ? (locales[0] || "en") : (locales || "en");
                    this._opts = options || {};
                }
                compare(a, b) {
                    a = String(a); b = String(b);
                    if (this._opts.sensitivity === "base" || this._opts.sensitivity === "accent") {
                        a = a.toLowerCase(); b = b.toLowerCase();
                    }
                    if (this._opts.numeric) {
                        return a.localeCompare(b, this._locale, { numeric: true });
                    }
                    return a < b ? -1 : a > b ? 1 : 0;
                }
                resolvedOptions() {
                    return Object.assign({ locale: this._locale }, this._opts);
                }
            }
            Collator.supportedLocalesOf = function(locales) {
                return Array.isArray(locales) ? locales.slice() : [locales];
            };
            Intl.Collator = Collator;

            // Intl.PluralRules — English-only stub.
            class PluralRules {
                constructor(locales, options) {
                    this._locale = Array.isArray(locales) ? (locales[0] || "en") : (locales || "en");
                    this._opts = options || {};
                }
                select(n) {
                    n = Math.abs(Number(n));
                    if (this._opts.type === "ordinal") {
                        if (n % 100 >= 11 && n % 100 <= 13) return "other";
                        if (n % 10 === 1) return "one";
                        if (n % 10 === 2) return "two";
                        if (n % 10 === 3) return "few";
                        return "other";
                    }
                    return n === 1 ? "one" : "other";
                }
                resolvedOptions() {
                    return Object.assign({ locale: this._locale }, this._opts);
                }
            }
            PluralRules.supportedLocalesOf = function(locales) {
                return Array.isArray(locales) ? locales.slice() : [locales];
            };
            Intl.PluralRules = PluralRules;

            // Intl.Segmenter — string-width, slice-words, and any
            // grapheme/word/sentence iteration use this. Real impl uses
            // CLDR's break tables; we approximate with simple iteration:
            //   granularity 'grapheme': iterate code points (UTF-16
            //     surrogate-pair aware) treating each code point as a
            //     segment. Misses combining-mark/joiner clustering but
            //     covers the dominant cases.
            //   granularity 'word': split on whitespace/word boundaries
            //     via a regex.
            //   granularity 'sentence': split on sentence-ending punct.
            class Segmenter {
                constructor(locales, options) {
                    this._locale = Array.isArray(locales) ? (locales[0] || "en") : (locales || "en");
                    this._opts = options || { granularity: "grapheme" };
                    this._gran = this._opts.granularity || "grapheme";
                }
                segment(str) {
                    str = String(str);
                    const gran = this._gran;
                    // Precompute segment boundaries so the Segments object
                    // can support both [Symbol.iterator] and .containing(i).
                    const segs = [];
                    if (gran === "grapheme") {
                        let i = 0;
                        while (i < str.length) {
                            const code = str.codePointAt(i);
                            const w = code > 0xFFFF ? 2 : 1;
                            segs.push({ segment: str.substring(i, i + w), index: i, input: str });
                            i += w;
                        }
                    } else if (gran === "word") {
                        let i = 0;
                        while (i < str.length) {
                            const start = i;
                            if (/\s/.test(str[i])) {
                                while (i < str.length && /\s/.test(str[i])) i++;
                            } else {
                                while (i < str.length && !/\s/.test(str[i])) i++;
                            }
                            const segment = str.substring(start, i);
                            segs.push({ segment, index: start, input: str, isWordLike: /\w/.test(segment) });
                        }
                    } else {
                        // sentence
                        let i = 0;
                        while (i < str.length) {
                            const start = i;
                            while (i < str.length && !/[.!?]/.test(str[i])) i++;
                            if (i < str.length) i++;
                            while (i < str.length && /\s/.test(str[i])) i++;
                            segs.push({ segment: str.substring(start, i), index: start, input: str });
                        }
                    }
                    return {
                        [Symbol.iterator]() {
                            let k = 0;
                            return {
                                next() {
                                    if (k >= segs.length) return { done: true };
                                    return { value: segs[k++], done: false };
                                },
                            };
                        },
                        // Intl.Segments.containing(codeUnitIndex) — returns
                        // the segment containing the code unit, or undefined.
                        // slice-ansi (and others) use this for cluster-aware
                        // truncation.
                        containing(index) {
                            index = Number(index) || 0;
                            if (index < 0 || index >= str.length) return undefined;
                            for (const s of segs) {
                                if (s.index <= index && index < s.index + s.segment.length) return s;
                            }
                            return undefined;
                        },
                    };
                }
                resolvedOptions() {
                    return Object.assign({ locale: this._locale, granularity: this._gran }, this._opts);
                }
            }
            Segmenter.supportedLocalesOf = function(locales) {
                return Array.isArray(locales) ? locales.slice() : [locales];
            };
            Intl.Segmenter = Segmenter;

            // Intl.ListFormat — and-conjunction English stub.
            class ListFormat {
                constructor(locales, options) {
                    this._locale = Array.isArray(locales) ? (locales[0] || "en") : (locales || "en");
                    this._opts = options || {};
                }
                format(list) {
                    const arr = Array.from(list).map(String);
                    if (arr.length === 0) return "";
                    if (arr.length === 1) return arr[0];
                    if (arr.length === 2) return arr.join(this._opts.type === "disjunction" ? " or " : " and ");
                    const conj = this._opts.type === "disjunction" ? "or" : "and";
                    return arr.slice(0, -1).join(", ") + ", " + conj + " " + arr[arr.length - 1];
                }
            }
            Intl.ListFormat = ListFormat;

            // Intl.RelativeTimeFormat — English stub.
            class RelativeTimeFormat {
                constructor(locales, options) {
                    this._locale = Array.isArray(locales) ? (locales[0] || "en") : (locales || "en");
                    this._opts = options || { numeric: "always" };
                }
                format(value, unit) {
                    const v = Number(value);
                    const u = String(unit).replace(/s$/, "");
                    if (this._opts.numeric === "auto") {
                        if (v === 0 && u === "day") return "today";
                        if (v === 1 && u === "day") return "tomorrow";
                        if (v === -1 && u === "day") return "yesterday";
                    }
                    if (v >= 0) return "in " + v + " " + u + (Math.abs(v) === 1 ? "" : "s");
                    return Math.abs(v) + " " + u + (Math.abs(v) === 1 ? "" : "s") + " ago";
                }
            }
            Intl.RelativeTimeFormat = RelativeTimeFormat;

            Intl.getCanonicalLocales = function(locales) {
                if (!locales) return [];
                if (typeof locales === "string") return [locales];
                return Array.from(locales);
            };

            // Per spec, Number#toLocaleString(locales, options) is
            // equivalent to new Intl.NumberFormat(locales, options).format(this).
            // pretty-bytes (E.35), formatter libs, dashboards depend on
            // this to honor maximumFractionDigits / useGrouping.
            const _origNumberToLocaleString = Number.prototype.toLocaleString;
            Number.prototype.toLocaleString = function(locales, options) {
                if (options || (locales !== undefined && locales !== null)) {
                    return new NumberFormat(locales, options).format(this.valueOf());
                }
                return _origNumberToLocaleString.call(this);
            };

            // Date#toLocaleString / toLocaleDateString / toLocaleTimeString.
            const _origDateToLocaleString = Date.prototype.toLocaleString;
            Date.prototype.toLocaleString = function(locales, options) {
                if (options || (locales !== undefined && locales !== null)) {
                    return new DateTimeFormat(locales, options).format(this);
                }
                return _origDateToLocaleString.call(this);
            };

            // String#localeCompare composes on Collator.
            const _origStringLocaleCompare = String.prototype.localeCompare;
            String.prototype.localeCompare = function(other, locales, options) {
                if (options) return new Collator(locales, options).compare(this.valueOf(), other);
                return _origStringLocaleCompare.call(this, other, locales);
            };
        })();
    "#)?;
    Ok(())
}

fn install_node_extra_builtins_js<'js>(ctx: &Ctx<'js>) -> JsResult<()> {
    // E.17 chain continued: node:timers, node:timers/promises, node:console,
    // node:fs/promises, node:stream/web, node:test, node:worker_threads.
    // All compose on existing surfaces (globalThis setTimeout, console,
    // ReadableStream, fs); test + worker_threads are throwing stubs since
    // fastify and friends top-level-import them but only use them under
    // opt-in flags.
    ctx.eval::<(), _>(r#"
        (function() {
            globalThis.nodeTimers = {
                setTimeout: globalThis.setTimeout,
                setInterval: globalThis.setInterval,
                setImmediate: globalThis.setImmediate,
                clearTimeout: globalThis.clearTimeout,
                clearInterval: globalThis.clearInterval,
                clearImmediate: globalThis.clearImmediate,
            };

            // node:timers/promises — Promise-based timer wrappers.
            globalThis.nodeTimersPromises = {
                setTimeout: function(ms, value, opts) {
                    return new Promise((resolve, reject) => {
                        const id = globalThis.setTimeout(() => resolve(value), ms);
                        if (opts && opts.signal) {
                            const sig = opts.signal;
                            if (sig.aborted) {
                                globalThis.clearTimeout(id);
                                const e = new Error("The operation was aborted");
                                e.name = "AbortError";
                                reject(e);
                                return;
                            }
                            sig.addEventListener("abort", () => {
                                globalThis.clearTimeout(id);
                                const e = new Error("The operation was aborted");
                                e.name = "AbortError";
                                reject(e);
                            });
                        }
                    });
                },
                setImmediate: function(value) {
                    return new Promise(resolve => globalThis.setImmediate(() => resolve(value)));
                },
                setInterval: async function*(ms, value) {
                    while (true) {
                        await new Promise(r => globalThis.setTimeout(r, ms));
                        yield value;
                    }
                },
                scheduler: { wait: (ms) => new Promise(r => globalThis.setTimeout(r, ms)) },
            };

            // node:console — Console class + bound methods.
            globalThis.nodeConsoleModule = {
                Console: function Console(stdout, stderr) {
                    return globalThis.console;
                },
                log: globalThis.console.log.bind(globalThis.console),
                warn: globalThis.console.warn.bind(globalThis.console),
                error: globalThis.console.error.bind(globalThis.console),
                info: globalThis.console.info
                    ? globalThis.console.info.bind(globalThis.console)
                    : globalThis.console.log.bind(globalThis.console),
                debug: globalThis.console.debug
                    ? globalThis.console.debug.bind(globalThis.console)
                    : globalThis.console.log.bind(globalThis.console),
            };

            // node:fs/promises — re-export of nodeFs as Promise-returning fns
            // where a sync counterpart exists. Real async I/O isn't pumped
            // through the event loop; we resolve synchronously.
            const fs = globalThis.fs || {};
            globalThis.nodeFsPromises = {
                readFile: async (p, opts) => {
                    if (opts && (opts.encoding || typeof opts === "string")) {
                        return fs.readFileSyncUtf8 ? fs.readFileSyncUtf8(p) : fs.readFileSync(p, "utf8");
                    }
                    return fs.readFileSyncBytes ? fs.readFileSyncBytes(p) : fs.readFileSync(p);
                },
                writeFile: async (p, data, opts) => {
                    if (fs.writeFileSync) return fs.writeFileSync(p, data, opts);
                    throw new Error("fs.writeFile not supported");
                },
                mkdir: async (p, opts) => {
                    if (fs.mkdirSyncRecursive) return fs.mkdirSyncRecursive(p);
                    throw new Error("fs.mkdir not supported");
                },
                rm: async (p, opts) => {
                    if (fs.unlinkSync) return fs.unlinkSync(p);
                    throw new Error("fs.rm not supported");
                },
                stat: async (p) => {
                    if (fs.isFileSync && fs.isDirectorySync) {
                        const isFile = fs.isFileSync(p);
                        const isDir = fs.isDirectorySync(p);
                        return { isFile: () => isFile, isDirectory: () => isDir };
                    }
                    throw new Error("fs.stat not supported");
                },
                lstat: async function(p) { return this.stat(p); },
                access: async (p) => {
                    if (!fs.existsSync(p)) throw new Error("ENOENT: " + p);
                },
                readdir: async (p, opts) => {
                    if (fs.readdirSync) return fs.readdirSync(p, opts);
                    throw new Error("fs.readdir not yet implemented");
                },
                unlink: async (p) => {
                    if (fs.unlinkSync) return fs.unlinkSync(p);
                    throw new Error("fs.unlink not supported");
                },
                readlink: async (_p) => {
                    throw new Error("fs.readlink not supported");
                },
                realpath: async (p) => p,
                rename: async (_a, _b) => { throw new Error("fs.rename not supported"); },
                rmdir: async (p) => { if (fs.rmdirSyncRecursive) return fs.rmdirSyncRecursive(p); },
                chmod: async () => undefined,
                chown: async () => undefined,
                utimes: async () => undefined,
                appendFile: async (p, data, opts) => {
                    const prior = fs.existsSync(p) ? fs.readFileSync(p, opts) : "";
                    return fs.writeFileSync(p, prior + data, opts);
                },
                copyFile: async (a, b) => {
                    const bytes = fs.readFileSync(a);
                    return fs.writeFileSync(b, bytes);
                },
                open: async (_p, _flags) => {
                    throw new Error("fs.open not supported (no FileHandle yet)");
                },
                constants: {
                    F_OK: 0, R_OK: 4, W_OK: 2, X_OK: 1,
                    O_RDONLY: 0, O_WRONLY: 1, O_RDWR: 2,
                    O_CREAT: 64, O_EXCL: 128, O_TRUNC: 512, O_APPEND: 1024,
                    COPYFILE_EXCL: 1,
                },
                watch: async function* () {},
                cp: async (src, dest) => fs.writeFileSync(dest, fs.readFileSync(src)),
                lchmod: async () => {},
                lchown: async () => {},
                link: async () => {},
                lutimes: async () => {},
                mkdtemp: async (prefix) => {
                    const suffix = Math.random().toString(36).slice(2, 8);
                    const path = prefix + suffix;
                    if (fs.mkdirSyncRecursive) fs.mkdirSyncRecursive(path);
                    return path;
                },
                opendir: async () => { throw new Error("fs.opendir not supported"); },
                symlink: async () => { throw new Error("fs.symlink not supported"); },
                truncate: async () => { throw new Error("fs.truncate not supported"); },
            };

            // node:stream/web → WHATWG streams (already on globalThis).
            globalThis.nodeStreamWeb = {
                ReadableStream: globalThis.ReadableStream,
                WritableStream: globalThis.WritableStream,
                TransformStream: globalThis.TransformStream,
                ByteLengthQueuingStrategy: globalThis.ByteLengthQueuingStrategy,
                CountQueuingStrategy: globalThis.CountQueuingStrategy,
                TextEncoderStream: globalThis.TextEncoderStream,
                TextDecoderStream: globalThis.TextDecoderStream,
            };

            // node:test → no-op stubs; the rusty-bun-host doesn't run Node's
            // built-in test runner. Libraries import this only when their
            // own test mode is active (a self-test guarded by a flag).
            const noop = () => {};
            const noopTest = (name, opts, fn) => { /* skip */ };
            noopTest.skip = noop;
            noopTest.todo = noop;
            noopTest.only = noop;
            globalThis.nodeTest = {
                test: noopTest,
                describe: noopTest,
                it: noopTest,
                before: noop, after: noop,
                beforeEach: noop, afterEach: noop,
                mock: { fn: () => () => {}, method: noop, getter: noop },
            };

            // node:worker_threads → minimal stub. We're always the "main
            // thread"; calling Worker throws.
            class Worker {
                constructor() {
                    throw new Error("rusty-bun-host: node:worker_threads.Worker not supported");
                }
            }
            globalThis.nodeWorkerThreads = {
                Worker,
                isMainThread: true,
                parentPort: null,
                workerData: undefined,
                threadId: 0,
                MessageChannel: class { constructor() { this.port1 = {}; this.port2 = {}; } },
                MessagePort: class {},
            };

            // node:http2 → throwing stub. Fastify top-level-imports this
            // but only exercises it under explicit allowHTTP2 opts (off by
            // default). Plain http path remains intact.
            const http2Throws = (name) => () => {
                throw new Error("rusty-bun-host: node:http2." + name + " not supported");
            };
            globalThis.nodeHttp2 = {
                constants: {
                    HTTP2_HEADER_PATH: ":path",
                    HTTP2_HEADER_METHOD: ":method",
                    HTTP2_HEADER_STATUS: ":status",
                },
                createServer: http2Throws("createServer"),
                createSecureServer: http2Throws("createSecureServer"),
                connect: http2Throws("connect"),
                Http2ServerRequest: class {},
                Http2ServerResponse: class {},
            };

            // node:vm — minimal stub for function-timeout and other libs
            // that import vm only for optional sandboxing. Script.run*
            // delegates to (0,eval) without true isolation or timeout
            // enforcement; consumers that depend on real timeout get
            // a documented divergence but their non-timeout path works.
            class VmScript {
                constructor(code, opts) {
                    this.code = String(code);
                    this._opts = opts || {};
                }
                runInThisContext(_opts) {
                    return (0, eval)(this.code);
                }
                runInNewContext(context, _opts) {
                    if (!context || typeof context !== "object") {
                        return (0, eval)(this.code);
                    }
                    // Wrap context in a Proxy whose `has` returns true
                    // unconditionally — inside `with (proxy) { code }`,
                    // every bare-name read AND write goes through the
                    // proxy, including assignments to names not yet
                    // present on the context (which would otherwise
                    // create globals). This is the variable-binding
                    // semantics of node:vm.Script.runInNewContext.
                    // Real isolation + timeout watchdog are deferred
                    // (the cut floor for S-vm is at L4 idiom semantics;
                    // below-floor is the isolation boundary, an
                    // accepted divergence).
                    const proxy = new Proxy(context, {
                        has() { return true; },
                        get(target, key) {
                            if (key in target) return target[key];
                            return globalThis[key];
                        },
                        set(target, key, value) {
                            target[key] = value;
                            return true;
                        },
                    });
                    const wrapped = new Function(
                        "__ctx__",
                        "with (__ctx__) { " + this.code + "\n}"
                    );
                    return wrapped(proxy);
                }
                runInContext(context, opts) { return this.runInNewContext(context, opts); }
            }
            function vmCreateContext(initial) {
                return initial ? Object.assign({}, initial) : {};
            }
            globalThis.nodeVm = {
                Script: VmScript,
                createContext: vmCreateContext,
                runInNewContext(code, ctx, opts) {
                    return new VmScript(code, opts).runInNewContext(ctx, opts);
                },
                runInThisContext(code, opts) {
                    return new VmScript(code, opts).runInThisContext(opts);
                },
                compileFunction(code, params, opts) {
                    return new Function(...(params || []), code);
                },
                isContext(_v) { return true; },
            };

            // node:string_decoder — Bun's runtime composes streams that
            // use StringDecoder for chunk-boundary-aware utf8 decoding.
            // glob's minified bundle pulls it through node:stream's
            // Readable. Our impl uses TextDecoder with stream: true,
            // which honors multi-byte UTF-8 boundaries the same way.
            class StringDecoder {
                constructor(encoding) {
                    this._enc = (encoding || "utf8").toLowerCase().replace(/-/g, "");
                    if (this._enc === "utf8") {
                        this._dec = new TextDecoder("utf-8", { fatal: false });
                    } else if (this._enc === "latin1" || this._enc === "binary") {
                        this._dec = new TextDecoder("latin1", { fatal: false });
                    } else if (this._enc === "ucs2" || this._enc === "utf16le") {
                        // No TextDecoder for utf-16le in our polyfill; emulate.
                        this._dec = null;
                        this._utf16le = true;
                        this._pending = null;
                    } else {
                        this._dec = new TextDecoder("utf-8", { fatal: false });
                    }
                }
                _decodeUtf16le(buf, final) {
                    let bytes = buf;
                    if (this._pending) {
                        const merged = new Uint8Array(this._pending.length + bytes.length);
                        merged.set(this._pending); merged.set(bytes, this._pending.length);
                        bytes = merged;
                        this._pending = null;
                    }
                    const evenLen = bytes.length & ~1;
                    let s = "";
                    for (let i = 0; i + 1 < evenLen; i += 2) {
                        s += String.fromCharCode(bytes[i] | (bytes[i + 1] << 8));
                    }
                    if (!final && evenLen < bytes.length) {
                        this._pending = bytes.slice(evenLen);
                    }
                    return s;
                }
                write(buf) {
                    const u8 = buf === undefined || buf === null ? new Uint8Array(0)
                        : (buf instanceof Uint8Array ? buf : new Uint8Array(buf));
                    if (this._utf16le) return this._decodeUtf16le(u8, false);
                    return this._dec.decode(u8, { stream: true });
                }
                end(buf) {
                    if (this._utf16le) {
                        const u8 = buf === undefined || buf === null ? new Uint8Array(0)
                            : (buf instanceof Uint8Array ? buf : new Uint8Array(buf));
                        return this._decodeUtf16le(u8, true);
                    }
                    if (buf !== undefined && buf !== null) {
                        const u8 = buf instanceof Uint8Array ? buf : new Uint8Array(buf);
                        return this._dec.decode(u8);
                    }
                    return this._dec.decode();
                }
            }
            globalThis.nodeStringDecoder = { StringDecoder };

            // node:readline — REPL-style line reader. Most consumers
            // import this only for completeness (some node tooling
            // libraries import it for terminal detection); we stub a
            // throwing createInterface and accept the divergence per
            // the L5 cut.
            const rlThrows = (n) => () => {
                throw new Error("rusty-bun-host: node:readline." + n + " not supported");
            };
            globalThis.nodeReadline = {
                createInterface: rlThrows("createInterface"),
                Interface: class {},
                emitKeypressEvents() {},
                clearLine() { return true; },
                clearScreenDown() { return true; },
                cursorTo() { return true; },
                moveCursor() { return true; },
            };
            globalThis.nodeReadlinePromises = {
                createInterface: rlThrows("createInterface"),
                Interface: class {},
                Readline: class {},
            };

            // node:module — createRequire is the common consumer call;
            // returns a require fn that resolves from a given URL/path.
            // We compose on globalThis.require which already resolves
            // node:* builtins and walks node_modules from cwd. yargs-parser,
            // many ESM-from-CJS libs use this.
            const builtinModulesList = [
                "fs","path","http","https","crypto","buffer","url","os",
                "process","dns","events","util","stream","querystring",
                "assert","child_process","net","tty","zlib","vm",
                "diagnostics_channel","perf_hooks","async_hooks","timers",
                "timers/promises","console","fs/promises","stream/web",
                "test","worker_threads","http2","string_decoder","readline",
                "readline/promises","module"
            ];
            globalThis.nodeModule = {
                createRequire(url) {
                    // Per Node spec: returns a require() bound to the URL's
                    // containing directory. Strip file:// prefix and use the
                    // dirname as the resolution anchor.
                    let path = String(url);
                    if (path.startsWith("file://")) path = path.slice(7);
                    const slash = path.lastIndexOf("/");
                    const dir = slash > 0 ? path.substring(0, slash) : "/";
                    const NB = globalThis.__cjs.NODE_BUILTINS;
                    const r = function require(spec) {
                        if (typeof spec !== "string") throw new TypeError("require: spec must be a string");
                        if (NB && Object.prototype.hasOwnProperty.call(NB, spec)) {
                            return NB[spec]();
                        }
                        const resolved = globalThis.__cjs.resolvePath(spec, dir);
                        return globalThis.__cjs.loadModule(resolved);
                    };
                    r.cache = globalThis.__cjs.moduleCache;
                    r.resolve = function (spec) {
                        if (NB && Object.prototype.hasOwnProperty.call(NB, spec)) return spec;
                        return globalThis.__cjs.resolvePath(spec, dir);
                    };
                    return r;
                },
                builtinModules: builtinModulesList,
                isBuiltin(name) {
                    const n = name.startsWith("node:") ? name.substring(5) : name;
                    return builtinModulesList.includes(n);
                },
                Module: class {},
                syncBuiltinESMExports() {},
            };
            // node:v8 stub — V8-specific module. We use QuickJS, so the
            // engine-specific bits don't apply; consumers (prettier's bundled
            // utilities, some logging libs) check typeof for fallback paths.
            globalThis.nodeV8 = {
                serialize(value) {
                    // structuredClone via JSON for the safe-subset; real Node
                    // uses HostObject IDs for cycles/typed-arrays. Sufficient
                    // for consumers that round-trip plain data.
                    return new TextEncoder().encode(JSON.stringify(value));
                },
                deserialize(buf) {
                    const bytes = buf instanceof Uint8Array ? buf : new Uint8Array(buf);
                    return JSON.parse(new TextDecoder().decode(bytes));
                },
                getHeapStatistics() {
                    return {
                        total_heap_size: 0, total_heap_size_executable: 0,
                        total_physical_size: 0, total_available_size: 0,
                        used_heap_size: 0, heap_size_limit: 0,
                        malloced_memory: 0, peak_malloced_memory: 0,
                        does_zap_garbage: 0, number_of_native_contexts: 1,
                        number_of_detached_contexts: 0,
                    };
                },
                getHeapSpaceStatistics() { return []; },
                cachedDataVersionTag() { return 0; },
                setFlagsFromString() {},
                writeHeapSnapshot() { return ""; },
                // V8 promise/microtask helpers used by some libs
                getHeapCodeStatistics() {
                    return { code_and_metadata_size: 0, bytecode_and_metadata_size: 0,
                             external_script_source_size: 0 };
                },
            };
            // node:constants — deprecated namespace alias for os.constants
            // and fs constants. Stub with the most-commonly-checked values;
            // libs typically just check existence not exact byte values.
            globalThis.nodeConstants = {
                O_RDONLY: 0, O_WRONLY: 1, O_RDWR: 2,
                O_CREAT: 64, O_EXCL: 128, O_TRUNC: 512, O_APPEND: 1024,
                S_IFMT: 0o170000, S_IFREG: 0o100000, S_IFDIR: 0o40000,
                S_IFLNK: 0o120000, S_IFCHR: 0o20000, S_IFBLK: 0o60000,
                EACCES: 13, EEXIST: 17, ENOENT: 2, EPERM: 1, EBADF: 9,
                EINVAL: 22, EIO: 5, EISDIR: 21, ENOTDIR: 20, ENOSPC: 28,
            };
            // node:cluster stub — rusty-bun is single-process. Master flag
            // is true, worker is undefined. Surface matches the read-mostly
            // pattern (libs check isMaster / worker.id to derive a unique
            // ID and otherwise no-op).
            globalThis.nodeCluster = {
                isMaster: true,
                isPrimary: true,
                isWorker: false,
                worker: undefined,
                workers: {},
                schedulingPolicy: 2,
                SCHED_NONE: 1,
                SCHED_RR: 2,
                fork() { return null; },
                disconnect(cb) { if (typeof cb === "function") cb(); },
                setupMaster() {},
                setupPrimary() {},
                on() {},
                off() {},
                once() {},
                emit() { return false; },
                removeListener() {},
                removeAllListeners() {},
            };
        })();
    "#)?;
    Ok(())
}

fn install_node_net_js<'js>(ctx: &Ctx<'js>) -> JsResult<()> {
    // node:net — minimal Socket class composing on Π2.6.b non-blocking TCP.
    // Surface: net.Socket / net.connect(port, host?, cb?) / .createConnection /
    // EventEmitter shape (on/once/emit) with data/error/end/close/connect events.
    // Server (net.createServer) deferred — Bun.serve covers the HTTP-server case.
    ctx.eval::<(), _>(r#"
        (function() {
            class Socket {
                constructor() {
                    this._sid = null;
                    this._listeners = new Map();
                    this._closed = false;
                    this._encoding = null;
                }
                on(ev, cb) {
                    if (!this._listeners.has(ev)) this._listeners.set(ev, []);
                    this._listeners.get(ev).push({ cb, once: false });
                    return this;
                }
                once(ev, cb) {
                    if (!this._listeners.has(ev)) this._listeners.set(ev, []);
                    this._listeners.get(ev).push({ cb, once: true });
                    return this;
                }
                off(ev, cb) {
                    const arr = this._listeners.get(ev);
                    if (!arr) return this;
                    this._listeners.set(ev, arr.filter(l => l.cb !== cb));
                    return this;
                }
                removeListener(ev, cb) { return this.off(ev, cb); }
                emit(ev, ...args) {
                    const arr = this._listeners.get(ev);
                    if (!arr || arr.length === 0) return false;
                    const snapshot = arr.slice();
                    this._listeners.set(ev, arr.filter(l => !l.once));
                    for (const l of snapshot) {
                        try { l.cb.apply(this, args); } catch (e) {
                            if (ev !== "error") this.emit("error", e);
                        }
                    }
                    return true;
                }
                setEncoding(enc) { this._encoding = enc; return this; }
                connect(port, host, cb) {
                    if (typeof host === "function") { cb = host; host = "127.0.0.1"; }
                    host = host || "127.0.0.1";
                    if (cb) this.once("connect", cb);
                    try {
                        this._sid = globalThis.TCP.connect(host + ":" + port);
                        globalThis.TCP.setNonblocking(this._sid, true);
                    } catch (e) {
                        queueMicrotask(() => this.emit("error", e));
                        return this;
                    }
                    // Register in __keepAlive so the eval loop pumps __tick.
                    if (globalThis.__keepAlive) globalThis.__keepAlive.add(this);
                    queueMicrotask(() => this.emit("connect"));
                    return this;
                }
                write(data, encOrCb, cb) {
                    if (typeof encOrCb === "function") cb = encOrCb;
                    if (this._closed || this._sid == null) return false;
                    const bytes = (typeof data === "string")
                        ? new TextEncoder().encode(data)
                        : (data instanceof Uint8Array ? data : new Uint8Array(data));
                    globalThis.TCP.writeAll(this._sid, bytes);
                    if (cb) queueMicrotask(cb);
                    return true;
                }
                end(data, encOrCb) {
                    if (data !== undefined) this.write(data, encOrCb);
                    this._endRequested = true;
                    return this;
                }
                destroy(err) {
                    if (this._closed) return;
                    this._closed = true;
                    if (this._sid != null) {
                        try { globalThis.TCP.close(this._sid); } catch (_) {}
                        this._sid = null;
                    }
                    if (globalThis.__keepAlive) globalThis.__keepAlive.delete(this);
                    if (err) this.emit("error", err);
                    queueMicrotask(() => this.emit("close", !!err));
                }
                // __tick: called by the eval-loop keep-alive pump. Reads any
                // available bytes from the socket, emits "data" event, handles
                // EOF + close lifecycle. Returns true if work was done.
                __tick() {
                    if (this._closed || this._sid == null) return false;
                    const chunk = globalThis.TCP.tryRead(this._sid, 65536);
                    if (chunk === null) return false; // would-block
                    if (chunk.length === 0) {
                        // FIN — orderly close
                        this.emit("end");
                        this.destroy();
                        return true;
                    }
                    const payload = this._encoding === "utf8" || this._encoding === "utf-8"
                        ? new TextDecoder().decode(chunk)
                        : (typeof Buffer !== "undefined" ? Buffer.from(chunk) : chunk);
                    this.emit("data", payload);
                    if (this._endRequested) {
                        this.destroy();
                    }
                    return true;
                }
            }

            function connect(arg1, arg2, arg3) {
                // net.connect(port, host?, cb?) | net.connect({port, host}, cb?)
                let port, host, cb;
                if (typeof arg1 === "object") {
                    port = arg1.port; host = arg1.host || "127.0.0.1"; cb = arg2;
                } else {
                    port = arg1; host = arg2; cb = arg3;
                    if (typeof host === "function") { cb = host; host = "127.0.0.1"; }
                }
                const s = new Socket();
                s.connect(port, host, cb);
                return s;
            }

            globalThis.nodeNet = {
                Socket,
                connect,
                createConnection: connect,
                createServer: () => {
                    throw new Error("net.createServer not implemented; use Bun.serve for HTTP");
                },
                isIP: (s) => {
                    if (typeof s !== "string") return 0;
                    // IPv4: a.b.c.d with octets 0-255
                    if (/^(\d{1,3}\.){3}\d{1,3}$/.test(s)) {
                        const parts = s.split(".").map(Number);
                        if (parts.every(o => o >= 0 && o <= 255)) return 4;
                    }
                    // IPv6: at least one : and all hex/colons
                    if (s.includes(":") && /^[0-9a-fA-F:]+$/.test(s)) return 6;
                    return 0;
                },
                isIPv4: function (s) { return this.isIP(s) === 4; },
                isIPv6: function (s) { return this.isIP(s) === 6; },
                BlockList: class BlockList {
                    constructor() { this._rules = []; }
                    addAddress() {}
                    addRange() {}
                    addSubnet() {}
                    check() { return false; }
                },
                SocketAddress: class SocketAddress {
                    constructor(opts) { Object.assign(this, opts || {}); }
                },
            };

            // node:tls — TLSSocket composing on the Π1.4 __tls registry.
            // Surface matches node:net.Socket so consumers that swap net→tls
            // (nodemailer, mongodb driver, redis client) get the same shape.
            class TLSSocket {
                constructor() {
                    this._sid = null;
                    this._listeners = new Map();
                    this._closed = false;
                    this._encoding = null;
                    this.authorized = true;
                }
                on(ev, cb) {
                    if (!this._listeners.has(ev)) this._listeners.set(ev, []);
                    this._listeners.get(ev).push({ cb, once: false });
                    return this;
                }
                once(ev, cb) {
                    if (!this._listeners.has(ev)) this._listeners.set(ev, []);
                    this._listeners.get(ev).push({ cb, once: true });
                    return this;
                }
                off(ev, cb) {
                    const arr = this._listeners.get(ev);
                    if (!arr) return this;
                    this._listeners.set(ev, arr.filter(l => l.cb !== cb));
                    return this;
                }
                removeListener(ev, cb) { return this.off(ev, cb); }
                emit(ev, ...args) {
                    const arr = this._listeners.get(ev);
                    if (!arr || arr.length === 0) return false;
                    const snapshot = arr.slice();
                    this._listeners.set(ev, arr.filter(l => !l.once));
                    for (const l of snapshot) {
                        try { l.cb.apply(this, args); } catch (e) {
                            if (ev !== "error") this.emit("error", e);
                        }
                    }
                    return true;
                }
                setEncoding(enc) { this._encoding = enc; return this; }
                connect(opts, cb) {
                    const host = opts.host || opts.servername || "127.0.0.1";
                    const port = opts.port || 443;
                    if (cb) this.once("secureConnect", cb);
                    // Load CA bundle once via the same fallback chain fetch uses.
                    let caPem = null;
                    if (opts.ca) caPem = opts.ca;
                    else {
                        const env = (process && process.env) || {};
                        const candidates = [env.RUSTY_BUN_CA, env.NODE_EXTRA_CA_CERTS,
                            "/etc/ssl/certs/ca-certificates.crt",
                            "/etc/pki/tls/certs/ca-bundle.crt",
                            "/etc/ssl/cert.pem"].filter(Boolean);
                        for (const p of candidates) {
                            try { caPem = require("node:fs").readFileSync(p, "utf8"); if (caPem) break; }
                            catch (_) {}
                        }
                    }
                    if (!caPem) {
                        queueMicrotask(() => this.emit("error", new Error("node:tls: no CA bundle found")));
                        return this;
                    }
                    try {
                        this._sid = globalThis.__tls.connect(host, port, caPem);
                    } catch (e) {
                        queueMicrotask(() => this.emit("error", e));
                        return this;
                    }
                    if (globalThis.__keepAlive) globalThis.__keepAlive.add(this);
                    queueMicrotask(() => {
                        this.emit("secureConnect");
                        this.emit("connect");
                    });
                    return this;
                }
                write(data, encOrCb, cb) {
                    if (typeof encOrCb === "function") cb = encOrCb;
                    if (this._closed || this._sid == null) return false;
                    const bytes = (typeof data === "string")
                        ? new TextEncoder().encode(data)
                        : (data instanceof Uint8Array ? data : new Uint8Array(data));
                    globalThis.__tls.write(this._sid, Array.from(bytes));
                    if (cb) queueMicrotask(cb);
                    return true;
                }
                end(data, encOrCb) {
                    if (data !== undefined) this.write(data, encOrCb);
                    this._endRequested = true;
                    return this;
                }
                destroy(err) {
                    if (this._closed) return;
                    this._closed = true;
                    if (this._sid != null) {
                        try { globalThis.__tls.close(this._sid); } catch (_) {}
                        this._sid = null;
                    }
                    if (globalThis.__keepAlive) globalThis.__keepAlive.delete(this);
                    if (err) this.emit("error", err);
                    queueMicrotask(() => this.emit("close", !!err));
                }
                __tick() {
                    if (this._closed || this._sid == null) return false;
                    const chunk = globalThis.__tls.read(this._sid);
                    if (chunk === null || chunk.length === 0) {
                        // For now treat empty as no-data (we don't have a non-blocking
                        // tryRead for TLS). Consumers that need streaming pull bytes
                        // via explicit calls.
                        return false;
                    }
                    const payload = this._encoding === "utf8" || this._encoding === "utf-8"
                        ? new TextDecoder().decode(new Uint8Array(chunk))
                        : (typeof Buffer !== "undefined" ? Buffer.from(chunk) : new Uint8Array(chunk));
                    this.emit("data", payload);
                    if (this._endRequested) { this.destroy(); }
                    return true;
                }
                getPeerCertificate() { return {}; }
                getCipher() { return { name: "TLS_AES_128_GCM_SHA256", version: "TLSv1.3" }; }
                getProtocol() { return "TLSv1.3"; }
            }

            function tlsConnect(opts, cb) {
                // tls.connect(port, host?, opts?, cb?) | tls.connect(opts, cb?)
                if (typeof opts === "number") {
                    const port = opts;
                    let host = "127.0.0.1", options = {}, callback = null;
                    for (const a of [arguments[1], arguments[2], arguments[3]]) {
                        if (typeof a === "string") host = a;
                        else if (typeof a === "function") callback = a;
                        else if (a && typeof a === "object") options = a;
                    }
                    opts = Object.assign({ port, host }, options);
                    cb = callback;
                }
                const s = new TLSSocket();
                s.connect(opts, cb);
                return s;
            }

            globalThis.nodeTls = {
                TLSSocket,
                connect: tlsConnect,
                createSecureContext: () => ({}),
                rootCertificates: [],
                DEFAULT_MIN_VERSION: "TLSv1.2",
                DEFAULT_MAX_VERSION: "TLSv1.3",
                DEFAULT_CIPHERS: "",
                checkServerIdentity: () => undefined,
                createServer: () => {
                    throw new Error("tls.createServer not implemented; use Bun.serve for HTTPS");
                },
            };
        })();
    "#)?;
    Ok(())
}

fn install_node_child_process_js<'js>(ctx: &Ctx<'js>) -> JsResult<()> {
    // node:child_process minimal shim. spawnSync composes on Bun.spawnSync
    // (the already-wired std::process::Command wrapper); other entry points
    // throw a clear error if called. Many libraries top-level-require this
    // for optional features (commander's executable subcommands) but never
    // actually invoke it under normal usage — those libs now load cleanly.
    ctx.eval::<(), _>(r#"
        (function() {
            const stub = (name) => () => {
                throw new Error("rusty-bun-host: node:child_process." + name +
                    " not implemented (only spawnSync composes on Bun.spawnSync)");
            };
            // ChildProcess class — stub for libs that import the class
            // (typeof check + instanceof). Real subprocess work routes
            // through spawnSync.
            class ChildProcess {
                constructor() {
                    this.stdin = null; this.stdout = null; this.stderr = null;
                    this.pid = -1; this.exitCode = null; this.signalCode = null;
                    this.killed = false; this._listeners = new Map();
                }
                on(ev, cb) {
                    if (!this._listeners.has(ev)) this._listeners.set(ev, []);
                    this._listeners.get(ev).push(cb); return this;
                }
                off() { return this; }
                kill() { this.killed = true; return true; }
                disconnect() {}
                ref() {} unref() {}
                send() { return false; }
            }
            globalThis.nodeChildProcess = {
                ChildProcess,
                spawnSync(command, args, options) {
                    const opts = options || {};
                    const argv = [String(command)].concat((args || []).map(String));
                    const stdinText = (typeof opts.input === "string") ? opts.input :
                        (opts.input && opts.input.toString) ? opts.input.toString() : undefined;
                    const r = Bun.spawnSync(argv, {
                        stdin: stdinText,
                        cwd: opts.cwd,
                    });
                    return {
                        pid: 0,
                        status: r.exitCode,
                        signal: null,
                        output: [null, r.stdout, r.stderr],
                        stdout: typeof Buffer !== "undefined" ? Buffer.from(r.stdout) : r.stdout,
                        stderr: typeof Buffer !== "undefined" ? Buffer.from(r.stderr) : r.stderr,
                    };
                },
                execSync(cmd, options) {
                    const opts = options || {};
                    const argv = ["/bin/sh", "-c", String(cmd)];
                    const r = Bun.spawnSync(argv, { cwd: opts.cwd });
                    if (r.exitCode !== 0) {
                        const err = new Error("Command failed: " + cmd);
                        err.status = r.exitCode;
                        err.stderr = r.stderr;
                        throw err;
                    }
                    return typeof Buffer !== "undefined" ? Buffer.from(r.stdout) : r.stdout;
                },
                spawn: stub("spawn"),
                exec: stub("exec"),
                execFile: stub("execFile"),
                execFileSync: stub("execFileSync"),
                fork: stub("fork"),
            };
        })();
    "#)?;
    Ok(())
}

fn install_node_assert_js<'js>(ctx: &Ctx<'js>) -> JsResult<()> {
    // Π3.x: node:assert. The canonical test-framework primitive most
    // npm test infrastructure uses as a fallback. Composes on
    // nodeUtil.isDeepStrictEqual (Π3.10) for deepStrictEqual.
    //
    // Per M9 (spec-first against Bun): throws AssertionError with the
    // documented shape; default vs strict modes behave per Node.
    ctx.eval::<(), _>(r#"
        (function() {
            class AssertionError extends Error {
                constructor(opts) {
                    opts = opts || {};
                    const message = opts.message || (
                        (opts.actual !== undefined && opts.expected !== undefined)
                            ? "Expected " + Bun.inspect(opts.expected) + " but got " + Bun.inspect(opts.actual)
                            : "Assertion failed");
                    super(message);
                    this.name = "AssertionError";
                    this.code = "ERR_ASSERTION";
                    this.actual = opts.actual;
                    this.expected = opts.expected;
                    this.operator = opts.operator;
                    this.generatedMessage = opts.message === undefined;
                }
            }

            function ok(value, message) {
                if (!value) throw new AssertionError({
                    actual: value, expected: true, operator: "==",
                    message: message,
                });
            }
            function equal(actual, expected, message) {
                if (actual != expected) throw new AssertionError({
                    actual, expected, operator: "==", message,
                });
            }
            function notEqual(actual, expected, message) {
                if (actual == expected) throw new AssertionError({
                    actual, expected, operator: "!=", message,
                });
            }
            function strictEqual(actual, expected, message) {
                if (!Object.is(actual, expected)) throw new AssertionError({
                    actual, expected, operator: "===", message,
                });
            }
            function notStrictEqual(actual, expected, message) {
                if (Object.is(actual, expected)) throw new AssertionError({
                    actual, expected, operator: "!==", message,
                });
            }
            function deepStrictEqual(actual, expected, message) {
                if (!globalThis.nodeUtil.isDeepStrictEqual(actual, expected)) {
                    throw new AssertionError({
                        actual, expected, operator: "deepStrictEqual", message,
                    });
                }
            }
            function notDeepStrictEqual(actual, expected, message) {
                if (globalThis.nodeUtil.isDeepStrictEqual(actual, expected)) {
                    throw new AssertionError({
                        actual, expected, operator: "notDeepStrictEqual", message,
                    });
                }
            }
            // deepEqual in non-strict mode uses == on primitives + recursive
            // for objects. We delegate to isDeepStrictEqual which is
            // structurally what consumers expect even in non-strict mode.
            const deepEqual = deepStrictEqual;
            const notDeepEqual = notDeepStrictEqual;

            function throws(block, errSpec, message) {
                let thrown = null;
                try { block(); } catch (e) { thrown = e; }
                if (thrown === null) throw new AssertionError({
                    actual: undefined, expected: errSpec, operator: "throws",
                    message: message || "Expected function to throw",
                });
                if (errSpec instanceof RegExp) {
                    if (!errSpec.test(String(thrown.message))) throw new AssertionError({
                        actual: thrown.message, expected: errSpec, operator: "throws", message,
                    });
                } else if (typeof errSpec === "function") {
                    if (!(thrown instanceof errSpec)) throw new AssertionError({
                        actual: thrown, expected: errSpec, operator: "throws", message,
                    });
                } else if (errSpec && typeof errSpec === "object") {
                    for (const k of Object.keys(errSpec)) {
                        if (thrown[k] !== errSpec[k]) throw new AssertionError({
                            actual: thrown[k], expected: errSpec[k],
                            operator: "throws", message: message || ("Property '" + k + "' mismatch"),
                        });
                    }
                }
            }
            function doesNotThrow(block, errSpec, message) {
                try { block(); } catch (e) {
                    throw new AssertionError({
                        actual: e, expected: undefined, operator: "doesNotThrow",
                        message: message || "Expected function not to throw: " + e.message,
                    });
                }
            }
            async function rejects(promiseOrFn, errSpec, message) {
                const p = typeof promiseOrFn === "function" ? promiseOrFn() : promiseOrFn;
                let thrown = null;
                try { await p; } catch (e) { thrown = e; }
                if (thrown === null) throw new AssertionError({
                    actual: undefined, expected: errSpec, operator: "rejects",
                    message: message || "Expected promise to reject",
                });
                // Match thrown against errSpec same as throws().
                if (errSpec instanceof RegExp) {
                    if (!errSpec.test(String(thrown.message))) throw new AssertionError({
                        actual: thrown.message, expected: errSpec, operator: "rejects", message,
                    });
                } else if (typeof errSpec === "function") {
                    if (!(thrown instanceof errSpec)) throw new AssertionError({
                        actual: thrown, expected: errSpec, operator: "rejects", message,
                    });
                }
            }
            async function doesNotReject(promiseOrFn, message) {
                const p = typeof promiseOrFn === "function" ? promiseOrFn() : promiseOrFn;
                try { await p; } catch (e) {
                    throw new AssertionError({
                        actual: e, expected: undefined, operator: "doesNotReject",
                        message: message || "Expected promise not to reject: " + e.message,
                    });
                }
            }
            function match(string, regexp, message) {
                if (!(regexp instanceof RegExp)) throw new TypeError("match: regexp required");
                if (!regexp.test(String(string))) throw new AssertionError({
                    actual: string, expected: regexp, operator: "match", message,
                });
            }
            function doesNotMatch(string, regexp, message) {
                if (!(regexp instanceof RegExp)) throw new TypeError("doesNotMatch: regexp required");
                if (regexp.test(String(string))) throw new AssertionError({
                    actual: string, expected: regexp, operator: "doesNotMatch", message,
                });
            }
            function fail(message) {
                throw new AssertionError({
                    message: message || "Failed",
                    operator: "fail",
                });
            }
            function ifError(value) {
                if (value !== null && value !== undefined) {
                    throw new AssertionError({
                        actual: value, expected: null, operator: "ifError",
                        message: "ifError got unwanted: " + (value && value.message ? value.message : String(value)),
                    });
                }
            }

            // Default assert is callable AND has the methods attached.
            const assertFn = function assert(value, message) { ok(value, message); };
            Object.assign(assertFn, {
                ok, equal, notEqual, strictEqual, notStrictEqual,
                deepEqual, notDeepEqual, deepStrictEqual, notDeepStrictEqual,
                throws, doesNotThrow, rejects, doesNotReject,
                match, doesNotMatch, fail, ifError,
                AssertionError,
            });
            // Strict mode: equal == strictEqual, deepEqual == deepStrictEqual.
            const strictFn = function strict(value, message) { ok(value, message); };
            Object.assign(strictFn, assertFn, {
                equal: strictEqual,
                notEqual: notStrictEqual,
                deepEqual: deepStrictEqual,
                notDeepEqual: notDeepStrictEqual,
            });
            assertFn.strict = strictFn;

            globalThis.nodeAssert = assertFn;
            globalThis.nodeAssertStrict = strictFn;
        })();
    "#)?;
    Ok(())
}

fn install_keep_alive_js<'js>(ctx: &Ctx<'js>) -> JsResult<()> {
    // Π2.6: async-runtime auto-keep-alive. eval_esm_module's main loop
    // continues ticking entries in globalThis.__keepAlive after the
    // microtask queue drains. Each entry is an object with a
    // synchronous __tick(maxWaitMs) -> boolean method that returns
    // true if it did productive work, false if it polled-and-was-idle.
    // The loop exits when both microtask queue and __keepAlive are
    // quiescent (no work for a documented number of consecutive ticks).
    //
    // Per seed §A8.16: __keepAlive is a process-global resource and
    // must be guarded if the host ever supports parallel evals; for now
    // the host runs one eval at a time so a plain Set suffices.
    //
    // Composes with Bun.serve(opts) which, when called with
    // opts.autoServe truthy, calls listen() then registers the server
    // in __keepAlive. server.stop() removes it. The canonical real-Bun
    // shape `Bun.serve({fetch, port})` is unlocked when consumers opt
    // into autoServe.
    ctx.eval::<(), _>(r#"
        (function() {
            globalThis.__keepAlive = new Set();
            globalThis.__tickKeepAlive = function() {
                let didWork = false;
                // Copy to a list to allow add/remove during tick.
                const items = Array.from(globalThis.__keepAlive);
                for (const item of items) {
                    if (item && typeof item.__tick === "function") {
                        try {
                            if (item.__tick(0)) didWork = true;
                        } catch (e) {
                            // Tick errors are surfaced to stderr but don't
                            // halt the loop; consumer-side error handling
                            // is per-server.
                            globalThis.__stderrBuf += "keepAlive tick error: " +
                                (e && e.message ? e.message : String(e)) + "\n";
                        }
                    }
                }
                return didWork;
            };
        })();
    "#)?;
    Ok(())
}

fn install_bun_small_utilities_js<'js>(ctx: &Ctx<'js>) -> JsResult<()> {
    // Π4: Bun namespace small utilities — load-bearing subset visible
    // to real OSS consumer code:
    //   - Bun.write(dest, src) sync file write
    //   - Bun.fileURLToPath / Bun.pathToFileURL — alias to nodeUrl
    //   - Bun.deepEquals — alias to util.isDeepStrictEqual
    //   - Bun.inspect — alias to util.inspect
    //   - Bun.CryptoHasher — incremental hash builder via crypto.subtle
    //   - Bun.Glob — minimal glob with match() boolean
    //   - Bun.gzipSync / Bun.gunzipSync / Bun.deflateSync / Bun.inflateSync —
    //     composes on Π1.3 compression decode (encode deferred).
    //
    // Bun.SQLite, Bun.connect (real async), Bun.YAML —
    // deferred per seed §VII.A and the architectural-cost discipline.
    // Bun.password ships this round (Π4.14.c) on the rusty-web-crypto
    // argon2id substrate.
    ctx.eval::<(), _>(r#"
        (function() {
            globalThis.Bun = globalThis.Bun || {};

            // Bun.write — sync write of string|Uint8Array to a path.
            // Real Bun returns a Promise<number>; we return synchronously
            // for the host's no-real-event-loop regime and Bun-compat
            // consumers can still await the result.
            Bun.write = function write(destPath, content) {
                let bytes;
                if (typeof content === "string") {
                    bytes = new TextEncoder().encode(content);
                } else if (content instanceof Uint8Array) {
                    bytes = content;
                } else if (content && typeof content.text === "function") {
                    // Blob / File / Response — caller awaits text(),
                    // we synchronously read via bytes() if present.
                    if (typeof content.bytes === "function") {
                        bytes = new Uint8Array(content.bytes());
                    } else {
                        throw new TypeError("Bun.write: source needs bytes() for sync write");
                    }
                } else {
                    throw new TypeError("Bun.write: unsupported source type");
                }
                // Convert path target (BunFile, URL, string) to filesystem path.
                let path = destPath;
                if (destPath && typeof destPath === "object" && destPath._path) path = destPath._path;
                else if (destPath instanceof URL) path = globalThis.nodeUrl.fileURLToPath(destPath);
                // Write via fs.writeFileSync.
                const arr = Array.from(bytes);
                globalThis.fs.writeFileSync(String(path), arr);
                return bytes.length;
            };

            // Bun.fileURLToPath / Bun.pathToFileURL aliases.
            Bun.fileURLToPath = function fileURLToPath(url) {
                return globalThis.nodeUrl.fileURLToPath(url);
            };
            Bun.pathToFileURL = function pathToFileURL(path) {
                return globalThis.nodeUrl.pathToFileURL(path);
            };

            // Bun.deepEquals / Bun.inspect aliases.
            Bun.deepEquals = function deepEquals(a, b) {
                return globalThis.nodeUtil.isDeepStrictEqual(a, b);
            };
            Bun.inspect = function inspect(value, options) {
                return globalThis.nodeUtil.inspect(value, options);
            };

            // Bun.CryptoHasher — incremental hash builder. Real Bun
            // supports many algorithms; we expose the most common ones
            // that compose on crypto.subtle.digest. update() accumulates
            // into an internal buffer; digest() runs the hash and resets.
            class CryptoHasher {
                constructor(algorithm) {
                    const a = String(algorithm || "sha256").toLowerCase();
                    const supported = ["sha1", "sha256", "sha384", "sha512", "sha512-256"];
                    if (!supported.includes(a)) {
                        throw new TypeError("Bun.CryptoHasher: unsupported algorithm '" + a + "'");
                    }
                    this._algo = a;
                    this._chunks = [];
                }
                update(chunk) {
                    let bytes;
                    if (typeof chunk === "string") bytes = new TextEncoder().encode(chunk);
                    else if (chunk instanceof Uint8Array) bytes = chunk;
                    else if (chunk instanceof ArrayBuffer) bytes = new Uint8Array(chunk);
                    else throw new TypeError("CryptoHasher.update: chunk must be string, Uint8Array, or ArrayBuffer");
                    this._chunks.push(bytes);
                    return this;
                }
                async _digestBytes() {
                    let total = 0;
                    for (const c of this._chunks) total += c.length;
                    const combined = new Uint8Array(total);
                    let off = 0;
                    for (const c of this._chunks) { combined.set(c, off); off += c.length; }
                    this._chunks = [];
                    const subtleAlgo = this._algo === "sha512-256" ? "SHA-512"
                        : this._algo.replace("sha", "SHA-").toUpperCase();
                    const buf = await crypto.subtle.digest(subtleAlgo, combined);
                    return new Uint8Array(buf);
                }
                digest(encoding) {
                    // Real Bun is sync; we approximate via a then-able
                    // shape that consumers can await. Most consumers
                    // call await hash.digest() anyway.
                    return this._digestBytes().then(bytes => {
                        if (encoding === "hex" || encoding === undefined) {
                            // Default is hex per Bun convention.
                            return encoding === undefined ? bytes : Array.from(bytes).map(b => b.toString(16).padStart(2, "0")).join("");
                        }
                        if (encoding === "base64") {
                            let s = "";
                            for (const b of bytes) s += String.fromCharCode(b);
                            return btoa(s);
                        }
                        if (encoding === "buffer" || encoding === "bytes") return bytes;
                        return bytes;
                    });
                }
            }
            Bun.CryptoHasher = CryptoHasher;

            // Bun.Glob — minimal glob with match(path) boolean. Real Bun
            // also exposes scanSync/scan iterators; we expose match()
            // only this round (the most common usage).
            class Glob {
                constructor(pattern) {
                    this.pattern = String(pattern);
                    this._regex = Glob._toRegex(this.pattern);
                }
                static _toRegex(p) {
                    // Minimal glob → regex translation: ** → .*, * → [^/]*,
                    // ? → [^/], plus regex escape for special chars.
                    let re = "^";
                    let i = 0;
                    while (i < p.length) {
                        const c = p[i];
                        if (c === "*") {
                            if (p[i + 1] === "*") {
                                re += ".*";
                                i += 2;
                                if (p[i] === "/") i += 1;  // collapse **/
                            } else {
                                re += "[^/]*";
                                i += 1;
                            }
                        } else if (c === "?") {
                            re += "[^/]";
                            i += 1;
                        } else if ("\\^$.|+()[]{}".indexOf(c) >= 0) {
                            re += "\\" + c;
                            i += 1;
                        } else {
                            re += c;
                            i += 1;
                        }
                    }
                    re += "$";
                    return new RegExp(re);
                }
                match(path) { return this._regex.test(String(path)); }
            }
            Bun.Glob = Glob;

            // Bun.gunzipSync / Bun.inflateSync — compose on Π1.3 decode.
            // Bun.gzipSync / Bun.deflateSync compose on Π1.3.b stored-block
            // encoders — wire-format compatible (any inflater accepts), with
            // compression ratio 1.0 until the LZ77+Huffman encoder lands.
            Bun.gunzipSync = function gunzipSync(input) {
                const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
                return new Uint8Array(globalThis.__compression.gunzip(Array.from(bytes)));
            };
            Bun.inflateSync = function inflateSync(input) {
                const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
                return new Uint8Array(globalThis.__compression.http_deflate_inflate(Array.from(bytes)));
            };
            Bun.gzipSync = function gzipSync(input) {
                const bytes = (typeof input === "string")
                    ? new TextEncoder().encode(input)
                    : (input instanceof Uint8Array ? input : new Uint8Array(input));
                return new Uint8Array(globalThis.__compression.gzip_deflate_stored(Array.from(bytes)));
            };
            Bun.deflateSync = function deflateSync(input) {
                const bytes = (typeof input === "string")
                    ? new TextEncoder().encode(input)
                    : (input instanceof Uint8Array ? input : new Uint8Array(input));
                // Bun.deflateSync produces zlib-wrapped output by default (matches Node zlib.deflateSync).
                return new Uint8Array(globalThis.__compression.zlib_deflate_stored(Array.from(bytes)));
            };

            // Bun.YAML — safe-subset YAML parser + serializer. Covers the
            // common config-file surface: scalars (null/bool/int/float/string),
            // flow-mode [a,b] and {k:v}, block-mode lists and maps with
            // indent-significant structure, single/double-quoted strings with
            // basic escapes, # line comments, and | / > block scalars. Anchor/
            // alias, tags, and merge-keys are out of scope (the "safe" subset).
            Bun.YAML = (function() {
                function parseScalar(s) {
                    if (s === "" || s === "~" || s === "null" || s === "Null" || s === "NULL") return null;
                    if (s === "true" || s === "True" || s === "TRUE") return true;
                    if (s === "false" || s === "False" || s === "FALSE") return false;
                    if (/^[-+]?\d+$/.test(s)) return parseInt(s, 10);
                    if (/^[-+]?\d*\.\d+([eE][-+]?\d+)?$/.test(s)) return parseFloat(s);
                    if (/^[-+]?\d+[eE][-+]?\d+$/.test(s)) return parseFloat(s);
                    if (s === ".inf" || s === ".Inf" || s === ".INF") return Infinity;
                    if (s === "-.inf" || s === "-.Inf" || s === "-.INF") return -Infinity;
                    if (s === ".nan" || s === ".NaN" || s === ".NAN") return NaN;
                    return s;
                }
                function unescapeDouble(s) {
                    return s.replace(/\\([nrt"\\\/bfu])/g, (_, c) => {
                        if (c === "n") return "\n";
                        if (c === "r") return "\r";
                        if (c === "t") return "\t";
                        if (c === "b") return "\b";
                        if (c === "f") return "\f";
                        if (c === '"') return '"';
                        if (c === "\\") return "\\";
                        if (c === "/") return "/";
                        return c;
                    });
                }
                function parseFlow(text, i) {
                    function skipWs() { while (i < text.length && /[\s]/.test(text[i])) i++; }
                    function parseValue() {
                        skipWs();
                        const c = text[i];
                        if (c === "[") { return parseArr(); }
                        if (c === "{") { return parseObj(); }
                        if (c === '"') { return parseDQ(); }
                        if (c === "'") { return parseSQ(); }
                        let j = i;
                        while (j < text.length && !/[,\]\}\n]/.test(text[j])) j++;
                        const tok = text.slice(i, j).trim();
                        i = j;
                        return parseScalar(tok);
                    }
                    function parseArr() {
                        i++; const arr = [];
                        skipWs();
                        if (text[i] === "]") { i++; return arr; }
                        while (i < text.length) {
                            arr.push(parseValue());
                            skipWs();
                            if (text[i] === ",") { i++; skipWs(); continue; }
                            if (text[i] === "]") { i++; return arr; }
                            throw new SyntaxError("YAML flow array: expected , or ]");
                        }
                        throw new SyntaxError("YAML flow array: unterminated");
                    }
                    function parseObj() {
                        i++; const obj = {};
                        skipWs();
                        if (text[i] === "}") { i++; return obj; }
                        while (i < text.length) {
                            skipWs();
                            let key;
                            if (text[i] === '"') key = parseDQ();
                            else if (text[i] === "'") key = parseSQ();
                            else {
                                let j = i;
                                while (j < text.length && text[j] !== ":" && text[j] !== "}") j++;
                                key = text.slice(i, j).trim();
                                i = j;
                            }
                            skipWs();
                            if (text[i] !== ":") throw new SyntaxError("YAML flow object: expected :");
                            i++; skipWs();
                            obj[key] = parseValue();
                            skipWs();
                            if (text[i] === ",") { i++; continue; }
                            if (text[i] === "}") { i++; return obj; }
                            throw new SyntaxError("YAML flow object: expected , or }");
                        }
                        throw new SyntaxError("YAML flow object: unterminated");
                    }
                    function parseDQ() {
                        i++; let j = i;
                        while (j < text.length && text[j] !== '"') {
                            if (text[j] === "\\") j += 2; else j++;
                        }
                        const s = unescapeDouble(text.slice(i, j));
                        i = j + 1;
                        return s;
                    }
                    function parseSQ() {
                        i++; let j = i, s = "";
                        while (j < text.length) {
                            if (text[j] === "'" && text[j+1] === "'") { s += text.slice(i, j) + "'"; j += 2; i = j; continue; }
                            if (text[j] === "'") break;
                            j++;
                        }
                        s += text.slice(i, j);
                        i = j + 1;
                        return s;
                    }
                    return { value: parseValue(), endIndex: i };
                }
                function parse(text) {
                    if (typeof text !== "string") text = String(text);
                    // Strip BOM, normalize CRLF.
                    if (text.charCodeAt(0) === 0xFEFF) text = text.slice(1);
                    text = text.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
                    // Strip document markers and comments.
                    const rawLines = text.split("\n");
                    const lines = [];
                    for (const rl of rawLines) {
                        let l = rl;
                        // Strip # comments unless inside quotes (best-effort).
                        let inS = false, inD = false, cut = -1;
                        for (let k = 0; k < l.length; k++) {
                            const ch = l[k];
                            if (ch === "'" && !inD) inS = !inS;
                            else if (ch === '"' && !inS) inD = !inD;
                            else if (ch === "\x23" && !inS && !inD && (k === 0 || /\s/.test(l[k-1]))) {
                                cut = k; break;
                            }
                        }
                        if (cut >= 0) l = l.slice(0, cut);
                        l = l.replace(/\s+$/, "");
                        if (l === "---" || l === "...") continue;
                        lines.push(l);
                    }
                    let idx = 0;
                    function lineIndent(l) {
                        let n = 0;
                        while (n < l.length && l[n] === " ") n++;
                        return n;
                    }
                    function parseBlock(minIndent) {
                        // Skip blanks
                        while (idx < lines.length && lines[idx].trim() === "") idx++;
                        if (idx >= lines.length) return null;
                        const indent = lineIndent(lines[idx]);
                        if (indent < minIndent) return null;
                        const first = lines[idx].slice(indent);
                        // List?
                        if (first.startsWith("- ") || first === "-") {
                            const arr = [];
                            while (idx < lines.length) {
                                if (lines[idx].trim() === "") { idx++; continue; }
                                const li = lineIndent(lines[idx]);
                                if (li < indent) break;
                                if (li > indent) break;
                                const t = lines[idx].slice(li);
                                if (!t.startsWith("-")) break;
                                const after = t === "-" ? "" : t.slice(2);
                                if (after === "") {
                                    idx++;
                                    arr.push(parseBlock(indent + 1));
                                } else if (/^[a-zA-Z_$\"\'][^:]*:(\s|$)/.test(after) || /^[\w-]+:(\s|$)/.test(after)) {
                                    // Inline map starting on the - line
                                    // Treat the rest of this line as the first map line of a nested map at indent+2
                                    const lead = " ".repeat(indent + 2) + after;
                                    lines[idx] = lead;
                                    arr.push(parseBlock(indent + 2));
                                } else {
                                    idx++;
                                    arr.push(parseInlineValue(after, indent + 2));
                                }
                            }
                            return arr;
                        }
                        // Map?
                        if (/^[^\s].*:(\s|$)/.test(first) || /^"[^"]*":(\s|$)/.test(first) || /^'[^']*':(\s|$)/.test(first)) {
                            const obj = {};
                            while (idx < lines.length) {
                                if (lines[idx].trim() === "") { idx++; continue; }
                                const li = lineIndent(lines[idx]);
                                if (li < indent) break;
                                if (li > indent) break;
                                const t = lines[idx].slice(li);
                                const colonPos = (() => {
                                    let inS = false, inD = false;
                                    for (let k = 0; k < t.length; k++) {
                                        const ch = t[k];
                                        if (ch === "'" && !inD) inS = !inS;
                                        else if (ch === '"' && !inS) inD = !inD;
                                        else if (ch === ":" && !inS && !inD && (k + 1 === t.length || /\s/.test(t[k+1]))) {
                                            return k;
                                        }
                                    }
                                    return -1;
                                })();
                                if (colonPos < 0) break;
                                let key = t.slice(0, colonPos).trim();
                                if (key[0] === '"' && key[key.length-1] === '"') key = unescapeDouble(key.slice(1, -1));
                                else if (key[0] === "'" && key[key.length-1] === "'") key = key.slice(1, -1).replace(/''/g, "'");
                                const after = t.slice(colonPos + 1).trim();
                                idx++;
                                if (after === "" || after === "|" || after === ">" || after === "|-" || after === ">-") {
                                    if (after === "|" || after === "|-" || after === ">" || after === ">-") {
                                        // Block scalar
                                        const folded = after.startsWith(">");
                                        const strip = after.endsWith("-");
                                        const chunks = [];
                                        const scalarIndent = (idx < lines.length) ? lineIndent(lines[idx]) : indent + 2;
                                        if (scalarIndent <= indent) { obj[key] = ""; continue; }
                                        while (idx < lines.length) {
                                            if (lines[idx] === "") { chunks.push(""); idx++; continue; }
                                            const li2 = lineIndent(lines[idx]);
                                            if (li2 < scalarIndent) break;
                                            chunks.push(lines[idx].slice(scalarIndent));
                                            idx++;
                                        }
                                        // Trim trailing empty lines (default chomp keeps one \n).
                                        while (chunks.length > 0 && chunks[chunks.length - 1] === "") chunks.pop();
                                        let s = folded ? chunks.join(" ") : chunks.join("\n");
                                        if (!strip) s += "\n";
                                        obj[key] = s;
                                    } else {
                                        const v = parseBlock(indent + 1);
                                        obj[key] = v === null ? null : v;
                                    }
                                } else {
                                    obj[key] = parseInlineValue(after, indent + 2);
                                }
                            }
                            return obj;
                        }
                        // Bare scalar / flow scalar at root
                        idx++;
                        return parseInlineValue(first, indent);
                    }
                    function parseInlineValue(s, contIndent) {
                        s = s.trim();
                        if (s === "") return null;
                        if (s[0] === "[" || s[0] === "{") {
                            const r = parseFlow(s, 0);
                            return r.value;
                        }
                        if (s[0] === '"') {
                            // Find end quote (no embedded newlines for safe subset)
                            const end = (() => {
                                for (let k = 1; k < s.length; k++) {
                                    if (s[k] === "\\") { k++; continue; }
                                    if (s[k] === '"') return k;
                                }
                                return -1;
                            })();
                            if (end > 0) return unescapeDouble(s.slice(1, end));
                            return s;
                        }
                        if (s[0] === "'") {
                            const end = s.lastIndexOf("'");
                            if (end > 0) return s.slice(1, end).replace(/''/g, "'");
                            return s;
                        }
                        return parseScalar(s);
                    }
                    return parseBlock(0);
                }
                function stringify(value, indent) {
                    const ind = (typeof indent === "number") ? indent : 2;
                    function quoteIfNeeded(s) {
                        if (s === "" || s === "null" || s === "true" || s === "false") return '"' + s + '"';
                        if (/^[-+]?\d/.test(s) || /^[\[{>|*&!#%@\`]/.test(s)) return '"' + s + '"';
                        if (/[:#\n]/.test(s) || /^\s/.test(s) || /\s$/.test(s)) {
                            return '"' + s.replace(/\\/g, "\\\\").replace(/"/g, '\\"')
                                .replace(/\n/g, "\\n").replace(/\r/g, "\\r").replace(/\t/g, "\\t") + '"';
                        }
                        return s;
                    }
                    function emit(v, level) {
                        const pad = " ".repeat(level * ind);
                        if (v === null || v === undefined) return "null";
                        if (typeof v === "boolean") return v ? "true" : "false";
                        if (typeof v === "number") {
                            if (!isFinite(v)) return v !== v ? ".nan" : (v > 0 ? ".inf" : "-.inf");
                            return String(v);
                        }
                        if (typeof v === "string") return quoteIfNeeded(v);
                        if (Array.isArray(v)) {
                            if (v.length === 0) return "[]";
                            return v.map(item => "\n" + pad + "- " + emit(item, level + 1).replace(/^\n/, "")).join("");
                        }
                        if (typeof v === "object") {
                            const keys = Object.keys(v);
                            if (keys.length === 0) return "{}";
                            return keys.map(k => {
                                const out = emit(v[k], level + 1);
                                if (out.startsWith("\n")) return "\n" + pad + k + ":" + out;
                                return "\n" + pad + k + ": " + out;
                            }).join("");
                        }
                        return String(v);
                    }
                    const result = emit(value, 0);
                    return result.startsWith("\n") ? result.slice(1) + "\n" : result + "\n";
                }
                return { parse, stringify };
            })();

            // Bun.connect — async TCP client. Returns Promise<Socket>.
            // Composes on TCP.connect + setNonblocking + tryRead +
            // __keepAlive + the cooperative-loop pump (Pi2.6.b).
            // Surface mirrors Bun's: { hostname, port, socket: { open,
            // data, close, error, drain } } in, Socket with write/end/
            // terminate/flush out.
            Bun.connect = function connect(opts) {
                const host = opts.hostname || opts.host || "127.0.0.1";
                const port = opts.port || 0;
                const handlers = opts.socket || {};
                return new Promise((resolve, reject) => {
                    let sid;
                    try {
                        sid = globalThis.TCP.connect(host + ":" + port);
                        globalThis.TCP.setNonblocking(sid, true);
                    } catch (e) {
                        if (typeof handlers.error === "function") {
                            try { handlers.error(null, e); } catch (_) {}
                        }
                        reject(e);
                        return;
                    }
                    const socket = {
                        _sid: sid,
                        _closed: false,
                        data: opts.data,
                        readyState: "open",
                        remoteAddress: host,
                        write(chunk) {
                            if (this._closed) return 0;
                            const bytes = (typeof chunk === "string")
                                ? new TextEncoder().encode(chunk)
                                : (chunk instanceof Uint8Array ? chunk : new Uint8Array(chunk));
                            globalThis.TCP.writeAll(this._sid, bytes);
                            return bytes.length;
                        },
                        end(chunk) {
                            if (chunk !== undefined) this.write(chunk);
                            this._endRequested = true;
                            return this;
                        },
                        terminate() {
                            if (this._closed) return;
                            this._closed = true;
                            this.readyState = "closed";
                            try { globalThis.TCP.close(this._sid); } catch (_) {}
                            if (globalThis.__keepAlive) globalThis.__keepAlive.delete(this);
                            if (typeof handlers.close === "function") {
                                try { handlers.close(this); } catch (_) {}
                            }
                        },
                        flush() { /* writeAll already flushes; no-op */ },
                        ref() {},
                        unref() {},
                        __tick() {
                            if (this._closed) return false;
                            const chunk = globalThis.TCP.tryRead(this._sid, 65536);
                            if (chunk === null) return false;
                            if (chunk.length === 0) {
                                this.terminate();
                                return true;
                            }
                            if (typeof handlers.data === "function") {
                                try {
                                    handlers.data(this, typeof Buffer !== "undefined"
                                        ? Buffer.from(chunk) : new Uint8Array(chunk));
                                } catch (e) {
                                    if (typeof handlers.error === "function") {
                                        try { handlers.error(this, e); } catch (_) {}
                                    }
                                }
                            }
                            if (this._endRequested) this.terminate();
                            return true;
                        },
                    };
                    if (globalThis.__keepAlive) globalThis.__keepAlive.add(socket);
                    queueMicrotask(() => {
                        if (typeof handlers.open === "function") {
                            try { handlers.open(socket); } catch (e) {
                                if (typeof handlers.error === "function") {
                                    try { handlers.error(socket, e); } catch (_) {}
                                }
                            }
                        }
                        resolve(socket);
                    });
                });
            };

            // Bun.escapeHTML — common micro-helper.
            Bun.escapeHTML = function escapeHTML(input) {
                return String(input)
                    .replace(/&/g, "&amp;")
                    .replace(/</g, "&lt;")
                    .replace(/>/g, "&gt;")
                    .replace(/"/g, "&quot;")
                    .replace(/'/g, "&#39;");
            };

            // Bun.nanoseconds — high-res monotonic time (ns).
            Bun.nanoseconds = function nanoseconds() {
                // Real Bun returns a Number (not BigInt); match its shape.
                return Math.floor(performance.now() * 1e6);
            };

            // Bun.sleep — Promise that resolves after ms milliseconds.
            // Composes on setTimeout (already installed).
            Bun.sleep = function sleep(ms) {
                return new Promise(resolve => setTimeout(resolve, Number(ms)));
            };

            // Bun.sleepSync — synchronous sleep stub (Bun has it real).
            // We return immediately in the host's no-real-clock regime;
            // consumers using this for timing-sensitive work see a
            // documented divergence.
            Bun.sleepSync = function sleepSync(_ms) {
                // No-op in test runtime: real wall-clock pauses aren't
                // representable without real async I/O.
            };

            // Bun.password — argon2id-backed password hashing. Real Bun
            // returns a PHC-format string from hash(); verify() parses
            // the PHC string and recomputes. Defaults from Bun docs:
            // algorithm=argon2id, time_cost=2, memory_cost=65536, hash_len=32.
            // Single-lane (p=1) only; multi-lane support tracked separately.
            const __b64nopad_encode = (bytes) => {
                let s = "";
                for (const b of bytes) s += String.fromCharCode(b);
                return btoa(s).replace(/=+$/, "");
            };
            const __b64nopad_decode = (str) => {
                const pad = (4 - (str.length % 4)) % 4;
                const padded = str + "=".repeat(pad);
                const bin = atob(padded);
                const out = new Uint8Array(bin.length);
                for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
                return out;
            };
            const __argon2id_run = (password, salt, t, m, tau) => {
                const pwBytes = typeof password === "string"
                    ? new TextEncoder().encode(password)
                    : (password instanceof Uint8Array ? password : new Uint8Array(password));
                const out = globalThis.crypto.subtle.argon2idBytes(
                    Array.from(pwBytes), Array.from(salt), t, m, tau,
                );
                return new Uint8Array(out);
            };
            const __ct_equal = (a, b) => {
                if (a.length !== b.length) return false;
                let diff = 0;
                for (let i = 0; i < a.length; i++) diff |= a[i] ^ b[i];
                return diff === 0;
            };

            Bun.password = {
                async hash(password, options) {
                    const opts = (typeof options === "string") ? { algorithm: options } : (options || {});
                    const algo = opts.algorithm || "argon2id";
                    if (algo !== "argon2id" && algo !== "argon2d" && algo !== "argon2i") {
                        throw new TypeError("Bun.password: only argon2 family supported");
                    }
                    if (algo !== "argon2id") {
                        throw new TypeError("Bun.password: only argon2id supported in this build");
                    }
                    const timeCost = opts.timeCost ?? 2;
                    const memoryCost = opts.memoryCost ?? 65536;
                    const tau = 32;
                    const salt = new Uint8Array(16);
                    crypto.getRandomValues(salt);
                    const tag = __argon2id_run(password, salt, timeCost, memoryCost, tau);
                    const saltB64 = __b64nopad_encode(salt);
                    const hashB64 = __b64nopad_encode(tag);
                    return `$argon2id$v=19$m=${memoryCost},t=${timeCost},p=1$${saltB64}$${hashB64}`;
                },
                async verify(password, encoded, algorithm) {
                    if (typeof encoded !== "string") return false;
                    const parts = encoded.split("$");
                    // Form: ["", "argon2id", "v=19", "m=...,t=...,p=...", saltB64, hashB64]
                    if (parts.length !== 6) return false;
                    if (parts[1] !== "argon2id") return false;
                    if (algorithm && algorithm !== parts[1]) return false;
                    if (parts[2] !== "v=19") return false;
                    const paramSegs = parts[3].split(",");
                    let m = 0, t = 0, p = 0;
                    for (const seg of paramSegs) {
                        const [k, v] = seg.split("=");
                        if (k === "m") m = parseInt(v, 10);
                        else if (k === "t") t = parseInt(v, 10);
                        else if (k === "p") p = parseInt(v, 10);
                    }
                    if (p !== 1) return false;
                    const salt = __b64nopad_decode(parts[4]);
                    const expected = __b64nopad_decode(parts[5]);
                    const tau = expected.length;
                    const got = __argon2id_run(password, salt, t, m, tau);
                    return __ct_equal(got, expected);
                },
                hashSync(password, options) {
                    const opts = (typeof options === "string") ? { algorithm: options } : (options || {});
                    if (opts.algorithm && opts.algorithm !== "argon2id") {
                        throw new TypeError("Bun.password.hashSync: only argon2id supported");
                    }
                    const timeCost = opts.timeCost ?? 2;
                    const memoryCost = opts.memoryCost ?? 65536;
                    const salt = new Uint8Array(16);
                    crypto.getRandomValues(salt);
                    const tag = __argon2id_run(password, salt, timeCost, memoryCost, 32);
                    return `$argon2id$v=19$m=${memoryCost},t=${timeCost},p=1$${__b64nopad_encode(salt)}$${__b64nopad_encode(tag)}`;
                },
                verifySync(password, encoded, algorithm) {
                    if (typeof encoded !== "string") return false;
                    const parts = encoded.split("$");
                    if (parts.length !== 6) return false;
                    if (parts[1] !== "argon2id") return false;
                    if (algorithm && algorithm !== parts[1]) return false;
                    if (parts[2] !== "v=19") return false;
                    let m = 0, t = 0, p = 0;
                    for (const seg of parts[3].split(",")) {
                        const [k, v] = seg.split("=");
                        if (k === "m") m = parseInt(v, 10);
                        else if (k === "t") t = parseInt(v, 10);
                        else if (k === "p") p = parseInt(v, 10);
                    }
                    if (p !== 1) return false;
                    const salt = __b64nopad_decode(parts[4]);
                    const expected = __b64nopad_decode(parts[5]);
                    const got = __argon2id_run(password, salt, t, m, expected.length);
                    return __ct_equal(got, expected);
                },
            };
        })();
    "#)?;
    Ok(())
}

fn install_commonjs_loader_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(COMMONJS_LOADER_JS)?;
    Ok(())
}

// ════════════════════════════════════════════════════════════════════════
// Tier-H.4: timers, queueMicrotask, performance
// ════════════════════════════════════════════════════════════════════════
//
// Real consumer code uses setTimeout/setImmediate/queueMicrotask
// pervasively. E.54 (delay ^7) closure: timers honor wall-clock ms.
//
// Deadline-based scheduling: setTimeout stores { fn, args, deadline }.
// __tickTimers (called from host eval loop) fires due timers in
// deadline order. __nextTimerDelay reports how long until the earliest
// pending timer, so the host can sleep instead of spinning.
//
// performance.now() / .timeOrigin: backed by std::time::Instant via a
// Rust closure. timeOrigin is captured at runtime construction.

const TIMERS_AND_PERF_JS: &str = r#"
(function() {
    const timers = new Map();  // id → { cleared, fn, args, deadline, interval }
    let nextId = 1;

    function setTimeoutImpl(fn, ms, ...args) {
        if (typeof fn !== "function") {
            // Per WHATWG: string fn is allowed but pilot rejects.
            throw new TypeError("setTimeout requires a function");
        }
        const id = nextId++;
        const delay = Math.max(0, Number(ms) || 0);
        timers.set(id, {
            cleared: false, fn, args,
            deadline: Date.now() + delay,
            interval: -1,
        });
        return id;
    }

    function clearTimeoutImpl(id) {
        const entry = timers.get(id);
        if (entry) {
            entry.cleared = true;
            timers.delete(id);
        }
    }

    function setIntervalImpl(fn, ms, ...args) {
        if (typeof fn !== "function") {
            throw new TypeError("setInterval requires a function");
        }
        const id = nextId++;
        const period = Math.max(1, Number(ms) || 1);
        timers.set(id, {
            cleared: false, fn, args,
            deadline: Date.now() + period,
            interval: period,
        });
        return id;
    }

    // Called from host eval loop after microtask drain. Fires all timers
    // whose deadline has passed, in deadline order. Returns true if any
    // fired (so caller can re-drain microtasks before sleeping).
    globalThis.__tickTimers = function() {
        const now = Date.now();
        const due = [];
        for (const [id, e] of timers) {
            if (!e.cleared && e.deadline <= now) due.push([id, e]);
        }
        if (due.length === 0) return false;
        due.sort((a, b) => a[1].deadline - b[1].deadline);
        for (const [id, e] of due) {
            if (e.cleared) continue;
            if (e.interval > 0) {
                // Reschedule before invoking so a throwing handler
                // doesn't desynchronize the interval cadence.
                e.deadline = now + e.interval;
            } else {
                timers.delete(id);
            }
            try { e.fn.apply(undefined, e.args); }
            catch (err) {
                if (typeof console !== "undefined" && console.error) {
                    console.error("uncaught in timer:", err);
                }
            }
        }
        return true;
    };

    // Returns ms until the earliest pending timer, or -1 if none.
    globalThis.__nextTimerDelay = function() {
        let earliest = -1;
        for (const [_, e] of timers) {
            if (e.cleared) continue;
            if (earliest === -1 || e.deadline < earliest) earliest = e.deadline;
        }
        if (earliest === -1) return -1;
        return Math.max(0, earliest - Date.now());
    };

    globalThis.__hasPendingTimers = function() {
        for (const [_, e] of timers) if (!e.cleared) return true;
        return false;
    };

    globalThis.setTimeout = setTimeoutImpl;
    globalThis.clearTimeout = clearTimeoutImpl;
    globalThis.setImmediate = function setImmediate(fn, ...args) {
        return setTimeoutImpl(fn, 0, ...args);
    };
    globalThis.clearImmediate = clearTimeoutImpl;
    globalThis.setInterval = setIntervalImpl;
    globalThis.clearInterval = clearTimeoutImpl;

    globalThis.queueMicrotask = function queueMicrotask(fn) {
        if (typeof fn !== "function") {
            throw new TypeError("queueMicrotask requires a function");
        }
        Promise.resolve().then(() => {
            try { fn(); }
            catch (e) {
                if (typeof console !== "undefined" && console.error) {
                    console.error("uncaught in queueMicrotask:", e);
                }
            }
        });
    };
})();
"#;

fn install_timers_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(TIMERS_AND_PERF_JS)?;
    Ok(())
}

// ════════════════════════════════════════════════════════════════════════
// Tier-H.4 #2: URL class (WHATWG)
// ════════════════════════════════════════════════════════════════════════
//
// JS-side instantiation per Pattern 4 (seed §III.A8.2bis). The WHATWG
// URL Standard's full state-machine parser is ~hundreds of LOC and
// covers IDN/percent-encoding/host-validation edge cases beyond pilot
// scope. This implementation handles the common-consumer subset:
//
//   - http(s):// ws(s):// ftp:// file:// schemes
//   - host[:port]/pathname?search#hash decomposition
//   - default-port omission per scheme
//   - relative URL resolution against a base URL
//   - searchParams live-bound to .search via the wired URLSearchParams
//   - toString() / toJSON() / href setter (re-parses)
//
// What's deliberately scoped out: full percent-encoding tables (we use
// encodeURI for path, encodeURIComponent for individual components),
// IDN (assumes ASCII hostnames), authority-only URLs without paths,
// non-special-scheme URLs. Real consumer code touching these edges
// is rare; closing the gap is a follow-up if a Tier-J consumer hits it.

const URL_CLASS_JS: &str = r##"
(function() {
    const SPECIAL_SCHEMES = {
        "http:": 80,
        "https:": 443,
        "ws:": 80,
        "wss:": 443,
        "ftp:": 21,
        "file:": null,
    };

    function isSpecial(scheme) {
        return Object.prototype.hasOwnProperty.call(SPECIAL_SCHEMES, scheme);
    }

    function defaultPortFor(scheme) {
        return SPECIAL_SCHEMES[scheme];
    }

    function parseScheme(input) {
        // Match a leading "scheme:" if it conforms to ALPHA *( ALPHA / DIGIT / + / - / . ).
        const m = /^([a-zA-Z][a-zA-Z0-9+\-.]*):/.exec(input);
        if (!m) return null;
        return { scheme: m[1].toLowerCase() + ":", rest: input.substring(m[0].length) };
    }

    function parseAuthority(rest) {
        // After "scheme://", split off the authority up to the first /, ?, #, or end.
        if (!rest.startsWith("//")) return { hasAuthority: false, rest };
        rest = rest.substring(2);
        let end = rest.length;
        for (let i = 0; i < rest.length; i++) {
            const c = rest.charCodeAt(i);
            if (c === 47 /* / */ || c === 63 /* ? */ || c === 35 /* # */) { end = i; break; }
        }
        const authority = rest.substring(0, end);
        const remainder = rest.substring(end);

        // Split userinfo@host from authority.
        let userinfo = "", host = authority;
        const atIdx = authority.lastIndexOf("@");
        if (atIdx >= 0) {
            userinfo = authority.substring(0, atIdx);
            host = authority.substring(atIdx + 1);
        }
        let username = "", password = "";
        if (userinfo.length > 0) {
            const colonIdx = userinfo.indexOf(":");
            if (colonIdx >= 0) {
                username = userinfo.substring(0, colonIdx);
                password = userinfo.substring(colonIdx + 1);
            } else {
                username = userinfo;
            }
        }

        // Split host:port (handle IPv6 [::1]:port form).
        let hostname = host, port = "";
        if (host.startsWith("[")) {
            const closeIdx = host.indexOf("]");
            if (closeIdx >= 0) {
                hostname = host.substring(0, closeIdx + 1);
                if (closeIdx + 1 < host.length && host.charAt(closeIdx + 1) === ":") {
                    port = host.substring(closeIdx + 2);
                }
            }
        } else {
            const colonIdx = host.lastIndexOf(":");
            if (colonIdx >= 0) {
                hostname = host.substring(0, colonIdx);
                port = host.substring(colonIdx + 1);
            }
        }

        return {
            hasAuthority: true,
            username,
            password,
            hostname: hostname.toLowerCase(),
            port,
            rest: remainder,
        };
    }

    function parsePathQueryFragment(rest) {
        let pathname = "", search = "", hash = "";
        const hashIdx = rest.indexOf("#");
        if (hashIdx >= 0) {
            hash = rest.substring(hashIdx);
            rest = rest.substring(0, hashIdx);
        }
        const qIdx = rest.indexOf("?");
        if (qIdx >= 0) {
            search = rest.substring(qIdx);
            rest = rest.substring(0, qIdx);
        }
        pathname = rest;
        return { pathname, search, hash };
    }

    function resolveAgainstBase(input, base) {
        // Minimal relative-resolution per RFC 3986 §5.3.
        if (!base) throw new TypeError("Invalid URL: " + input);
        // If input has its own scheme, ignore base.
        if (parseScheme(input)) return input;
        // Otherwise build absolute by replacing the appropriate component of base.
        if (input.startsWith("//")) {
            return base.protocol + input;
        }
        const baseAuthority = base.username || base.password || base.hostname || base.port
            ? "//" + (base.username
                ? base.username + (base.password ? ":" + base.password : "") + "@"
                : "")
              + base.hostname + (base.port ? ":" + base.port : "")
            : "";
        if (input.startsWith("/")) {
            return base.protocol + baseAuthority + input;
        }
        if (input.startsWith("?")) {
            return base.protocol + baseAuthority + base.pathname + input;
        }
        if (input.startsWith("#")) {
            return base.protocol + baseAuthority + base.pathname + base.search + input;
        }
        if (input.length === 0) {
            return base.protocol + baseAuthority + base.pathname + base.search;
        }
        // Relative path: merge with base.pathname's directory.
        const basePath = base.pathname;
        const lastSlash = basePath.lastIndexOf("/");
        const dir = lastSlash >= 0 ? basePath.substring(0, lastSlash + 1) : "/";
        return base.protocol + baseAuthority + normalizePathSegments(dir + input);
    }

    function normalizePathSegments(p) {
        const isAbs = p.startsWith("/");
        const parts = p.split("/").filter((s) => s.length > 0);
        const out = [];
        for (const seg of parts) {
            if (seg === ".") continue;
            if (seg === "..") {
                if (out.length > 0) out.pop();
                continue;
            }
            out.push(seg);
        }
        return (isAbs ? "/" : "") + out.join("/");
    }

    class URL {
        constructor(input, base) {
            input = String(input);
            let parsed;
            // If input is relative, resolve against base.
            if (!parseScheme(input) && base !== undefined) {
                const baseUrl = base instanceof URL ? base : new URL(String(base));
                input = resolveAgainstBase(input, baseUrl);
            }
            const schemeParse = parseScheme(input);
            if (!schemeParse) throw new TypeError("Invalid URL: " + input);
            const protocol = schemeParse.scheme;
            const auth = parseAuthority(schemeParse.rest);
            const pqf = parsePathQueryFragment(auth.rest);

            this._protocol = protocol;
            this._username = auth.username || "";
            this._password = auth.password || "";
            this._hostname = auth.hostname || "";
            // Drop port if it equals the default for the scheme.
            const dflt = defaultPortFor(protocol);
            const portStr = auth.port && Number(auth.port) !== dflt ? auth.port : "";
            this._port = portStr;
            this._pathname = pqf.pathname || (auth.hasAuthority && isSpecial(protocol) ? "/" : "");
            this._search = pqf.search;
            this._hash = pqf.hash;
            // searchParams live-bound: writes propagate to ._search.
            this._searchParams = new URLSearchParams(this._search.replace(/^\?/, ""));
            const self = this;
            // Wrap mutating methods to keep .search in sync.
            const proxy = ["append", "delete", "set", "sort"];
            for (const m of proxy) {
                const orig = this._searchParams[m].bind(this._searchParams);
                this._searchParams[m] = function (...args) {
                    const r = orig(...args);
                    const s = self._searchParams.toString();
                    self._search = s.length > 0 ? "?" + s : "";
                    return r;
                };
            }
        }
        get protocol() { return this._protocol; }
        set protocol(v) { this._protocol = String(v).endsWith(":") ? String(v) : String(v) + ":"; }
        get username() { return this._username; }
        set username(v) { this._username = String(v); }
        get password() { return this._password; }
        set password(v) { this._password = String(v); }
        get host() {
            return this._hostname + (this._port ? ":" + this._port : "");
        }
        set host(v) {
            const s = String(v);
            const c = s.lastIndexOf(":");
            if (s.startsWith("[")) {
                this._hostname = s; this._port = "";
            } else if (c >= 0) {
                this._hostname = s.substring(0, c).toLowerCase();
                this._port = s.substring(c + 1);
            } else {
                this._hostname = s.toLowerCase(); this._port = "";
            }
        }
        get hostname() { return this._hostname; }
        set hostname(v) { this._hostname = String(v).toLowerCase(); }
        get port() { return this._port; }
        set port(v) {
            const s = String(v);
            this._port = s === "" ? "" : (Number(s) === defaultPortFor(this._protocol) ? "" : s);
        }
        get pathname() { return this._pathname; }
        set pathname(v) {
            let s = String(v);
            if (isSpecial(this._protocol) && this._hostname && !s.startsWith("/")) s = "/" + s;
            this._pathname = s;
        }
        get search() { return this._search; }
        set search(v) {
            let s = String(v);
            if (s.length > 0 && !s.startsWith("?")) s = "?" + s;
            this._search = s;
            // Reseed searchParams.
            const newParams = new URLSearchParams(s.replace(/^\?/, ""));
            this._searchParams._pairs = newParams._pairs;
        }
        get hash() { return this._hash; }
        set hash(v) {
            let s = String(v);
            if (s.length > 0 && !s.startsWith("#")) s = "#" + s;
            this._hash = s;
        }
        get searchParams() { return this._searchParams; }
        get origin() {
            if (!isSpecial(this._protocol)) return "null";
            if (this._protocol === "file:") return "null";
            return this._protocol + "//" + this._hostname + (this._port ? ":" + this._port : "");
        }
        get href() {
            let userinfo = "";
            if (this._username || this._password) {
                userinfo = this._username + (this._password ? ":" + this._password : "") + "@";
            }
            const authority = this._hostname || userinfo
                ? "//" + userinfo + this._hostname + (this._port ? ":" + this._port : "")
                : (isSpecial(this._protocol) ? "//" : "");
            return this._protocol + authority + this._pathname + this._search + this._hash;
        }
        set href(v) {
            // Re-parse and copy fields.
            const fresh = new URL(String(v));
            this._protocol = fresh._protocol;
            this._username = fresh._username;
            this._password = fresh._password;
            this._hostname = fresh._hostname;
            this._port = fresh._port;
            this._pathname = fresh._pathname;
            this._search = fresh._search;
            this._hash = fresh._hash;
            this._searchParams = fresh._searchParams;
        }
        toString() { return this.href; }
        toJSON() { return this.href; }
    }

    URL.canParse = function (input, base) {
        try { new URL(input, base); return true; }
        catch (e) { return false; }
    };

    URL.createObjectURL = function () {
        throw new Error("URL.createObjectURL is not supported in rusty-bun-host");
    };
    URL.revokeObjectURL = function () {
        // No-op (matches the no-objectURL substrate state).
    };

    globalThis.URL = URL;

    // WHATWG FormData — multipart/form-data form construction. Stored as
    // an array of [name, value, filename?] entries. multipart serialization
    // for fetch body lives in the fetch path (substrate-deferred to first
    // consumer that needs it).
    globalThis.FormData = class FormData {
        constructor() { this._entries = []; }
        append(name, value, filename) {
            const e = [String(name), value];
            if (filename !== undefined) e.push(String(filename));
            this._entries.push(e);
        }
        delete(name) {
            const n = String(name);
            this._entries = this._entries.filter(e => e[0] !== n);
        }
        get(name) {
            const n = String(name);
            const e = this._entries.find(e => e[0] === n);
            return e ? e[1] : null;
        }
        getAll(name) {
            const n = String(name);
            return this._entries.filter(e => e[0] === n).map(e => e[1]);
        }
        has(name) {
            const n = String(name);
            return this._entries.some(e => e[0] === n);
        }
        set(name, value, filename) {
            this.delete(name);
            this.append(name, value, filename);
        }
        *entries() {
            for (const e of this._entries) yield [e[0], e[1]];
        }
        *keys() { for (const e of this._entries) yield e[0]; }
        *values() { for (const e of this._entries) yield e[1]; }
        forEach(cb, thisArg) {
            for (const e of this._entries) cb.call(thisArg, e[1], e[0], this);
        }
        [Symbol.iterator]() { return this.entries(); }
    };
})();
"##;

fn install_url_class_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(URL_CLASS_JS)?;
    Ok(())
}

fn wire_performance<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let time_origin_ms: f64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64() * 1000.0)
        .unwrap_or(0.0);
    let start = std::time::Instant::now();

    let perf = Object::new(ctx.clone())?;
    perf.set(
        "now",
        Function::new(ctx.clone(), move || -> f64 {
            start.elapsed().as_secs_f64() * 1000.0
        })?,
    )?;
    perf.set("timeOrigin", time_origin_ms)?;
    global.set("performance", perf)?;
    Ok(())
}

// ─────────────────── Eval helpers ────────────────────────────────────

pub fn eval_string(source: &str) -> Result<String, String> {
    let (_runtime, context) = new_runtime().map_err(|e| format!("init: {:?}", e))?;
    context.with(|ctx| {
        ctx.eval::<String, _>(source).map_err(|e| format!("eval: {:?}", e))
    })
}

pub fn eval_bool(source: &str) -> Result<bool, String> {
    let (_runtime, context) = new_runtime().map_err(|e| format!("init: {:?}", e))?;
    context.with(|ctx| {
        ctx.eval::<bool, _>(source).map_err(|e| format!("eval: {:?}", e))
    })
}

pub fn eval_i64(source: &str) -> Result<i64, String> {
    let (_runtime, context) = new_runtime().map_err(|e| format!("init: {:?}", e))?;
    context.with(|ctx| {
        ctx.eval::<i64, _>(source).map_err(|e| format!("eval: {:?}", e))
    })
}

/// Load `entry_path` as a CommonJS module via bootRequire, drive the
/// microtask queue, then read `globalThis.__asyncResult` as a string.
/// Used for Tier-J consumer fixtures that complete asynchronously.
pub fn eval_cjs_module_async(entry_path: &str) -> Result<String, String> {
    let (runtime, context) = new_runtime().map_err(|e| format!("init: {:?}", e))?;
    let bootstrap = format!(
        r#"globalThis.__asyncResult = undefined;
        globalThis.__asyncError = undefined;
        bootRequire({});"#,
        serde_json::to_string(entry_path).unwrap()
    );
    context.with(|ctx| -> Result<(), String> {
        ctx.eval::<(), _>(bootstrap.as_str())
            .map_err(|e| format!("boot: {:?}", e))
    })?;
    let mut consecutive_idle = 0;
    for _ in 0..1_000_000 {
        match runtime.execute_pending_job() {
            Ok(true) => { consecutive_idle = 0; continue; }
            Ok(false) => {}
            Err(_) => break,
        }
        let (timer_fired, sleep_ms) = context.with(|ctx| -> (bool, i64) {
            let g = ctx.globals();
            let fired = match g.get::<_, Option<rquickjs::Function>>("__tickTimers") {
                Ok(Some(f)) => f.call::<_, bool>(()).unwrap_or(false),
                _ => false,
            };
            let sleep_ms = if !fired {
                match g.get::<_, Option<rquickjs::Function>>("__nextTimerDelay") {
                    Ok(Some(f)) => f.call::<_, i64>(()).unwrap_or(-1),
                    _ => -1,
                }
            } else { -1 };
            (fired, sleep_ms)
        });
        if timer_fired { consecutive_idle = 0; continue; }
        if sleep_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(sleep_ms.min(50) as u64));
            continue;
        }
        consecutive_idle += 1;
        if consecutive_idle > 2 { break; }
    }
    context.with(|ctx| -> Result<String, String> {
        let g = ctx.globals();
        let err: Option<String> = g.get::<_, Option<String>>("__asyncError")
            .map_err(|e| format!("read err: {:?}", e))?;
        if let Some(e) = err { return Err(format!("js error: {}", e)); }
        let res: Option<String> = g.get::<_, Option<String>>("__asyncResult")
            .map_err(|e| format!("read result: {:?}", e))?;
        if let Some(r) = res { return Ok(r); }
        let stdout: Option<String> = g.get::<_, Option<String>>("__stdoutBuf")
            .map_err(|e| format!("read stdout: {:?}", e))?;
        match stdout {
            Some(s) if !s.is_empty() => Ok(s.trim_end_matches('\n').to_string()),
            _ => Err("module did not set __asyncResult or __stdoutBuf".to_string()),
        }
    })
}

/// Evaluate the ESM module at `entry_path` (absolute path); after the
/// module's top-level executes, read `globalThis.__esmResult` as a string.
/// Tier-H.3 #2: ESM with node-style resolution.
pub fn eval_esm_module(entry_path: &str) -> Result<String, String> {
    let (runtime, context) = new_runtime().map_err(|e| format!("init: {:?}", e))?;
    let source = std::fs::read_to_string(entry_path)
        .map_err(|e| format!("read entry: {}", e))?;
    // Apply the same preprocessors as the FsLoader's load path. The
    // entry source bypasses FsLoader, but if it contains a `/v` regex
    // literal or `\-` inside a /u class, QuickJS will reject it at
    // parse time exactly as it would for an imported file.
    let source = strip_reserved_class_field_decls(&source);
    let source = rewrite_destructure_exports(&source);
    let source = strip_string_module_exports_alias(&source);
    let source = rewrite_regex_u_class_escapes(&source);
    let entry_name = entry_path.to_string();
    context.with(|ctx| -> Result<(), String> {
        ctx.globals().set("__esmResult", rquickjs::Value::new_undefined(ctx.clone()))
            .map_err(|e| format!("init result slot: {:?}", e))?;
        // Gate marker: async eval context. Used by node:http.Server.listen
        // to decide whether to bridge to Bun.serve (only safe when this
        // eval loop is driving microtasks + __keepAlive).
        ctx.globals().set("__asyncEvalActive", true)
            .map_err(|e| format!("init async flag: {:?}", e))?;
        // Π2: declare then evaluate so we can populate import.meta.url
        // (Node ESM idiom — dotenv/many libs derive __dirname from it).
        let declared = Module::declare(ctx.clone(), entry_name.as_str(), source.as_str())
            .map_err(|e| format!("declare entry: {:?}", e))?;
        let meta = declared.meta().map_err(|e| format!("meta: {:?}", e))?;
        let file_url = format!("file://{}", entry_name);
        meta.set("url", file_url).map_err(|e| format!("meta.url: {:?}", e))?;
        meta.set("main", true).map_err(|e| format!("meta.main: {:?}", e))?;
        let (_evaluated, _promise) = declared.eval().map_err(|e| format!("eval entry: {:?}", e))?;
        Ok(())
    })?;
    // Π2.6: drain microtasks AND tick keep-alive registry until both
    // are quiescent. The keep-alive registry is populated by
    // Bun.serve(opts, {autoServe: true}) and removed by server.stop().
    //
    // Termination invariant: exit when microtask queue is empty AND
    // (the keep-alive registry is empty OR __tickKeepAlive has been
    // idle for max_consecutive_idle iterations). Bounded by max_total
    // to prevent runaway loops.
    //
    // Performance discipline (§A8.17): when __keepAlive is empty (the
    // common case for fixtures that don't use autoServe), skip the
    // tick-loop entirely — the prior behavior of microtask-drain-then-
    // exit. This keeps inner-loop budget intact for non-Π2.6 fixtures.
    let max_total: usize = 5_000_000;
    let max_consecutive_idle: usize = 1000;
    let mut total: usize = 0;
    let mut consecutive_idle: usize = 0;
    // Wall-clock cap on the whole eval. Recurring setInterval timers
    // (fastify's plugin chain installs several) keep __nextTimerDelay
    // permanently positive, so timer-pending alone cannot be a
    // termination invariant. Cap total wall-clock to keep fixtures
    // bounded; downstream tests are expected to call .close() to drain
    // pending intervals when they need a clean exit.
    // Wall-clock cap, env-configurable for slow-test runs.
    // RUSTY_BUN_WALLCLOCK_SECS overrides the 15s inner-loop default.
    let wallclock_secs: u64 = std::env::var("RUSTY_BUN_WALLCLOCK_SECS")
        .ok().and_then(|s| s.parse().ok()).unwrap_or(15);
    let wallclock_deadline = std::time::Instant::now() + std::time::Duration::from_secs(wallclock_secs);
    // After this many idle ticks where the ONLY work is recurring timers
    // (no microtask, no keep-alive), give up — the test's productive
    // work is done.
    let max_timer_only_idle: usize = 200;
    let mut timer_only_idle: usize = 0;
    while total < max_total && std::time::Instant::now() < wallclock_deadline {
        total += 1;
        match runtime.execute_pending_job() {
            Ok(true) => { consecutive_idle = 0; timer_only_idle = 0; continue; }
            Ok(false) => {}
            Err(_) => break,
        }
        // Microtask queue empty. Three sources of further work:
        //   1. __keepAlive registry (Bun.serve autoServe) — Π2.6
        //   2. Pending wall-clock timers — E.54 closure
        //   3. Nothing → exit
        let (alive_count, ka_did_work, timer_did_work, sleep_ms) = context.with(|ctx| -> (i32, bool, bool, i64) {
            let g = ctx.globals();
            let alive_count = g
                .get::<_, Option<rquickjs::Object>>("__keepAlive")
                .ok().flatten()
                .and_then(|s| s.get::<_, i32>("size").ok())
                .unwrap_or(0);
            let ka_did = if alive_count > 0 {
                match g.get::<_, Option<rquickjs::Function>>("__tickKeepAlive") {
                    Ok(Some(f)) => f.call::<_, bool>(()).unwrap_or(false),
                    _ => false,
                }
            } else { false };
            let timer_did = match g.get::<_, Option<rquickjs::Function>>("__tickTimers") {
                Ok(Some(f)) => f.call::<_, bool>(()).unwrap_or(false),
                _ => false,
            };
            // If neither fired and keep-alive is idle, see if any timer
            // is pending and report how long until it fires.
            let sleep_ms: i64 = if !timer_did && !ka_did {
                match g.get::<_, Option<rquickjs::Function>>("__nextTimerDelay") {
                    Ok(Some(f)) => f.call::<_, i64>(()).unwrap_or(-1),
                    _ => -1,
                }
            } else { -1 };
            (alive_count, ka_did, timer_did, sleep_ms)
        });
        // Π2.6.c.b: fourth idle source — mio reactor. When microtasks
        // are empty AND keep-alive + timers didn't fire, if any fds are
        // registered with the reactor, poll mio with a short cap so the
        // in-process server's __tickKeepAlive (still thread-per-listener
        // until Π2.6.c.c) keeps running between polls. Drain resolves
        // pending TCP.waitReadable promises.
        let reactor_did_work = if !ka_did_work && !timer_did_work {
            let count = crate::reactor::registered_count();
            if count > 0 {
                let cap_ms: i64 = if sleep_ms > 0 { sleep_ms.min(5) } else { 5 };
                let _ = crate::reactor::poll_once(cap_ms);
                let drained: f64 = context.with(|ctx| -> f64 {
                    let g = ctx.globals();
                    match g.get::<_, Option<rquickjs::Function>>("__reactorDrain") {
                        Ok(Some(f)) => f.call::<_, f64>(()).unwrap_or(0.0),
                        _ => 0.0,
                    }
                });
                drained > 0.0
            } else { false }
        } else { false };
        let did_work = ka_did_work || timer_did_work || reactor_did_work;
        // Exit when nothing keeps us alive and no timer is pending AND
        // no fds are registered with the reactor.
        let reactor_alive = crate::reactor::registered_count() > 0;
        if !did_work && alive_count == 0 && sleep_ms < 0 && !reactor_alive { break; }
        if did_work {
            consecutive_idle = 0;
            // Track ticks where the only work was timer ticks (no
            // keep-alive ticks). If timers fire but produce no
            // microtask work, count as idle for runaway-interval cap.
            // Π2.6.c.b: if the reactor has fds registered, an in-flight
            // fetch is parked on readiness — don't fire the timer-only
            // bail-out (a recurring middleware timer firing while fetch
            // waits is not "the test is done").
            if timer_did_work && !ka_did_work && !reactor_did_work && !reactor_alive {
                timer_only_idle += 1;
                if timer_only_idle > max_timer_only_idle { break; }
            } else {
                timer_only_idle = 0;
            }
        } else {
            // Sleep until the next timer is due (capped so a long-delay
            // timer doesn't hold the loop unresponsive to other work).
            if sleep_ms > 0 {
                let cap = sleep_ms.min(50) as u64;
                std::thread::sleep(std::time::Duration::from_millis(cap));
                consecutive_idle = 0;
                continue;
            }
            consecutive_idle += 1;
            if consecutive_idle > max_consecutive_idle { break; }
        }
    }
    context.with(|ctx| -> Result<String, String> {
        let g = ctx.globals();
        let res: Option<String> = g.get::<_, Option<String>>("__esmResult")
            .map_err(|e| format!("read result: {:?}", e))?;
        if let Some(r) = res { return Ok(r); }
        // Fall back to process.stdout.write buffer — fixtures using the
        // Bun-portable process.stdout.write path write here. Strip trailing
        // newline to match Bun's `.trim()`-shaped differential expectations.
        let stdout: Option<String> = g.get::<_, Option<String>>("__stdoutBuf")
            .map_err(|e| format!("read stdout: {:?}", e))?;
        match stdout {
            Some(s) if !s.is_empty() => Ok(s.trim_end_matches('\n').to_string()),
            _ => Err("module did not set __esmResult or __stdoutBuf".to_string()),
        }
    })
}

/// Evaluate `source` whose top-level expression resolves to a Promise<string>.
/// Drives the QuickJS microtask queue until the promise settles, then returns
/// the resolved string. Used for streams / async-iteration tests.
pub fn eval_string_async(source: &str) -> Result<String, String> {
    let (runtime, context) = new_runtime().map_err(|e| format!("init: {:?}", e))?;
    // Wrap source in a self-invoking async block that stores result on globalThis.
    let wrapped = format!(
        r#"
        globalThis.__asyncResult = undefined;
        globalThis.__asyncError = undefined;
        globalThis.__asyncEvalActive = true;
        Promise.resolve().then(async () => {{
            try {{
                globalThis.__asyncResult = await (async () => {{ {} }})();
            }} catch (e) {{
                globalThis.__asyncError = String(e && e.message ? e.message : e);
            }}
        }});
        "#,
        source
    );
    context.with(|ctx| -> Result<(), String> {
        ctx.eval::<(), _>(wrapped.as_str()).map_err(|e| format!("eval: {:?}", e))
    })?;
    // Pump microtasks + wall-clock timers (E.54 closure).
    let mut iters = 0;
    let mut executed = 0;
    let mut consecutive_idle = 0;
    for _ in 0..1_000_000 {
        iters += 1;
        match runtime.execute_pending_job() {
            Ok(true) => { executed += 1; consecutive_idle = 0; continue; }
            Ok(false) => {}
            Err(_) => break,
        }
        // Microtask queue empty: tick keep-alive + timers + reactor.
        // Π2.6.c.b: mirror the eval_esm_module loop's full set of
        // idle sources here so eval_string_async can also host
        // autoServe + same-process fetch round-trips (the
        // autoserve_self_fetch_round_trips test runs this path).
        let (ka_did, timer_fired, sleep_ms) = context.with(|ctx| -> (bool, bool, i64) {
            let g = ctx.globals();
            let alive_count = g
                .get::<_, Option<rquickjs::Object>>("__keepAlive")
                .ok().flatten()
                .and_then(|s| s.get::<_, i32>("size").ok())
                .unwrap_or(0);
            let ka_did = if alive_count > 0 {
                match g.get::<_, Option<rquickjs::Function>>("__tickKeepAlive") {
                    Ok(Some(f)) => f.call::<_, bool>(()).unwrap_or(false),
                    _ => false,
                }
            } else { false };
            let fired = match g.get::<_, Option<rquickjs::Function>>("__tickTimers") {
                Ok(Some(f)) => f.call::<_, bool>(()).unwrap_or(false),
                _ => false,
            };
            let sleep_ms = if !fired && !ka_did {
                match g.get::<_, Option<rquickjs::Function>>("__nextTimerDelay") {
                    Ok(Some(f)) => f.call::<_, i64>(()).unwrap_or(-1),
                    _ => -1,
                }
            } else { -1 };
            (ka_did, fired, sleep_ms)
        });
        if ka_did || timer_fired { consecutive_idle = 0; continue; }
        // Reactor poll for parked TCP.waitReadable promises.
        let reactor_did = if crate::reactor::registered_count() > 0 {
            let cap_ms: i64 = if sleep_ms > 0 { sleep_ms.min(5) } else { 5 };
            let _ = crate::reactor::poll_once(cap_ms);
            let drained: f64 = context.with(|ctx| -> f64 {
                let g = ctx.globals();
                match g.get::<_, Option<rquickjs::Function>>("__reactorDrain") {
                    Ok(Some(f)) => f.call::<_, f64>(()).unwrap_or(0.0),
                    _ => 0.0,
                }
            });
            drained > 0.0
        } else { false };
        if reactor_did { consecutive_idle = 0; continue; }
        if sleep_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(sleep_ms.min(50) as u64));
            continue;
        }
        // No microtask, no keep-alive, no timer, no reactor activity:
        // settled (or stuck). Allow one extra grace iteration if any
        // fds remain registered, since the next reactor poll might
        // produce a delayed wake.
        consecutive_idle += 1;
        let stuck_cap = if crate::reactor::registered_count() > 0 { 200 } else { 2 };
        if consecutive_idle > stuck_cap { break; }
    }
    if std::env::var("RUSTY_BUN_HOST_DEBUG").is_ok() {
        eprintln!("[host] pump iters={} executed={}", iters, executed);
    }
    context.with(|ctx| -> Result<String, String> {
        let g = ctx.globals();
        let err: Option<String> = g.get::<_, Option<String>>("__asyncError")
            .map_err(|e| format!("read err: {:?}", e))?;
        if let Some(e) = err { return Err(format!("js error: {}", e)); }
        let res: Option<String> = g.get::<_, Option<String>>("__asyncResult")
            .map_err(|e| format!("read result: {:?}", e))?;
        res.ok_or_else(|| "promise did not settle".to_string())
    })
}
