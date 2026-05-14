//! Built-in intrinsics tests (Math / JSON / global functions / console).

use rusty_js_runtime::{run_module, Value};

fn run(src: &str) -> Value {
    run_module(src).expect(&format!("run failed: {:?}", src))
}

// ─────────── Math ───────────

#[test]
fn math_abs() {
    if let Value::Number(n) = run("return Math.abs(-7);") {
        assert_eq!(n, 7.0);
    } else { panic!(); }
}

#[test]
fn math_floor_ceil_round() {
    if let Value::Number(n) = run("return Math.floor(3.7);") { assert_eq!(n, 3.0); } else { panic!(); }
    if let Value::Number(n) = run("return Math.ceil(3.2);") { assert_eq!(n, 4.0); } else { panic!(); }
    if let Value::Number(n) = run("return Math.round(3.5);") { assert_eq!(n, 4.0); } else { panic!(); }
    if let Value::Number(n) = run("return Math.round(3.4);") { assert_eq!(n, 3.0); } else { panic!(); }
}

#[test]
fn math_sqrt() {
    if let Value::Number(n) = run("return Math.sqrt(16);") {
        assert_eq!(n, 4.0);
    } else { panic!(); }
}

#[test]
fn math_pow() {
    if let Value::Number(n) = run("return Math.pow(2, 10);") {
        assert_eq!(n, 1024.0);
    } else { panic!(); }
}

#[test]
fn math_max_min() {
    if let Value::Number(n) = run("return Math.max(1, 7, 3, 5);") { assert_eq!(n, 7.0); } else { panic!(); }
    if let Value::Number(n) = run("return Math.min(1, 7, 3, 5);") { assert_eq!(n, 1.0); } else { panic!(); }
}

#[test]
fn math_constants() {
    if let Value::Number(n) = run("return Math.PI;") {
        assert!((n - std::f64::consts::PI).abs() < 1e-10);
    } else { panic!(); }
    if let Value::Number(n) = run("return Math.E;") {
        assert!((n - std::f64::consts::E).abs() < 1e-10);
    } else { panic!(); }
}

#[test]
fn math_trig() {
    if let Value::Number(n) = run("return Math.sin(0);") {
        assert!(n.abs() < 1e-10);
    } else { panic!(); }
    if let Value::Number(n) = run("return Math.cos(0);") {
        assert!((n - 1.0).abs() < 1e-10);
    } else { panic!(); }
}

#[test]
fn math_in_arithmetic() {
    let s = r#"
        function dist(x, y) { return Math.sqrt(x * x + y * y); }
        return dist(3, 4);
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 5.0);
    } else { panic!(); }
}

// ─────────── Global functions ───────────

#[test]
fn parse_int() {
    if let Value::Number(n) = run("return parseInt('42');") { assert_eq!(n, 42.0); } else { panic!(); }
    if let Value::Number(n) = run("return parseInt('ff', 16);") { assert_eq!(n, 255.0); } else { panic!(); }
}

#[test]
fn parse_float() {
    if let Value::Number(n) = run("return parseFloat('3.14');") {
        assert!((n - 3.14).abs() < 1e-9);
    } else { panic!(); }
}

#[test]
fn is_nan_global() {
    assert!(matches!(run("return isNaN(NaN);"), Value::Boolean(_)));
    assert!(matches!(run("return isNaN(0/0);"), Value::Boolean(true)));
    assert!(matches!(run("return isNaN(42);"), Value::Boolean(false)));
}

#[test]
fn is_finite_global() {
    assert!(matches!(run("return isFinite(42);"), Value::Boolean(true)));
    assert!(matches!(run("return isFinite(1/0);"), Value::Boolean(false)));
}

// ─────────── JSON ───────────

#[test]
fn json_stringify_primitives() {
    if let Value::String(s) = run("return JSON.stringify(null);") { assert_eq!(s.as_str(), "null"); } else { panic!(); }
    if let Value::String(s) = run("return JSON.stringify(true);") { assert_eq!(s.as_str(), "true"); } else { panic!(); }
    if let Value::String(s) = run("return JSON.stringify(42);") { assert_eq!(s.as_str(), "42"); } else { panic!(); }
    if let Value::String(s) = run("return JSON.stringify('hi');") { assert_eq!(s.as_str(), "\"hi\""); } else { panic!(); }
}

#[test]
fn json_stringify_object() {
    if let Value::String(s) = run("return JSON.stringify({a: 1, b: 'x'});") {
        // Property order is hash-map order — accept either permutation
        let str = s.as_str();
        assert!(
            str == "{\"a\":1,\"b\":\"x\"}" || str == "{\"b\":\"x\",\"a\":1}",
            "got: {}", str
        );
    } else { panic!(); }
}

#[test]
fn json_stringify_array() {
    if let Value::String(s) = run("return JSON.stringify([1, 2, 3]);") {
        assert_eq!(s.as_str(), "[1,2,3]");
    } else { panic!(); }
}

#[test]
fn json_parse_primitives() {
    if let Value::Number(n) = run("return JSON.parse('42');") { assert_eq!(n, 42.0); } else { panic!(); }
    if let Value::Boolean(b) = run("return JSON.parse('true');") { assert!(b); } else { panic!(); }
    if let Value::Null = run("return JSON.parse('null');") { } else { panic!(); }
    if let Value::String(s) = run("return JSON.parse('\"hi\"');") { assert_eq!(s.as_str(), "hi"); } else { panic!(); }
}

#[test]
fn json_parse_object() {
    if let Value::Number(n) = run("let o = JSON.parse('{\"a\":42}'); return o.a;") {
        assert_eq!(n, 42.0);
    } else { panic!(); }
}

#[test]
fn json_parse_array() {
    if let Value::Number(n) = run("let a = JSON.parse('[10, 20, 30]'); return a[1];") {
        assert_eq!(n, 20.0);
    } else { panic!(); }
}

#[test]
fn json_round_trip() {
    let s = r#"
        let s = JSON.stringify({x: 1, y: 'hi'});
        let o = JSON.parse(s);
        return o.x;
    "#;
    if let Value::Number(n) = run(s) {
        assert_eq!(n, 1.0);
    } else { panic!(); }
}
