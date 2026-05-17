//! node:zlib stub — Tier-Ω.5.y. Import-time + shape probe only;
//! every method throws a clear "not yet implemented" message.

use crate::register::{new_object, register_method};
use rusty_js_runtime::{Runtime, RuntimeError, Value};
use std::rc::Rc;

fn stub(name: &'static str) -> impl Fn(&mut Runtime, &[Value]) -> Result<Value, RuntimeError> {
    move |_rt, _args| {
        Err(RuntimeError::Thrown(Value::String(Rc::new(format!(
            "TypeError: node:zlib.{name} not yet implemented (Tier-Ω.5.y stub)"
        )))))
    }
}

pub fn install(rt: &mut Runtime) {
    let z = new_object(rt);
    for name in &[
        "deflate", "deflateSync", "deflateRaw", "deflateRawSync",
        "inflate", "inflateSync", "inflateRaw", "inflateRawSync",
        "gzip", "gzipSync", "gunzip", "gunzipSync",
        "brotliCompress", "brotliDecompress",
        "brotliCompressSync", "brotliDecompressSync",
        "createDeflate", "createInflate",
        "createGzip", "createGunzip",
        "createDeflateRaw", "createInflateRaw",
        "createBrotliCompress", "createBrotliDecompress",
    ] {
        register_method(rt, z, name, stub(name));
    }
    // Constructor placeholders for `util.inherits(X, zlib.Inflate)` and
    // `class X extends zlib.Inflate {}` patterns (pngjs Inflate, etc.).
    // Each is an Object with a `prototype` carrying a `constructor` backref;
    // util.inherits reads `super_.prototype` and that's the shape it needs.
    // Call/construct semantics are not wired — consumer code that actually
    // instantiates these will fail downstream, but module-load substrate
    // (the import-and-shape parity layer) only needs the slots to exist.
    for name in &[
        "Zlib",
        "Deflate", "Inflate",
        "DeflateRaw", "InflateRaw",
        "Gzip", "Gunzip",
        "BrotliCompress", "BrotliDecompress",
    ] {
        let ctor = new_object(rt);
        let proto = new_object(rt);
        rt.object_set(ctor, "prototype".into(), Value::Object(proto));
        rt.object_set(proto, "constructor".into(), Value::Object(ctor));
        rt.object_set(z, name.to_string(), Value::Object(ctor));
    }
    rt.globals.insert("zlib".into(), Value::Object(z));
}
