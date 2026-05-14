//! Tier-Ω.5.e acceptance: binding-shared closure capture per
//! ECMA-262 §8.1 / §10.2. Replaces the value-snapshot semantics from
//! Tier-Ω.5.c. Each captured outer-frame binding is a single shared
//! location — writes through any handle (outer frame, inner closure,
//! sibling closure) are visible to the others.

use rusty_js_runtime::{run_module, Runtime, Value};

fn run(src: &str) -> Value {
    run_module(src).unwrap_or_else(|e| panic!("run failed for {:?}: {:?}", src, e))
}

fn run_rt(src: &str) -> (Runtime, Value) {
    let module = rusty_js_bytecode::compile_module(src)
        .unwrap_or_else(|e| panic!("compile: {:?}", e));
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    let v = rt.run_module(&module).unwrap_or_else(|e| panic!("run: {:?}", e));
    (rt, v)
}

// 1. Counter factory — function-expression form.
#[test]
fn t01_counter_factory_fn() {
    let src = r#"
        function c(){ let n = 0; return function(){ n = n + 1; return n; }; }
        let f = c();
        f(); f();
        return f();
    "#;
    assert_eq!(run(src), Value::Number(3.0));
}

// 2. Counter factory — arrow form with prefix increment.
#[test]
fn t02_counter_factory_arrow() {
    let src = r#"
        function c(){ let n = 0; return () => ++n; }
        let f = c();
        f(); f();
        return f();
    "#;
    assert_eq!(run(src), Value::Number(3.0));
}

// 3. Isolation across factory calls — each call yields a fresh binding.
#[test]
fn t03_factory_isolation() {
    let src = r#"
        function c(){ let n = 0; return () => ++n; }
        let f = c();
        let g = c();
        f(); f();
        return g();
    "#;
    assert_eq!(run(src), Value::Number(1.0));
}

// 4. Outer-frame visibility: outer `let` sees writes through closure.
#[test]
fn t04_outer_frame_visibility() {
    let src = r#"
        let n = 0;
        let inc = () => { n = n + 1; };
        inc(); inc();
        return n;
    "#;
    assert_eq!(run(src), Value::Number(2.0));
}

// 5. Two closures sharing one binding.
#[test]
fn t05_shared_binding_pair() {
    let src = r#"
        function pair(){ let n = 0; return [() => ++n, () => n]; }
        let p = pair();
        let inc = p[0];
        let get = p[1];
        inc(); inc(); inc();
        return get();
    "#;
    assert_eq!(run(src), Value::Number(3.0));
}

// 6. Transitive capture three deep. The innermost closure mutates the
// outermost binding through two layers. We grab the innermost closure
// via chained immediate calls (a pre-existing compiler limitation makes
// the let-intermediated form fail to bind nested function constants
// correctly; that is separate from the runtime upvalue cell mechanism
// under test here).
#[test]
fn t06_transitive_three_deep() {
    let src = r#"
        function a(){
            let x = 0;
            return function(){
                return function(){ x = x + 1; return x; };
            };
        }
        let inner = a()();
        inner(); inner();
        return inner();
    "#;
    assert_eq!(run(src), Value::Number(3.0));
}

// 7. forEach accumulator over outer let.
#[test]
fn t07_foreach_accumulator() {
    let src = r#"
        let n = 0;
        [1,2,3].forEach(x => { n = n + x; });
        __record(n);
    "#;
    let (rt, _) = run_rt(src);
    let recorded = rt.globals.get("__last_recorded").cloned().unwrap_or(Value::Undefined);
    assert_eq!(recorded, Value::Number(6.0));
}

// 8. Map side-effect into outer-let trail string.
#[test]
fn t08_map_side_effect_trail() {
    let src = r#"
        let trail = "";
        [1,2,3].map(x => { trail = trail + x; return x * 2; });
        return trail;
    "#;
    if let Value::String(s) = run(src) { assert_eq!(s.as_str(), "123"); }
    else { panic!("expected string"); }
}

