//! End-to-end runtime tests: parse + compile + run + assert result.
//!
//! For v1 round 3.d.b, the runtime returns the last value remaining on
//! the operand stack at module exit. Since modules end with ReturnUndef,
//! we use explicit `return` to surface the result.

use rusty_js_runtime::{run_module, Value};

fn run(src: &str) -> Value {
    run_module(src).expect(&format!("run failed: {:?}", src))
}

// ─────────── Literals ───────────

#[test]
fn return_null() {
    assert!(matches!(run("return null;"), Value::Null));
}

#[test]
fn return_true() {
    assert!(matches!(run("return true;"), Value::Boolean(true)));
}

#[test]
fn return_false() {
    assert!(matches!(run("return false;"), Value::Boolean(false)));
}

#[test]
fn return_integer() {
    if let Value::Number(n) = run("return 42;") {
        assert_eq!(n, 42.0);
    } else { panic!("expected number"); }
}

#[test]
fn return_float() {
    if let Value::Number(n) = run("return 3.14;") {
        assert!((n - 3.14).abs() < 1e-9);
    } else { panic!("expected number"); }
}

#[test]
fn return_string() {
    if let Value::String(s) = run("return 'hello';") {
        assert_eq!(s.as_str(), "hello");
    } else { panic!("expected string"); }
}

// ─────────── Arithmetic ───────────

#[test]
fn add_integers() {
    if let Value::Number(n) = run("return 1 + 2;") {
        assert_eq!(n, 3.0);
    } else { panic!(); }
}

#[test]
fn subtract() {
    if let Value::Number(n) = run("return 10 - 3;") {
        assert_eq!(n, 7.0);
    } else { panic!(); }
}

#[test]
fn multiply() {
    if let Value::Number(n) = run("return 6 * 7;") {
        assert_eq!(n, 42.0);
    } else { panic!(); }
}

#[test]
fn divide() {
    if let Value::Number(n) = run("return 12 / 4;") {
        assert_eq!(n, 3.0);
    } else { panic!(); }
}

#[test]
fn modulo() {
    if let Value::Number(n) = run("return 10 % 3;") {
        assert_eq!(n, 1.0);
    } else { panic!(); }
}

#[test]
fn power() {
    if let Value::Number(n) = run("return 2 ** 8;") {
        assert_eq!(n, 256.0);
    } else { panic!(); }
}

#[test]
fn negation() {
    if let Value::Number(n) = run("return -5;") {
        assert_eq!(n, -5.0);
    } else { panic!(); }
}

#[test]
fn precedence_correct_at_runtime() {
    // 1 + 2 * 3 = 7 (not 9)
    if let Value::Number(n) = run("return 1 + 2 * 3;") {
        assert_eq!(n, 7.0);
    } else { panic!(); }
}

// ─────────── String concatenation ───────────

#[test]
fn string_concat() {
    if let Value::String(s) = run("return 'hello' + ' ' + 'world';") {
        assert_eq!(s.as_str(), "hello world");
    } else { panic!(); }
}

#[test]
fn string_plus_number_coerces_to_string() {
    if let Value::String(s) = run("return 'x=' + 42;") {
        assert_eq!(s.as_str(), "x=42");
    } else { panic!(); }
}

// ─────────── Comparison ───────────

#[test]
fn less_than_true() {
    assert!(matches!(run("return 1 < 2;"), Value::Boolean(true)));
}

#[test]
fn less_than_false() {
    assert!(matches!(run("return 2 < 1;"), Value::Boolean(false)));
}

#[test]
fn greater_than() {
    assert!(matches!(run("return 5 > 3;"), Value::Boolean(true)));
}

#[test]
fn strict_equal_numbers() {
    assert!(matches!(run("return 1 === 1;"), Value::Boolean(true)));
}

#[test]
fn strict_equal_different_types() {
    assert!(matches!(run("return 1 === '1';"), Value::Boolean(false)));
}

#[test]
fn loose_equal_string_number() {
    assert!(matches!(run("return 1 == '1';"), Value::Boolean(true)));
}

#[test]
fn loose_equal_null_undefined() {
    assert!(matches!(run("return null == undefined;"), Value::Boolean(true)));
}

#[test]
fn nan_not_strictly_equal_to_itself() {
    // NaN/NaN per division: 0/0 = NaN; NaN === NaN is false.
    assert!(matches!(run("return (0/0) === (0/0);"), Value::Boolean(false)));
}

// ─────────── Bitwise ───────────

#[test]
fn bitwise_and() {
    if let Value::Number(n) = run("return 5 & 3;") {
        assert_eq!(n, 1.0);
    } else { panic!(); }
}

#[test]
fn shift_left() {
    if let Value::Number(n) = run("return 1 << 4;") {
        assert_eq!(n, 16.0);
    } else { panic!(); }
}

// ─────────── Logical ───────────

