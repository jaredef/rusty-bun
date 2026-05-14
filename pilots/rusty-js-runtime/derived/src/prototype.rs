//! Intrinsic prototype objects per Tier-Ω.5.a.
//!
//! Allocates and installs %Object.prototype%, %Array.prototype%,
//! %String.prototype%, %Function.prototype%, %Promise.prototype%, and
//! %Number.prototype%. Each prototype is an ordinary Object stashed on
//! `Runtime` so the alloc-time proto-wiring path (see
//! `Runtime::alloc_object`) finds it by InternalKind. Primitive method
//! dispatch (string.toUpperCase(), (5).toFixed(2)) routes through the
//! property-read paths in `interp.rs`'s GetProp handler, which look up
//! `string_prototype` / `number_prototype` directly when the receiver is
//! a primitive — no wrapper allocation.
//!
//! `this` reaches each prototype method via `Runtime::current_this()`,
//! which `call_function` stashes around every NativeFn invocation.

use crate::abstract_ops;
use crate::interp::{Runtime, RuntimeError};
use crate::value::{
    FunctionInternals, InternalKind, NativeFn, Object, ObjectRef,
    PromiseReaction, PromiseStatus, Value,
    BoundFunctionInternals,
};
use std::collections::HashMap;
use std::rc::Rc;

impl Runtime {
    /// Allocate and wire every intrinsic prototype. Must run before any
    /// other intrinsic so subsequent `alloc_object` calls pick up the
    /// stashes. Idempotent in practice (each call would clobber prior
    /// stashes — install_intrinsics calls it exactly once).
    pub fn install_prototypes(&mut self) {
        // The Object.prototype object is itself an Ordinary, but it must
        // not inherit from itself — explicitly install with proto: None
        // before any stash is set, which the alloc-time wiring respects
        // since no proto is installed yet.
        let object_proto = self.alloc_object(Object::new_ordinary());
        self.object_prototype = Some(object_proto);

        // Now allocate the rest; each will pick up object_prototype as
        // its `proto` automatically via the alloc-time wiring, which is
        // exactly what Array/Function/Promise/String/Number prototypes
        // want per spec (every prototype inherits from Object.prototype).
        let array_proto    = self.alloc_object(Object::new_ordinary());
        let function_proto = self.alloc_object(Object::new_ordinary());
        let promise_proto  = self.alloc_object(Object::new_ordinary());
        let string_proto   = self.alloc_object(Object::new_ordinary());
        let number_proto   = self.alloc_object(Object::new_ordinary());
        self.array_prototype    = Some(array_proto);
        self.function_prototype = Some(function_proto);
        self.promise_prototype  = Some(promise_proto);
        self.string_prototype   = Some(string_proto);
        self.number_prototype   = Some(number_proto);

        install_object_proto(self, object_proto);
        install_array_proto(self, array_proto);
        install_string_proto(self, string_proto);
        install_function_proto(self, function_proto);
        install_promise_proto(self, promise_proto);
        install_number_proto(self, number_proto);
    }
}

// ──────────────── %Object.prototype% ────────────────

