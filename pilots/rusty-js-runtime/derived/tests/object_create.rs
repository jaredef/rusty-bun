//! Tier-Ω.5.v acceptance: Object.create(proto, propertiesObject?).
//!
//! Per ECMA-262 §20.1.2.2: prototype must be Object or null; allocates an
//! ordinary object whose [[Prototype]] is set accordingly. The optional
//! propertiesObject is handled at the subset level by reading each
//! descriptor's `value` field (matching defineProperty's subset).

use rusty_js_runtime::{Runtime, Value};

fn run_rt(src: &str) -> Runtime {
    let module = rusty_js_bytecode::compile_module(src)
        .unwrap_or_else(|e| panic!("compile {:?}: {:?}", src, e));
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    rt.run_module(&module).unwrap_or_else(|e| panic!("run {:?}: {:?}", src, e));
    rt
}

fn last(rt: &Runtime) -> Value {
    rt.globals.get("__last_recorded").cloned().unwrap_or(Value::Undefined)
}

// 1. Object.create(null) allocates an ordinary object.
#[test]
fn t01_create_null_proto() {
    let rt = run_rt("const o = Object.create(null); __record(typeof o === \"object\");");
    assert_eq!(last(&rt), Value::Boolean(true));
}

// 2. Prototype chain hookup: methods from proto are reachable.
#[test]
fn t02_create_proto_chain() {
    let rt = run_rt(r#"
        const proto = {greet() { return "hi"; }};
        const o = Object.create(proto);
        __record(o.greet());
    "#);
    assert!(matches!(last(&rt), Value::String(ref s) if &**s == "hi"));
}

// 3. Non-object, non-null prototype must throw TypeError.
#[test]
fn t03_create_bad_proto_throws() {
    let src = r#"Object.create("not an object");"#;
    let module = rusty_js_bytecode::compile_module(src).unwrap();
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    let err = rt.run_module(&module).expect_err("Object.create with non-object proto must error");
    let msg = format!("{:?}", err);
    assert!(msg.contains("Object.create"), "unexpected error: {}", msg);
}

// 4. propertiesObject — value field becomes the new own property.
#[test]
fn t04_create_with_properties() {
    let rt = run_rt("const o = Object.create({}, {x: {value: 7}}); __record(o.x);");
    assert_eq!(last(&rt), Value::Number(7.0));
}
