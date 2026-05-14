//! Tier-Ω.5.a acceptance bar: prototype-chain instance-method dispatch
//! with proper `this` threading. Covers Object/Array/String/Function/
//! Promise/Number prototypes. Numbered to the round's spec.

use rusty_js_runtime::{run_module, Runtime, Value};
use rusty_js_runtime::interp::RuntimeError;

fn run(src: &str) -> Value {
    run_module(src).unwrap_or_else(|e| panic!("run failed for {:?}: {:?}", src, e))
}

fn run_drain(src: &str) -> Value {
    let module = rusty_js_bytecode::compile_module(src)
        .unwrap_or_else(|e| panic!("compile: {:?}", e));
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    rt.run_module(&module).unwrap_or_else(|e| panic!("run: {:?}", e));
    // Drain microtasks to fire Promise reaction chains.
    let _ = rt.run_to_completion();
    rt.globals.get("__last_recorded").cloned().unwrap_or(Value::Undefined)
}

// ─────────── 1. ({}).toString() === "[object Object]" ───────────
#[test]
fn t01_object_to_string() {
    if let Value::String(s) = run("return ({}).toString();") {
        assert_eq!(s.as_str(), "[object Object]");
    } else { panic!("not a string"); }
}

// ─────────── 2. ({a:1}).hasOwnProperty("a") === true ───────────
#[test]
fn t02_object_has_own_property() {
    assert_eq!(run("return ({a:1}).hasOwnProperty(\"a\");"), Value::Boolean(true));
    assert_eq!(run("return ({a:1}).hasOwnProperty(\"b\");"), Value::Boolean(false));
}

// ─────────── 3. [1,2,3].map(x => x * 2) -> [2,4,6] ───────────
#[test]
fn t03_array_map() {
    // Check elements via join (avoid array equality dance).
    if let Value::String(s) = run("return [1,2,3].map(x => x * 2).join(\",\");") {
        assert_eq!(s.as_str(), "2,4,6");
    } else { panic!(); }
}

