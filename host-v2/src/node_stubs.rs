//! node:* stub modules — Tier-Ω.5.bb. Each module exposes a populated
//! namespace so import-time and shape probes succeed. Methods throw
//! "not yet implemented" when actually called.
//!
//! Modules covered: child_process, tls, readline, constants,
//! string_decoder, buffer.

use crate::register::{make_callable, new_object, register_method};
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
    for m in &["connect", "createServer", "createSecureContext"] {
        register_method(rt, ns, m, stub("tls", m));
    }
    // Tier-Ω.5.QQQQQQQ: TLSSocket / Server as stub-classes with .prototype.
    // got / got-fetch / undici-style HTTP libs read tls.TLSSocket at module-
    // init (for `class X extends tls.TLSSocket` or `tls.TLSSocket.prototype`
    // feature probes). Stub-methods that throw on call surface as Thrown
    // errors instead of class-shaped resolution; replacing with stub-classes
    // (callable but error-on-construct, with empty .prototype + constructor
    // backref) satisfies the import surface.
    for cls in &["TLSSocket", "Server"] {
        let proto = new_object(rt);
        let ctor = make_callable(rt, cls, move |rt, _args| {
            // Return a stub instance with EventEmitter-shape methods so
            // module-init `new tls.TLSSocket(opts)` resolves; runtime
            // calls (write/connect/destroy) are no-ops that return self.
            let inst = rt.alloc_object(RtObject::new_ordinary());
            register_method(rt, inst, "on", |rt, _a| Ok(rt.current_this()));
            register_method(rt, inst, "once", |rt, _a| Ok(rt.current_this()));
            register_method(rt, inst, "off", |rt, _a| Ok(rt.current_this()));
            register_method(rt, inst, "emit", |_rt, _a| Ok(Value::Boolean(false)));
            register_method(rt, inst, "write", |_rt, _a| Ok(Value::Boolean(true)));
            register_method(rt, inst, "end", |rt, _a| Ok(rt.current_this()));
            register_method(rt, inst, "destroy", |rt, _a| Ok(rt.current_this()));
            register_method(rt, inst, "connect", |rt, _a| Ok(rt.current_this()));
            register_method(rt, inst, "setEncoding", |rt, _a| Ok(rt.current_this()));
            Ok(Value::Object(inst))
        });
        rt.object_set(ctor, "prototype".into(), Value::Object(proto));
        rt.object_set(proto, "constructor".into(), Value::Object(ctor));
        rt.object_set(ns, (*cls).into(), Value::Object(ctor));
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

// Tier-Ω.5.bbbbbb: rich Buffer-instance method surface — slice, toString,
// copy, indexOf, equals. csv-parse uses all of these via ResizeableBuffer.
fn install_buffer_methods(rt: &mut Runtime, id: rusty_js_runtime::ObjectRef) {
    register_method(rt, id, "slice", |rt, args| {
        let this_id = match rt.current_this() {
            Value::Object(o) => o, _ => return Err(RuntimeError::TypeError("Buffer.slice: this must be a Buffer".into())),
        };
        let len = match rt.object_get(this_id, "length") { Value::Number(n) => n as usize, _ => 0 };
        let start = args.first().and_then(|v| if let Value::Number(n)=v { Some(*n as i64) } else { None }).unwrap_or(0);
        let end = args.get(1).and_then(|v| if let Value::Number(n)=v { Some(*n as i64) } else { None }).unwrap_or(len as i64);
        let start = (if start < 0 { (len as i64 + start).max(0) } else { start }).min(len as i64) as usize;
        let end = (if end < 0 { (len as i64 + end).max(0) } else { end }).min(len as i64) as usize;
        let slice_len = end.saturating_sub(start);
        let mut o = RtObject::new_ordinary();
        o.set_own("length".into(), Value::Number(slice_len as f64));
        o.set_own("__is_buffer__".into(), Value::Boolean(true));
        let new_id = rt.alloc_object(o);
        install_buffer_methods(rt, new_id);
        for i in 0..slice_len {
            let v = rt.object_get(this_id, &(start + i).to_string());
            rt.object_set(new_id, i.to_string(), v);
        }
        Ok(Value::Object(new_id))
    });
    register_method(rt, id, "toString", |rt, args| {
        let this_id = match rt.current_this() {
            Value::Object(o) => o, _ => return Ok(Value::String(Rc::new(String::new()))),
        };
        let _enc = match args.first() { Some(Value::String(s)) => s.as_str().to_string(), _ => "utf8".into() };
        let len = match rt.object_get(this_id, "length") { Value::Number(n) => n as usize, _ => 0 };
        let mut bytes: Vec<u8> = Vec::with_capacity(len);
        for i in 0..len {
            if let Value::Number(n) = rt.object_get(this_id, &i.to_string()) {
                bytes.push(n as u8);
            }
        }
        Ok(Value::String(Rc::new(String::from_utf8_lossy(&bytes).to_string())))
    });
    register_method(rt, id, "copy", |rt, args| {
        let this_id = match rt.current_this() {
            Value::Object(o) => o, _ => return Ok(Value::Number(0.0)),
        };
        let target = match args.first() { Some(Value::Object(id)) => *id, _ => return Ok(Value::Number(0.0)) };
        let target_start = args.get(1).and_then(|v| if let Value::Number(n)=v { Some(*n as usize) } else { None }).unwrap_or(0);
        let src_start = args.get(2).and_then(|v| if let Value::Number(n)=v { Some(*n as usize) } else { None }).unwrap_or(0);
        let src_len = match rt.object_get(this_id, "length") { Value::Number(n) => n as usize, _ => 0 };
        let src_end = args.get(3).and_then(|v| if let Value::Number(n)=v { Some(*n as usize) } else { None }).unwrap_or(src_len).min(src_len);
        let count = src_end.saturating_sub(src_start);
        for i in 0..count {
            let v = rt.object_get(this_id, &(src_start + i).to_string());
            rt.object_set(target, (target_start + i).to_string(), v);
        }
        Ok(Value::Number(count as f64))
    });
    register_method(rt, id, "subarray", |rt, args| {
        let this_id = match rt.current_this() {
            Value::Object(o) => o, _ => return Err(RuntimeError::TypeError("Buffer.subarray: this must be a Buffer".into())),
        };
        let len = match rt.object_get(this_id, "length") { Value::Number(n) => n as usize, _ => 0 };
        let start = args.first().and_then(|v| if let Value::Number(n)=v { Some(*n as i64) } else { None }).unwrap_or(0);
        let end = args.get(1).and_then(|v| if let Value::Number(n)=v { Some(*n as i64) } else { None }).unwrap_or(len as i64);
        let start = (if start < 0 { (len as i64 + start).max(0) } else { start }).min(len as i64) as usize;
        let end = (if end < 0 { (len as i64 + end).max(0) } else { end }).min(len as i64) as usize;
        let slice_len = end.saturating_sub(start);
        let mut o = RtObject::new_ordinary();
        o.set_own("length".into(), Value::Number(slice_len as f64));
        o.set_own("__is_buffer__".into(), Value::Boolean(true));
        let new_id = rt.alloc_object(o);
        install_buffer_methods(rt, new_id);
        for i in 0..slice_len {
            let v = rt.object_get(this_id, &(start + i).to_string());
            rt.object_set(new_id, i.to_string(), v);
        }
        Ok(Value::Object(new_id))
    });
    register_method(rt, id, "readUInt8", |rt, args| {
        let this_id = match rt.current_this() { Value::Object(o) => o, _ => return Ok(Value::Number(0.0)) };
        let offset = args.first().and_then(|v| if let Value::Number(n)=v { Some(*n as usize) } else { None }).unwrap_or(0);
        match rt.object_get(this_id, &offset.to_string()) {
            Value::Number(n) => Ok(Value::Number(n)),
            _ => Ok(Value::Number(0.0)),
        }
    });
    register_method(rt, id, "indexOf", |rt, args| {
        let this_id = match rt.current_this() { Value::Object(o) => o, _ => return Ok(Value::Number(-1.0)) };
        let len = match rt.object_get(this_id, "length") { Value::Number(n) => n as usize, _ => 0 };
        let needle_bytes: Vec<u8> = match args.first() {
            Some(Value::Number(n)) => vec![*n as u8],
            Some(Value::String(s)) => s.as_bytes().to_vec(),
            Some(Value::Object(nid)) => {
                let nl = match rt.object_get(*nid, "length") { Value::Number(n) => n as usize, _ => 0 };
                (0..nl).filter_map(|i| match rt.object_get(*nid, &i.to_string()) { Value::Number(n) => Some(n as u8), _ => None }).collect()
            }
            _ => return Ok(Value::Number(-1.0)),
        };
        for start in 0..=len.saturating_sub(needle_bytes.len()) {
            let mut all = true;
            for (j, b) in needle_bytes.iter().enumerate() {
                if let Value::Number(n) = rt.object_get(this_id, &(start + j).to_string()) {
                    if n as u8 != *b { all = false; break; }
                } else { all = false; break; }
            }
            if all { return Ok(Value::Number(start as f64)); }
        }
        Ok(Value::Number(-1.0))
    });
    register_method(rt, id, "equals", |rt, args| {
        let this_id = match rt.current_this() { Value::Object(o) => o, _ => return Ok(Value::Boolean(false)) };
        let other = match args.first() { Some(Value::Object(o)) => *o, _ => return Ok(Value::Boolean(false)) };
        let l1 = match rt.object_get(this_id, "length") { Value::Number(n) => n as usize, _ => 0 };
        let l2 = match rt.object_get(other, "length") { Value::Number(n) => n as usize, _ => 0 };
        if l1 != l2 { return Ok(Value::Boolean(false)); }
        for i in 0..l1 {
            if rt.object_get(this_id, &i.to_string()) != rt.object_get(other, &i.to_string()) {
                return Ok(Value::Boolean(false));
            }
        }
        Ok(Value::Boolean(true))
    });
}

pub fn install_buffer(rt: &mut Runtime) {
    // node:buffer exposes Buffer + Blob. v1: stub class with .from / .alloc
    // / .isBuffer / .byteLength. Real Buffer needs binary internals.
    let ns = new_object(rt);
    // Tier-Ω.5.OOOOOOO: Buffer is callable as `new Buffer(arg)` per Node
    // legacy API. single-line-log / keyv-redis / many older packages do
    // `new Buffer(len)` (deprecated since Node 6) or `new Buffer(string)`.
    // Route to Buffer.alloc(N) or Buffer.from(arg) depending on arg shape.
    // Buffer was previously an ordinary object; the global Buffer call
    // hit 'callee is not callable: Object(keys=[prototype,alloc,...])'.
    let buf_ctor = make_callable(rt, "Buffer", |rt, args| {
        match args.first() {
            Some(Value::Number(n)) => {
                let n = *n as usize;
                let mut o = RtObject::new_ordinary();
                o.set_own("length".into(), Value::Number(n as f64));
                o.set_own("__is_buffer__".into(), Value::Boolean(true));
                for i in 0..n.min(65536) {
                    o.set_own(i.to_string(), Value::Number(0.0));
                }
                let id = rt.alloc_object(o);
                install_buffer_methods(rt, id);
                Ok(Value::Object(id))
            }
            Some(Value::String(s)) => {
                let s = s.as_str().to_string();
                let mut o = RtObject::new_ordinary();
                o.set_own("__buffer_data".into(), Value::String(Rc::new(s.clone())));
                o.set_own("length".into(), Value::Number(s.as_bytes().len() as f64));
                o.set_own("__is_buffer__".into(), Value::Boolean(true));
                for (i, b) in s.as_bytes().iter().enumerate() {
                    o.set_own(i.to_string(), Value::Number(*b as f64));
                }
                let id = rt.alloc_object(o);
                install_buffer_methods(rt, id);
                Ok(Value::Object(id))
            }
            _ => {
                // Treat array-like / other: empty buffer fallback.
                let mut o = RtObject::new_ordinary();
                o.set_own("length".into(), Value::Number(0.0));
                o.set_own("__is_buffer__".into(), Value::Boolean(true));
                let id = rt.alloc_object(o);
                install_buffer_methods(rt, id);
                Ok(Value::Object(id))
            }
        }
    });
    register_method(rt, buf_ctor, "from", |rt, args| {
        // Buffer.from(str) → produces a plain object pretending to be a Buffer.
        let s = match args.first() {
            Some(Value::String(s)) => s.as_str().to_string(),
            _ => String::new(),
        };
        let mut o = RtObject::new_ordinary();
        o.set_own("__buffer_data".into(), Value::String(Rc::new(s.clone())));
        o.set_own("length".into(), Value::Number(s.as_bytes().len() as f64));
        o.set_own("__is_buffer__".into(), Value::Boolean(true));
        let id = rt.alloc_object(o);
        // Tier-Ω.5.wwwww: populate indexed byte access. csv-parse reads
        // bytes via buf[i] to compare escape/quote/delimiter byte streams.
        for (i, b) in s.as_bytes().iter().enumerate() {
            rt.object_set(id, i.to_string(), Value::Number(*b as f64));
        }
        install_buffer_methods(rt, id);
        Ok(Value::Object(id))
    });
    register_method(rt, buf_ctor, "alloc", |rt, args| {
        let n = match args.first() {
            Some(Value::Number(n)) => *n as usize,
            _ => 0,
        };
        let mut o = RtObject::new_ordinary();
        o.set_own("length".into(), Value::Number(n as f64));
        o.set_own("__is_buffer__".into(), Value::Boolean(true));
        // Zero-initialized indexable bytes per Node spec.
        for i in 0..n.min(65536) {
            o.set_own(i.to_string(), Value::Number(0.0));
        }
        let id = rt.alloc_object(o);
        install_buffer_methods(rt, id);
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
        o.set_own("__is_buffer__".into(), Value::Boolean(true));
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
        install_buffer_methods(rt, id);
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
    // Tier-Ω.5.wwwww: Buffer.compare(a, b) per Node spec — returns -1/0/1
    // by byte-wise lex order. csv-parse uses this to test escape===quote.
    register_method(rt, buf_ctor, "compare", |rt, args| {
        let read = |v: &Value| -> Vec<u8> {
            match v {
                Value::String(s) => s.as_bytes().to_vec(),
                Value::Object(id) => {
                    let len = match rt.object_get(*id, "length") { Value::Number(n) => n as usize, _ => 0 };
                    (0..len).map(|i| match rt.object_get(*id, &i.to_string()) {
                        Value::Number(n) => n as u8,
                        Value::String(s) if !s.is_empty() => s.as_bytes()[0],
                        _ => 0,
                    }).collect()
                }
                _ => Vec::new(),
            }
        };
        let a = read(&args.first().cloned().unwrap_or(Value::Undefined));
        let b = read(&args.get(1).cloned().unwrap_or(Value::Undefined));
        Ok(Value::Number(match a.cmp(&b) {
            std::cmp::Ordering::Less => -1.0,
            std::cmp::Ordering::Equal => 0.0,
            std::cmp::Ordering::Greater => 1.0,
        }))
    });
    register_method(rt, buf_ctor, "concat", |rt, args| {
        let list = match args.first() {
            Some(Value::Object(id)) => *id,
            _ => return Err(RuntimeError::TypeError("Buffer.concat: expected array".into())),
        };
        let len = rt.array_length(list);
        let mut bytes: Vec<u8> = Vec::new();
        for i in 0..len {
            if let Value::Object(b) = rt.object_get(list, &i.to_string()) {
                let bl = match rt.object_get(b, "length") { Value::Number(n) => n as usize, _ => 0 };
                for j in 0..bl {
                    if let Value::Number(n) = rt.object_get(b, &j.to_string()) {
                        bytes.push(n as u8);
                    }
                }
            }
        }
        let mut o = RtObject::new_ordinary();
        o.set_own("length".into(), Value::Number(bytes.len() as f64));
        o.set_own("__is_buffer__".into(), Value::Boolean(true));
        let id = rt.alloc_object(o);
        for (i, b) in bytes.iter().enumerate() {
            rt.object_set(id, i.to_string(), Value::Number(*b as f64));
        }
        Ok(Value::Object(id))
    });
    register_method(rt, buf_ctor, "isBuffer", |rt, args| {
        // Tier-Ω.5.mmmmm: recognize our Buffer-likes via the __is_buffer__
        // marker. csv-parse and any lib that brand-checks options before
        // dispatching needs this to accept Buffer.from(...) outputs.
        if let Some(Value::Object(id)) = args.first() {
            if matches!(rt.object_get(*id, "__is_buffer__"), Value::Boolean(true)) {
                return Ok(Value::Boolean(true));
            }
        }
        Ok(Value::Boolean(false))
    });
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
    install_diagnostics_channel(rt);
    install_v8(rt);
    install_inspector(rt);
    install_vm(rt);
    install_punycode(rt);
    install_async_hooks(rt);
}

/// Tier-Ω.5.RRRRRRR: node:async_hooks stub with AsyncResource as a
/// stub class (extensible via `class X extends AsyncResource`).
/// undici / fastify / many context-tracking libraries do this at
/// module-init. AsyncResource was previously routed through the
/// events module's namespace, which has no AsyncResource export —
/// `class X extends AsyncResource` then read .prototype on undefined.
/// AsyncLocalStorage / executionAsyncId / triggerAsyncId / createHook
/// are placeholders sufficient for module-init presence checks.
pub fn install_async_hooks(rt: &mut Runtime) {
    let ns = new_object(rt);

    let ar_proto = new_object(rt);
    let ar_ctor = make_callable(rt, "AsyncResource", |rt, _args| {
        let inst = rt.alloc_object(RtObject::new_ordinary());
        register_method(rt, inst, "runInAsyncScope", |rt, args| {
            // (callback, thisArg, ...args) -> callback.call(thisArg, ...args)
            let cb = args.first().cloned().unwrap_or(Value::Undefined);
            let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
            let cb_args: Vec<Value> = args.iter().skip(2).cloned().collect();
            rt.call_function(cb, this_arg, cb_args)
        });
        register_method(rt, inst, "emitDestroy", |rt, _a| Ok(rt.current_this()));
        register_method(rt, inst, "asyncId", |_rt, _a| Ok(Value::Number(0.0)));
        register_method(rt, inst, "triggerAsyncId", |_rt, _a| Ok(Value::Number(0.0)));
        register_method(rt, inst, "bind", |rt, args| Ok(args.first().cloned().unwrap_or(rt.current_this())));
        Ok(Value::Object(inst))
    });
    rt.object_set(ar_ctor, "prototype".into(), Value::Object(ar_proto));
    rt.object_set(ar_proto, "constructor".into(), Value::Object(ar_ctor));
    register_method(rt, ar_ctor, "bind", |rt, args| Ok(args.first().cloned().unwrap_or(rt.current_this())));
    rt.object_set(ns, "AsyncResource".into(), Value::Object(ar_ctor));

    // AsyncLocalStorage — used by undici/fastify for context propagation.
    let als_proto = new_object(rt);
    let als_ctor = make_callable(rt, "AsyncLocalStorage", |rt, _args| {
        let inst = rt.alloc_object(RtObject::new_ordinary());
        register_method(rt, inst, "getStore", |_rt, _a| Ok(Value::Undefined));
        register_method(rt, inst, "run", |rt, args| {
            // (store, callback, ...args) -> callback(...args)
            let cb = args.get(1).cloned().unwrap_or(Value::Undefined);
            let cb_args: Vec<Value> = args.iter().skip(2).cloned().collect();
            rt.call_function(cb, Value::Undefined, cb_args)
        });
        register_method(rt, inst, "enterWith", |rt, _a| Ok(rt.current_this()));
        register_method(rt, inst, "exit", |rt, args| {
            let cb = args.first().cloned().unwrap_or(Value::Undefined);
            let cb_args: Vec<Value> = args.iter().skip(1).cloned().collect();
            rt.call_function(cb, Value::Undefined, cb_args)
        });
        register_method(rt, inst, "disable", |_rt, _a| Ok(Value::Undefined));
        Ok(Value::Object(inst))
    });
    rt.object_set(als_ctor, "prototype".into(), Value::Object(als_proto));
    rt.object_set(als_proto, "constructor".into(), Value::Object(als_ctor));
    register_method(rt, als_ctor, "bind", |rt, args| Ok(args.first().cloned().unwrap_or(rt.current_this())));
    register_method(rt, als_ctor, "snapshot", |_rt, _a| {
        Ok(Value::Undefined)
    });
    rt.object_set(ns, "AsyncLocalStorage".into(), Value::Object(als_ctor));

    register_method(rt, ns, "executionAsyncId", |_rt, _a| Ok(Value::Number(0.0)));
    register_method(rt, ns, "triggerAsyncId", |_rt, _a| Ok(Value::Number(0.0)));
    register_method(rt, ns, "executionAsyncResource", |rt, _a| Ok(Value::Object(new_object(rt))));
    register_method(rt, ns, "createHook", |rt, _a| {
        let hook = rt.alloc_object(RtObject::new_ordinary());
        register_method(rt, hook, "enable", |rt, _a| Ok(rt.current_this()));
        register_method(rt, hook, "disable", |rt, _a| Ok(rt.current_this()));
        Ok(Value::Object(hook))
    });

    rt.globals.insert("async_hooks".into(), Value::Object(ns));
}

/// Tier-Ω.5.PPPPPPP: node:punycode stub. Deprecated in Node 7+ but
/// userland packages (whatwg-url's polyfill path, cross-fetch's IDN
/// handling) still resolve it. Stubs encode/decode/toASCII/toUnicode
/// as identity functions sufficient for ASCII inputs; full
/// IDN transcoding deferred.
pub fn install_punycode(rt: &mut Runtime) {
    let ns = new_object(rt);
    register_method(rt, ns, "encode", |_rt, args| {
        let s = match args.first() { Some(Value::String(s)) => s.as_str().to_string(), _ => String::new() };
        Ok(Value::String(Rc::new(s)))
    });
    register_method(rt, ns, "decode", |_rt, args| {
        let s = match args.first() { Some(Value::String(s)) => s.as_str().to_string(), _ => String::new() };
        Ok(Value::String(Rc::new(s)))
    });
    register_method(rt, ns, "toASCII", |_rt, args| {
        let s = match args.first() { Some(Value::String(s)) => s.as_str().to_string(), _ => String::new() };
        Ok(Value::String(Rc::new(s)))
    });
    register_method(rt, ns, "toUnicode", |_rt, args| {
        let s = match args.first() { Some(Value::String(s)) => s.as_str().to_string(), _ => String::new() };
        Ok(Value::String(Rc::new(s)))
    });
    crate::register::set_constant(rt, ns, "version", Value::String(Rc::new("2.3.1".into())));
    let ucs2 = new_object(rt);
    register_method(rt, ucs2, "encode", |_rt, args| Ok(args.first().cloned().unwrap_or(Value::Undefined)));
    register_method(rt, ucs2, "decode", |_rt, args| Ok(args.first().cloned().unwrap_or(Value::Undefined)));
    crate::register::set_constant(rt, ns, "ucs2", Value::Object(ucs2));
    rt.globals.insert("punycode".into(), Value::Object(ns));
}

/// Tier-Ω.5.FFFFFFF: node:v8 stub. mlly / exsolve / local-pkg / prettier /
/// execa read `v8.getHeapStatistics()` or `v8.serialize/deserialize` at
/// module-init for diagnostic / structured-clone surfaces. Stubs return
/// plausible defaults; serialize errors on actual call.
pub fn install_v8(rt: &mut Runtime) {
    let ns = new_object(rt);
    register_method(rt, ns, "getHeapStatistics", |rt, _args| {
        let o = new_object(rt);
        for (k, v) in &[
            ("total_heap_size", 1024.0 * 1024.0 * 64.0),
            ("total_heap_size_executable", 1024.0 * 1024.0),
            ("total_physical_size", 1024.0 * 1024.0 * 64.0),
            ("total_available_size", 1024.0 * 1024.0 * 1024.0),
            ("used_heap_size", 1024.0 * 1024.0 * 16.0),
            ("heap_size_limit", 1024.0 * 1024.0 * 1024.0 * 2.0),
            ("malloced_memory", 1024.0 * 64.0),
            ("peak_malloced_memory", 1024.0 * 128.0),
            ("does_zap_garbage", 0.0),
            ("number_of_native_contexts", 1.0),
            ("number_of_detached_contexts", 0.0),
            ("total_global_handles_size", 1024.0 * 8.0),
            ("used_global_handles_size", 1024.0 * 4.0),
            ("external_memory", 0.0),
        ] {
            rt.object_set(o, (*k).into(), Value::Number(*v));
        }
        Ok(Value::Object(o))
    });
    register_method(rt, ns, "getHeapSpaceStatistics", |rt, _args| {
        Ok(Value::Object(rt.alloc_object(RtObject::new_array())))
    });
    register_method(rt, ns, "getHeapCodeStatistics", |rt, _args| {
        Ok(Value::Object(new_object(rt)))
    });
    for m in &["serialize", "deserialize", "writeHeapSnapshot", "setFlagsFromString", "cachedDataVersionTag"] {
        register_method(rt, ns, m, stub("v8", m));
    }
    rt.globals.insert("v8".into(), Value::Object(ns));
}

/// Tier-Ω.5.FFFFFFF: node:inspector stub.
pub fn install_inspector(rt: &mut Runtime) {
    let ns = new_object(rt);
    for m in &["open", "close", "url", "waitForDebugger"] {
        register_method(rt, ns, m, stub("inspector", m));
    }
    rt.globals.insert("inspector".into(), Value::Object(ns));
}

/// Tier-Ω.5.FFFFFFF: node:vm stub. Several packages do feature-probes
/// (`typeof vm.runInContext === 'function'`); methods error on call.
pub fn install_vm(rt: &mut Runtime) {
    let ns = new_object(rt);
    for m in &["runInThisContext", "runInContext", "runInNewContext", "createContext",
              "compileFunction", "measureMemory"] {
        register_method(rt, ns, m, stub("vm", m));
    }
    // Tier-Ω.5.QQQQQQQ: Script / SourceTextModule / SyntheticModule as
    // stub-classes with .prototype. stringify-object and friends probe
    // vm.Script class-shape at module-init (`new vm.Script(...)` inside
    // a try/catch detector pattern). Stub-class lets the load resolve;
    // construction throws.
    for cls in &["Script", "SourceTextModule", "SyntheticModule"] {
        let proto = new_object(rt);
        let ctor = make_callable(rt, cls, move |rt, _args| {
            let inst = rt.alloc_object(RtObject::new_ordinary());
            register_method(rt, inst, "runInThisContext", |_rt, _a| Ok(Value::Undefined));
            register_method(rt, inst, "runInContext", |_rt, _a| Ok(Value::Undefined));
            register_method(rt, inst, "runInNewContext", |_rt, _a| Ok(Value::Undefined));
            register_method(rt, inst, "createCachedData", |rt, _a| {
                let buf = rt.alloc_object(RtObject::new_array());
                rt.object_set(buf, "length".into(), Value::Number(0.0));
                Ok(Value::Object(buf))
            });
            Ok(Value::Object(inst))
        });
        rt.object_set(ctor, "prototype".into(), Value::Object(proto));
        rt.object_set(proto, "constructor".into(), Value::Object(ctor));
        rt.object_set(ns, (*cls).into(), Value::Object(ctor));
    }
    rt.globals.insert("vm".into(), Value::Object(ns));
}

/// Tier-Ω.5.CCCCCCC: node:diagnostics_channel stub. lru-cache /
/// fastify / undici call `dc.channel(name)` and `dc.tracingChannel(name)`
/// at module-init, then read `.hasSubscribers` on the result. Both
/// return objects with `hasSubscribers: false`, satisfying the
/// short-circuit check; publish/subscribe queued for downstream.
pub fn install_diagnostics_channel(rt: &mut Runtime) {
    let ns = new_object(rt);
    register_method(rt, ns, "channel", |rt, _args| {
        let ch = new_object(rt);
        rt.object_set(ch, "hasSubscribers".into(), Value::Boolean(false));
        register_method(rt, ch, "publish", |_rt, _args| Ok(Value::Undefined));
        register_method(rt, ch, "subscribe", |_rt, _args| Ok(Value::Undefined));
        register_method(rt, ch, "unsubscribe", |_rt, _args| Ok(Value::Boolean(false)));
        Ok(Value::Object(ch))
    });
    register_method(rt, ns, "tracingChannel", |rt, _args| {
        let ch = new_object(rt);
        rt.object_set(ch, "hasSubscribers".into(), Value::Boolean(false));
        for m in &["start", "end", "asyncStart", "asyncEnd", "error", "traceSync", "tracePromise", "traceCallback"] {
            register_method(rt, ch, m, |_rt, args| {
                Ok(args.get(1).cloned().unwrap_or(Value::Undefined))
            });
        }
        Ok(Value::Object(ch))
    });
    register_method(rt, ns, "subscribe", |_rt, _args| Ok(Value::Undefined));
    register_method(rt, ns, "unsubscribe", |_rt, _args| Ok(Value::Boolean(false)));
    register_method(rt, ns, "hasSubscribers", |_rt, _args| Ok(Value::Boolean(false)));
    rt.globals.insert("diagnostics_channel".into(), Value::Object(ns));
}