// 9. Reduce canary — built-in accumulator. Should remain green.
#[test]
fn t09_reduce_canary() {
    let src = r#"
        return [1,2,3].reduce((acc, x) => acc + x, 0);
    "#;
    assert_eq!(run(src), Value::Number(6.0));
}

// 10. for-of per-iteration binding for `const`. ECMA-262 §14.7.5.5
// requires a fresh binding per iteration for `let`/`const` heads.
// Delivered Tier-Ω.5.g.1 via Op::ResetLocalCell: at each iteration entry
// the compiler detaches the previous iteration's upvalue cell from the
// frame slot, so closures captured in iteration N retain their handle
// to N's cell while iteration N+1 promotes to a fresh one.
#[test]
fn t10_for_of_per_iteration_binding() {
    let src = r#"
        let closures = [];
        for (const i of [1,2,3]) { closures.push(() => i); }
        return closures[0]() + "," + closures[1]() + "," + closures[2]();
    "#;
    if let Value::String(s) = run(src) { assert_eq!(s.as_str(), "1,2,3"); }
    else { panic!("expected string"); }
}

// 10b. Same as t10 but with `let` head — let and const behave identically
// under §14.7.5.5 for-of per-iteration binding.
#[test]
fn t10b_for_of_let_head_per_iteration() {
    let src = r#"
        let closures = [];
        for (let i of [1,2,3]) { closures.push(() => i); }
        return closures[0]() + "," + closures[1]() + "," + closures[2]();
    "#;
    if let Value::String(s) = run(src) { assert_eq!(s.as_str(), "1,2,3"); }
    else { panic!("expected string"); }
}

// 10c. `var` head in for-of stays function-scoped and shared across
// iterations per §14.7.5.5 — only let/const get per-iteration fresh
// bindings. Locks in the var-vs-let distinction.
#[test]
fn t10c_for_of_var_head_shared() {
    let src = r#"
        let closures = [];
        for (var i of [1,2,3]) { closures.push(() => i); }
        return closures[0]() + "," + closures[1]() + "," + closures[2]();
    "#;
    if let Value::String(s) = run(src) { assert_eq!(s.as_str(), "3,3,3"); }
    else { panic!("expected string"); }
}

// 10d. continue mid-loop: skip iteration where i === 2; first/third
// closures still capture their iteration's binding.
#[test]
fn t10d_for_of_per_iteration_with_continue() {
    let src = r#"
        let closures = [];
        for (const i of [1,2,3]) {
            if (i === 2) { continue; }
            closures.push(() => i);
        }
        return closures[0]() + "," + closures[1]();
    "#;
    if let Value::String(s) = run(src) { assert_eq!(s.as_str(), "1,3"); }
    else { panic!("expected string"); }
}

// 10e. break mid-loop: closures collected before the break observe
// their own iteration's binding, not the loop's exit value.
#[test]
fn t10e_for_of_per_iteration_with_break() {
    let src = r#"
        let closures = [];
        for (const i of [1,2,3,4]) {
            closures.push(() => i);
            if (i === 2) { break; }
        }
        return closures[0]() + "," + closures[1]();
    "#;
    if let Value::String(s) = run(src) { assert_eq!(s.as_str(), "1,2"); }
    else { panic!("expected string"); }
}

// 11. Closure outlives the creating frame.
#[test]
fn t11_closure_outlives_frame() {
    let src = r#"
        function make(){ let x = 42; return () => x; }
        let f = make();
        return f();
    "#;
    assert_eq!(run(src), Value::Number(42.0));
}

// 12. Two factory invocations produce independent bindings — writes
// through a's closure-pair don't affect b's.
#[test]
fn t12_independent_bindings_across_factory_calls() {
    let src = r#"
        function make(){ let x = 0; return { inc: () => { x = x + 1; }, get: () => x }; }
        let a = make();
        let b = make();
        a.inc(); a.inc();
        b.inc();
        return a.get() * 10 + b.get();
    "#;
    // a.get == 2, b.get == 1 -> 21
    assert_eq!(run(src), Value::Number(21.0));
}
