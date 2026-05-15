//! Tier-Ω.5.c Stage 2 — iterator protocol helpers.
//!
//! v1 representation: a Symbol.iterator value is the well-known string
//! `"@@iterator"`. The `Symbol` global is an ordinary object whose
//! `iterator` slot holds that string, so JS code `obj[Symbol.iterator]`
//! evaluates to property-key `"@@iterator"`. The iterable protocols
//! (Array.prototype, String.prototype, etc.) register a method under
//! that key.
//!
//! String iteration walks the source by Unicode scalar values (Rust
//! `char` iteration) rather than UTF-16 code units. This is a deliberate
//! v1 deviation from ECMA-262 §22.1.5.1 which iterates by code points
//! but yields surrogate pairs as single elements only on the BMP path.
//! UTF-8/char iteration matches consumer expectations for ASCII +
//! single-codepoint inputs which cover the corpus.

use crate::interp::{Runtime, RuntimeError};
use crate::value::{
    FunctionInternals, InternalKind, NativeFn, Object, ObjectRef, PropertyDescriptor, Value,
};
use std::collections::HashMap;
use std::rc::Rc;

/// Make an Array iterator object — `{ next: () => {value, done}, _arr, _i }`.
/// The iterator carries its source array id and a current-index cursor
/// stored as a regular property `_i` (v1 — a real engine would intern in
/// internal slots).
pub fn make_array_iterator(rt: &mut Runtime, src: ObjectRef) -> ObjectRef {
    let iter = rt.alloc_object(Object::new_ordinary());
    rt.object_set(iter, "_arr".into(), Value::Object(src));
    rt.object_set(iter, "_i".into(), Value::Number(0.0));
    install_next(rt, iter, |rt, _args| {
        let it = match rt.current_this() {
            Value::Object(id) => id,
            _ => return Err(RuntimeError::TypeError("array iterator next: this is not an iterator".into())),
        };
        let src_id = match rt.object_get(it, "_arr") {
            Value::Object(id) => id,
            _ => return Ok(iter_result_done(rt)),
        };
        let i = match rt.object_get(it, "_i") {
            Value::Number(n) => n as usize,
            _ => 0,
        };
        let len = rt.array_length(src_id);
        if i >= len {
            return Ok(iter_result_done(rt));
        }
        let v = rt.object_get(src_id, &i.to_string());
        rt.object_set(it, "_i".into(), Value::Number((i + 1) as f64));
        Ok(iter_result_value(rt, v))
    });
    iter
}

/// Make a String iterator. Pre-collects the chars into a Vec stored on
/// the iterator's _chars property (as an Array of single-character
/// strings) for simplicity. _i tracks the cursor.
pub fn make_string_iterator(rt: &mut Runtime, s: String) -> ObjectRef {
    let chars: Vec<char> = s.chars().collect();
    let arr = rt.alloc_object(Object::new_array());
    for (i, c) in chars.iter().enumerate() {
        rt.object_set(arr, i.to_string(), Value::String(Rc::new(c.to_string())));
    }
    rt.object_set(arr, "length".into(), Value::Number(chars.len() as f64));
    make_array_iterator(rt, arr)
}

fn install_next<F>(rt: &mut Runtime, host: ObjectRef, f: F)
where F: Fn(&mut Runtime, &[Value]) -> Result<Value, RuntimeError> + 'static {
    let native: NativeFn = Rc::new(f);
    let fn_obj = Object {
        proto: None,
        extensible: true,
        properties: HashMap::new(),
        internal_kind: InternalKind::Function(FunctionInternals { name: "next".into(), native }),
    };
    let fn_id = rt.alloc_object(fn_obj);
    rt.object_set(host, "next".into(), Value::Object(fn_id));
}

/// Build `{ value, done: false }`.
pub fn iter_result_value(rt: &mut Runtime, v: Value) -> Value {
    let id = rt.alloc_object(Object::new_ordinary());
    rt.obj_mut(id).properties.insert("value".into(), PropertyDescriptor {
        value: v, writable: true, enumerable: true, configurable: true, getter: None, setter: None,
    });
    rt.obj_mut(id).properties.insert("done".into(), PropertyDescriptor {
        value: Value::Boolean(false), writable: true, enumerable: true, configurable: true, getter: None, setter: None,
    });
    Value::Object(id)
}

/// Build `{ value: undefined, done: true }`.
pub fn iter_result_done(rt: &mut Runtime) -> Value {
    let id = rt.alloc_object(Object::new_ordinary());
    rt.obj_mut(id).properties.insert("value".into(), PropertyDescriptor {
        value: Value::Undefined, writable: true, enumerable: true, configurable: true, getter: None, setter: None,
    });
    rt.obj_mut(id).properties.insert("done".into(), PropertyDescriptor {
        value: Value::Boolean(true), writable: true, enumerable: true, configurable: true, getter: None, setter: None,
    });
    Value::Object(id)
}
