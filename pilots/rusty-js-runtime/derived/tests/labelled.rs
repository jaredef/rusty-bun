//! Tier-Ω.5.o acceptance — labelled statements + labelled break/continue.

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
fn t01_labelled_break_outer() {
    let rt = run_rt(r#"
        let r = "";
        outer: for (let i = 0; i < 3; i++) {
            for (let j = 0; j < 3; j++) {
                if (i === 1 && j === 1) break outer;
                r = r + i + "" + j + ",";
            }
        }
        __record(r);
    "#);
    if let Value::String(s) = recorded(&rt) { assert_eq!(s.as_str(), "00,01,02,10,"); } else { panic!(); }
}

#[test]
fn t02_labelled_continue_outer() {
    let rt = run_rt(r#"
        let r = "";
        outer: for (let i = 0; i < 3; i++) {
            for (let j = 0; j < 3; j++) {
                if (j === 1) continue outer;
                r = r + i + "" + j + ",";
            }
        }
        __record(r);
    "#);
    if let Value::String(s) = recorded(&rt) { assert_eq!(s.as_str(), "00,10,20,"); } else { panic!(); }
}

#[test]
fn t03_plain_break_only_inner() {
    let rt = run_rt(r#"
        let r = "";
        outer: for (let i = 0; i < 2; i++) {
            for (let j = 0; j < 3; j++) {
                if (j === 1) break;
                r = r + i + "" + j + ",";
            }
            r = r + "|";
        }
        __record(r);
    "#);
    if let Value::String(s) = recorded(&rt) { assert_eq!(s.as_str(), "00,|10,|"); } else { panic!(); }
}

#[test]
fn t04_labelled_while() {
    let rt = run_rt(r#"
        let i = 0;
        loop: while (true) {
            i = i + 1;
            if (i > 3) break loop;
        }
        __record(i);
    "#);
    assert_eq!(recorded(&rt), Value::Number(4.0));
}

#[test]
fn t05_labelled_block_break() {
    let rt = run_rt(r#"
        let r = 0;
        outer: {
            r = 1;
            break outer;
            r = 99;
        }
        __record(r);
    "#);
    assert_eq!(recorded(&rt), Value::Number(1.0));
}
