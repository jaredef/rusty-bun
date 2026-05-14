//! Tier-Ω.5.g.3 acceptance: destructuring pattern lowering.
//!
//! Substrate-amortization signal per Doc 714 §VI Consequence 7: every test
//! here exercises a path that composes the Ω.5.g.2 substrate
//! (BindingPattern + BindingElement + ObjectPatternProperty + PropertyKey
//! + collect_names) with the existing identifier/parameter/for-of lowering.

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

// 1. Plain array destructure.
#[test]
fn t01_array_basic() {
    let rt = run_rt("const [a, b] = [1, 2]; __record(a === 1 && b === 2);");
    assert_eq!(last(&rt), Value::Boolean(true));
}

// 2. Array destructure with missing tail → undefined.
#[test]
fn t02_array_short_source() {
    let rt = run_rt("const [a, b, c] = [1, 2]; __record(c);");
    assert_eq!(last(&rt), Value::Undefined);
}

// 3. Default applies when source slot is undefined.
#[test]
fn t03_array_default_applied() {
    let rt = run_rt("const [a, b = 99] = [1]; __record(b);");
    assert_eq!(last(&rt), Value::Number(99.0));
}

// 4. Default skipped when source provides value.
#[test]
fn t04_array_default_skipped() {
    let rt = run_rt("const [a, b = 99] = [1, 5]; __record(b);");
    assert_eq!(last(&rt), Value::Number(5.0));
}

// 5. Array rest.
#[test]
fn t05_array_rest() {
    let rt = run_rt(
        "const [a, ...rest] = [1, 2, 3, 4]; \
         __record(rest.length === 3 && rest[0] === 2 && rest[2] === 4);"
    );
    assert_eq!(last(&rt), Value::Boolean(true));
}

// 6. Object destructure shorthand.
#[test]
fn t06_object_shorthand() {
    let rt = run_rt("const {x, y} = {x: 1, y: 2}; __record(x === 1 && y === 2);");
    assert_eq!(last(&rt), Value::Boolean(true));
}

// 7. Object destructure with rename.
#[test]
fn t07_object_rename() {
    let rt = run_rt("const {a: alias} = {a: 7}; __record(alias);");
    assert_eq!(last(&rt), Value::Number(7.0));
}

// 8. Object destructure default.
#[test]
fn t08_object_default() {
    let rt = run_rt("const {x = 99} = {}; __record(x);");
    assert_eq!(last(&rt), Value::Number(99.0));
}

// 9. Parameter destructure.
#[test]
fn t09_param_array() {
    let rt = run_rt("function f([a, b]) { return a + b; } __record(f([3, 4]));");
    assert_eq!(last(&rt), Value::Number(7.0));
}

// 10. for-of with destructure head.
#[test]
fn t10_forof_destructure_head() {
    let rt = run_rt(
        "let sum = 0; for (const [a, b] of [[1,2],[3,4]]) { sum = sum + a + b; } __record(sum);"
    );
    assert_eq!(last(&rt), Value::Number(10.0));
}

// 11. Object rest collects remaining keys.
#[test]
fn t11_object_rest() {
    let rt = run_rt(
        "const {a, b, ...rest} = {a:1, b:2, c:3, d:4}; \
         __record(rest.c === 3 && rest.d === 4 && rest.a === undefined && rest.b === undefined);"
    );
    assert_eq!(last(&rt), Value::Boolean(true));
}

// 12. Nested pattern: {a: [b, c]}.
#[test]
fn t12_nested_pattern() {
    let rt = run_rt("const {a: [b, c]} = {a: [10, 20]}; __record(b === 10 && c === 20);");
    assert_eq!(last(&rt), Value::Boolean(true));
}

// 13. Array elision holes.
#[test]
fn t13_array_elision() {
    let rt = run_rt("const [, b, , d] = [1, 2, 3, 4]; __record(b === 2 && d === 4);");
    assert_eq!(last(&rt), Value::Boolean(true));
}
