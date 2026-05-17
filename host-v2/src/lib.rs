//! rusty-bun-host-v2 — host wiring for the rusty-js-runtime engine.
//! Per specs/omega-4-host-migration-design.md.
//!
//! Round Ω.4.b scope: Cargo skeleton + binary entry point + first
//! intrinsics (path / os / process minimal). Subsequent rounds add fs,
//! http, TLS, WebSocket, crypto.subtle, mio reactor integration, and
//! the CJS↔ESM bridge.

pub mod assert;
pub mod crypto;
pub mod events;
pub mod fs;
pub mod module_ns;
pub mod node_stubs;
pub mod http;
pub mod https;
pub mod os;
pub mod path;
pub mod timer;
pub mod process;
pub mod register;
pub mod stream;
pub mod tty;
pub mod url;
pub mod util;
pub mod zlib;

use rusty_js_runtime::{HostHook, Runtime, Value};

/// Install the Bun-host surface onto the engine. Call after
/// `rt.install_intrinsics()` (which installs Math / JSON / console /
/// Promise / globals from the engine itself).
///
/// Installs the PollIo host hook that drains the fs PendingIo queue —
/// this is what lets `fs.readFile().then(...)` actually resolve under
/// `run_to_completion`. Tier-Omega.4.d.
pub fn install_bun_host(rt: &mut Runtime, argv: Vec<String>) {
    path::install(rt);
    os::install(rt);
    process::install(rt, argv);
    fs::install(rt);
    fs::install_poll_io(rt);
    // Ω.5.P37.E1.timers: setTimeout / setInterval / setImmediate /
    // queueMicrotask globals. Install after install_poll_io so the
    // shared PollIo hook (in fs.rs) can consult timer::drain_due_pairs.
    timer::install(rt);
    // Ω.5.P16.E2.ns-default-synth: Doc 717 Tuple A/B closure.
    module_ns::install(rt);
    http::install(rt);
    crypto::install(rt);
    // Tier-Ω.5.s: bundle of small built-in stubs.
    assert::install(rt);
    https::install(rt);
    stream::install(rt);
    url::install(rt);
    util::install(rt);
    zlib::install(rt);
    tty::install(rt);
    events::install(rt);
    // Tier-Ω.5.MMMMMMMM: post-install pass to wire stream.EventEmitter from
    // the events ctor (Node re-exports it on node:stream for legacy compat).
    // Done as a post-step rather than reordering events-before-stream
    // because earlier installs (http, crypto, etc.) depend on stream being
    // resolved at their times.
    if let (Some(Value::Object(ee_ctor)), Some(Value::Object(stream_ns))) =
        (rt.globals.get("events").cloned(), rt.globals.get("stream").cloned())
    {
        rt.object_set(stream_ns, "EventEmitter".into(), Value::Object(ee_ctor));
        rt.object_set(stream_ns, "EventEmitterAsyncResource".into(), Value::Object(ee_ctor));
    }
    node_stubs::install_all(rt);
    install_builtin_module_resolver(rt);
    // Tier-Ω.5.t: re-snapshot globalThis so host-v2's added globals
    // (path/os/process/fs/...) become visible on globalThis. install_intrinsics
    // ran globalThis-install before host-v2 wired its globals, so its initial
    // snapshot missed them. Re-running it produces a fresh snapshot that
    // includes everything currently in globals.
    rt.install_global_this_refresh();
}

/// Tier-Ω.5.b: install the ResolveBuiltinModule host hook that maps
/// `node:fs`, `node:path`, `node:os`, `node:process` to the intrinsic
/// objects already registered on `rt.globals` by the install_* functions
/// above. The hook is consulted by `Runtime::evaluate_module` whenever
/// an import specifier starts with `node:`.
///
/// Reuse rationale: the intrinsic install_* functions already build the
/// namespace shape we want (`fs` has `existsSync` / `readFileSync` /
/// `writeFileSync` / ...; `path` has `join` / `basename` / ...). Treating
/// the global as the namespace object is the cleanest closure point.
pub fn install_builtin_module_resolver(rt: &mut Runtime) {
    rt.install_host_hook(HostHook::ResolveBuiltinModule(Box::new(|rt, specifier| {
        // Tier-Ω.5.j.cjs: accept un-prefixed names as well so
        // `require("fs")` works alongside `require("node:fs")` /
        // `import ... from "node:fs"`.
        let global_name = match specifier {
            "node:fs" | "fs" | "node:fs/promises" | "fs/promises" => "fs",
            "node:path" | "path" => "path",
            "node:os" | "os" => "os",
            "node:process" | "process" => "process",
            // Tier-Ω.5.r: http + crypto stubs.
            "node:http" | "http" => "http",
            "node:crypto" | "crypto" => "crypto",
            // Tier-Ω.5.s: assert / https / stream / url / util stubs.
            "node:assert" | "assert" => "assert",
            "node:https" | "https" => "https",
            "node:stream" | "stream" | "node:stream/promises" | "stream/promises" | "node:stream/web" | "stream/web" | "node:stream/consumers" | "stream/consumers" => "stream",
            "node:url" | "url" => "url",
            "node:util" | "util" => "util",
            // Tier-Ω.5.y: zlib + tty stubs.
            "node:zlib" | "zlib" => "zlib",
            "node:tty" | "tty" => "tty",
            "node:events" | "events" => "events",
            // Tier-Ω.5.bb: six more node:* stubs.
            "node:child_process" | "child_process" => "child_process",
            "node:tls" | "tls" => "tls",
            "node:readline" | "readline" => "readline",
            "node:constants" | "constants" => "constants",
            "node:string_decoder" | "string_decoder" => "string_decoder",
            "node:buffer" | "buffer" => "buffer",
            // Tier-Ω.5.kkkk/llll: dns + module stubs.
            "node:dns" | "dns" | "node:dns/promises" | "dns/promises" => "dns",
            "node:module" | "module" => "module",
            "node:http2" | "http2" => "http2",
            // Tier-Ω.5.nnnnnn: additional node:* stubs from broader basket
            "node:net" | "net" => "tls",
            "node:diagnostics_channel" | "diagnostics_channel" => "diagnostics_channel",
            // Tier-Ω.5.FFFFFFF
            "node:v8" | "v8" => "v8",
            "node:inspector" | "inspector" => "inspector",
            "node:vm" | "vm" => "vm",
            // Tier-Ω.5.PPPPPPP
            "node:punycode" | "punycode" => "punycode",
            // Tier-Ω.5.RRRRRRR — surfaced via cheerio tail-walk
            "node:console" | "console" => "console",
            "node:util/types" | "util/types" => "util",
            "node:domain" | "domain" => "domain",
            "node:async_hooks" | "async_hooks" => "async_hooks",
            "node:perf_hooks" | "perf_hooks" => "perf_hooks",
            "node:worker_threads" | "worker_threads" => "events",
            "node:querystring" | "querystring" => "url",
            "node:timers" | "timers" | "node:timers/promises" | "timers/promises" => "events",
            "node:string_decoder" => "string_decoder",
            _ => return Ok(None),
        };
        match rt.globals.get(global_name) {
            Some(Value::Object(id)) => Ok(Some(*id)),
            _ => Ok(None),
        }
    })));
}
