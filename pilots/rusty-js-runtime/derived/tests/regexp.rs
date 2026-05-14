//! Tier-Ω.5.i acceptance: regex literal AST → runtime RegExp object,
//! prototype methods (.test / .exec / .toString), accessor surface, and
//! the regex-aware String.prototype methods. Plus the RegExp constructor.

use rusty_js_runtime::{Runtime, Value};

fn run_rt(src: &str) -> Runtime {
    let module = rusty_js_bytecode::compile_module(src)
        .unwrap_or_else(|e| panic!("compile {:?}: {:?}", src, e));
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    rt.run_module(&module).unwrap_or_else(|e| panic!("run {:?}: {:?}", src, e));
    rt
}

fn last_recorded(rt: &Runtime) -> Value {
    rt.globals.get("__last_recorded").cloned().unwrap_or(Value::Undefined)
}

fn last_bool(rt: &Runtime) -> bool {
    match last_recorded(rt) {
        Value::Boolean(b) => b,
        other => panic!("expected bool, got {:?}", other),
    }
}

fn last_string(rt: &Runtime) -> String {
    match last_recorded(rt) {
        Value::String(s) => (*s).clone(),
        other => panic!("expected string, got {:?}", other),
    }
}

fn last_num(rt: &Runtime) -> f64 {
    match last_recorded(rt) {
        Value::Number(n) => n,
        other => panic!("expected number, got {:?}", other),
    }
}

#[test]
fn t01_literal_parses_and_compiles() {
    // No assertion needed beyond compile+run not throwing.
    let _rt = run_rt(r#"let r = /foo/; __record("ok");"#);
}

#[test]
fn t02_source_accessor() {
    let rt = run_rt(r#"__record(/foo/.source);"#);
    assert_eq!(last_string(&rt), "foo");
}

#[test]
fn t03_flags_accessor_contains_gim() {
    let rt = run_rt(r#"__record(/foo/gim.flags);"#);
    let f = last_string(&rt);
    assert!(f.contains('g'), "missing g: {:?}", f);
    assert!(f.contains('i'), "missing i: {:?}", f);
    assert!(f.contains('m'), "missing m: {:?}", f);
}

#[test]
fn t04_test_true() {
    let rt = run_rt(r#"__record(/foo/.test("foobar"));"#);
    assert!(last_bool(&rt));
}

#[test]
fn t05_test_false() {
    let rt = run_rt(r#"__record(/foo/.test("bar"));"#);
    assert!(!last_bool(&rt));
}

#[test]
fn t06_case_insensitive() {
    let rt = run_rt(r#"__record(/foo/i.test("FOO"));"#);
    assert!(last_bool(&rt));
}

#[test]
fn t07_exec_returns_array_with_index_input() {
    let rt = run_rt(r#"
        let m = /(\w+)\s+(\w+)/.exec("hello world");
        __record(m[0] + "|" + m[1] + "|" + m[2] + "|" + m.index + "|" + m.input);
    "#);
    assert_eq!(last_string(&rt), "hello world|hello|world|0|hello world");
}

#[test]
fn t08_exec_no_match_null() {
    let rt = run_rt(r#"
        let m = /xyz/.exec("hello");
        __record(m === null);
    "#);
    assert!(last_bool(&rt));
}

#[test]
fn t09_global_stateful_exec() {
    let rt = run_rt(r#"
        let r = /\d+/g;
        let a = r.exec("a1b22c333");
        let b = r.exec("a1b22c333");
        let c = r.exec("a1b22c333");
        __record(a[0] + "|" + b[0] + "|" + c[0] + "|" + r.lastIndex);
    "#);
    // After third match "333" ending at byte index 9, lastIndex = 9.
    assert_eq!(last_string(&rt), "1|22|333|9");
}

#[test]
fn t10_string_match_groups() {
    let rt = run_rt(r#"
        let m = "hello world".match(/(\w+) (\w+)/);
        __record(m[2]);
    "#);
    assert_eq!(last_string(&rt), "world");
}

#[test]
fn t11_string_replace_single() {
    let rt = run_rt(r#"__record("hello".replace(/l/, "L"));"#);
    assert_eq!(last_string(&rt), "heLlo");
}

#[test]
fn t12_string_replace_global() {
    let rt = run_rt(r#"__record("hello".replace(/l/g, "L"));"#);
    assert_eq!(last_string(&rt), "heLLo");
}

#[test]
fn t13_string_split_regex() {
    let rt = run_rt(r#"
        let parts = "a1b2c3".split(/\d/);
        __record(parts.length + "|" + parts[0] + "|" + parts[1] + "|" + parts[2] + "|" + parts[3]);
    "#);
    // "a1b2c3" split by /\d/ → ["a","b","c",""] in JS.
    assert_eq!(last_string(&rt), "4|a|b|c|");
}

#[test]
fn t14_constructor_form() {
    let rt = run_rt(r#"
        let r = new RegExp("\\d+", "g");
        __record(r.test("abc 123") + "|" + r.flags);
    "#);
    let s = last_string(&rt);
    assert!(s.starts_with("true|"), "got {:?}", s);
    assert!(s.contains('g'), "got {:?}", s);
}

#[test]
fn t15_search_returns_index() {
    let rt = run_rt(r#"__record("hello world".search(/world/));"#);
    assert_eq!(last_num(&rt), 6.0);
}

#[test]
fn t16_replace_all() {
    let rt = run_rt(r#"__record("ababab".replaceAll(/a/g, "X"));"#);
    assert_eq!(last_string(&rt), "XbXbXb");
}
