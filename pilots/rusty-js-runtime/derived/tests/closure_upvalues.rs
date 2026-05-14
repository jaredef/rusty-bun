//! Tier-Ω.5.c Stage 1 acceptance: closure upvalues (read-path).
//!
//! Tier-Ω.5.e: closures are now binding-shared (ECMA-262 §8.1 / §10.2).
//! The two tests that encoded the value-capture deviation (t03, t04)
//! were updated to assert spec-faithful semantics. The detailed
//! binding-shared acceptance bar lives in tests/binding_capture.rs.

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

// 1. Inner arrow reads outer local.
#[test]
fn t01_arrow_reads_outer_local() {
    assert_eq!(run("let n = 7; let f = () => n; return f();"), Value::Number(7.0));
}

// 2. Inner function reads outer parameter.
#[test]
fn t02_function_reads_outer_param() {
    assert_eq!(run("function outer(x){ let inner = function(){ return x + 1; }; return inner(); } return outer(10);"),
        Value::Number(11.0));
}

// 3. forEach accumulator via outer let — the headline acceptance case.
#[test]
fn t03_foreach_accumulator() {
    let src = r#"
        let n = 0;
        [1,2,3].forEach(x => { n = n + x; });
        __record(n);
    "#;
    let (rt, _) = run_rt(src);
    // v1 value-capture: the arrow's `n` is a snapshot. Mutation inside the
    // arrow does NOT propagate to the outer `n` for v1. This deviation is
    // documented; the test verifies engine *runs* this code without panic
    // and the recorded value is the snapshot result.
    let recorded = rt.globals.get("__last_recorded").cloned().unwrap_or(Value::Undefined);
    // Binding-shared: arrow's writes propagate to outer `n`. 0+1+2+3 = 6.
    assert_eq!(recorded, Value::Number(6.0));
}

// 4. Counter factory — exercises capture-of-param.
#[test]
fn t04_counter_factory_reads() {
    // Binding-shared: f();f();f() now returns 3 per spec.
    let src = r#"
        function counter() { let c = 0; return () => ++c; }
        let f = counter();
        f(); f();
        return f();
    "#;
    assert_eq!(run(src), Value::Number(3.0));
}

// 5. Multiple levels of nesting (transitive upvalue).
#[test]
fn t05_transitive_upvalue() {
    let src = r#"
        function outer(){
            let x = 42;
            function mid(){
                function inner(){ return x; }
                return inner();
            }
            return mid();
        }
        return outer();
    "#;
    assert_eq!(run(src), Value::Number(42.0));
}

// 6. Closure captures across map callback — read-only pattern works.
#[test]
fn t06_map_with_outer_factor() {
    let src = r#"
        let factor = 3;
        return [1,2,3].map(x => x * factor).join(",");
    "#;
    if let Value::String(s) = run(src) {
        assert_eq!(s.as_str(), "3,6,9");
    } else { panic!(); }
}

// 7. Captured value lives independently per closure invocation.
#[test]
fn t07_per_call_capture() {
    let src = r#"
        function make(v) { return () => v; }
        return make("a")() + make("b")();
    "#;
    if let Value::String(s) = run(src) { assert_eq!(s.as_str(), "ab"); } else { panic!(); }
}

// 8. Closure capturing nothing — degenerate case still works.
#[test]
fn t08_no_captures() {
    assert_eq!(run("let f = () => 99; return f();"), Value::Number(99.0));
}

// 9. Inner closure inside method body — `this` and captures coexist.
#[test]
fn t09_method_body_closure() {
    let src = r#"
        let obj = { v: 5, get: function(){ let local = this.v; return (() => local)(); } };
        return obj.get();
    "#;
    assert_eq!(run(src), Value::Number(5.0));
}
