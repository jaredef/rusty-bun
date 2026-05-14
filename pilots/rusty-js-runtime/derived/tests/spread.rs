//! Tier-Ω.5.k acceptance: spread in object literals and spread call
//! arguments.
//!
//! Substrate-amortization signal per Doc 714 §VI Consequence 7: every test
//! here exercises a path that composes the Ω.5.k spread-helper substrate
//! (__object_spread / __array_push_single / __array_extend / __apply /
//! __construct) with existing object-literal, call, method-call, and `new`
//! lowering. Zero new AST nodes, zero new opcodes — pure closure over the
//! existing surface.

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

// 1. Object spread basic.
#[test]
fn t01_object_spread_basic() {
    let rt = run_rt("const o = {...{a:1, b:2}}; __record(o.a === 1 && o.b === 2);");
    assert_eq!(last(&rt), Value::Boolean(true));
}

// 2. Object spread with explicit keys, spread wins over earlier, later wins
//    over spread.
#[test]
fn t02_object_spread_with_keys() {
    let rt = run_rt(
        "const o = {x: 1, ...{x: 2, y: 3}, z: 4}; \
         __record(o.x === 2 && o.y === 3 && o.z === 4);");
    assert_eq!(last(&rt), Value::Boolean(true));
}

// 3. Multiple spreads compose left-to-right.
#[test]
fn t03_object_spread_multiple() {
    let rt = run_rt(
        "const o = {...{a:1}, ...{b:2}, ...{c:3}}; \
         __record(o.a === 1 && o.b === 2 && o.c === 3);");
    assert_eq!(last(&rt), Value::Boolean(true));
}

// 4. Later property overrides spread.
#[test]
fn t04_object_spread_override_order() {
    let rt = run_rt(
        "const o = {...{x:1, y:2}, x: 99}; \
         __record(o.x === 99 && o.y === 2);");
    assert_eq!(last(&rt), Value::Boolean(true));
}

// 5. Spread of empty object is a no-op.
#[test]
fn t05_object_spread_empty() {
    let rt = run_rt("const o = {...{}, x: 1}; __record(o.x === 1);");
    assert_eq!(last(&rt), Value::Boolean(true));
}

// 6. Spread args into a plain function call.
#[test]
fn t06_spread_args_basic() {
    let rt = run_rt(
        "function f(a, b, c) { return a+b+c; } \
         const args = [2, 3, 4]; \
         __record(f(...args));");
    assert_eq!(last(&rt), Value::Number(9.0));
}

// 7. Spread args sandwiched between leading and trailing fixed args.
#[test]
fn t07_spread_args_sandwich() {
    let rt = run_rt(
        "function f(a, b, c, d, e) { return a*10000+b*1000+c*100+d*10+e; } \
         __record(f(1, ...[2,3,4], 5));");
    assert_eq!(last(&rt), Value::Number(12345.0));
}

// 8. Multiple spreads in the argument list compose: positional pickup
//    from concatenated args.
#[test]
fn t08_spread_args_multiple() {
    let rt = run_rt(
        "function f(a, b, c, d, e) { return a*10000+b*1000+c*100+d*10+e; } \
         __record(f(...[1, 2], ...[3, 4, 5]));");
    assert_eq!(last(&rt), Value::Number(12345.0));
}

// 9. Spread in a method call: `this` must thread to the receiver.
#[test]
fn t09_spread_method_call() {
    let rt = run_rt(
        "const o = {sum: function(a, b) { return a + b; }}; \
         __record(o.sum(...[3, 4]));");
    assert_eq!(last(&rt), Value::Number(7.0));
}

// 10. Spread into Math.max — a native variadic.
#[test]
fn t10_spread_math_max() {
    let rt = run_rt("__record(Math.max(...[1, 5, 3, 9, 2]));");
    assert_eq!(last(&rt), Value::Number(9.0));
}

// 11. Stretch: spread in `new`.
#[test]
fn t11_spread_new() {
    let rt = run_rt(
        "class C { constructor(a, b) { this.s = a + b; } } \
         __record(new C(...[3, 4]).s);");
    assert_eq!(last(&rt), Value::Number(7.0));
}
