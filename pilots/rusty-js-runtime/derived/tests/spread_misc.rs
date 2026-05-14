//! Tier-Ω.5.l: small closure surfaces bundled as one round.
//!   - rest parameter (`function f(...all)`)
//!   - object method shorthand (`{ name(p) { body } }`)
//!   - array literal spread (`[...arr]`)

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
fn rest_parameter_collects() {
    let rt = run_rt("function f(...all) { return all.length; } __record(f(1,2,3,4));");
    assert_eq!(last(&rt), Value::Number(4.0));
}

#[test]
fn rest_parameter_after_named() {
    let rt = run_rt("function f(a, b, ...rest) { return a + b + rest.length; } __record(f(10, 20, 1, 2, 3));");
    assert_eq!(last(&rt), Value::Number(33.0));
}

#[test]
fn rest_parameter_empty() {
    let rt = run_rt("function f(...all) { return all.length; } __record(f());");
    assert_eq!(last(&rt), Value::Number(0.0));
}

#[test]
fn rest_parameter_indexed_access() {
    let rt = run_rt("function f(...all) { return all[0] + all[1]; } __record(f(7, 35));");
    assert_eq!(last(&rt), Value::Number(42.0));
}

#[test]
fn rest_parameter_arrow() {
    let rt = run_rt("const f = (...all) => all.length; __record(f(1,2,3));");
    assert_eq!(last(&rt), Value::Number(3.0));
}

#[test]
fn method_shorthand_basic() {
    let rt = run_rt("const o = { add(a, b) { return a + b; } }; __record(o.add(3, 4));");
    assert_eq!(last(&rt), Value::Number(7.0));
}

#[test]
fn method_shorthand_this() {
    let rt = run_rt("const o = { x: 7, get() { return this.x; } }; __record(o.get());");
    assert_eq!(last(&rt), Value::Number(7.0));
}

#[test]
fn method_shorthand_multiple_methods_call_each_other() {
    let rt = run_rt("const o = { a() { return 1; }, b() { return 2; }, c() { return this.a() + this.b(); } }; __record(o.c());");
    assert_eq!(last(&rt), Value::Number(3.0));
}

#[test]
fn array_spread_basic() {
    let rt = run_rt("__record([...[1,2,3]].length);");
    assert_eq!(last(&rt), Value::Number(3.0));
}

#[test]
fn array_spread_with_other_elements() {
    let rt = run_rt("__record([0, ...[1,2,3], 4].length);");
    assert_eq!(last(&rt), Value::Number(5.0));
}

#[test]
fn array_spread_sums_across_boundaries() {
    let rt = run_rt("const a = [...[1,2,3], ...[4,5]]; __record(a[0] + a[4]);");
    assert_eq!(last(&rt), Value::Number(6.0));
}

#[test]
fn array_spread_of_array_iterable() {
    // Arrays expose @@iterator from %Array.prototype% (Ω.5.c).
    // String-as-iterable is deferred — collect_iterable currently only
    // walks Value::Object iterators; primitives need a String-prototype
    // routing hop that isn't yet wired.
    let rt = run_rt("const inner = [10, 20, 30]; const a = [0, ...inner, 40]; __record(a.length);");
    assert_eq!(last(&rt), Value::Number(5.0));
}
