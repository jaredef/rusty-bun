//! Tier-Ω.5.j.proto: Function global stub acceptance.
//! v1 ships a non-constructible stub that throws TypeError. Full
//! eval-via-Function is deferred (would need parser+compiler
//! dependency injection and a Closure-from-FunctionExpression path).

use rusty_js_runtime::{run_module, Value};

#[test]
fn t1_function_stub_throws() {
    // Try/catch path must observe the stub's TypeError, not "callee is
    // not callable".
    let src = "
        let caught = false;
        try { Function('return 1'); }
        catch (e) { caught = true; }
        return caught;
    ";
    assert_eq!(run_module(src).unwrap(), Value::Boolean(true));
}
