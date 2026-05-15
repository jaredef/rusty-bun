//! Tier-Ω.5.y acceptance: computed class member names + node:zlib/tty stubs.

use rusty_js_runtime::{Runtime, Value};

fn run_rt(src: &str) -> Runtime {
    let module = rusty_js_bytecode::compile_module(src)
        .unwrap_or_else(|e| panic!("compile {:?}: {:?}", src, e));
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    rt.run_module(&module).unwrap_or_else(|e| panic!("run {:?}: {:?}", src, e));
    rt
}

fn last(rt: &Runtime) -> Value {
    rt.globals.get("__last_recorded").cloned().unwrap_or(Value::Undefined)
}

#[test]
fn computed_class_method_via_variable() {
    let rt = run_rt("const k = \"foo\"; class C { [k]() { return 7; } } __record(new C().foo());");
    assert_eq!(last(&rt), Value::Number(7.0));
}

#[test]
fn computed_class_method_via_expression() {
    let rt = run_rt("class C { [\"bar\" + 1]() { return 99; } } __record(new C().bar1());");
    assert_eq!(last(&rt), Value::Number(99.0));
}

#[test]
fn computed_class_static_method() {
    let rt = run_rt("const k = \"compute\"; class C { static [k]() { return 100; } } __record(C.compute());");
    assert_eq!(last(&rt), Value::Number(100.0));
}

#[test]
fn computed_class_field() {
    let rt = run_rt("const k = \"x\"; class C { [k] = 42; } __record(new C().x);");
    assert_eq!(last(&rt), Value::Number(42.0));
}

#[test]
fn computed_class_static_field() {
    let rt = run_rt("const k = \"y\"; class C { static [k] = 13; } __record(C.y);");
    assert_eq!(last(&rt), Value::Number(13.0));
}
