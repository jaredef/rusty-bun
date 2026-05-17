//! Ω.5.P16.E2.ns-default-synth — integration coverage for the
//! HostFinalizeModuleNamespace install in host-v2.

use rusty_bun_host_v2::module_ns;
use rusty_js_runtime::{Runtime, Value};

fn new_rt() -> Runtime {
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    module_ns::install(&mut rt);
    rt
}

#[test]
fn tuple_a_synthesizes_default_as_namespace_when_absent() {
    let mut rt = new_rt();
    let src = "const a = 1; const b = 2; export { a, b };";
    let ns = rt.evaluate_module(src, "test-a").expect("evaluate");
    let o = rt.obj(ns);
    assert!(o.properties.contains_key("a"));
    assert!(o.properties.contains_key("b"));
    let default = o.properties.get("default").expect("Tuple A: default synthesized");
    match &default.value {
        Value::Object(id) => assert_eq!(*id, ns, "default points at namespace itself"),
        other => panic!("expected default to be namespace object, got {:?}", other),
    }
}

#[test]
fn tuple_b_synthesizes_named_from_default_object_keys() {
    let mut rt = new_rt();
    let src = "export default { x: 1, y: 2 };";
    let ns = rt.evaluate_module(src, "test-b").expect("evaluate");
    let o = rt.obj(ns);
    assert!(o.properties.contains_key("default"), "default still present");
    assert!(o.properties.contains_key("x"), "Tuple B: 'x' spread from default");
    assert!(o.properties.contains_key("y"), "Tuple B: 'y' spread from default");
}

#[test]
fn does_not_shadow_when_both_default_and_named_exist() {
    let mut rt = new_rt();
    let src = r#"
        const a = 10;
        export { a };
        export default { a: 999, z: 7 };
    "#;
    let ns = rt.evaluate_module(src, "test-c").expect("evaluate");
    let o = rt.obj(ns);
    // Named export 'a' must be preserved (10, not 999 from default).
    match o.properties.get("a").map(|d| &d.value) {
        Some(Value::Number(n)) => assert_eq!(*n, 10.0, "named 'a' preserved, not shadowed by default.a"),
        other => panic!("expected named 'a' = 10, got {:?}", other),
    }
    // default still present unchanged.
    assert!(matches!(o.properties.get("default").map(|d| &d.value), Some(Value::Object(_))));
    // Tuple B branch must NOT fire (named exports already exist), so 'z'
    // from default should NOT have been spread.
    assert!(!o.properties.contains_key("z"),
        "Tuple B suppressed when named exports already present");
}
