//! node:https intrinsic stub — Tier-Ω.5.s.
//!
//! Mirror of host-v2/src/http.rs (Ω.5.r). Exposes enough shape that
//! `import https from "node:https"` succeeds and node-fetch's import-time
//! probe finds a populated namespace. All callable surface throws
//! TypeError("not yet implemented") — real TLS lives behind a future
//! Tier-Π wiring round (rusty-mtls, rusty-x509, rusty-websocket).

use crate::register::{new_object, register_method, set_constant};
use rusty_js_runtime::{Runtime, RuntimeError, Value};

pub fn install(rt: &mut Runtime) {
    let https = new_object(rt);

    register_method(rt, https, "request", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:https https.request: not yet implemented (Tier-Ω.5.s stub)".into(),
        ))
    });
    register_method(rt, https, "get", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:https https.get: not yet implemented (Tier-Ω.5.s stub)".into(),
        ))
    });
    register_method(rt, https, "createServer", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:https https.createServer: not yet implemented (Tier-Ω.5.s stub)".into(),
        ))
    });
    register_method(rt, https, "Agent", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:https https.Agent: not yet implemented (Tier-Ω.5.s stub)".into(),
        ))
    });

    set_constant(rt, https, "default", Value::Object(https));
    rt.globals.insert("https".into(), Value::Object(https));
}
