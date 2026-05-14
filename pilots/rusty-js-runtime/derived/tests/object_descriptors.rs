//! Tier-Ω.5.j.proto: Object.defineProperty / defineProperties /
//! getOwnPropertyDescriptor / getOwnPropertyNames acceptance.

use rusty_js_runtime::{run_module, Value};

fn run(src: &str) -> Value {
    run_module(src).unwrap_or_else(|e| panic!("run failed for {:?}: {:?}", src, e))
}

#[test]
fn t1_define_property_value() {
    let src = "
        const o = {};
        Object.defineProperty(o, 'x', {value: 42});
        return o.x;
    ";
    assert_eq!(run(src), Value::Number(42.0));
}

#[test]
fn t2_define_properties() {
    let src = "
        const o = Object.defineProperties({}, {a: {value: 1}, b: {value: 2}});
        return o.a + o.b;
    ";
    assert_eq!(run(src), Value::Number(3.0));
}

#[test]
fn t3_get_own_property_descriptor_shape() {
    let src = "
        const o = {x: 7};
        const d = Object.getOwnPropertyDescriptor(o, 'x');
        return (d.value === 7) + ',' + (d.enumerable === true);
    ";
    if let Value::String(s) = run(src) {
        assert_eq!(s.as_str(), "true,true");
    } else { panic!(); }
}

#[test]
fn t4_get_own_property_descriptor_missing() {
    // Undefined renders to the JS string "undefined" via template; we
    // verify directly via strict-equality return.
    let src = "return Object.getOwnPropertyDescriptor({}, 'x') === undefined;";
    assert_eq!(run(src), Value::Boolean(true));
}

#[test]
fn t5_get_own_property_names_count() {
    let src = "return Object.getOwnPropertyNames({a:1,b:2}).length;";
    assert_eq!(run(src), Value::Number(2.0));
}

#[test]
fn t6_define_property_returns_object() {
    let src = "
        const o = {};
        return Object.defineProperty(o, 'x', {value: 1}) === o;
    ";
    assert_eq!(run(src), Value::Boolean(true));
}