fn install_object_proto(rt: &mut Runtime, host: ObjectRef) {
    register_method(rt, host, "toString", |rt, _args| {
        let this = rt.current_this();
        let s = match this {
            Value::Undefined => "[object Undefined]".to_string(),
            Value::Null => "[object Null]".to_string(),
            Value::Object(id) => {
                let tag = match &rt.obj(id).internal_kind {
                    InternalKind::Array => "Array",
                    InternalKind::Function(_)
                    | InternalKind::Closure(_)
                    | InternalKind::BoundFunction(_) => "Function",
                    InternalKind::Promise(_) => "Promise",
                    InternalKind::Error => "Error",
                    _ => "Object",
                };
                format!("[object {}]", tag)
            }
            _ => format!("[object {}]", this.type_of()),
        };
        Ok(Value::String(Rc::new(s)))
    });
    register_method(rt, host, "hasOwnProperty", |rt, args| {
        let key = arg_string(args, 0);
        let owns = match rt.current_this() {
            Value::Object(id) => rt.obj(id).properties.contains_key(&key),
            _ => false,
        };
        Ok(Value::Boolean(owns))
    });
    register_method(rt, host, "valueOf", |rt, _args| Ok(rt.current_this()));
    register_method(rt, host, "isPrototypeOf", |rt, args| {
        let target = match args.first() {
            Some(Value::Object(id)) => *id,
            _ => return Ok(Value::Boolean(false)),
        };
        let this_id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Ok(Value::Boolean(false)),
        };
        let mut cur = rt.obj(target).proto;
        while let Some(c) = cur {
            if c == this_id { return Ok(Value::Boolean(true)); }
            cur = rt.obj(c).proto;
        }
        Ok(Value::Boolean(false))
    });
}

// ──────────────── %Array.prototype% ────────────────