#[test]
fn logical_not() {
    assert!(matches!(run("return !true;"), Value::Boolean(false)));
    assert!(matches!(run("return !false;"), Value::Boolean(true)));
    assert!(matches!(run("return !0;"), Value::Boolean(true)));
    assert!(matches!(run("return !1;"), Value::Boolean(false)));
}

// ─────────── Variables ───────────

#[test]
fn let_and_read() {
    if let Value::Number(n) = run("let x = 42; return x;") {
        assert_eq!(n, 42.0);
    } else { panic!(); }
}

#[test]
fn let_with_expression_initializer() {
    if let Value::Number(n) = run("let x = 2 + 3; return x;") {
        assert_eq!(n, 5.0);
    } else { panic!(); }
}

#[test]
fn const_binding() {
    if let Value::Number(n) = run("const c = 100; return c;") {
        assert_eq!(n, 100.0);
    } else { panic!(); }
}

#[test]
fn multiple_locals() {
    if let Value::Number(n) = run("let a = 1; let b = 2; let c = 3; return a + b + c;") {
        assert_eq!(n, 6.0);
    } else { panic!(); }
}

// ─────────── Typeof ───────────

// ─────────── Control flow ───────────

#[test]
fn if_true_branch() {
    if let Value::Number(n) = run("if (true) { return 1; } return 2;") {
        assert_eq!(n, 1.0);
    } else { panic!(); }
}

#[test]
fn if_false_branch() {
    if let Value::Number(n) = run("if (false) { return 1; } return 2;") {
        assert_eq!(n, 2.0);
    } else { panic!(); }
}

#[test]
fn if_else_branch() {
    if let Value::Number(n) = run("if (false) { return 1; } else { return 2; }") {
        assert_eq!(n, 2.0);
    } else { panic!(); }
}

#[test]
fn if_condition_evaluated() {
    if let Value::Number(n) = run("let x = 5; if (x > 3) { return 100; } return 200;") {
        assert_eq!(n, 100.0);
    } else { panic!(); }
}

#[test]
fn while_loop_executes() {
    let s = r#"
        let i = 0;
        let sum = 0;
        while (i < 5) {
            sum = sum + i;
            i = i + 1;
        }
        return sum;
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 10.0);  // 0 + 1 + 2 + 3 + 4
    } else { panic!(); }
}

#[test]
fn for_c_style_loop() {
    let s = r#"
        let total = 0;
        for (let i = 1; i <= 10; i = i + 1) {
            total = total + i;
        }
        return total;
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 55.0);  // sum 1..10
    } else { panic!(); }
}

#[test]
fn do_while_runs_body_at_least_once() {
    let s = r#"
        let i = 10;
        do { i = i + 1; } while (i < 5);
        return i;
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 11.0);  // body ran once
    } else { panic!(); }
}

#[test]
fn break_exits_loop() {
    let s = r#"
        let i = 0;
        while (true) {
            if (i === 3) { break; }
            i = i + 1;
        }
        return i;
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 3.0);
    } else { panic!(); }
}

#[test]
fn continue_skips_iteration() {
    let s = r#"
        let sum = 0;
        for (let i = 0; i < 5; i = i + 1) {
            if (i === 2) { continue; }
            sum = sum + i;
        }
        return sum;
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 8.0);  // 0 + 1 + 3 + 4
    } else { panic!(); }
}

// ─────────── Short-circuit / conditional ───────────

#[test]
fn logical_and_short_circuit() {
    if let Value::Number(n) = run("return 5 && 10;") {
        assert_eq!(n, 10.0);
    } else { panic!(); }
}

#[test]
fn logical_and_falsy_returns_lhs() {
    if let Value::Number(n) = run("return 0 && 10;") {
        assert_eq!(n, 0.0);
    } else { panic!(); }
}

#[test]
fn logical_or_returns_first_truthy() {
    if let Value::Number(n) = run("return 0 || 7;") {
        assert_eq!(n, 7.0);
    } else { panic!(); }
}

#[test]
fn logical_or_truthy_lhs_kept() {
    if let Value::Number(n) = run("return 3 || 7;") {
        assert_eq!(n, 3.0);
    } else { panic!(); }
}

#[test]
fn nullish_coalesce_null() {
    if let Value::Number(n) = run("return null ?? 5;") {
        assert_eq!(n, 5.0);
    } else { panic!(); }
}

#[test]
fn nullish_coalesce_undefined() {
    if let Value::Number(n) = run("return undefined ?? 7;") {
        assert_eq!(n, 7.0);
    } else { panic!(); }
}

#[test]
fn nullish_coalesce_zero_kept() {
    // 0 is not nullish, so should be returned (this is the key difference vs ||)
    if let Value::Number(n) = run("return 0 ?? 5;") {
        assert_eq!(n, 0.0);
    } else { panic!(); }
}

#[test]
fn conditional_true() {
    if let Value::Number(n) = run("return true ? 1 : 2;") {
        assert_eq!(n, 1.0);
    } else { panic!(); }
}

#[test]
fn conditional_false() {
    if let Value::Number(n) = run("return false ? 1 : 2;") {
        assert_eq!(n, 2.0);
    } else { panic!(); }
}

