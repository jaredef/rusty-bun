//! rusty-bun-host-v2 — host wiring for the rusty-js-runtime engine.
//! Per specs/omega-4-host-migration-design.md.
//!
//! Round Ω.4.b scope: Cargo skeleton + binary entry point + first
//! intrinsics (path / os / process minimal). Subsequent rounds add fs,
//! http, TLS, WebSocket, crypto.subtle, mio reactor integration, and
//! the CJS↔ESM bridge.

pub mod path;
pub mod os;
pub mod process;
pub mod register;

use rusty_js_runtime::Runtime;

/// Install the Bun-host surface onto the engine. Call after
/// `rt.install_intrinsics()` (which installs Math / JSON / console /
/// Promise / globals from the engine itself).
pub fn install_bun_host(rt: &mut Runtime, argv: Vec<String>) {
    path::install(rt);
    os::install(rt);
    process::install(rt, argv);
}
