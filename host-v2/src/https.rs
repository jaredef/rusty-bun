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
    // Ω.5.P49.E4: https.Agent benign at module-init. Playwright's
    // happyEyeballs.ts constructs `new https.Agent({...})` at top level
    // (an outer-scope default value). A throwing stub kills the import;
    // a benign one returning an empty object lets the bundle finish
    // initializing. Actual TLS-pooling behavior is still unwired.
    register_method(rt, https, "Agent", |rt, _args| {
        let id = rt.alloc_object(rusty_js_runtime::Object::new_ordinary());
        Ok(Value::Object(id))
    });

    set_constant(rt, https, "default", Value::Object(https));
    rt.globals.insert("https".into(), Value::Object(https));
}
