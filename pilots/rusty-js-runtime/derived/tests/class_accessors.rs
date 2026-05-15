//! Tier-Omega.5.u — class-member getter / setter compiler coverage.
//!
//! v1 deviation: accessor-descriptor semantics are dropped. A class
//! getter / setter is lowered as a plain function-valued property on
//! the prototype (instance) or constructor (static). The accessor
//! function is reachable by name; reading the property returns the
//! function itself, not the result of invoking it. Real getter / setter
//! behavior is queued for the substrate round that wires
//! Object.defineProperty's get / set fields end-to-end. Mirrors the
//! object-literal treatment landed in Ω.5.p.parse.

use rusty_js_runtime::{run_module, Value};

fn run(src: &str) -> Value {
    run_module(src).unwrap_or_else(|e| panic!("run failed for {:?}: {:?}", src, e))
}

// 1. Class with getter parses + compiles. v1 deviation: `new C().foo`
//    yields the getter *function*, not its return value.
#[test]
fn t01_class_getter_compiles() {
    let src = r#"
        class C { get foo() { return 42; } }
        return typeof new C().foo;
    "#;
    assert_eq!(run(src), Value::String(std::rc::Rc::new("function".into())));
}

// 2. Class with setter parses + compiles. v1 deviation: `new C().bar`
//    yields the setter function as a plain property.
#[test]
fn t02_class_setter_compiles() {
    let src = r#"
        class C { set bar(v) { this.x = v; } }
        return typeof new C().bar;
    "#;
    assert_eq!(run(src), Value::String(std::rc::Rc::new("function".into())));
}

// 3. Disambiguation — `get` / `set` as method names (followed by `(`),
//    not accessor modifiers.
#[test]
fn t03_get_set_as_method_names() {
    let src = r#"
        class C {
            get() { return 1; }
            set(v) { return v + 1; }
        }
        const c = new C();
        return c.get() + c.set(2);
    "#;
    assert_eq!(run(src), Value::Number(4.0));
}

// 4. Static getter lands on the constructor. v1 deviation: `C.count`
//    yields the getter function.
#[test]
fn t04_static_getter() {
    let src = r#"
        class C { static get count() { return 5; } }
        return typeof C.count;
    "#;
    assert_eq!(run(src), Value::String(std::rc::Rc::new("function".into())));
}

// 5. Both getter and setter on the same key — both compile. v1
//    deviation: they share a single prototype slot, last-write wins.
#[test]
fn t05_getter_and_setter_same_key() {
    let src = r#"
        class C {
            get x() { return 1; }
            set x(v) { return v; }
        }
        return typeof new C().x;
    "#;
    assert_eq!(run(src), Value::String(std::rc::Rc::new("function".into())));
}