// ─────────── Try / catch ───────────

#[test]
fn try_catch_catches_throw() {
    let s = r#"
        try { throw 42; }
        catch (e) { return e; }
        return 0;
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 42.0);
    } else { panic!(); }
}

#[test]
fn try_without_throw_skips_handler() {
    let s = r#"
        let x = 0;
        try { x = 1; }
        catch (e) { x = 99; }
        return x;
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 1.0);
    } else { panic!(); }
}

// ─────────── Object / Array literals ───────────

#[test]
fn array_literal_creates_array() {
    let v = run("return [1, 2, 3];");
    if let Value::Object(_) = v {
        // good — produced an array-kind object
    } else { panic!("expected object"); }
}

#[test]
fn array_index_access() {
    if let Value::Number(n) = run("let a = [10, 20, 30]; return a[1];") {
        assert_eq!(n, 20.0);
    } else { panic!(); }
}

#[test]
fn object_literal_property_access() {
    if let Value::Number(n) = run("let o = {x: 1, y: 2}; return o.x;") {
        assert_eq!(n, 1.0);
    } else { panic!(); }
}

#[test]
fn object_computed_index() {
    if let Value::Number(n) = run("let o = {a: 42}; return o['a'];") {
        assert_eq!(n, 42.0);
    } else { panic!(); }
}

#[test]
fn nested_object() {
    if let Value::Number(n) = run("let o = {a: {b: 99}}; return o.a.b;") {
        assert_eq!(n, 99.0);
    } else { panic!(); }
}

#[test]
fn object_set_property() {
    let s = r#"
        let o = {};
        o.x = 10;
        o.y = 20;
        return o.x + o.y;
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 30.0);
    } else { panic!(); }
}

#[test]
fn property_undefined_on_missing() {
    if let Value::Undefined = run("let o = {a: 1}; return o.b;") {
        // good
    } else { panic!("expected undefined"); }
}

#[test]
fn typeof_object_literal() {
    if let Value::String(s) = run("return typeof {};") {
        assert_eq!(s.as_str(), "object");
    } else { panic!(); }
}

// ─────────── Function calls ───────────

#[test]
fn call_function_no_args() {
    let s = r#"
        function f() { return 42; }
        return f();
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 42.0);
    } else { panic!(); }
}

#[test]
fn call_function_with_args() {
    let s = r#"
        function add(a, b) { return a + b; }
        return add(3, 4);
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 7.0);
    } else { panic!(); }
}

#[test]
fn arrow_function_call() {
    let s = r#"
        let sq = (x) => x * x;
        return sq(7);
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 49.0);
    } else { panic!(); }
}

#[test]
fn arrow_function_block_body() {
    let s = r#"
        let f = (x, y) => { return x + y * 2; };
        return f(1, 3);
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 7.0);
    } else { panic!(); }
}

#[test]
fn function_returning_string() {
    let s = r#"
        function greet(name) { return 'hello ' + name; }
        return greet('world');
    "#;
    if let Value::String(s) = run(s) {
        assert_eq!(s.as_str(), "hello world");
    } else { panic!(); }
}

#[test]
fn typeof_function() {
    let s = r#"
        function f() {}
        return typeof f;
    "#;
    if let Value::String(s) = run(s) {
        assert_eq!(s.as_str(), "function");
    } else { panic!(); }
}

#[test]
fn function_with_local_variable() {
    let s = r#"
        function compute(x) {
            let result = x * 2;
            result = result + 1;
            return result;
        }
        return compute(10);
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 21.0);
    } else { panic!(); }
}

#[test]
fn function_with_conditional() {
    let s = r#"
        function abs(x) {
            if (x < 0) { return -x; }
            return x;
        }
        return abs(-7) + abs(3);
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 10.0);
    } else { panic!(); }
}

#[test]
fn call_in_arithmetic() {
    let s = r#"
        function double(x) { return x * 2; }
        return double(3) + double(4);
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 14.0);
    } else { panic!(); }
}

#[test]
fn higher_order_function() {
    let s = r#"
        function apply(f, x) { return f(x); }
        let cube = (x) => x * x * x;
        return apply(cube, 3);
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 27.0);
    } else { panic!(); }
}

#[test]
fn typeof_primitives() {
    if let Value::String(s) = run("return typeof 42;") {
        assert_eq!(s.as_str(), "number");
    } else { panic!(); }
    if let Value::String(s) = run("return typeof 'x';") {
        assert_eq!(s.as_str(), "string");
    } else { panic!(); }
    if let Value::String(s) = run("return typeof true;") {
        assert_eq!(s.as_str(), "boolean");
    } else { panic!(); }
    if let Value::String(s) = run("return typeof undefined;") {
        assert_eq!(s.as_str(), "undefined");
    } else { panic!(); }
    if let Value::String(s) = run("return typeof null;") {
        assert_eq!(s.as_str(), "object"); // per §13.5.3
    } else { panic!(); }
}
