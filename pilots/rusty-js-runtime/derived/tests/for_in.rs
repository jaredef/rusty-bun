//! Tier-Ω.5.m acceptance — for-in over own enumerable string keys.
//!
//! Spec deviations (documented):
//!   - Own enumerable keys only; no proto-chain walk.
//!   - No Symbol-key exclusion (we don't ship real Symbols).
//!   - Enumeration order delegated to Object.keys (integer-like first in
//!     ascending order for Array internals; insertion order otherwise).

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

// 1. Basic: collects all own keys.
#[test]
fn t01_basic_keys() {
    let rt = run_rt(r#"
        const o = {a:1, b:2, c:3};
        const ks = Object.keys(o).join("");
        let r = "";
        for (const k in o) r = r + k;
        // Compare against Object.keys' own enumeration so the test is
        // robust against the underlying property-order convention.
        __record(r === ks);
    "#);
    assert_eq!(recorded(&rt), Value::Boolean(true));
}

// 2. for-in mirrors Object.keys order exactly.
#[test]
fn t02_matches_object_keys() {
    let rt = run_rt(r#"
        const o = {3: "x", 1: "y", name: "z"};
        const ks = Object.keys(o).join(",");
        let r = "";
        for (const k in o) r = r + k + ",";
        // Trim trailing comma to match Object.keys.join.
        r = r.slice(0, r.length - 1);
        __record(r === ks);
    "#);
    assert_eq!(recorded(&rt), Value::Boolean(true));
}

// 3. `let` head — per-iteration fresh binding (closures see their own key).
#[test]
fn t03_let_head_per_iter() {
    let rt = run_rt(r#"
        const cs = [];
        const o = {a:1, b:2, c:3};
        for (let k in o) cs.push(() => k);
        const expect = Object.keys(o).join("");
        const got = cs.map(f => f()).join("");
        __record(got === expect);
    "#);
    assert_eq!(recorded(&rt), Value::Boolean(true));
}

// 4. `var` head — function-scoped, shared across iterations.
#[test]
fn t04_var_head_shared() {
    let rt = run_rt(r#"
        const cs = [];
        const o = {a:1, b:2, c:3};
        for (var k in o) cs.push(() => k);
        const ks = Object.keys(o);
        const last = ks[ks.length - 1];
        const got = cs.map(f => f()).join("");
        const expect = last + last + last;
        __record(got === expect);
    "#);
    assert_eq!(recorded(&rt), Value::Boolean(true));
}

// 5. Empty object — body does not execute.
#[test]
fn t05_empty_object() {
    let rt = run_rt(r#"
        let n = 0;
        for (const k in {}) n = n + 1;
        __record(n);
    "#);
    assert_eq!(recorded(&rt), Value::Number(0.0));
}

// 6. Index into object using the key.
#[test]
fn t06_index_via_key() {
    let rt = run_rt(r#"
        const o = {x: 1, y: 2};
        let s = 0;
        for (const k in o) s = s + o[k];
        __record(s);
    "#);
    assert_eq!(recorded(&rt), Value::Number(3.0));
}
