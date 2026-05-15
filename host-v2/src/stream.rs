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
    // Tier-Ω.5.gggg: stream.pipeline / .finished return undefined
    // (instead of throwing). get-stream and many libs import these
    // for presence-checks at module-load and only call them at
    // runtime — letting the import succeed is the substrate goal.
    register_method(rt, stream, "pipeline", |_rt, _args| Ok(Value::Undefined));
    register_method(rt, stream, "finished", |_rt, _args| Ok(Value::Undefined));

    set_constant(rt, stream, "default", Value::Object(stream));
    rt.globals.insert("stream".into(), Value::Object(stream));
}
