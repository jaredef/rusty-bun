//! Tier-Ω.5.g.3 acceptance: template literal substitution lowering.
//!
//! Lowered as a left-to-right Add chain seeded by the first quasi. op_add
//! coerces non-string operands to string when LHS is String, so no explicit
//! ToString is needed in the emitted bytecode.

use rusty_js_runtime::{Runtime, Value};

fn run_rt(src: &str) -> Runtime {
    let module = rusty_js_bytecode::compile_module(src)
        .unwrap_or_else(|e| panic!("compile {:?}: {:?}", src, e));
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    rt.run_module(&module).unwrap_or_else(|e| panic!("run {:?}: {:?}", src, e));
    rt
}

fn last_str(rt: &Runtime) -> String {
    match rt.globals.get("__last_recorded").cloned().unwrap() {
        Value::String(s) => (*s).clone(),
        other => panic!("expected string, got {:?}", other),
    }
}

#[test]
fn t01_basic_number_substitution() {
    let rt = run_rt(r#"__record(`hello ${42} world`);"#);
    assert_eq!(last_str(&rt), "hello 42 world");
}

#[test]
fn t02_back_to_back_substitutions() {
    let rt = run_rt(r#"__record(`${1}${2}${3}`);"#);
    assert_eq!(last_str(&rt), "123");
}

#[test]
fn t03_substitution_with_expression() {
    let rt = run_rt(r#"let a = 2, b = 3; __record(`${a + b}`);"#);
    assert_eq!(last_str(&rt), "5");
}

#[test]
fn t04_string_substitution_only() {
    let rt = run_rt(r#"__record(`${"x"}`);"#);
    assert_eq!(last_str(&rt), "x");
}

#[test]
fn t05_mixed_types() {
    let rt = run_rt(r#"__record(`n=${5}, s=${"hi"}, b=${true}`);"#);
    assert_eq!(last_str(&rt), "n=5, s=hi, b=true");
}

#[test]
fn t06_nested_templates() {
    let rt = run_rt(r#"__record(`outer ${`inner ${7}`}`);"#);
    assert_eq!(last_str(&rt), "outer inner 7");
}
