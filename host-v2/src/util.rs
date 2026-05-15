//! node:util intrinsic — Tier-Ω.5.s.
//!
//! Mixed: `inspect` / `format` / `inherits` / `deprecate` / `types` are
//! actually implemented enough for pathe + common callers; `promisify`
//! and `callbackify` are stubs.

use crate::register::{new_object, register_method, set_constant};
use rusty_js_runtime::abstract_ops;
use rusty_js_runtime::value::{InternalKind, Object};
use rusty_js_runtime::{Runtime, RuntimeError, Value};
use std::rc::Rc;

pub fn install(rt: &mut Runtime) {
    let util = new_object(rt);

    // inspect(v) → JSON.stringify(v, null, 2). Close enough for v1.
    register_method(rt, util, "inspect", |rt, args| {
        let v = args.first().cloned().unwrap_or(Value::Undefined);
        let s = json_stringify_via_intrinsic(rt, &v)?;
        Ok(Value::String(Rc::new(s)))
    });

    // format(fmt, ...args) → printf-style substitution with %s/%d/%j.
    register_method(rt, util, "format", |rt, args| {
        if args.is_empty() {
            return Ok(Value::String(Rc::new(String::new())));
        }
        let fmt = match &args[0] {
            Value::String(s) => s.as_str().to_string(),
            other => abstract_ops::to_string(other).as_str().to_string(),
        };
        let mut out = String::new();
        let mut chars = fmt.chars().peekable();
        let mut arg_idx = 1usize;
        while let Some(c) = chars.next() {
            if c == '%' {
                match chars.next() {
                    Some('s') => {
                        let a = args.get(arg_idx).cloned().unwrap_or(Value::Undefined);
                        arg_idx += 1;
                        out.push_str(&abstract_ops::to_string(&a));
                    }
                    Some('d') | Some('i') => {
                        let a = args.get(arg_idx).cloned().unwrap_or(Value::Undefined);
                        arg_idx += 1;
                        let n = abstract_ops::to_number(&a);
                        if n.is_nan() {
                            out.push_str("NaN");
                        } else {
                            out.push_str(&(n.trunc() as i64).to_string());
                        }
                    }
                    Some('f') => {
                        let a = args.get(arg_idx).cloned().unwrap_or(Value::Undefined);
                        arg_idx += 1;
                        out.push_str(&abstract_ops::to_number(&a).to_string());
                    }
                    Some('j') => {
                        let a = args.get(arg_idx).cloned().unwrap_or(Value::Undefined);
                        arg_idx += 1;
                        let s = json_stringify_via_intrinsic(rt, &a)?;
                        out.push_str(&s);
                    }
                    Some('%') => out.push('%'),
                    Some(other) => {
                        out.push('%');
                        out.push(other);
                    }
                    None => out.push('%'),
                }
            } else {
                out.push(c);
            }
        }
        // Trailing args appended space-separated, per Node semantics.
        for i in arg_idx..args.len() {
            out.push(' ');
            out.push_str(&abstract_ops::to_string(&args[i]));
        }
        Ok(Value::String(Rc::new(out)))
    });

    // inherits(ctor, super_): ctor.super_ = super_;
    //   ctor.prototype = Object.create(super_.prototype, {constructor:{value:ctor}})
    register_method(rt, util, "inherits", |rt, args| {
        let ctor_id = match args.first() {
            Some(Value::Object(id)) => *id,
            _ => return Err(RuntimeError::TypeError(
                "util.inherits: ctor must be an object".into())),
        };
        let super_id = match args.get(1) {
            Some(Value::Object(id)) => *id,
            _ => return Err(RuntimeError::TypeError(
                "util.inherits: super must be an object".into())),
        };
        rt.object_set(ctor_id, "super_".into(), Value::Object(super_id));
        let super_proto = rt.object_get(super_id, "prototype");
        let new_proto = rt.alloc_object(Object::new_ordinary());
        if let Value::Object(sp) = super_proto {
            // Set [[Prototype]] of new_proto to super_proto.
            let _ = rt.obj_mut(new_proto);
            rt.obj_mut(new_proto).proto = Some(sp);
        }
        rt.object_set(new_proto, "constructor".into(), Value::Object(ctor_id));
        rt.object_set(ctor_id, "prototype".into(), Value::Object(new_proto));
        Ok(Value::Undefined)
    });

    // Tier-Ω.5.ddd: promisify / callbackify v1 stub. Real semantics
    // (callback-style → Promise-returning wrapper) need a full Promise
    // implementation in the runtime; for module-load-time evaluation,
    // returning the input function unchanged lets dependent libraries
    // (node-fetch, etc.) at least load and probe their namespaces.
    register_method(rt, util, "promisify", |_rt, args| {
        Ok(args.first().cloned().unwrap_or(Value::Undefined))
    });
    register_method(rt, util, "callbackify", |_rt, args| {
        Ok(args.first().cloned().unwrap_or(Value::Undefined))
    });
    register_method(rt, util, "deprecate", |_rt, args| {
        // Return fn unchanged; v1 drops the deprecation warning.
        Ok(args.first().cloned().unwrap_or(Value::Undefined))
    });

    // types subobject with InternalKind-based checks.
    let types = new_object(rt);
    register_method(rt, types, "isPromise", |rt, args| {
        Ok(Value::Boolean(matches!(args.first(),
            Some(Value::Object(id)) if matches!(rt.obj(*id).internal_kind, InternalKind::Promise(_)))))
    });
    register_method(rt, types, "isRegExp", |rt, args| {
        Ok(Value::Boolean(matches!(args.first(),
            Some(Value::Object(id)) if matches!(rt.obj(*id).internal_kind, InternalKind::RegExp(_)))))
    });
    register_method(rt, types, "isMap", |_rt, _args| Ok(Value::Boolean(false)));
    register_method(rt, types, "isSet", |_rt, _args| Ok(Value::Boolean(false)));
    register_method(rt, types, "isDate", |_rt, _args| Ok(Value::Boolean(false)));
    register_method(rt, types, "isNativeError", |rt, args| {
        Ok(Value::Boolean(matches!(args.first(),
            Some(Value::Object(id)) if matches!(rt.obj(*id).internal_kind, InternalKind::Error))))
    });
    register_method(rt, types, "isArrayBuffer", |_rt, _args| Ok(Value::Boolean(false)));
    register_method(rt, types, "isTypedArray", |_rt, _args| Ok(Value::Boolean(false)));
    set_constant(rt, util, "types", Value::Object(types));

    set_constant(rt, util, "default", Value::Object(util));
    rt.globals.insert("util".into(), Value::Object(util));
}

fn json_stringify_via_intrinsic(rt: &mut Runtime, v: &Value) -> Result<String, RuntimeError> {
    let json = rt
        .globals
        .get("JSON")
        .cloned()
        .ok_or_else(|| RuntimeError::TypeError("JSON intrinsic missing".into()))?;
    let json_id = match json {
        Value::Object(id) => id,
        _ => return Err(RuntimeError::TypeError("JSON is not an object".into())),
    };
    let stringify = rt.object_get(json_id, "stringify");
    let s = rt.call_function(stringify, Value::Object(json_id), vec![v.clone()])?;
    Ok(match s {
        Value::String(s) => s.as_str().to_string(),
        _ => String::new(),
    })
}
