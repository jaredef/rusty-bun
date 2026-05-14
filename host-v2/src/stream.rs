//! node:stream intrinsic stub — Tier-Ω.5.s.
//!
//! Used by ndjson, split2, and any package that imports the constructor
//! classes at module load to subclass them. v1 exposes the class names
//! as stub-throw NativeFns so import succeeds and `typeof Stream.Readable
//! === "function"` is true; instantiation throws.

use crate::register::{new_object, register_method, set_constant};
use rusty_js_runtime::{Runtime, RuntimeError, Value};

pub fn install(rt: &mut Runtime) {
    let stream = new_object(rt);

    for name in &["Readable", "Writable", "Transform", "Duplex", "PassThrough"] {
        let nm = *name;
        register_method(rt, stream, name, move |_rt, _args| {
            Err(RuntimeError::TypeError(format!(
                "node:stream {nm}: not yet implemented (Tier-Ω.5.s stub)"
            )))
        });
    }
    register_method(rt, stream, "pipeline", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:stream pipeline: not yet implemented (Tier-Ω.5.s stub)".into(),
        ))
    });
    register_method(rt, stream, "finished", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:stream finished: not yet implemented (Tier-Ω.5.s stub)".into(),
        ))
    });

    set_constant(rt, stream, "default", Value::Object(stream));
    rt.globals.insert("stream".into(), Value::Object(stream));
}
