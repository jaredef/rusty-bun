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
    rt.globals.insert("zlib".into(), Value::Object(z));
}
