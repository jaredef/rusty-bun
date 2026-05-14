//! Tier-Ω.5.o acceptance — public class fields.

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

#[test]
fn t01_field_with_initializer() {
    let rt = run_rt(r#"
        class C { x = 7; }
        __record(new C().x);
    "#);
    assert_eq!(recorded(&rt), Value::Number(7.0));
}

#[test]
fn t02_field_without_initializer() {
    let rt = run_rt(r#"
        class C { x; }
        __record(new C().x);
    "#);
    assert_eq!(recorded(&rt), Value::Undefined);
}

#[test]
fn t03_field_uses_this() {
    let rt = run_rt(r#"
        class C { x = 7; y = this.x * 2; }
        __record(new C().y);
    "#);
    assert_eq!(recorded(&rt), Value::Number(14.0));
}

#[test]
fn t04_field_before_constructor_body() {
    let rt = run_rt(r#"
        class C { x = 7; constructor() { this.y = this.x + 1; } }
        __record(new C().y);
    "#);
    assert_eq!(recorded(&rt), Value::Number(8.0));
}

#[test]
fn t05_static_field() {
    let rt = run_rt(r#"
        class C { static count = 42; }
        __record(C.count);
    "#);
    assert_eq!(recorded(&rt), Value::Number(42.0));
}

#[test]
fn t06_inherited_fields_dont_shadow() {
    let rt = run_rt(r#"
        class A { x = 1; }
        class B extends A { y = 2; }
        const b = new B();
        __record(b.x + b.y);
    "#);
    assert_eq!(recorded(&rt), Value::Number(3.0));
}

#[test]
fn t07_private_field_clear_error() {
    let src = r#"class C { #x = 7; } new C();"#;
    let err = rusty_js_bytecode::compile_module(src).err().expect("expected compile error");
    let msg = format!("{:?}", err);
    assert!(msg.contains("private") && msg.contains("not yet supported"),
        "error did not mention private not-yet-supported: {}", msg);
}