// ─────────── 4. [1,2,3].forEach(fn) fires three times ───────────
#[test]
fn t04_array_forEach() {
    // forEach must invoke the callback 3 times. Engine v1 does not yet
    // capture outer locals as upvalues, so we use a global-scoped slot
    // (assignment to an undeclared identifier writes globalThis) and
    // verify via __record at the end.
    run(r#"
        totalForEach = 0;
        [10, 20, 30].forEach(function(v){ totalForEach = totalForEach + v; });
        __record(totalForEach);
    "#);
    // Re-execute via a Runtime we can inspect.
    let module = rusty_js_bytecode::compile_module(r#"
        totalForEach = 0;
        [10, 20, 30].forEach(function(v){ totalForEach = totalForEach + v; });
        __record(totalForEach);
    "#).unwrap();
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    rt.run_module(&module).unwrap();
    assert_eq!(rt.globals.get("__last_recorded").cloned().unwrap_or(Value::Undefined),
        Value::Number(60.0));
}

// ─────────── 5. [1,2,3].length === 3 ───────────
#[test]
fn t05_array_length() {
    assert_eq!(run("return [1,2,3].length;"), Value::Number(3.0));
}

// ─────────── 6. [1,2,3].push(4) mutates + returns 4 (new length) ───────────
#[test]
fn t06_array_push() {
    if let Value::String(s) = run(r#"
        let a = [1,2,3];
        let r = a.push(4);
        return r + ":" + a.length + ":" + a.join(",");
    "#) {
        assert_eq!(s.as_str(), "4:4:1,2,3,4");
    } else { panic!(); }
}

// ─────────── 7. [1,2,3].indexOf(2) === 1 ───────────
#[test]
fn t07_array_indexOf() {
    assert_eq!(run("return [1,2,3].indexOf(2);"), Value::Number(1.0));
    assert_eq!(run("return [1,2,3].indexOf(99);"), Value::Number(-1.0));
}

// ─────────── 8. "hello".toUpperCase() === "HELLO" ───────────
#[test]
fn t08_string_to_upper_case() {
    if let Value::String(s) = run("return \"hello\".toUpperCase();") {
        assert_eq!(s.as_str(), "HELLO");
    } else { panic!(); }
}

// ─────────── 9. "hello".length === 5 ───────────
#[test]
fn t09_string_length() {
    assert_eq!(run("return \"hello\".length;"), Value::Number(5.0));
}

// ─────────── 10. "hello".charAt(1) === "e" ───────────
#[test]
fn t10_string_charAt() {
    if let Value::String(s) = run("return \"hello\".charAt(1);") {
        assert_eq!(s.as_str(), "e");
    } else { panic!(); }
}

// ─────────── 11. "hello".slice(1,3) === "el" ───────────
#[test]
fn t11_string_slice() {
    if let Value::String(s) = run("return \"hello\".slice(1, 3);") {
        assert_eq!(s.as_str(), "el");
    } else { panic!(); }
}

// ─────────── 12. Promise.resolve(42).then(x => x * 2).then(__record) records 84 ───────────
#[test]
fn t12_promise_then_chain_instance_form() {
    let v = run_drain(r#"
        Promise.resolve(42).then(function(x){ return x * 2; }).then(function(x){ __record(x); });
    "#);
    assert_eq!(v, Value::Number(84.0));
}

// ─────────── 13. fn.call(thisVal, ...args) ───────────
#[test]
fn t13_function_call() {
    if let Value::Number(n) = run(r#"
        function add(a, b) { return this + a + b; }
        return add.call(10, 1, 2);
    "#) {
        assert_eq!(n, 13.0);
    } else { panic!(); }
}

// ─────────── 14. fn.apply(thisVal, [args]) ───────────
#[test]
fn t14_function_apply() {
    if let Value::Number(n) = run(r#"
        function add(a, b) { return this + a + b; }
        return add.apply(100, [3, 4]);
    "#) {
        assert_eq!(n, 107.0);
    } else { panic!(); }
}

// ─────────── stretch: array filter / reduce / some / every ───────────
#[test]
fn stretch_array_filter() {
    if let Value::String(s) = run("return [1,2,3,4].filter(x => x > 2).join(\",\");") {
        assert_eq!(s.as_str(), "3,4");
    } else { panic!(); }
}

#[test]
fn stretch_array_reduce() {
    if let Value::Number(n) = run("return [1,2,3,4].reduce(function(a,b){ return a + b; }, 0);") {
        assert_eq!(n, 10.0);
    } else { panic!(); }
}

#[test]
fn stretch_array_some_every_find() {
    assert_eq!(run("return [1,2,3].some(x => x > 2);"), Value::Boolean(true));
    assert_eq!(run("return [1,2,3].every(x => x > 0);"), Value::Boolean(true));
    assert_eq!(run("return [1,2,3].every(x => x > 1);"), Value::Boolean(false));
    assert_eq!(run("return [1,2,3].find(x => x > 1);"), Value::Number(2.0));
}

#[test]
fn stretch_function_bind() {
    if let Value::Number(n) = run(r#"
        function add(a, b, c) { return a + b + c; }
        let bound = add.bind(null, 1, 2);
        return bound(10);
    "#) {
        assert_eq!(n, 13.0);
    } else { panic!(); }
}

#[test]
fn stretch_string_split_join_round_trip() {
    if let Value::String(s) = run(r#"return "a,b,c".split(",").join("-");"#) {
        assert_eq!(s.as_str(), "a-b-c");
    } else { panic!(); }
}

#[test]
fn stretch_number_to_fixed() {
    if let Value::String(s) = run("return (3.14159).toFixed(2);") {
        assert_eq!(s.as_str(), "3.14");
    } else { panic!(); }
}

// Suppress unused-import warning when promise_then_chain isn't drained.
fn _silence() -> Result<(), RuntimeError> { Ok(()) }
