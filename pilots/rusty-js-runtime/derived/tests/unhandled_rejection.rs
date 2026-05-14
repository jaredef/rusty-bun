//! HostPromiseRejectionTracker per ECMA-262 §27.2.1.9. A rejection
//! landing with no reject handler is surfaced to the host.

use rusty_js_runtime::promise::{new_promise, reject_promise};
use rusty_js_runtime::value::Value;
use rusty_js_runtime::Runtime;
use std::rc::Rc;

#[test]
fn unhandled_rejection_surfaces() {
    let mut rt = Runtime::new();
    let p = new_promise(&mut rt);
    reject_promise(&mut rt, p, Value::String(Rc::new("bang".into())));
    let drained = rt.drain_unhandled_rejections();
    assert_eq!(drained.len(), 1);
    match &drained[0].1 {
        Value::String(s) => assert_eq!(s.as_str(), "bang"),
        other => panic!("unexpected reason: {:?}", other),
    }
}

#[test]
fn drain_is_idempotent() {
    let mut rt = Runtime::new();
    let p = new_promise(&mut rt);
    reject_promise(&mut rt, p, Value::Number(1.0));
    assert_eq!(rt.drain_unhandled_rejections().len(), 1);
    assert_eq!(rt.drain_unhandled_rejections().len(), 0);
}
