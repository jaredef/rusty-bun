//! Tier-Ω.5.s: Number static constants + predicates.
//!
//! Acceptance items (per Tier-Ω.5.s locked design):
//!   (1) Number.MAX_SAFE_INTEGER === 9007199254740991
//!   (2) Number.EPSILON > 0 && < 0.001
//!   (3) Number.isInteger predicate semantics
//!   (4) Number.isSafeInteger boundary at 2**53
//!   (5) Number.isFinite — typeof-gated, no coercion
//!   (6) Number.isNaN — typeof-gated, no coercion

use rusty_js_runtime::{Runtime, Value};

fn fresh() -> Runtime {
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    rt
}

fn run_eval(rt: &mut Runtime, src: &str) -> Value {
    let url = format!("file:///tmp/number_statics_{}.mjs",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let wrapped = format!("const __r = {};\nexport {{ __r }};\n", src);
    let ns = rt.evaluate_module(&wrapped, &url).expect("evaluate_module");
    rt.object_get(ns, "__r")
}

#[test]
fn t01_max_safe_integer() {
    let mut rt = fresh();
    let v = run_eval(&mut rt, "Number.MAX_SAFE_INTEGER === 9007199254740991");
    assert!(matches!(v, Value::Boolean(true)), "got {:?}", v);
}

#[test]
fn t02_epsilon_positive_small() {
    let mut rt = fresh();
    let v = run_eval(&mut rt, "Number.EPSILON > 0 && Number.EPSILON < 0.001");
    assert!(matches!(v, Value::Boolean(true)), "got {:?}", v);
}

#[test]
fn t03_is_integer_semantics() {
    let mut rt = fresh();
    let v = run_eval(&mut rt,
        "Number.isInteger(42) && !Number.isInteger(42.5) && !Number.isInteger(\"42\")");
    assert!(matches!(v, Value::Boolean(true)), "got {:?}", v);
}

#[test]
fn t04_is_safe_integer_boundary() {
    let mut rt = fresh();
    // 2**53 - 1 is safe; 2**53 is not.
    let v = run_eval(&mut rt,
        "Number.isSafeInteger(9007199254740991) && !Number.isSafeInteger(9007199254740992)");
    assert!(matches!(v, Value::Boolean(true)), "got {:?}", v);
}

#[test]
fn t05_is_finite_typeof_gated() {
    let mut rt = fresh();
    let v = run_eval(&mut rt,
        "Number.isFinite(1) && !Number.isFinite(Number.POSITIVE_INFINITY) && !Number.isFinite(\"1\")");
    assert!(matches!(v, Value::Boolean(true)), "got {:?}", v);
}

#[test]
fn t06_is_nan_typeof_gated() {
    let mut rt = fresh();
    let v = run_eval(&mut rt,
        "Number.isNaN(Number.NaN) && !Number.isNaN(\"NaN\")");
    assert!(matches!(v, Value::Boolean(true)), "got {:?}", v);
}
