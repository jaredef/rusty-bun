//! rusty-bun-host-v2 — host wiring for the rusty-js-runtime engine.
//! Per specs/omega-4-host-migration-design.md.
//!
//! Round Ω.4.b scope: Cargo skeleton + binary entry point + first
//! intrinsics (path / os / process minimal). Subsequent rounds add fs,
//! http, TLS, WebSocket, crypto.subtle, mio reactor integration, and
//! the CJS↔ESM bridge.

pub mod assert;
pub mod crypto;
pub mod fs;
pub mod http;
pub mod https;
pub mod os;
pub mod path;
pub mod process;
pub mod register;
pub mod stream;
pub mod url;
pub mod util;

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
    http::install(rt);
    crypto::install(rt);
    // Tier-Ω.5.s: bundle of small built-in stubs.
    assert::install(rt);
    https::install(rt);
    stream::install(rt);
    url::install(rt);
    util::install(rt);
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
            "node:fs" | "fs" => "fs",
            "node:path" | "path" => "path",
            "node:os" | "os" => "os",
            "node:process" | "process" => "process",
            // Tier-Ω.5.r: http + crypto stubs.
            "node:http" | "http" => "http",
            "node:crypto" | "crypto" => "crypto",
            // Tier-Ω.5.s: assert / https / stream / url / util stubs.
            "node:assert" | "assert" => "assert",
            "node:https" | "https" => "https",
            "node:stream" | "stream" => "stream",
            "node:url" | "url" => "url",
            "node:util" | "util" => "util",
            _ => return Ok(None),
        };
        match rt.globals.get(global_name) {
            Some(Value::Object(id)) => Ok(Some(*id)),
            _ => Ok(None),
        }
    })));
}
