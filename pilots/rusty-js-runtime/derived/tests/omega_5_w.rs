//! Tier-Ω.5.w acceptance: Symbol() callable + async/generator class methods
//! + private class fields. All three are v1-deviation closures:
//!   - Symbol() returns Value::String "@@sym:<counter>:<desc>"; not a real
//!     primitive distinct from string at the value-tag level.
//!   - Async / generator class methods compile as ordinary functions;
//!     await/yield inside the body still error.
//!   - Private fields are name-mangled to "#name" properties; privacy is
//!     not enforced.

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

// ─── Symbol() callable ──────────────────────────────────────────────

#[test]
fn symbol_callable_typeof() {
    let rt = run_rt("__record(typeof Symbol);");
    assert_eq!(last(&rt), Value::String("function".to_string().into()));
}

#[test]
fn symbol_returns_string() {
    let rt = run_rt("__record(typeof Symbol(\"foo\"));");
    assert_eq!(last(&rt), Value::String("string".to_string().into()));
}

#[test]
fn symbol_distinct_calls_unique() {
    let rt = run_rt("__record(Symbol(\"x\") !== Symbol(\"x\"));");
    assert_eq!(last(&rt), Value::Boolean(true));
}

#[test]
fn symbol_iterator_preserved() {
    let rt = run_rt("__record(typeof Symbol.iterator);");
    assert_eq!(last(&rt), Value::String("string".to_string().into()));
}

// ─── async / generator class methods ────────────────────────────────

#[test]
fn class_async_method_compiles() {
    let rt = run_rt("class C { async foo() { return 1; } } __record(typeof new C().foo);");
    assert_eq!(last(&rt), Value::String("function".to_string().into()));
}

#[test]
fn class_generator_method_compiles() {
    let rt = run_rt("class C { *bar() { return 2; } } __record(typeof new C().bar);");
    assert_eq!(last(&rt), Value::String("function".to_string().into()));
}

#[test]
fn class_static_async_method_compiles() {
    let rt = run_rt("class C { static async baz() { return 3; } } __record(typeof C.baz);");
    assert_eq!(last(&rt), Value::String("function".to_string().into()));
}

// ─── private class fields + methods ──────────────────────────────────

#[test]
fn private_field_with_initializer() {
    let rt = run_rt("class C { #x = 7; reveal() { return this.#x; } } __record(new C().reveal());");
    assert_eq!(last(&rt), Value::Number(7.0));
}

#[test]
fn private_method_callable_within_class() {
    let rt = run_rt("class C { #m() { return 1; } call_m() { return this.#m(); } } __record(new C().call_m());");
    assert_eq!(last(&rt), Value::Number(1.0));
}

#[test]
fn private_field_set_in_constructor() {
    let rt = run_rt("class C { #x; constructor() { this.#x = 42; } reveal() { return this.#x; } } __record(new C().reveal());");
    assert_eq!(last(&rt), Value::Number(42.0));
}

#[test]
fn multiple_private_fields_sum() {
    let rt = run_rt("class C { #a = 1; #b = 2; sum() { return this.#a + this.#b; } } __record(new C().sum());");
    assert_eq!(last(&rt), Value::Number(3.0));
}
