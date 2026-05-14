//! node:url intrinsic — Tier-Ω.5.s.
//!
//! Provides `fileURLToPath` / `pathToFileURL` actually implemented (cheap
//! and load-bearing for pathe), and re-exposes globalThis.URL /
//! URLSearchParams when available.

use crate::register::{new_object, register_method, set_constant};
use rusty_js_runtime::{Runtime, RuntimeError, Value};
use std::rc::Rc;

pub fn install(rt: &mut Runtime) {
    let url_ns = new_object(rt);

    register_method(rt, url_ns, "fileURLToPath", |_rt, args| {
        let s = match args.first() {
            Some(Value::String(s)) => s.as_str().to_string(),
            Some(v) => rusty_js_runtime::abstract_ops::to_string(v).as_str().to_string(),
            None => return Err(RuntimeError::TypeError(
                "url.fileURLToPath: missing argument".into())),
        };
        let stripped = s.strip_prefix("file://").unwrap_or(&s).to_string();
        Ok(Value::String(Rc::new(stripped)))
    });
    register_method(rt, url_ns, "pathToFileURL", |_rt, args| {
        let s = match args.first() {
            Some(Value::String(s)) => s.as_str().to_string(),
            Some(v) => rusty_js_runtime::abstract_ops::to_string(v).as_str().to_string(),
            None => return Err(RuntimeError::TypeError(
                "url.pathToFileURL: missing argument".into())),
        };
        Ok(Value::String(Rc::new(format!("file://{}", s))))
    });
    register_method(rt, url_ns, "parse", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:url url.parse: not yet implemented (Tier-Ω.5.s stub; legacy API)".into(),
        ))
    });

    // URL / URLSearchParams: re-expose globals if installed; else stub.
    if let Some(g) = rt.globals.get("URL").cloned() {
        set_constant(rt, url_ns, "URL", g);
    } else {
        register_method(rt, url_ns, "URL", |_rt, _args| {
            Err(RuntimeError::TypeError(
                "node:url URL constructor: not yet implemented (Tier-Ω.5.s stub)".into(),
            ))
        });
    }
    if let Some(g) = rt.globals.get("URLSearchParams").cloned() {
        set_constant(rt, url_ns, "URLSearchParams", g);
    } else {
        register_method(rt, url_ns, "URLSearchParams", |_rt, _args| {
            Err(RuntimeError::TypeError(
                "node:url URLSearchParams: not yet implemented (Tier-Ω.5.s stub)".into(),
            ))
        });
    }

    set_constant(rt, url_ns, "default", Value::Object(url_ns));
    rt.globals.insert("url".into(), Value::Object(url_ns));
}