fn install_array_proto(rt: &mut Runtime, host: ObjectRef) {
    register_method(rt, host, "push", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Err(RuntimeError::TypeError("Array.prototype.push: this is not an Array".into())),
        };
        let mut len = rt.array_length(id);
        for a in args {
            rt.object_set(id, len.to_string(), a.clone());
            len += 1;
        }
        // Keep a synthetic length to outpace property-derivation in edge cases.
        rt.object_set(id, "length".into(), Value::Number(len as f64));
        Ok(Value::Number(len as f64))
    });
    register_method(rt, host, "pop", |rt, _args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Ok(Value::Undefined),
        };
        let len = rt.array_length(id);
        if len == 0 { return Ok(Value::Undefined); }
        let last_key = (len - 1).to_string();
        let v = rt.object_get(id, &last_key);
        rt.obj_mut(id).properties.remove(&last_key);
        rt.object_set(id, "length".into(), Value::Number((len - 1) as f64));
        Ok(v)
    });
    register_method(rt, host, "shift", |rt, _args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Ok(Value::Undefined),
        };
        let len = rt.array_length(id);
        if len == 0 { return Ok(Value::Undefined); }
        let first = rt.object_get(id, "0");
        for i in 1..len {
            let v = rt.object_get(id, &i.to_string());
            rt.object_set(id, (i - 1).to_string(), v);
        }
        rt.obj_mut(id).properties.remove(&(len - 1).to_string());
        rt.object_set(id, "length".into(), Value::Number((len - 1) as f64));
        Ok(first)
    });
    register_method(rt, host, "unshift", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Ok(Value::Number(0.0)),
        };
        let n = args.len();
        let len = rt.array_length(id);
        // Shift existing right by n.
        for i in (0..len).rev() {
            let v = rt.object_get(id, &i.to_string());
            rt.object_set(id, (i + n).to_string(), v);
        }
        for (i, a) in args.iter().enumerate() {
            rt.object_set(id, i.to_string(), a.clone());
        }
        let new_len = len + n;
        rt.object_set(id, "length".into(), Value::Number(new_len as f64));
        Ok(Value::Number(new_len as f64))
    });
    register_method(rt, host, "indexOf", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Ok(Value::Number(-1.0)),
        };
        let needle = args.first().cloned().unwrap_or(Value::Undefined);
        let len = rt.array_length(id);
        for i in 0..len {
            let v = rt.object_get(id, &i.to_string());
            if abstract_ops::is_strictly_equal(&v, &needle) {
                return Ok(Value::Number(i as f64));
            }
        }
        Ok(Value::Number(-1.0))
    });
    register_method(rt, host, "includes", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Ok(Value::Boolean(false)),
        };
        let needle = args.first().cloned().unwrap_or(Value::Undefined);
        let len = rt.array_length(id);
        for i in 0..len {
            let v = rt.object_get(id, &i.to_string());
            if abstract_ops::is_strictly_equal(&v, &needle) {
                return Ok(Value::Boolean(true));
            }
        }
        Ok(Value::Boolean(false))
    });
    register_method(rt, host, "slice", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Err(RuntimeError::TypeError("Array.prototype.slice: this is not an Array".into())),
        };
        let len = rt.array_length(id) as i64;
        let start_arg = args.first().map(abstract_ops::to_number).unwrap_or(0.0) as i64;
        let end_arg = args.get(1).map(abstract_ops::to_number).unwrap_or(len as f64) as i64;
        let start = clamp_index(start_arg, len);
        let end = clamp_index(end_arg, len);
        let out = rt.alloc_object(Object::new_array());
        let mut j: i64 = 0;
        let mut i = start;
        while i < end {
            let v = rt.object_get(id, &i.to_string());
            rt.object_set(out, j.to_string(), v);
            j += 1;
            i += 1;
        }
        rt.object_set(out, "length".into(), Value::Number(j as f64));
        Ok(Value::Object(out))
    });
    register_method(rt, host, "concat", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Err(RuntimeError::TypeError("Array.prototype.concat: this not Array".into())),
        };
        let len = rt.array_length(id);
        let out = rt.alloc_object(Object::new_array());
        let mut j = 0usize;
        for i in 0..len {
            let v = rt.object_get(id, &i.to_string());
            rt.object_set(out, j.to_string(), v);
            j += 1;
        }
        for a in args {
            match a {
                Value::Object(aid) => {
                    if matches!(rt.obj(*aid).internal_kind, InternalKind::Array) {
                        let al = rt.array_length(*aid);
                        for i in 0..al {
                            let v = rt.object_get(*aid, &i.to_string());
                            rt.object_set(out, j.to_string(), v);
                            j += 1;
                        }
                    } else {
                        rt.object_set(out, j.to_string(), a.clone());
                        j += 1;
                    }
                }
                _ => {
                    rt.object_set(out, j.to_string(), a.clone());
                    j += 1;
                }
            }
        }
        rt.object_set(out, "length".into(), Value::Number(j as f64));
        Ok(Value::Object(out))
    });
    register_method(rt, host, "join", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Ok(Value::String(Rc::new(String::new()))),
        };
        let sep = match args.first() {
            Some(Value::Undefined) | None => ",".to_string(),
            Some(v) => abstract_ops::to_string(v).as_str().to_string(),
        };
        let len = rt.array_length(id);
        let mut parts = Vec::with_capacity(len);
        for i in 0..len {
            let v = rt.object_get(id, &i.to_string());
            let s = match v {
                Value::Undefined | Value::Null => String::new(),
                other => abstract_ops::to_string(&other).as_str().to_string(),
            };
            parts.push(s);
        }
        Ok(Value::String(Rc::new(parts.join(&sep))))
    });
    register_method(rt, host, "at", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Ok(Value::Undefined),
        };
        let len = rt.array_length(id) as i64;
        let i = args.first().map(abstract_ops::to_number).unwrap_or(0.0) as i64;
        let idx = if i < 0 { len + i } else { i };
        if idx < 0 || idx >= len { return Ok(Value::Undefined); }
        Ok(rt.object_get(id, &idx.to_string()))
    });
    register_method(rt, host, "map", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Err(RuntimeError::TypeError("Array.prototype.map: this not Array".into())),
        };
        let cb = args.first().cloned().ok_or_else(||
            RuntimeError::TypeError("Array.prototype.map: callback required".into()))?;
        let len = rt.array_length(id);
        let out = rt.alloc_object(Object::new_array());
        for i in 0..len {
            let v = rt.object_get(id, &i.to_string());
            let mapped = rt.call_function(cb.clone(), Value::Undefined,
                vec![v, Value::Number(i as f64), Value::Object(id)])?;
            rt.object_set(out, i.to_string(), mapped);
        }
        rt.object_set(out, "length".into(), Value::Number(len as f64));
        Ok(Value::Object(out))
    });
    register_method(rt, host, "forEach", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Err(RuntimeError::TypeError("forEach: this not Array".into())),
        };
        let cb = args.first().cloned().ok_or_else(||
            RuntimeError::TypeError("Array.prototype.forEach: callback required".into()))?;
        let len = rt.array_length(id);
        for i in 0..len {
            let v = rt.object_get(id, &i.to_string());
            rt.call_function(cb.clone(), Value::Undefined,
                vec![v, Value::Number(i as f64), Value::Object(id)])?;
        }
        Ok(Value::Undefined)
    });
    register_method(rt, host, "filter", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Err(RuntimeError::TypeError("filter: this not Array".into())),
        };
        let cb = args.first().cloned().ok_or_else(||
            RuntimeError::TypeError("Array.prototype.filter: callback required".into()))?;
        let len = rt.array_length(id);
        let out = rt.alloc_object(Object::new_array());
        let mut j = 0usize;
        for i in 0..len {
            let v = rt.object_get(id, &i.to_string());
            let r = rt.call_function(cb.clone(), Value::Undefined,
                vec![v.clone(), Value::Number(i as f64), Value::Object(id)])?;
            if abstract_ops::to_boolean(&r) {
                rt.object_set(out, j.to_string(), v);
                j += 1;
            }
        }
        rt.object_set(out, "length".into(), Value::Number(j as f64));
        Ok(Value::Object(out))
    });
    register_method(rt, host, "reduce", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Err(RuntimeError::TypeError("reduce: this not Array".into())),
        };
        let cb = args.first().cloned().ok_or_else(||
            RuntimeError::TypeError("Array.prototype.reduce: callback required".into()))?;
        let len = rt.array_length(id);
        let has_init = args.len() >= 2;
        let mut i = 0usize;
        let mut acc = if has_init {
            args[1].clone()
        } else {
            if len == 0 { return Err(RuntimeError::TypeError("reduce of empty array with no initial value".into())); }
            i = 1;
            rt.object_get(id, "0")
        };
        while i < len {
            let v = rt.object_get(id, &i.to_string());
            acc = rt.call_function(cb.clone(), Value::Undefined,
                vec![acc, v, Value::Number(i as f64), Value::Object(id)])?;
            i += 1;
        }
        Ok(acc)
    });
    register_method(rt, host, "find", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Ok(Value::Undefined),
        };
        let cb = args.first().cloned().ok_or_else(||
            RuntimeError::TypeError("find: callback required".into()))?;
        let len = rt.array_length(id);
        for i in 0..len {
            let v = rt.object_get(id, &i.to_string());
            let r = rt.call_function(cb.clone(), Value::Undefined,
                vec![v.clone(), Value::Number(i as f64), Value::Object(id)])?;
            if abstract_ops::to_boolean(&r) { return Ok(v); }
        }
        Ok(Value::Undefined)
    });
    register_method(rt, host, "some", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Ok(Value::Boolean(false)),
        };
        let cb = args.first().cloned().ok_or_else(||
            RuntimeError::TypeError("some: callback required".into()))?;
        let len = rt.array_length(id);
        for i in 0..len {
            let v = rt.object_get(id, &i.to_string());
            let r = rt.call_function(cb.clone(), Value::Undefined,
                vec![v, Value::Number(i as f64), Value::Object(id)])?;
            if abstract_ops::to_boolean(&r) { return Ok(Value::Boolean(true)); }
        }
        Ok(Value::Boolean(false))
    });
    register_method(rt, host, "@@iterator", |rt, _args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Err(RuntimeError::TypeError("@@iterator: this is not an Array".into())),
        };
        Ok(Value::Object(crate::iterator::make_array_iterator(rt, id)))
    });
    // Tier-Ω.5.j.proto: Array.prototype.sort(comparator?).
    // Mutates `this` in place, returns `this`. Stable.
    // - No comparator: ToString each element, lexicographic compare.
    // - With comparator: call comparator(a,b); sign of return → Ordering.
    // v1 ignores spec's sparse-array semantics; sorts dense own indices 0..length-1.
    register_method(rt, host, "sort", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Err(RuntimeError::TypeError("Array.prototype.sort: this is not an Array".into())),
        };
        let comparator = args.first().cloned().filter(|v| !matches!(v, Value::Undefined));
        let len = rt.array_length(id);
        let mut items: Vec<Value> = (0..len).map(|i| rt.object_get(id, &i.to_string())).collect();
        // Stable sort. With comparator, use call_function; on error propagate.
        // sort_by needs a non-fallible cmp, so collect errors via interior state.
        let mut err: Option<RuntimeError> = None;
        match comparator {
            None => {
                items.sort_by(|a, b| {
                    let sa = abstract_ops::to_string(a);
                    let sb = abstract_ops::to_string(b);
                    sa.as_str().cmp(sb.as_str())
                });
            }
            Some(cb) => {
                items.sort_by(|a, b| {
                    if err.is_some() { return std::cmp::Ordering::Equal; }
                    match rt.call_function(cb.clone(), Value::Undefined, vec![a.clone(), b.clone()]) {
                        Ok(v) => {
                            let n = abstract_ops::to_number(&v);
                            if n.is_nan() { std::cmp::Ordering::Equal }
                            else if n < 0.0 { std::cmp::Ordering::Less }
                            else if n > 0.0 { std::cmp::Ordering::Greater }
                            else { std::cmp::Ordering::Equal }
                        }
                        Err(e) => { err = Some(e); std::cmp::Ordering::Equal }
                    }
                });
            }
        }
        if let Some(e) = err { return Err(e); }
        for (i, v) in items.into_iter().enumerate() {
            rt.object_set(id, i.to_string(), v);
        }
        rt.object_set(id, "length".into(), Value::Number(len as f64));
        Ok(Value::Object(id))
    });
    register_method(rt, host, "every", |rt, args| {
        let id = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Ok(Value::Boolean(true)),
        };
        let cb = args.first().cloned().ok_or_else(||
            RuntimeError::TypeError("every: callback required".into()))?;
        let len = rt.array_length(id);
        for i in 0..len {
            let v = rt.object_get(id, &i.to_string());
            let r = rt.call_function(cb.clone(), Value::Undefined,
                vec![v, Value::Number(i as f64), Value::Object(id)])?;
            if !abstract_ops::to_boolean(&r) { return Ok(Value::Boolean(false)); }
        }
        Ok(Value::Boolean(true))
    });
}

