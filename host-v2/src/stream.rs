//! node:stream intrinsic stub — Tier-Ω.5.s.
//!
//! Used by ndjson, split2, and any package that imports the constructor
//! classes at module load to subclass them. v1 exposes the class names
//! as stub-throw NativeFns so import succeeds and `typeof Stream.Readable
//! === "function"` is true; instantiation throws.

use crate::register::{new_object, register_method, set_constant};
use rusty_js_runtime::value::Object as RtObject;
use rusty_js_runtime::{Runtime, RuntimeError, Value};

// Tier-Ω.5.yyyy: minimal stream base-class constructors. Returns an object
// with empty _readableState / _writableState + no-op event emitter / write /
// pipe methods. Enough for `class X extends Transform {}` modules to load
// and run their constructors without throwing. ndjson, split2, through2,
// pump cluster aimed at module-load presence rather than real I/O.
fn make_stream_instance(rt: &mut Runtime, opts: Option<rusty_js_runtime::ObjectRef>, receiver: Option<rusty_js_runtime::ObjectRef>) -> rusty_js_runtime::ObjectRef {
    // Tier-Ω.5.yyyy: mutate the receiver when called as a super-ctor
    // (Transform.call(this) / super(opts) from subclass). Only allocate
    // fresh when invoked bare. Without this, subclass instances never
    // get _writableState et al because the fresh object is discarded.
    let id = if let Some(r) = receiver { r } else { rt.alloc_object(RtObject::new_ordinary()) };
    // State objects required by many libs for shape probes.
    let rs = rt.alloc_object(RtObject::new_ordinary());
    let ws = rt.alloc_object(RtObject::new_ordinary());
    rt.object_set(rs, "highWaterMark".into(), Value::Number(16384.0));
    let rs_buf = rt.alloc_object(RtObject::new_array());
    rt.object_set(rs, "buffer".into(), Value::Object(rs_buf));
    rt.object_set(rs, "length".into(), Value::Number(0.0));
    rt.object_set(rs, "ended".into(), Value::Boolean(false));
    rt.object_set(ws, "highWaterMark".into(), Value::Number(16384.0));
    let ws_buf = rt.alloc_object(RtObject::new_array());
    rt.object_set(ws, "buffer".into(), Value::Object(ws_buf));
    rt.object_set(ws, "length".into(), Value::Number(0.0));
    rt.object_set(ws, "ended".into(), Value::Boolean(false));
    rt.object_set(id, "_readableState".into(), Value::Object(rs));
    rt.object_set(id, "_writableState".into(), Value::Object(ws));
    if let Some(o_id) = opts {
        rt.object_set(id, "_options".into(), Value::Object(o_id));
    }
    register_method(rt, id, "on", |rt, _args| Ok(rt.current_this()));
    register_method(rt, id, "once", |rt, _args| Ok(rt.current_this()));
    register_method(rt, id, "off", |rt, _args| Ok(rt.current_this()));
    register_method(rt, id, "removeListener", |rt, _args| Ok(rt.current_this()));
    register_method(rt, id, "emit", |_rt, _args| Ok(Value::Boolean(false)));
    register_method(rt, id, "pipe", |_rt, args| Ok(args.first().cloned().unwrap_or(Value::Undefined)));
    register_method(rt, id, "unpipe", |rt, _args| Ok(rt.current_this()));
    register_method(rt, id, "write", |_rt, _args| Ok(Value::Boolean(true)));
    register_method(rt, id, "end", |rt, _args| Ok(rt.current_this()));
    register_method(rt, id, "destroy", |rt, _args| Ok(rt.current_this()));
    register_method(rt, id, "pause", |rt, _args| Ok(rt.current_this()));
    register_method(rt, id, "resume", |rt, _args| Ok(rt.current_this()));
    register_method(rt, id, "read", |_rt, _args| Ok(Value::Null));
    register_method(rt, id, "push", |_rt, _args| Ok(Value::Boolean(true)));
    register_method(rt, id, "setEncoding", |rt, _args| Ok(rt.current_this()));
    register_method(rt, id, "_transform", |_rt, _args| Ok(Value::Undefined));
    register_method(rt, id, "_read", |_rt, _args| Ok(Value::Undefined));
    register_method(rt, id, "_write", |_rt, _args| Ok(Value::Undefined));
    register_method(rt, id, "_flush", |_rt, _args| Ok(Value::Undefined));
    id
}

pub fn install(rt: &mut Runtime) {
    let stream = new_object(rt);

    for name in &["Readable", "Writable", "Transform", "Duplex", "PassThrough"] {
        let ctor = new_object(rt);
        // The constructor's prototype is a placeholder so
        // `class X extends Transform { }` finds something callable to read
        // `.prototype` from. Real prototype-chain wiring is deferred — for
        // module-load substrate, the ctor body returning an object is
        // enough because Op::New picks up the returned object.
        let proto = new_object(rt);
        rt.object_set(ctor, "prototype".into(), Value::Object(proto));
        rt.object_set(proto, "constructor".into(), Value::Object(ctor));
        // The ctor itself: accepting an optional options object.
        let nm = *name;
        register_method(rt, ctor, "__call__", move |_rt, _args| {
            Err(RuntimeError::TypeError(format!("internal: {} called via __call__", nm)))
        });
        // Real call dispatch path: assign a native callable property on
        // the ctor object via a trampoline kept in stream.<Name>.
        let _ = nm;
        rt.object_set(stream, name.to_string(), Value::Object(ctor));
    }
    // Replace each ctor with a real native function that, when called,
    // produces a stream instance. The register_method trick above only
    // wires methods — for `new Transform()` to construct, the global
    // needs to itself be callable. We re-register each as a native.
    for name in &["Readable", "Writable", "Transform", "Duplex", "PassThrough"] {
        let nm = name.to_string();
        register_method(rt, stream, name, move |rt, args| {
            let opts = match args.first() {
                Some(Value::Object(id)) => Some(*id),
                _ => None,
            };
            let _ = &nm;
            // When invoked as `Transform.call(this, opts)` from a subclass
            // ctor, this is the subclass instance and we should mutate it.
            // When invoked as `new Transform(opts)`, this is a fresh object
            // and we mutate that. Bare `Transform(opts)` (no this) →
            // allocate.
            let receiver = match rt.current_this() {
                Value::Object(id) => Some(id),
                _ => None,
            };
            Ok(Value::Object(make_stream_instance(rt, opts, receiver)))
        });
        // Tier-Ω.5.pppppp: re-attach .prototype on the re-registered ctor.
        // The earlier loop set ctor.prototype but register_method here
        // overwrote the slot with a fresh native fn lacking .prototype.
        // userspace packages like iconv-lite / cheerio call
        // `Object.create(Stream.Transform.prototype)` → must be object.
        let proto = new_object(rt);
        if let Value::Object(ctor) = rt.object_get(stream, name) {
            rt.object_set(ctor, "prototype".into(), Value::Object(proto));
            rt.object_set(proto, "constructor".into(), Value::Object(ctor));
        }
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
