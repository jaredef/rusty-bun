//! Tier-Ω.5.v acceptance: complex assignment targets — destructuring
//! assignment (array / object patterns as AssignmentTarget, distinct from
//! binding declarations) plus compound assignment to computed-key and
//! nested member targets.

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

// 1. Static-key member compound: o.x += 5.
#[test]
fn t01_member_compound_static_key() {
    let rt = run_rt("const o = {x: 1}; o.x += 5; __record(o.x);");
    assert_eq!(last(&rt), Value::Number(6.0));
}

// 2. Computed-key compound: a[1] += 5.
#[test]
fn t02_member_compound_computed_key() {
    let rt = run_rt("const a = [10, 20, 30]; a[1] += 5; __record(a[1]);");
    assert_eq!(last(&rt), Value::Number(25.0));
}

// 3. Nested computed compound: a[0][1] += 10.
#[test]
fn t03_member_compound_nested_computed() {
    let rt = run_rt("const a = [[1,2],[3,4]]; a[0][1] += 10; __record(a[0][1]);");
    assert_eq!(last(&rt), Value::Number(12.0));
}

// 4. Nested static compound: o.n.m += 3.
#[test]
fn t04_member_compound_nested_static() {
    let rt = run_rt("const o = {n: {m: 5}}; o.n.m += 3; __record(o.n.m);");
    assert_eq!(last(&rt), Value::Number(8.0));
}

// 5. Computed key from a variable: o[k] *= 4.
#[test]
fn t05_member_compound_computed_var_key() {
    let rt = run_rt("const k = \"x\"; const o = {x: 1}; o[k] *= 4; __record(o.x);");
    assert_eq!(last(&rt), Value::Number(4.0));
}

// 6. Destructuring assignment — array pattern to existing locals.
#[test]
fn t06_destr_assign_array_to_locals() {
    let rt = run_rt("let a, b, c; [a, b, c] = [1, 2, 3]; __record(a === 1 && b === 2 && c === 3);");
    assert_eq!(last(&rt), Value::Boolean(true));
}

// 7. Destructuring assignment — parenthesized object pattern (kleur shape).
#[test]
fn t07_destr_assign_object_parenthesized() {
    let rt = run_rt("let x, y; ({x, y} = {x: 10, y: 20}); __record(x === 10 && y === 20);");
    assert_eq!(last(&rt), Value::Boolean(true));
}

// 8. Destructuring assignment — member-expression leaves.
#[test]
fn t08_destr_assign_to_members() {
    let rt = run_rt("const o = {}; [o.p, o.q] = [7, 8]; __record(o.p === 7 && o.q === 8);");
    assert_eq!(last(&rt), Value::Boolean(true));
}
