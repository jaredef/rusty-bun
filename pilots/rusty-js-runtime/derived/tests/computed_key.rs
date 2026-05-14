//! Tier-Ω.5.o acceptance — computed object keys `{[expr]: value}`.

use rusty_js_runtime::{Runtime, Value};

fn run_rt(src: &str) -> Runtime {
    let module = rusty_js_bytecode::compile_module(src)
        .unwrap_or_else(|e| panic!("compile: {:?}", e));
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    rt.run_module(&module).unwrap_or_else(|e| panic!("run: {:?}", e));
    rt
}

fn recorded(rt: &Runtime) -> Value {
    rt.globals.get("__last_recorded").cloned().unwrap()
}

#[test]
fn t01_basic_computed_key() {
    let rt = run_rt(r#"
        const o = {[("x")]: 1};
        __record(o.x);
    "#);
    assert_eq!(recorded(&rt), Value::Number(1.0));
}

#[test]
fn t02_computed_key_from_variable() {
    let rt = run_rt(r#"
        const k = "foo";
        const o = {[k]: 42};
        __record(o.foo);
    "#);
    assert_eq!(recorded(&rt), Value::Number(42.0));
}

#[test]
fn t03_numeric_computed_key() {
    let rt = run_rt(r#"
        const o = {[42]: "a"};
        __record(o[42]);
    "#);
    if let Value::String(s) = recorded(&rt) { assert_eq!(s.as_str(), "a"); } else { panic!(); }
}

#[test]
fn t04_mix_static_and_computed() {
    let rt = run_rt(r#"
        const k = "a";
        const o = {[k]: 1, b: 2};
        __record(o.a + o.b);
    "#);
    assert_eq!(recorded(&rt), Value::Number(3.0));
}

#[test]
fn t05_computed_key_from_expression() {
    let rt = run_rt(r#"
        const o = {["pre" + "fix"]: 7};
        __record(o.prefix);
    "#);
    assert_eq!(recorded(&rt), Value::Number(7.0));
}
