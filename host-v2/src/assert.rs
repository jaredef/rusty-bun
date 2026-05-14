//! node:assert intrinsic — Tier-Ω.5.s.
//!
//! Minimal v1 implementation. `ok` / `equal` / `deepEqual` are actually
//! implemented (cheap and load-bearing for yargs); the remaining surface
//! is stubbed so import-time and shape probes succeed.

use crate::register::{new_object, register_method, set_constant};
use rusty_js_runtime::abstract_ops;
use rusty_js_runtime::{Runtime, RuntimeError, Value};
use std::rc::Rc;

fn assertion_error(msg: String) -> RuntimeError {
    RuntimeError::Thrown(Value::String(Rc::new(format!("AssertionError: {msg}"))))
}

fn arg_msg(args: &[Value], idx: usize, fallback: &str) -> String {
    match args.get(idx) {
        Some(Value::String(s)) => s.as_str().to_string(),
        Some(other) => abstract_ops::to_string(other).as_str().to_string(),
        None => fallback.to_string(),
    }
}

fn ok_impl(_rt: &mut Runtime, args: &[Value]) -> Result<Value, RuntimeError> {
    let v = args.first().cloned().unwrap_or(Value::Undefined);
    if abstract_ops::to_boolean(&v) {
        Ok(Value::Undefined)
    } else {
        Err(assertion_error(arg_msg(args, 1, "assertion failed")))
    }
}

fn loose_eq(a: &Value, b: &Value) -> bool {
    // === for v1 — close enough for yargs's typical usage.
    abstract_ops::is_strictly_equal(a, b)
}

pub fn install(rt: &mut Runtime) {
    let assert = new_object(rt);

    register_method(rt, assert, "ok", ok_impl);
    register_method(rt, assert, "strict", ok_impl);

    register_method(rt, assert, "equal", |_rt, args| {
        let a = args.first().cloned().unwrap_or(Value::Undefined);
        let b = args.get(1).cloned().unwrap_or(Value::Undefined);
        if loose_eq(&a, &b) {
            Ok(Value::Undefined)
        } else {
            Err(assertion_error(arg_msg(args, 2, "equal failed")))
        }
    });
    register_method(rt, assert, "notEqual", |_rt, args| {
        let a = args.first().cloned().unwrap_or(Value::Undefined);
        let b = args.get(1).cloned().unwrap_or(Value::Undefined);
        if !loose_eq(&a, &b) {
            Ok(Value::Undefined)
        } else {
            Err(assertion_error(arg_msg(args, 2, "notEqual failed")))
        }
    });
    register_method(rt, assert, "strictEqual", |_rt, args| {
        let a = args.first().cloned().unwrap_or(Value::Undefined);
        let b = args.get(1).cloned().unwrap_or(Value::Undefined);
        if loose_eq(&a, &b) {
            Ok(Value::Undefined)
        } else {
            Err(assertion_error(arg_msg(args, 2, "strictEqual failed")))
        }
    });
    register_method(rt, assert, "notStrictEqual", |_rt, args| {
        let a = args.first().cloned().unwrap_or(Value::Undefined);
        let b = args.get(1).cloned().unwrap_or(Value::Undefined);
        if !loose_eq(&a, &b) {
            Ok(Value::Undefined)
        } else {
            Err(assertion_error(arg_msg(args, 2, "notStrictEqual failed")))
        }
    });

    // deepEqual: stringify both via JSON, compare. Round-trips through
    // the JSON intrinsic instead of duplicating a recursive walker.
    register_method(rt, assert, "deepEqual", |rt, args| {
        let a = args.first().cloned().unwrap_or(Value::Undefined);
        let b = args.get(1).cloned().unwrap_or(Value::Undefined);
        let sa = json_stringify_via_intrinsic(rt, &a)?;
        let sb = json_stringify_via_intrinsic(rt, &b)?;
        if sa == sb {
            Ok(Value::Undefined)
        } else {
            Err(assertion_error(arg_msg(args, 2, "deepEqual failed")))
        }
    });
    register_method(rt, assert, "notDeepEqual", |rt, args| {
        let a = args.first().cloned().unwrap_or(Value::Undefined);
        let b = args.get(1).cloned().unwrap_or(Value::Undefined);
        let sa = json_stringify_via_intrinsic(rt, &a)?;
        let sb = json_stringify_via_intrinsic(rt, &b)?;
        if sa != sb {
            Ok(Value::Undefined)
        } else {
            Err(assertion_error(arg_msg(args, 2, "notDeepEqual failed")))
        }
    });
    register_method(rt, assert, "deepStrictEqual", |rt, args| {
        let a = args.first().cloned().unwrap_or(Value::Undefined);
        let b = args.get(1).cloned().unwrap_or(Value::Undefined);
        let sa = json_stringify_via_intrinsic(rt, &a)?;
        let sb = json_stringify_via_intrinsic(rt, &b)?;
        if sa == sb {
            Ok(Value::Undefined)
        } else {
            Err(assertion_error(arg_msg(args, 2, "deepStrictEqual failed")))
        }
    });

    register_method(rt, assert, "fail", |_rt, args| {
        Err(assertion_error(arg_msg(args, 0, "fail")))
    });
    register_method(rt, assert, "throws", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:assert throws: not yet implemented (Tier-Ω.5.s stub)".into(),
        ))
    });
    register_method(rt, assert, "doesNotThrow", |_rt, _args| {
        Err(RuntimeError::TypeError(
            "node:assert doesNotThrow: not yet implemented (Tier-Ω.5.s stub)".into(),
        ))
    });

    set_constant(rt, assert, "default", Value::Object(assert));
    rt.globals.insert("assert".into(), Value::Object(assert));
}

/// Call JSON.stringify(v) via the installed intrinsic. Avoids
/// re-implementing recursive serialization here.
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
