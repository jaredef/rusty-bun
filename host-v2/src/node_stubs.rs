//! node:* stub modules — Tier-Ω.5.bb. Each module exposes a populated
//! namespace so import-time and shape probes succeed. Methods throw
//! "not yet implemented" when actually called.
//!
//! Modules covered: child_process, tls, readline, constants,
//! string_decoder, buffer.

use crate::register::{new_object, register_method};
use rusty_js_runtime::value::Object as RtObject;
use rusty_js_runtime::{Runtime, RuntimeError, Value};
use std::rc::Rc;

fn stub(module: &'static str, method: &'static str) -> impl Fn(&mut Runtime, &[Value]) -> Result<Value, RuntimeError> {
    move |_rt, _args| {
        Err(RuntimeError::Thrown(Value::String(Rc::new(format!(
            "TypeError: node:{module}.{method} not yet implemented (Tier-Ω.5.bb stub)"
        )))))
    }
}

pub fn install_child_process(rt: &mut Runtime) {
    let ns = new_object(rt);
    for m in &["spawn", "spawnSync", "exec", "execSync", "execFile", "execFileSync", "fork"] {
        register_method(rt, ns, m, stub("child_process", m));
    }
    rt.globals.insert("child_process".into(), Value::Object(ns));
}

pub fn install_tls(rt: &mut Runtime) {
    let ns = new_object(rt);
    for m in &["connect", "createServer", "createSecureContext", "TLSSocket", "Server"] {
        register_method(rt, ns, m, stub("tls", m));
    }
    rt.globals.insert("tls".into(), Value::Object(ns));
}

pub fn install_readline(rt: &mut Runtime) {
    let ns = new_object(rt);
    for m in &["createInterface", "Interface", "emitKeypressEvents", "cursorTo", "moveCursor", "clearLine", "clearScreenDown"] {
        register_method(rt, ns, m, stub("readline", m));
    }
    rt.globals.insert("readline".into(), Value::Object(ns));
}

pub fn install_constants(rt: &mut Runtime) {
    // node:constants is a flat namespace of numeric constants. v1: empty
    // object (consumers read constants; missing ones are undefined which
    // most code tolerates).
    let ns = new_object(rt);
    // A few of the more common ones:
    rt.object_set(ns, "O_RDONLY".into(), Value::Number(0.0));
    rt.object_set(ns, "O_WRONLY".into(), Value::Number(1.0));
    rt.object_set(ns, "O_RDWR".into(), Value::Number(2.0));
    rt.object_set(ns, "O_CREAT".into(), Value::Number(64.0));
    rt.object_set(ns, "O_EXCL".into(), Value::Number(128.0));
    rt.object_set(ns, "O_APPEND".into(), Value::Number(1024.0));
    rt.object_set(ns, "S_IFMT".into(), Value::Number(61440.0));
    rt.object_set(ns, "S_IFREG".into(), Value::Number(32768.0));
    rt.object_set(ns, "S_IFDIR".into(), Value::Number(16384.0));
    rt.globals.insert("constants".into(), Value::Object(ns));
}

pub fn install_string_decoder(rt: &mut Runtime) {
    let ns = new_object(rt);
    register_method(rt, ns, "StringDecoder", stub("string_decoder", "StringDecoder"));
    rt.globals.insert("string_decoder".into(), Value::Object(ns));
}

pub fn install_buffer(rt: &mut Runtime) {
    // node:buffer exposes Buffer + Blob. v1: stub class with .from / .alloc
    // / .isBuffer / .byteLength. Real Buffer needs binary internals.
    let ns = new_object(rt);
    let buf_ctor = new_object(rt);
    register_method(rt, buf_ctor, "from", |rt, args| {
        // Buffer.from(str) → produces a plain object pretending to be a Buffer.
        let s = match args.first() {
            Some(Value::String(s)) => s.as_str().to_string(),
            _ => String::new(),
        };
        let mut o = RtObject::new_ordinary();
        o.set_own("__buffer_data".into(), Value::String(Rc::new(s.clone())));
        o.set_own("length".into(), Value::Number(s.len() as f64));
        let id = rt.alloc_object(o);
        Ok(Value::Object(id))
    });
    register_method(rt, buf_ctor, "alloc", |rt, args| {
        let n = match args.first() {
            Some(Value::Number(n)) => *n as usize,
            _ => 0,
        };
        let mut o = RtObject::new_ordinary();
        o.set_own("length".into(), Value::Number(n as f64));
        let id = rt.alloc_object(o);
        Ok(Value::Object(id))
    });
    register_method(rt, buf_ctor, "isBuffer", |_rt, _args| Ok(Value::Boolean(false)));
    register_method(rt, buf_ctor, "byteLength", |rt, args| {
        let v = args.first().cloned().unwrap_or(Value::Undefined);
        let n = match &v {
            Value::String(s) => s.as_bytes().len(),
            _ => 0,
        };
        Ok(Value::Number(n as f64))
    });
    // Tier-Ω.5.bbb: Buffer.prototype as a real object so
    // `Object.create(Buffer.prototype)` (safe-buffer / many polyfills)
    // and inheritance chains terminate properly.
    let buf_proto = new_object(rt);
    rt.object_set(buf_ctor, "prototype".into(), Value::Object(buf_proto));
    rt.object_set(ns, "Buffer".into(), Value::Object(buf_ctor));
    register_method(rt, ns, "Blob", stub("buffer", "Blob"));
    rt.globals.insert("buffer".into(), Value::Object(ns));
    // Tier-Ω.5.oo: Buffer also visible as a top-level global per Node
    // convention. csv-parse + csv-parser + many others call
    // `Buffer.from(...)` at module level without importing node:buffer.
    rt.globals.insert("Buffer".into(), Value::Object(buf_ctor));
}

pub fn install_all(rt: &mut Runtime) {
    install_child_process(rt);
    install_tls(rt);
    install_readline(rt);
    install_constants(rt);
    install_string_decoder(rt);
    install_buffer(rt);
}
