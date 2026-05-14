//! Promise intrinsic end-to-end tests. Per ECMA-262 §27.2 + Doc 714 §VI
//! Consequence 5. Verifies that Promise reactions route through the
//! engine's JobQueue via HostEnqueuePromiseJob (engine-side microtask
//! enqueue).

use rusty_js_runtime::{Runtime, Value};

fn run_with_jobs(src: &str) -> Runtime {
    let module = rusty_js_bytecode::compile_module(src).expect("compile");
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    rt.run_module(&module).expect("run module");
    rt.run_to_completion().expect("run to completion");
    rt
}

fn recorded(rt: &Runtime) -> Option<Value> {
    rt.globals.get("__last_recorded").cloned()
}

// ─────────── Single-then chain ───────────

#[test]
fn promise_resolve_then_records() {
    let rt = run_with_jobs(r#"
        Promise.then(Promise.resolve(42), function(x) { __record(x); return x; });
    "#);
    if let Some(Value::Number(n)) = recorded(&rt) {
        assert_eq!(n, 42.0);
    } else { panic!("recorded: {:?}", recorded(&rt)); }
}

// ─────────── Two-then chain (the canonical end-to-end) ───────────

#[test]
fn promise_resolve_double_then_records_84() {
    let rt = run_with_jobs(r#"
        let p = Promise.resolve(42);
        let q = Promise.then(p, function(x) { return x * 2; });
        Promise.then(q, function(x) { __record(x); return x; });
    "#);
    if let Some(Value::Number(n)) = recorded(&rt) {
        assert_eq!(n, 84.0, "Promise.resolve(42).then(*2).then(record) → 84");
    } else { panic!("recorded: {:?}", recorded(&rt)); }
}

// ─────────── Three-then chain ───────────

#[test]
fn promise_chain_three_steps() {
    let rt = run_with_jobs(r#"
        let p = Promise.resolve(1);
        let p2 = Promise.then(p, function(x) { return x + 10; });
        let p3 = Promise.then(p2, function(x) { return x * 3; });
        Promise.then(p3, function(x) { __record(x); return x; });
    "#);
    if let Some(Value::Number(n)) = recorded(&rt) {
        // 1 → 1+10=11 → 11*3=33
        assert_eq!(n, 33.0);
    } else { panic!("recorded: {:?}", recorded(&rt)); }
}

// ─────────── Reject + catch ───────────

#[test]
fn promise_reject_with_catch() {
    let rt = run_with_jobs(r#"
        let p = Promise.reject('boom');
        Promise.catch_(p, function(reason) { __record(reason); return reason; });
    "#);
    if let Some(Value::String(s)) = recorded(&rt) {
        assert_eq!(s.as_str(), "boom");
    } else { panic!("recorded: {:?}", recorded(&rt)); }
}

// ─────────── Microtask ordering: Promise reactions interleave properly ───────────

#[test]
fn promise_reactions_run_after_module_body() {
    // Per §9.4.1: Promise reactions are microtasks. They MUST NOT run
    // during the module body's synchronous portion; they run only after
    // the module returns and run_to_completion enters the microtask drain.
    //
    // Verification: __record(0) runs synchronously; the reaction's
    // __record(99) runs after; the final value is 99 (reaction
    // overwrote the synchronous value).
    let rt = run_with_jobs(r#"
        __record(0);
        Promise.then(Promise.resolve(99), function(x) { __record(x); return x; });
    "#);
    if let Some(Value::Number(n)) = recorded(&rt) {
        assert_eq!(n, 99.0, "reaction overwrites synchronous record");
    } else { panic!(); }
}

// ─────────── Handler throw → chain rejects → catch catches ───────────

#[test]
fn handler_throw_propagates_to_catch() {
    let rt = run_with_jobs(r#"
        let p = Promise.resolve(1);
        let p2 = Promise.then(p, function(x) { throw 'handler-error'; });
        Promise.catch_(p2, function(reason) { __record(reason); return reason; });
    "#);
    if let Some(Value::String(s)) = recorded(&rt) {
        assert_eq!(s.as_str(), "handler-error");
    } else { panic!("recorded: {:?}", recorded(&rt)); }
}