fn clamp_index(i: i64, len: i64) -> i64 {
    let v = if i < 0 { (len + i).max(0) } else { i.min(len) };
    v
}

// ──────────────── %String.prototype% ────────────────

fn install_string_proto(rt: &mut Runtime, host: ObjectRef) {
    register_method(rt, host, "toUpperCase", |rt, _args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_uppercase();
        Ok(Value::String(Rc::new(s)))
    });
    register_method(rt, host, "toLowerCase", |rt, _args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_lowercase();
        Ok(Value::String(Rc::new(s)))
    });
    register_method(rt, host, "trim", |rt, _args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().trim().to_string();
        Ok(Value::String(Rc::new(s)))
    });
    register_method(rt, host, "charAt", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let i = args.first().map(abstract_ops::to_number).unwrap_or(0.0) as usize;
        let c = s.chars().nth(i).map(|c| c.to_string()).unwrap_or_default();
        Ok(Value::String(Rc::new(c)))
    });
    register_method(rt, host, "charCodeAt", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let i = args.first().map(abstract_ops::to_number).unwrap_or(0.0) as usize;
        match s.chars().nth(i) {
            Some(c) => Ok(Value::Number(c as u32 as f64)),
            None => Ok(Value::Number(f64::NAN)),
        }
    });
    register_method(rt, host, "slice", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len() as i64;
        let start = clamp_index(args.first().map(abstract_ops::to_number).unwrap_or(0.0) as i64, len);
        let end = clamp_index(args.get(1).map(abstract_ops::to_number).unwrap_or(len as f64) as i64, len);
        if end <= start { return Ok(Value::String(Rc::new(String::new()))); }
        let out: String = chars[start as usize..end as usize].iter().collect();
        Ok(Value::String(Rc::new(out)))
    });
    register_method(rt, host, "substring", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len() as i64;
        let mut a = args.first().map(abstract_ops::to_number).unwrap_or(0.0) as i64;
        let mut b = args.get(1).map(abstract_ops::to_number).unwrap_or(len as f64) as i64;
        a = a.clamp(0, len);
        b = b.clamp(0, len);
        if a > b { std::mem::swap(&mut a, &mut b); }
        let out: String = chars[a as usize..b as usize].iter().collect();
        Ok(Value::String(Rc::new(out)))
    });
    register_method(rt, host, "indexOf", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let needle = abstract_ops::to_string(&args.first().cloned().unwrap_or(Value::Undefined))
            .as_str().to_string();
        match s.find(&needle) {
            // .find returns byte offset; convert to char index by counting.
            Some(byte_off) => Ok(Value::Number(s[..byte_off].chars().count() as f64)),
            None => Ok(Value::Number(-1.0)),
        }
    });
    register_method(rt, host, "lastIndexOf", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let needle = abstract_ops::to_string(&args.first().cloned().unwrap_or(Value::Undefined))
            .as_str().to_string();
        match s.rfind(&needle) {
            Some(byte_off) => Ok(Value::Number(s[..byte_off].chars().count() as f64)),
            None => Ok(Value::Number(-1.0)),
        }
    });
    register_method(rt, host, "includes", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let needle = abstract_ops::to_string(&args.first().cloned().unwrap_or(Value::Undefined))
            .as_str().to_string();
        Ok(Value::Boolean(s.contains(&needle)))
    });
    register_method(rt, host, "startsWith", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let needle = abstract_ops::to_string(&args.first().cloned().unwrap_or(Value::Undefined))
            .as_str().to_string();
        Ok(Value::Boolean(s.starts_with(&needle)))
    });
    register_method(rt, host, "endsWith", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let needle = abstract_ops::to_string(&args.first().cloned().unwrap_or(Value::Undefined))
            .as_str().to_string();
        Ok(Value::Boolean(s.ends_with(&needle)))
    });
    register_method(rt, host, "split", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let out = rt.alloc_object(Object::new_array());
        let parts: Vec<String> = match args.first() {
            None | Some(Value::Undefined) => vec![s.clone()],
            Some(sep_v) => {
                let sep = abstract_ops::to_string(sep_v).as_str().to_string();
                if sep.is_empty() {
                    s.chars().map(|c| c.to_string()).collect()
                } else {
                    s.split(&sep).map(|s| s.to_string()).collect()
                }
            }
        };
        for (i, p) in parts.iter().enumerate() {
            rt.object_set(out, i.to_string(), Value::String(Rc::new(p.clone())));
        }
        rt.object_set(out, "length".into(), Value::Number(parts.len() as f64));
        Ok(Value::Object(out))
    });
    register_method(rt, host, "repeat", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let n = args.first().map(abstract_ops::to_number).unwrap_or(0.0) as usize;
        Ok(Value::String(Rc::new(s.repeat(n))))
    });
    register_method(rt, host, "replace", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let needle = abstract_ops::to_string(&args.first().cloned().unwrap_or(Value::Undefined))
            .as_str().to_string();
        let repl = abstract_ops::to_string(&args.get(1).cloned().unwrap_or(Value::Undefined))
            .as_str().to_string();
        Ok(Value::String(Rc::new(s.replacen(&needle, &repl, 1))))
    });
    register_method(rt, host, "at", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len() as i64;
        let i = args.first().map(abstract_ops::to_number).unwrap_or(0.0) as i64;
        let idx = if i < 0 { len + i } else { i };
        if idx < 0 || idx >= len { return Ok(Value::Undefined); }
        Ok(Value::String(Rc::new(chars[idx as usize].to_string())))
    });
    register_method(rt, host, "toString", |rt, _args| {
        Ok(Value::String(Rc::new(abstract_ops::to_string(&rt.current_this()).as_str().to_string())))
    });
    register_method(rt, host, "@@iterator", |rt, _args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        Ok(Value::Object(crate::iterator::make_string_iterator(rt, s)))
    });
}

