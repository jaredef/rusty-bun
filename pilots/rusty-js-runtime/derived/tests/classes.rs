//! Tier-Ω.5.f acceptance: class declarations + expressions, extends,
//! super, instance methods, static methods, constructors, and `new`.
//!
//! See pilots/rusty-js-runtime/derived/trajectory.md row 2026-05-14 for
//! the scope ceiling and the upvalue-vs-HomeObject lowering decision.

use rusty_js_runtime::{run_module, Value};

fn run(src: &str) -> Value {
    run_module(src).unwrap_or_else(|e| panic!("run failed for {:?}: {:?}", src, e))
}

// 1. Empty class. typeof new C() === "object".
#[test]
fn t01_empty_class() {
    let src = r#"
        class C {}
        const c = new C();
        return typeof c;
    "#;
    assert_eq!(run(src), Value::String(std::rc::Rc::new("object".into())));
}

// 2. Constructor with args populates the instance.
#[test]
fn t02_constructor_args() {
    let src = r#"
        class C {
            constructor(a, b) { this.a = a; this.b = b; }
        }
        const c = new C(1, 2);
        return c.a + c.b;
    "#;
    assert_eq!(run(src), Value::Number(3.0));
}

// 3. Instance method on the prototype.
#[test]
fn t03_instance_method() {
    let src = r#"
        class C { greet() { return "hi"; } }
        return new C().greet();
    "#;
    assert_eq!(run(src), Value::String(std::rc::Rc::new("hi".into())));
}

// 4. Method using `this` — repeated calls accumulate state.
#[test]
fn t04_method_this_state() {
    let src = r#"
        class Counter {
            constructor() { this.n = 0; }
            inc() { this.n = this.n + 1; return this.n; }
        }
        const c = new Counter();
        c.inc(); c.inc();
        return c.inc();
    "#;
    assert_eq!(run(src), Value::Number(3.0));
}

// 5. Multiple methods on one class.
#[test]
fn t05_multiple_methods() {
    let src = r#"
        class P {
            x() { return 1; }
            y() { return 2; }
        }
        const p = new P();
        return p.x() + p.y();
    "#;
    assert_eq!(run(src), Value::Number(3.0));
}

// 6. Static method on the constructor.
#[test]
fn t06_static_method() {
    let src = r#"
        class C { static who() { return "C"; } }
        return C.who();
    "#;
    assert_eq!(run(src), Value::String(std::rc::Rc::new("C".into())));
}

// 7. Extends without overriding — inherited instance method.
#[test]
fn t07_extends_inherit_method() {
    let src = r#"
        class A { greet() { return "A"; } }
        class B extends A {}
        return new B().greet();
    "#;
    assert_eq!(run(src), Value::String(std::rc::Rc::new("A".into())));
}

// 8. Subclass overrides parent method.
#[test]
fn t08_extends_override() {
    let src = r#"
        class A { greet() { return "A"; } }
        class B extends A { greet() { return "B"; } }
        return new B().greet();
    "#;
    assert_eq!(run(src), Value::String(std::rc::Rc::new("B".into())));
}

// 9. super.method() inside an overriding method.
#[test]
fn t09_super_method() {
    let src = r#"
        class A { greet() { return "A"; } }
        class B extends A { greet() { return "super:" + super.greet(); } }
        return new B().greet();
    "#;
    assert_eq!(run(src), Value::String(std::rc::Rc::new("super:A".into())));
}

// 10. super(...) in a subclass constructor.
#[test]
fn t10_super_constructor() {
    let src = r#"
        class A { constructor(x) { this.x = x; } }
        class B extends A { constructor() { super(7); } }
        return new B().x;
    "#;
    assert_eq!(run(src), Value::Number(7.0));
}

// 11. instanceof walks the prototype chain.
#[test]
fn t11_instanceof_chain() {
    let src = r#"
        class A {}
        class B extends A {}
        const b = new B();
        return (b instanceof B) && (b instanceof A);
    "#;
    assert_eq!(run(src), Value::Boolean(true));
}

// 12. Static method inherited via constructor's proto chain.
#[test]
fn t12_static_inheritance() {
    let src = r#"
        class A { static who() { return "A"; } }
        class B extends A {}
        return B.who();
    "#;
    assert_eq!(run(src), Value::String(std::rc::Rc::new("A".into())));
}

// 13. Multi-level inheritance (A → B → C).
#[test]
fn t13_multilevel_inheritance() {
    let src = r#"
        class A { f() { return 1; } }
        class B extends A {}
        class C extends B {}
        return new C().f();
    "#;
    assert_eq!(run(src), Value::Number(1.0));
}

// 14. Anonymous class expression.
#[test]
fn t14_class_expression_anonymous() {
    let src = r#"
        const C = class { f() { return 42; } };
        return new C().f();
    "#;
    assert_eq!(run(src), Value::Number(42.0));
}

// 15. Named class expression — binding works under the outer name.
#[test]
fn t15_class_expression_named() {
    let src = r#"
        const D = class Inner { name() { return "Inner"; } };
        return new D().name();
    "#;
    assert_eq!(run(src), Value::String(std::rc::Rc::new("Inner".into())));
}

// 16. Real-world shape: an EventEmitter-like class. Registers two handlers,
// `emit` invokes each with the passed argument; both fire.
#[test]
fn t16_event_emitter_shape() {
    let src = r#"
        class Bus {
            constructor() { this.handlers = []; }
            on(h) { this.handlers[this.handlers.length] = h; }
            emit(v) {
                let i = 0;
                let n = this.handlers.length;
                let sum = 0;
                while (i < n) {
                    sum = sum + this.handlers[i](v);
                    i = i + 1;
                }
                return sum;
            }
        }
        const b = new Bus();
        b.on(function(v){ return v + 1; });
        b.on(function(v){ return v * 2; });
        return b.emit(10);
    "#;
    // (10+1) + (10*2) = 11 + 20 = 31
    assert_eq!(run(src), Value::Number(31.0));
}
