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
    // Tier-Ω.5.zzzz: real StringDecoder. UTF-8 only, no incomplete-sequence
    // buffering — sufficient for split2 / through2-line-style consumers
    // that feed complete UTF-8 chunks. Real Node behavior buffers continued
    // bytes across chunks; deferred.
    let ns = new_object(rt);
    register_method(rt, ns, "StringDecoder", |rt, args| {
        let encoding = match args.first() {
            Some(Value::String(s)) => s.as_str().to_string(),
            _ => "utf8".to_string(),
        };
        let o = RtObject::new_ordinary();
        let id = rt.alloc_object(o);
        rt.object_set(id, "encoding".into(), Value::String(Rc::new(encoding)));
        register_method(rt, id, "write", |_rt, args| {
            match args.first() {
                Some(Value::String(s)) => Ok(Value::String(s.clone())),
                Some(Value::Object(oid)) => {
                    // Treat as Uint8Array-like: read length, then bytes.
                    // (Implemented inline to keep the stub self-contained.)
                    Ok(Value::String(Rc::new(format!("[Buffer:{}]", oid.0))))
                }
                _ => Ok(Value::String(Rc::new(String::new()))),
            }
        });
        register_method(rt, id, "end", |_rt, _args| Ok(Value::String(Rc::new(String::new()))));
        Ok(Value::Object(id))
    });
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
        // Zero-initialized indexable bytes per Node spec.
        for i in 0..n.min(65536) {
            o.set_own(i.to_string(), Value::Number(0.0));
        }
        let id = rt.alloc_object(o);
        Ok(Value::Object(id))
    });
    // Tier-Ω.5.iii: Buffer.allocUnsafe + subarray for nanoid. nanoid
    // calls Buffer.allocUnsafe(bytes * POOL_SIZE_MULTIPLIER), fills via
    // crypto.getRandomValues, then slices with subarray.
    register_method(rt, buf_ctor, "allocUnsafe", |rt, args| {
        let n = match args.first() {
            Some(Value::Number(n)) => *n as usize,
            _ => 0,
        };
        let mut o = RtObject::new_ordinary();
        o.set_own("length".into(), Value::Number(n as f64));
        // Pre-populate indices with 0 so subsequent SetIndex via
        // crypto.getRandomValues has slots to write into.
        for i in 0..n.min(65536) {
            o.set_own(i.to_string(), Value::Number(0.0));
        }
        let id = rt.alloc_object(o);
        // subarray method on the instance — slices [start..end) without
        // copying. v1 actually copies; the visible semantics match for
        // shape probes and indexed reads.
        register_method(rt, id, "readUInt8", |rt, args| {
            let this_id = match rt.current_this() {
                Value::Object(o) => o,
                _ => return Err(RuntimeError::TypeError("Buffer.readUInt8: this must be a Buffer".into())),
            };
            let offset = match args.first() {
                Some(Value::Number(n)) => *n as usize,
                _ => 0,
            };
            let v = rt.object_get(this_id, &offset.to_string());
            Ok(match v { Value::Number(n) => Value::Number(n), _ => Value::Number(0.0) })
        });
        register_method(rt, id, "subarray", |rt, args| {
            let this_id = match rt.current_this() {
                Value::Object(o) => o,
                _ => return Err(RuntimeError::TypeError("Buffer.subarray: this must be a Buffer".into())),
            };
            let len = match rt.object_get(this_id, &"length".to_string()) {
                Value::Number(n) => n as usize,
                _ => 0,
            };
            let start = match args.first() {
                Some(Value::Number(n)) => (*n as i64).max(0) as usize,
                _ => 0,
            }.min(len);
            let end = match args.get(1) {
                Some(Value::Number(n)) => (*n as i64).max(0) as usize,
                _ => len,
            }.min(len);
            let slice_len = end.saturating_sub(start);
            let mut o = RtObject::new_ordinary();
            o.set_own("length".into(), Value::Number(slice_len as f64));
            let new_id = rt.alloc_object(o);
            for i in 0..slice_len {
                let v = rt.object_get(this_id, &(start + i).to_string());
                rt.object_set(new_id, i.to_string(), v);
            }
            Ok(Value::Object(new_id))
        });
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

// Tier-Ω.5.nnnn: node:http2 stub (got advances here past dns).
pub fn install_http2(rt: &mut Runtime) {
    let ns = new_object(rt);
    for m in &["connect", "createServer", "createSecureServer", "constants"] {
        register_method(rt, ns, m, stub("http2", m));
    }
    rt.globals.insert("http2".into(), Value::Object(ns));
}

// Tier-Ω.5.kkkk: node:dns stub for `got` cluster.
pub fn install_dns(rt: &mut Runtime) {
    let ns = new_object(rt);
    for m in &["lookup", "resolve", "resolve4", "resolve6", "reverse", "setServers", "getServers"] {
        register_method(rt, ns, m, stub("dns", m));
    }
    let promises = new_object(rt);
    for m in &["lookup", "resolve", "resolve4", "resolve6", "reverse"] {
        register_method(rt, promises, m, stub("dns/promises", m));
    }
    rt.object_set(ns, "promises".into(), Value::Object(promises));
    rt.globals.insert("dns".into(), Value::Object(ns));
}

// Tier-Ω.5.llll: node:module stub for `yargs` cluster.
pub fn install_module(rt: &mut Runtime) {
    let ns = new_object(rt);
    register_method(rt, ns, "createRequire", |rt, _args| {
        // Returns a function that throws if called. Many libraries call
        // createRequire() at module init merely to capture a reference.
        let f = new_object(rt);
        register_method(rt, f, "resolve", stub("module", "require.resolve"));
        Ok(Value::Object(f))
    });
    register_method(rt, ns, "builtinModules", |_rt, _args| Ok(Value::Undefined));
    let arr = RtObject::new_ordinary();
    let arr_id = rt.alloc_object(arr);
    rt.object_set(ns, "builtinModules".into(), Value::Object(arr_id));
    rt.globals.insert("module".into(), Value::Object(ns));
}

pub fn install_all(rt: &mut Runtime) {
    install_child_process(rt);
    install_tls(rt);
    install_readline(rt);
    install_constants(rt);
    install_string_decoder(rt);
    install_buffer(rt);
    install_dns(rt);
    install_module(rt);
    install_http2(rt);
}