// ──────────────── %Function.prototype% ────────────────

fn install_function_proto(rt: &mut Runtime, host: ObjectRef) {
    register_method(rt, host, "call", |rt, args| {
        let f = rt.current_this();
        let this_arg = args.first().cloned().unwrap_or(Value::Undefined);
        let rest: Vec<Value> = args.iter().skip(1).cloned().collect();
        rt.call_function(f, this_arg, rest)
    });
    register_method(rt, host, "apply", |rt, args| {
        let f = rt.current_this();
        let this_arg = args.first().cloned().unwrap_or(Value::Undefined);
        let arr_v = args.get(1).cloned().unwrap_or(Value::Undefined);
        let call_args: Vec<Value> = match arr_v {
            Value::Object(aid) => {
                let len = rt.array_length(aid);
                (0..len).map(|i| rt.object_get(aid, &i.to_string())).collect()
            }
            Value::Null | Value::Undefined => Vec::new(),
            _ => return Err(RuntimeError::TypeError("apply: argsArray must be an Array".into())),
        };
        rt.call_function(f, this_arg, call_args)
    });
    register_method(rt, host, "bind", |rt, args| {
        let target = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Err(RuntimeError::TypeError("bind: this is not callable".into())),
        };
        let bound_this = args.first().cloned().unwrap_or(Value::Undefined);
        let bound_args: Vec<Value> = args.iter().skip(1).cloned().collect();
        let bf = Object {
            proto: None,
            extensible: true,
            properties: HashMap::new(),
            internal_kind: InternalKind::BoundFunction(BoundFunctionInternals {
                target,
                this: bound_this,
                args: bound_args,
            }),
        };
        let id = rt.alloc_object(bf);
        Ok(Value::Object(id))
    });
}

