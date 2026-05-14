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
