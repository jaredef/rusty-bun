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

/// Build a fresh rquickjs Runtime + Context with all rusty-bun pilots wired
/// into globalThis. Includes the ESM node-style module resolver/loader
/// (Tier-H.3); CommonJS is still wired JS-side via `bootRequire(absPath)`.
pub fn new_runtime() -> JsResult<(Runtime, Context)> {
    let runtime = Runtime::new()?;
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

fn try_directory_with_index(abs_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let pkg_json = abs_dir.join("package.json");
    if pkg_json.is_file() {
        if let Ok(text) = std::fs::read_to_string(&pkg_json) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                let main_str = parsed
                    .get("module")
                    .or_else(|| parsed.get("main"))
                    .and_then(|v| v.as_str());
                if let Some(main) = main_str {
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
                let sub = sub_path.trim_start_matches('/');
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
        "node:process" | "process"
    )
}

/// Generate an ESM re-export source for a node:* builtin module.
/// Per M8/M9: aligns ESM import semantics for node-builtins with Bun's
/// surface, so consumer code using `import x from "node:path"` works.
fn node_builtin_esm_source(name: &str) -> Option<String> {
    let (global_var, named_exports): (&str, &[&str]) = match name {
        "node:fs" | "fs" => ("fs", &["readFileSync", "readFileSyncUtf8", "readFileSyncBytes",
            "writeFileSync", "existsSync", "isFileSync", "isDirectorySync",
            "unlinkSync", "mkdirSyncRecursive", "rmdirSyncRecursive"]),
        "node:path" | "path" => ("path", &["basename", "dirname", "extname", "join",
            "normalize", "isAbsolute", "sep"]),
        "node:http" | "http" => ("nodeHttp", &["createServer", "request",
            "IncomingMessage", "ServerResponse", "ClientRequest", "Server"]),
        "node:crypto" | "crypto" => ("crypto", &["randomUUID", "subtle"]),
        "node:buffer" | "buffer" => ("Buffer", &[]),  // see special handling below
        "node:url" | "url" => ("URL", &[]),
        "node:os" | "os" => ("os", &["platform", "arch", "type", "tmpdir",
            "homedir", "hostname", "endianness", "EOL"]),
        "node:process" | "process" => ("process", &["argv", "env", "platform",
            "arch", "version", "versions", "cwd", "exit", "stdout", "stderr",
            "hrtime"]),
        _ => return None,
    };
    // node:buffer exports `{ Buffer }` not the Buffer itself.
    if name == "node:buffer" || name == "buffer" {
        return Some(
            "const __m = globalThis.Buffer;\nexport const Buffer = __m;\nexport default { Buffer: __m };\n".to_string()
        );
    }
    if name == "node:url" || name == "url" {
        return Some(
            "const __URL = globalThis.URL;\nconst __USP = globalThis.URLSearchParams;\nexport const URL = __URL;\nexport const URLSearchParams = __USP;\nexport default { URL: __URL, URLSearchParams: __USP };\n".to_string()
        );
    }
    let mut s = format!("const __m = globalThis.{};\nexport default __m;\n", global_var);
    for ex in named_exports {
        s.push_str(&format!("export const {} = __m.{};\n", ex, ex));
    }
    Some(s)
}

#[derive(Default, Clone, Copy)]
struct FsLoader;

impl Loader for FsLoader {
    fn load<'js>(&mut self, ctx: &Ctx<'js>, name: &str) -> JsResult<Module<'js, Declared>> {
        if let Some(src) = node_builtin_esm_source(name) {
            return Module::declare(ctx.clone(), name, src);
        }
        let source = std::fs::read_to_string(name)
            .map_err(|_| JsErr::new_loading(name))?;
        Module::declare(ctx.clone(), name, source)
    }
}