// ──────────────── %Promise.prototype% ────────────────
//
// `then` / `catch` delegate to the static-form logic in promise.rs. The
// receiver is the source promise. Since the static Promise.then native
// already expects (source, onF, onR) as positional args, we synthesize
// that argument list here.

fn install_promise_proto(rt: &mut Runtime, host: ObjectRef) {
    register_method(rt, host, "then", |rt, args| {
        let source = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Err(RuntimeError::TypeError("Promise.prototype.then: this is not a Promise".into())),
        };
        promise_then_impl(rt, source, args.first().cloned(), args.get(1).cloned())
    });
    register_method(rt, host, "catch", |rt, args| {
        let source = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Err(RuntimeError::TypeError("Promise.prototype.catch: this is not a Promise".into())),
        };
        promise_then_impl(rt, source, None, args.first().cloned())
    });
}

fn promise_then_impl(
    rt: &mut Runtime,
    source: ObjectRef,
    on_fulfilled: Option<Value>,
    on_rejected: Option<Value>,
) -> Result<Value, RuntimeError> {
    let chain = crate::promise::new_promise(rt);
    let (status, value) = {
        let s = rt.obj(source);
        match &s.internal_kind {
            InternalKind::Promise(ps) => (ps.status, ps.value.clone()),
            _ => return Err(RuntimeError::TypeError("then: source is not a Promise".into())),
        }
    };
    match status {
        PromiseStatus::Pending => {
            let src = rt.obj_mut(source);
            if let InternalKind::Promise(ps) = &mut src.internal_kind {
                ps.fulfill_reactions.push(PromiseReaction { handler: on_fulfilled, chain });
                ps.reject_reactions.push(PromiseReaction { handler: on_rejected, chain });
            }
        }
        PromiseStatus::Fulfilled => {
            enqueue_handler(rt, on_fulfilled, value, chain, false);
        }
        PromiseStatus::Rejected => {
            rt.pending_unhandled.remove(&source);
            enqueue_handler(rt, on_rejected, value, chain, true);
        }
    }
    Ok(Value::Object(chain))
}

