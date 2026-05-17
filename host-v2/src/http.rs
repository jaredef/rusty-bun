//! node:http intrinsic stub — Tier-Ω.5.r.
//!
//! Exposes enough shape that `import http from "node:http"` /
//! `require("node:http")` succeeds and `Object.keys(http).length > 0`,
//! which unblocks shape-probe parity passes for packages like node-fetch
//! that import the module unconditionally even when not all code paths
//! exercise it.
//!
//! All callable surface throws TypeError("not yet implemented") — the
//! goal is import-time success, not runtime functionality. Real HTTP
//! lives behind a future Tier-Π wiring round.

use crate::register::{new_object, register_method, set_constant};
use rusty_js_runtime::{Runtime, RuntimeError, Value};
use std::rc::Rc;

pub fn install(rt: &mut Runtime) {
    let http = new_object(rt);

    register_method(rt, http, "request", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:http http.request: not yet implemented (Tier-Ω.5.r stub)".into(),
        ))
    });
    register_method(rt, http, "get", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:http http.get: not yet implemented (Tier-Ω.5.r stub)".into(),
        ))
    });
    register_method(rt, http, "createServer", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:http http.createServer: not yet implemented (Tier-Ω.5.r stub)".into(),
        ))
    });
    // Ω.5.P49.E4: parallel to https.Agent — benign at module-init.
    register_method(rt, http, "Agent", |rt, _args| {
        let id = rt.alloc_object(rusty_js_runtime::Object::new_ordinary());
        Ok(Value::Object(id))
    });

    // STATUS_CODES — partial. Enough entries that callers probing
    // `STATUS_CODES[200]` / `STATUS_CODES[404]` get sensible strings.
    let codes = new_object(rt);
    for (code, msg) in &[
        (100, "Continue"),
        (101, "Switching Protocols"),
        (200, "OK"),
        (201, "Created"),
        (202, "Accepted"),
        (204, "No Content"),
        (301, "Moved Permanently"),
        (302, "Found"),
        (304, "Not Modified"),
        (307, "Temporary Redirect"),
        (308, "Permanent Redirect"),
        (400, "Bad Request"),
        (401, "Unauthorized"),
        (403, "Forbidden"),
        (404, "Not Found"),
        (405, "Method Not Allowed"),
        (408, "Request Timeout"),
        (409, "Conflict"),
        (410, "Gone"),
        (429, "Too Many Requests"),
        (500, "Internal Server Error"),
        (501, "Not Implemented"),
        (502, "Bad Gateway"),
        (503, "Service Unavailable"),
        (504, "Gateway Timeout"),
    ] {
        set_constant(rt, codes, &code.to_string(), Value::String(Rc::new((*msg).into())));
    }
    set_constant(rt, http, "STATUS_CODES", Value::Object(codes));

    // METHODS — list of supported HTTP method names. node-fetch and
    // similar shims occasionally read this.
    let methods = new_object(rt);
    let names = [
        "ACL", "BIND", "CHECKOUT", "CONNECT", "COPY", "DELETE", "GET", "HEAD",
        "LINK", "LOCK", "M-SEARCH", "MERGE", "MKACTIVITY", "MKCALENDAR", "MKCOL",
        "MOVE", "NOTIFY", "OPTIONS", "PATCH", "POST", "PROPFIND", "PROPPATCH",
        "PURGE", "PUT", "REBIND", "REPORT", "SEARCH", "SOURCE", "SUBSCRIBE",
        "TRACE", "UNBIND", "UNLINK", "UNLOCK", "UNSUBSCRIBE",
    ];
    for (i, n) in names.iter().enumerate() {
        set_constant(rt, methods, &i.to_string(), Value::String(Rc::new((*n).into())));
    }
    set_constant(rt, methods, "length", Value::Number(names.len() as f64));
    set_constant(rt, http, "METHODS", Value::Object(methods));

    // Tier-Ω.5.xxxxxx: http.ServerResponse / IncomingMessage / Server class
    // stubs with .prototype. compression/on-headers/koa-style middleware read
    // `http.ServerResponse.prototype.appendHeader` at module-init; without
    // ServerResponse the lookup throws on `.prototype`. The class is a
    // stub: constructor errors if called, prototype is an empty object,
    // sufficient for `class X extends http.ServerResponse` and
    // `typeof http.ServerResponse.prototype.foo === 'function'` probes.
    for class_name in &["ServerResponse", "IncomingMessage", "Server", "ClientRequest"] {
        let proto = new_object(rt);
        let ctor = crate::register::make_callable(rt, class_name, |_rt, _args| {
            Err(RuntimeError::TypeError(format!(
                "node:http class constructor not yet implemented (Tier-Ω.5.xxxxxx stub)",
            )))
        });
        rt.object_set(ctor, "prototype".into(), Value::Object(proto));
        rt.object_set(proto, "constructor".into(), Value::Object(ctor));
        set_constant(rt, http, class_name, Value::Object(ctor));
    }

    // Default export points at the namespace itself for CJS-interop
    // shape: `import http from "node:http"` reads `default` and falls
    // back to the namespace if absent, but writing it explicitly keeps
    // `http.default === http` round-trip honest for callers that probe.
    set_constant(rt, http, "default", Value::Object(http));

    rt.globals.insert("http".into(), Value::Object(http));
}