fn wire_globals<'js>(ctx: rquickjs::Ctx<'js>) -> JsResult<()> {
    let global = ctx.globals();
    wire_console(&ctx, &global)?;
    wire_atob_btoa(&ctx, &global)?;
    wire_path(&ctx, &global)?;
    wire_os(&ctx, &global)?;
    wire_crypto(&ctx, &global)?;
    wire_text_encoding(&ctx, &global)?;
    wire_buffer(&ctx, &global)?;
    install_buffer_class_js(&ctx)?;
    install_set_methods_polyfill(&ctx)?;
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
    wire_bun_namespace_static(&ctx, &global)?;
    install_bun_namespace_js(&ctx)?;
    wire_bun_serve_static(&ctx, &global)?;
    install_bun_serve_js(&ctx)?;
    wire_bun_spawn_static(&ctx, &global)?;
    install_bun_spawn_js(&ctx)?;
    install_structured_clone_js(&ctx)?;
    install_streams_js(&ctx)?;
    install_node_http_js(&ctx)?;
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
    process.set("version", "v0.0.0-rusty-bun-host")?;
    process.set("versions", {
        let v = Object::new(ctx.clone())?;
        v.set("node", "0.0.0")?;
        v.set("rusty_bun_host", "0.0.0")?;
        v
    })?;

    process.set("cwd", Function::new(ctx.clone(), || -> String {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "/".to_string())
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
    Ok(())
}

// ─────────────────── crypto + crypto.subtle ──────────────────────────

fn wire_crypto<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let crypto = Object::new(ctx.clone())?;
    crypto.set(
        "randomUUID",
        Function::new(ctx.clone(), || rusty_web_crypto::random_uuid_v4())?,
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

            // digest(algorithm, data) → Promise<ArrayBuffer>
            subtle.digest = async function digest(algorithm, data) {
                const alg = normalizeAlg(algorithm);
                if (alg.name !== "SHA256") {
                    throw new Error("Unsupported digest algorithm: " + alg.name);
                }
                return toArrayBuffer(subtle.digestSha256Bytes(toBytes(data)));
            };

            // importKey(format, keyData, algorithm, extractable, keyUsages)
            // Pilot scope: format="raw", algorithm={name:"HMAC", hash:"SHA-256"}.
            // Returns a CryptoKey-shaped object whose private _bytes carry the
            // key material. Async per spec (returns Promise).
            subtle.importKey = async function importKey(format, keyData, algorithm, extractable, keyUsages) {
                if (format !== "raw") {
                    throw new Error("Unsupported key format: " + format);
                }
                const alg = normalizeAlg(algorithm);
                if (alg.name !== "HMAC") {
                    throw new Error("Unsupported key algorithm: " + alg.name);
                }
                if (alg.hash !== "SHA256") {
                    throw new Error("Unsupported HMAC hash: " + alg.hash);
                }
                const bytes = toBytes(keyData);
                return {
                    type: "secret",
                    extractable: !!extractable,
                    algorithm: { name: "HMAC", hash: { name: "SHA-256" }, length: bytes.length * 8 },
                    usages: Array.isArray(keyUsages) ? keyUsages.slice() : [],
                    _bytes: bytes,  // implementation-private
                };
            };

            // sign(algorithm, key, data) → Promise<ArrayBuffer>
            subtle.sign = async function sign(algorithm, key, data) {
                const alg = normalizeAlg(algorithm);
                if (alg.name !== "HMAC") {
                    throw new Error("Unsupported sign algorithm: " + alg.name);
                }
                if (!key || !Array.isArray(key._bytes)) {
                    throw new TypeError("sign: key is not a valid HMAC key");
                }
                if (!key.usages.includes("sign")) {
                    throw new Error("Key not usable for sign");
                }
                return toArrayBuffer(subtle.hmacSha256Bytes(key._bytes, toBytes(data)));
            };

            // verify(algorithm, key, signature, data) → Promise<boolean>
            subtle.verify = async function verify(algorithm, key, signature, data) {
                const alg = normalizeAlg(algorithm);
                if (alg.name !== "HMAC") {
                    throw new Error("Unsupported verify algorithm: " + alg.name);
                }
                if (!key || !Array.isArray(key._bytes)) {
                    throw new TypeError("verify: key is not a valid HMAC key");
                }
                if (!key.usages.includes("verify")) {
                    throw new Error("Key not usable for verify");
                }
                const expected = subtle.hmacSha256Bytes(key._bytes, toBytes(data));
                return subtle.timingSafeEqualBytes(toBytes(signature), expected);
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
                if (input === undefined) return __te.encode();
                return __te.encode(input);
            }
        };
        globalThis.TextDecoder = class TextDecoder {
            constructor(label) { this._label = label; }
            get encoding() { return "utf-8"; }
            decode(bytes) {
                // Normalize Uint8Array / Buffer / typed-array views → plain
                // JS array (rquickjs Vec<u8> binding doesn't accept typed
                // arrays directly).
                if (bytes && typeof bytes === "object" && !Array.isArray(bytes)) {
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
        static from(input, encoding) {
            if (typeof input === "string") {
                // Currently only utf8 string→bytes is wired in S.
                const arr = S.from(input);
                return new Buffer(arr);
            }
            if (Array.isArray(input) || input instanceof Uint8Array || ArrayBuffer.isView(input)) {
                const buf = new Buffer(input.length || input.byteLength || 0);
                buf.set(input);
                return buf;
            }
            throw new TypeError("Buffer.from: unsupported input");
        }
        static alloc(size) { return new Buffer(size); }
        static byteLength(s) { return S.byteLength(s); }
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
            throw new Error("Unsupported encoding: " + encoding);
        }
    }
    globalThis.Buffer = Buffer;
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
        if (Array.isArray(this._body)) {
            return new TextDecoder().decode(this._body);
        }
        return String(this._body);
    }
    async arrayBuffer() {
        if (this._bodyUsed) throw new TypeError("Body already used");
        this._bodyUsed = true;
        if (this._body === null || this._body === undefined) return [];
        if (typeof this._body === "string") return new TextEncoder().encode(this._body);
        if (Array.isArray(this._body)) return this._body;
        return [];
    }
    async bytes() { return this.arrayBuffer(); }
    async json() {
        return JSON.parse(await this.text());
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
        if (Array.isArray(this._body)) return new TextDecoder().decode(this._body);
        return String(this._body);
    }
    async arrayBuffer() {
        if (this._bodyUsed) throw new TypeError("Body already used");
        this._bodyUsed = true;
        if (this._body === null || this._body === undefined) return [];
        if (typeof this._body === "string") return new TextEncoder().encode(this._body);
        if (Array.isArray(this._body)) return this._body;
        return [];
    }
    async bytes() { return this.arrayBuffer(); }
    async json() {
        return JSON.parse(await this.text());
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
            stop() { this._stopped = true; },
            reload(newOptions) {
                // Per spec: port + hostname preserved across reload.
                const port = this._port;
                const hostname = this._hostname;
                Object.assign(this, Bun.serve(newOptions));
                this._port = port;
                this._hostname = hostname;
            },
        };
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

    class IncomingMessage {
        constructor(init = {}) {
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
    }

    class ServerResponse {
        constructor() {
            this.statusCode = 200;
            this.statusMessage = "OK";
            this._headers = makeHeaders();
            this._body = [];
            this.headersSent = false;
            this.ended = false;
        }
        writeHead(statusCode, statusMessage, headers) {
            if (this.headersSent) return this;
            this.statusCode = statusCode;
            // statusMessage is optional; if it's an object it's the headers arg.
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
        }
        setHeader(name, value) { setHeader(this._headers, name, value); return this; }
        getHeader(name) { return getHeader(this._headers, name); }
        removeHeader(name) { removeHeader(this._headers, name); return this; }
        getHeaders() { return Object.assign({}, this._headers); }
        write(chunk) {
            if (this.ended) return false;
            this.headersSent = true;
            this._body.push(String(chunk));
            return true;
        }
        end(chunk) {
            if (this.ended) return this;
            if (chunk !== undefined) this._body.push(String(chunk));
            this.headersSent = true;
            this.ended = true;
            return this;
        }
        // Pilot helper: serialize body to string.
        body() { return this._body.join(""); }
    }

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
        }
        on(event, handler) {
            if (event === "request") this._handler = handler;
            return this;
        }
        listen(port, cb) {
            this._port = port;
            this._listening = true;
            if (typeof cb === "function") {
                Promise.resolve().then(cb);
            }
            return this;
        }
        close(cb) {
            this._listening = false;
            this._closed = true;
            if (typeof cb === "function") {
                Promise.resolve().then(cb);
            }
            return this;
        }
        get listening() { return this._listening; }
        get port() { return this._port; }
        // Pilot-only invocation: route a synthetic IncomingMessage through
        // the handler and return the populated ServerResponse. Real Node
        // delivers via socket; this is data-layer dispatch.
        dispatch(req) {
            const incoming = req instanceof IncomingMessage ? req : new IncomingMessage(req);
            const res = new ServerResponse();
            if (this._handler) this._handler(incoming, res);
            return res;
        }
    }

    function createServer(handler) {
        return new Server(handler);
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
        IncomingMessage,
        ServerResponse,
        ClientRequest,
        Server,
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

    function readSourceUtf8(absPath) {
        return fs.readFileSyncUtf8(absPath);
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

    function tryDirectoryWithIndex(absDir) {
        const pkgJson = absDir + "/package.json";
        if (pathExists(pkgJson)) {
            try {
                const pkg = JSON.parse(readSourceUtf8(pkgJson));
                // exports field — string or object with "." key.
                if (pkg.exports) {
                    let main = null;
                    if (typeof pkg.exports === "string") main = pkg.exports;
                    else if (typeof pkg.exports === "object") {
                        const dot = pkg.exports["."];
                        if (typeof dot === "string") main = dot;
                        else if (dot && typeof dot === "object") {
                            main = dot.require || dot.default || dot.node;
                        }
                    }
                    if (main) {
                        const resolved = tryExtensions(normalizePath(joinPath(absDir, main)));
                        if (resolved) return resolved;
                    }
                }
                // main field.
                if (typeof pkg.main === "string") {
                    const resolved = tryExtensions(normalizePath(joinPath(absDir, pkg.main)));
                    if (resolved) return resolved;
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
            throw e;
        }
        return moduleObj.exports;
    }

    globalThis.bootRequire = function bootRequire(absPath) {
        return loadModule(absPath);
    };
    // Expose resolution + cache for tests/diagnostics.
    globalThis.__cjs = { resolvePath, moduleCache, loadModule };
})();
"#;

fn install_commonjs_loader_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(COMMONJS_LOADER_JS)?;
    Ok(())
}

// ════════════════════════════════════════════════════════════════════════
// Tier-H.4: timers, queueMicrotask, performance
// ════════════════════════════════════════════════════════════════════════
//
// Real consumer code uses setTimeout/setImmediate/queueMicrotask
// pervasively, often with ms=0 for "next tick" semantics or with large
// ms for retry/timeout logic that doesn't fire during synchronous tests.
//
// Pilot scope: timers are scheduled as Promise.resolve().then() — i.e.,
// they run on the microtask queue rather than after a real wall-clock
// delay. This is sufficient for the dominant consumer pattern
// (setTimeout(fn, 0) or setImmediate(fn) for deferred work) and enough
// to validate that consumer code's "delay-then-do-X" paths execute
// without throwing. Real wall-clock delays for ms>0 are deferred to a
// follow-up round; they require the host pump to track timer expirations
// (currently the pump is microtask-only).
//
// performance.now() / .timeOrigin: backed by std::time::Instant via a
// Rust closure. timeOrigin is captured at runtime construction.

const TIMERS_AND_PERF_JS: &str = r#"
(function() {
    const timers = new Map();  // id → { cleared, fn, args }
    let nextId = 1;

    function setTimeoutImpl(fn, _ms, ...args) {
        if (typeof fn !== "function") {
            // Per WHATWG: string fn is allowed but pilot rejects.
            throw new TypeError("setTimeout requires a function");
        }
        const id = nextId++;
        const entry = { cleared: false, fn, args };
        timers.set(id, entry);
        // Pilot scope: schedule on microtask queue regardless of _ms.
        Promise.resolve().then(() => {
            if (entry.cleared) return;
            timers.delete(id);
            try { fn.apply(undefined, args); }
            catch (e) {
                // Per spec, exceptions in timer callbacks become
                // unhandled errors. Pilot logs to console.error.
                if (typeof console !== "undefined" && console.error) {
                    console.error("uncaught in setTimeout:", e);
                }
            }
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

    function setIntervalImpl(_fn, _ms, ..._args) {
        // setInterval requires real-time scheduling beyond the microtask
        // pump. Deferred to a follow-up round; throwing is preferable
        // to silently no-op'ing.
        throw new Error("setInterval is not yet supported in rusty-bun-host");
    }

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

    globalThis.URL = URL;
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
    for _ in 0..1_000_000 {
        match runtime.execute_pending_job() {
            Ok(true) => continue,
            Ok(false) => break,
            Err(_) => break,
        }
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
    let entry_name = entry_path.to_string();
    context.with(|ctx| -> Result<(), String> {
        ctx.globals().set("__esmResult", rquickjs::Value::new_undefined(ctx.clone()))
            .map_err(|e| format!("init result slot: {:?}", e))?;
        let _promise = Module::evaluate(ctx.clone(), entry_name.as_str(), source.as_str())
            .map_err(|e| format!("declare entry: {:?}", e))?;
        Ok(())
    })?;
    for _ in 0..1_000_000 {
        match runtime.execute_pending_job() {
            Ok(true) => continue,
            Ok(false) => break,
            Err(_) => break,
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
    // Pump microtasks.
    let mut iters = 0;
    let mut executed = 0;
    for _ in 0..100_000 {
        iters += 1;
        match runtime.execute_pending_job() {
            Ok(true) => { executed += 1; continue; }
            Ok(false) => break,
            Err(_) => break,
        }
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