fn enqueue_handler(
    rt: &mut Runtime,
    handler: Option<Value>,
    value: Value,
    chain: ObjectRef,
    is_rejected: bool,
) {
    rt.enqueue_microtask("PromiseReactionJob", move |rt| {
        match handler {
            Some(h) => match rt.call_function(h, Value::Undefined, vec![value]) {
                Ok(r) => { crate::promise::resolve_promise(rt, chain, r); }
                Err(e) => {
                    let thrown = match e {
                        RuntimeError::Thrown(v) => v,
                        other => Value::String(Rc::new(format!("{:?}", other))),
                    };
                    crate::promise::reject_promise(rt, chain, thrown);
                }
            }
            None => if is_rejected {
                crate::promise::reject_promise(rt, chain, value);
            } else {
                crate::promise::resolve_promise(rt, chain, value);
            }
        }
        Ok(())
    });
}

// ──────────────── %Number.prototype% ────────────────

fn install_number_proto(rt: &mut Runtime, host: ObjectRef) {
    register_method(rt, host, "toString", |rt, args| {
        let n = abstract_ops::to_number(&rt.current_this());
        let radix = args.first().map(abstract_ops::to_number).unwrap_or(10.0) as i32;
        if radix == 10 || radix == 0 {
            Ok(Value::String(Rc::new(abstract_ops::number_to_string(n))))
        } else if (2..=36).contains(&radix) && n.is_finite() && n.fract() == 0.0 {
            // Integer-radix only — fractional radix conversion is rare.
            let mut x = n as i64;
            if x == 0 { return Ok(Value::String(Rc::new("0".into()))); }
            let neg = x < 0;
            if neg { x = -x; }
            let mut digits = Vec::new();
            while x > 0 {
                let d = (x % radix as i64) as u32;
                let c = if d < 10 { (b'0' + d as u8) as char } else { (b'a' + (d - 10) as u8) as char };
                digits.push(c);
                x /= radix as i64;
            }
            if neg { digits.push('-'); }
            digits.reverse();
            Ok(Value::String(Rc::new(digits.into_iter().collect())))
        } else {
            Ok(Value::String(Rc::new(abstract_ops::number_to_string(n))))
        }
    });
    register_method(rt, host, "toFixed", |rt, args| {
        let n = abstract_ops::to_number(&rt.current_this());
        let digits = args.first().map(abstract_ops::to_number).unwrap_or(0.0) as usize;
        Ok(Value::String(Rc::new(format!("{:.*}", digits, n))))
    });
    register_method(rt, host, "valueOf", |rt, _args| Ok(rt.current_this()));
}

// ──────────────── helpers ────────────────

fn arg_string(args: &[Value], i: usize) -> String {
    args.get(i).map(|v| abstract_ops::to_string(v).as_str().to_string()).unwrap_or_default()
}

fn register_method<F>(rt: &mut Runtime, host: ObjectRef, name: &str, f: F)
where F: Fn(&mut Runtime, &[Value]) -> Result<Value, RuntimeError> + 'static {
    let native: NativeFn = Rc::new(f);
    let fn_obj = Object {
        proto: None, // function_prototype not yet installed when called from install_prototypes
        extensible: true,
        properties: HashMap::new(),
        internal_kind: InternalKind::Function(FunctionInternals {
            name: name.to_string(),
            native,
        }),
    };
    let fn_id = rt.alloc_object(fn_obj);
    rt.object_set(host, name.into(), Value::Object(fn_id));
}
