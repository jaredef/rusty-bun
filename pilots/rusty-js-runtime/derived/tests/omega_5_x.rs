//! Tier-Ω.5.x acceptance: class-name binding visible inside method
//! bodies (so `static foo() { return ThisClass.bar; }` works), plus
//! `await expr` lowers as a no-op (v1 deviation: suspension semantics
//! dropped; the argument's value passes through unchanged).

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

// ─── Class-name binding inside method bodies ─────────────────────────

#[test]
fn class_name_visible_in_public_static_method() {
    let rt = run_rt("class D { static count = 100; static getIt() { return D.count; } } __record(D.getIt());");
    assert_eq!(last(&rt), Value::Number(100.0));
}

#[test]
fn class_name_visible_in_private_static_method() {
    let rt = run_rt("class D { static #count = 50; static getPriv() { return D.#count; } } __record(D.getPriv());");
    assert_eq!(last(&rt), Value::Number(50.0));
}

#[test]
fn class_name_visible_in_instance_method() {
    let rt = run_rt("class K { static label = \"K\"; getLabel() { return K.label; } } __record(new K().getLabel());");
    assert_eq!(last(&rt), Value::String("K".to_string().into()));
}

// ─── Await as no-op ─────────────────────────────────────────────────

#[test]
fn await_returns_argument_unchanged() {
    let rt = run_rt("async function f(x) { return await x; } __record(f(7));");
    // v1 deviation: returns the value directly, not a Promise.
    assert_eq!(last(&rt), Value::Number(7.0));
}

#[test]
fn await_in_expression_position() {
    let rt = run_rt("async function f() { const x = await 42; return x + 1; } __record(f());");
    assert_eq!(last(&rt), Value::Number(43.0));
}

#[test]
fn await_typeof_passes_through() {
    let rt = run_rt("async function f() { return typeof await \"hello\"; } __record(f());");
    assert_eq!(last(&rt), Value::String("string".to_string().into()));
}
