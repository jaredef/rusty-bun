//! Tier-Ω.5.m acceptance — switch statement lowering.
//!
//! Covers ECMA-262 §14.12.4 semantics:
//!   - Strict-equality dispatch.
//!   - Fall-through between adjacent case bodies absent `break`.
//!   - `default` may appear anywhere in textual order.
//!   - `break` inside a switch targets the switch end.
//!   - `continue` inside a switch propagates to the enclosing loop.

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

// 1. Basic case match.
#[test]
fn t01_basic_case() {
    let rt = run_rt(r#"
        let r = "";
        switch (1) { case 1: r = "one"; break; }
        __record(r);
    "#);
    if let Value::String(s) = recorded(&rt) { assert_eq!(s.as_str(), "one"); } else { panic!(); }
}

// 2. Default branch.
#[test]
fn t02_default_branch() {
    let rt = run_rt(r#"
        let r = "";
        switch (99) { case 1: r = "one"; break; default: r = "other"; }
        __record(r);
    "#);
    if let Value::String(s) = recorded(&rt) { assert_eq!(s.as_str(), "other"); } else { panic!(); }
}

// 3. Fall-through between case bodies.
#[test]
fn t03_fall_through() {
    let rt = run_rt(r#"
        let r = "";
        switch (1) { case 1: r = r + "one;"; case 2: r = r + "two;"; break; }
        __record(r);
    "#);
    if let Value::String(s) = recorded(&rt) { assert_eq!(s.as_str(), "one;two;"); } else { panic!(); }
}

// 4. Break terminates fall-through.
#[test]
fn t04_break_in_chain() {
    let rt = run_rt(r#"
        let r = "";
        switch (1) { case 1: r = r + "a"; break; case 2: r = r + "b"; }
        __record(r);
    "#);
    if let Value::String(s) = recorded(&rt) { assert_eq!(s.as_str(), "a"); } else { panic!(); }
}

// 5. Default placed mid-switch is reached when nothing matches.
#[test]
fn t05_default_in_middle() {
    let rt = run_rt(r#"
        let r = "";
        switch (99) {
            case 1: r = "one"; break;
            default: r = "def"; break;
            case 2: r = "two";
        }
        __record(r);
    "#);
    if let Value::String(s) = recorded(&rt) { assert_eq!(s.as_str(), "def"); } else { panic!(); }
}

// 6. Default falls through to subsequent cases.
#[test]
fn t06_default_falls_through() {
    let rt = run_rt(r#"
        let r = "";
        switch (99) {
            default: r = r + "d;";
            case 1: r = r + "one;";
        }
        __record(r);
    "#);
    if let Value::String(s) = recorded(&rt) { assert_eq!(s.as_str(), "d;one;"); } else { panic!(); }
}

// 7. Strict equality — "1" does not match 1.
#[test]
fn t07_strict_equality() {
    let rt = run_rt(r#"
        let r = "";
        switch ("1") {
            case 1: r = "loose"; break;
            case "1": r = "strict"; break;
        }
        __record(r);
    "#);
    if let Value::String(s) = recorded(&rt) { assert_eq!(s.as_str(), "strict"); } else { panic!(); }
}

// 8. `break` inside a switch nested in a loop targets only the switch.
#[test]
fn t08_break_inner_switch_only() {
    let rt = run_rt(r#"
        let r = "";
        for (const i of [1, 2, 3]) {
            switch (i) {
                case 2: r = r + "X"; break;
                default: r = r + i;
            }
        }
        __record(r);
    "#);
    if let Value::String(s) = recorded(&rt) { assert_eq!(s.as_str(), "1X3"); } else { panic!(); }
}

// 9. `continue` inside a switch propagates to enclosing loop (extra coverage).
#[test]
fn t09_continue_propagates() {
    let rt = run_rt(r#"
        let r = "";
        for (const i of [1, 2, 3]) {
            switch (i) {
                case 2: continue;
                default: r = r + i;
            }
            r = r + "!";
        }
        __record(r);
    "#);
    if let Value::String(s) = recorded(&rt) { assert_eq!(s.as_str(), "1!3!"); } else { panic!(); }
}
