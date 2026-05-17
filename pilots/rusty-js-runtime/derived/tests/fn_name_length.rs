//! Tier-Ω.5.P15.E1: spec-mandated own .name and .length properties on
//! every function instance per ECMA-262 §10.2.9 (SetFunctionName) and
//! §10.2.10 (SetFunctionLength). Descriptors are
//! `{writable:false, enumerable:false, configurable:true}` — invisible to
//! Object.keys, visible to direct reads, overrideable by defineProperty.

use rusty_js_runtime::{run_module, Value};

fn run(src: &str) -> Value {
    run_module(src).unwrap_or_else(|e| panic!("run failed for {:?}: {:?}", src, e))
}

fn as_str(v: Value) -> String {
    if let Value::String(s) = v { s.as_str().to_string() } else { panic!("not a string: {:?}", v) }
}

#[test]
fn named_function_decl_has_name_and_length() {
    let s = r#"
        function foo(a, b) { return a + b; }
        return foo.name + "|" + foo.length;
    "#;
    assert_eq!(as_str(run(s)), "foo|2");
}

#[test]
fn anonymous_function_expression_in_paren_no_binding() {
    let s = r#"
        const f = (0, function () {});
        return f.name + "|" + f.length;
    "#;
    // Comma-sequence strips NamedEvaluation per spec; the outer binding's
    // hint does not propagate through Expr::Sequence.
    assert_eq!(as_str(run(s)), "|0");
}

#[test]
fn arrow_function_length_correct() {
    let s = r#"
        const baz = (x, y, z) => x;
        return baz.name + "|" + baz.length;
    "#;
    assert_eq!(as_str(run(s)), "baz|3");
}

#[test]
fn method_shorthand_takes_property_name() {
    let s = r#"
        const o = { qux(a) {} };
        return o.qux.name + "|" + o.qux.length;
    "#;
    assert_eq!(as_str(run(s)), "qux|1");
}

#[test]
fn named_function_expression_uses_inner_name() {
    let s = r#"
        const expr = function quux() {};
        return expr.name + "|" + expr.length;
    "#;
    assert_eq!(as_str(run(s)), "quux|0");
}

#[test]
fn rest_parameter_excluded_from_length() {
    let s = r#"
        function vrest(a, ...rest) {}
        return vrest.name + "|" + vrest.length;
    "#;
    assert_eq!(as_str(run(s)), "vrest|1");
}

#[test]
fn default_parameter_excluded_from_length() {
    let s = r#"
        function vdflt(a, b = 1, c) {}
        return vdflt.name + "|" + vdflt.length;
    "#;
    assert_eq!(as_str(run(s)), "vdflt|1");
}

#[test]
fn name_and_length_are_non_enumerable() {
    let s = r#"
        function foo(a, b) {}
        const keys = Object.keys(foo);
        return keys.indexOf("name") + "|" + keys.indexOf("length");
    "#;
    assert_eq!(as_str(run(s)), "-1|-1");
}

#[test]
fn const_anon_arrow_takes_binding_name() {
    let s = r#"
        const f = () => 1;
        return f.name;
    "#;
    assert_eq!(as_str(run(s)), "f");
}
