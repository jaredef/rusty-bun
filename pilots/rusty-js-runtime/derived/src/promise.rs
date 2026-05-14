//! Promise intrinsic + reaction routing through the JobQueue per
//! ECMA-262 §27.2 + the Doc 714 §VI Consequence 5 architectural shift
//! (event loop inside the engine).
//!
//! v1 (round 3.f.d) provides:
//!   - `Promise.resolve(v)` / `Promise.reject(v)` statics
//!   - `Promise.then(promise, onFulfilled[, onRejected])` static
//!     (instance-method form deferred until prototype-chain wiring)
//!   - Internal `resolve_promise` / `reject_promise` helpers that drain
//!     pending reactions through `Runtime::enqueue_microtask`
//!
//! Spec correspondence:
//!   - PromiseReactionJob (§27.2.2.1) implemented inline in the
//!     microtask closure
//!   - HostEnqueuePromiseJob (§9.5) implemented as `enqueue_microtask`
//!   - .then chaining produces a new Pending promise per §27.2.5.4
//!   - Thenable resolution (returning a Promise from a handler) is
//!     simplified in v1: returned Promises don't auto-flatten; the
//!     chain is fulfilled with the Promise value directly. Full
//!     thenable resolution is a follow-on per design spec §VII.

use crate::interp::{Runtime, RuntimeError};
use crate::value::{
    FunctionInternals, InternalKind, NativeFn, Object, ObjectRef, PromiseReaction,
    PromiseState, PromiseStatus, Value,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

impl Runtime {
    pub fn install_promise(&mut self) {
        let promise_obj = new_object();
        register_method(&promise_obj, "resolve", |rt, args| {
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            let p = new_promise();
            resolve_promise(rt, &p, v);
            Ok(Value::Object(p))
        });
        register_method(&promise_obj, "reject", |rt, args| {
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            let p = new_promise();
            reject_promise(rt, &p, v);
            Ok(Value::Object(p))
        });
        register_method(&promise_obj, "then", |rt, args| {
            let source = match args.first() {
                Some(Value::Object(o)) => o.clone(),
                _ => return Err(RuntimeError::TypeError("Promise.then: first arg must be a Promise".into())),
            };
            let on_fulfilled = args.get(1).cloned();
            let on_rejected = args.get(2).cloned();
            let chain = new_promise();
            let (status, value) = {
                let s = source.borrow();
                if let InternalKind::Promise(ps) = &s.internal_kind {
                    (ps.status, ps.value.clone())
                } else {
                    return Err(RuntimeError::TypeError("Promise.then: first arg not a Promise object".into()));
                }
            };
            match status {
                PromiseStatus::Pending => {
                    let mut src = source.borrow_mut();
                    if let InternalKind::Promise(ps) = &mut src.internal_kind {
                        ps.fulfill_reactions.push(PromiseReaction {
                            handler: on_fulfilled.clone(),
                            chain: chain.clone(),
                        });
                        ps.reject_reactions.push(PromiseReaction {
                            handler: on_rejected.clone(),
                            chain: chain.clone(),
                        });
                    }
                }
                PromiseStatus::Fulfilled => {
                    enqueue_reaction(rt, on_fulfilled, value, chain.clone(), false);
                }
                PromiseStatus::Rejected => {
                    enqueue_reaction(rt, on_rejected, value, chain.clone(), true);
                }
            }
            Ok(Value::Object(chain))
        });
        // catch(p, fn) — convenience for then(p, undefined, fn)
        register_method(&promise_obj, "catch_", |rt, args| {
            let source = match args.first() {
                Some(Value::Object(o)) => o.clone(),
                _ => return Err(RuntimeError::TypeError("Promise.catch_: first arg must be a Promise".into())),
            };
            let on_rejected = args.get(1).cloned();
            let chain = new_promise();
            let (status, value) = {
                let s = source.borrow();
                if let InternalKind::Promise(ps) = &s.internal_kind {
                    (ps.status, ps.value.clone())
                } else { return Err(RuntimeError::TypeError("not a Promise".into())); }
            };
            match status {
                PromiseStatus::Pending => {
                    let mut src = source.borrow_mut();
                    if let InternalKind::Promise(ps) = &mut src.internal_kind {
                        ps.fulfill_reactions.push(PromiseReaction { handler: None, chain: chain.clone() });
                        ps.reject_reactions.push(PromiseReaction { handler: on_rejected.clone(), chain: chain.clone() });
                    }
                }
                PromiseStatus::Fulfilled => {
                    enqueue_reaction(rt, None, value, chain.clone(), false);
                }
                PromiseStatus::Rejected => {
                    enqueue_reaction(rt, on_rejected, value, chain.clone(), true);
                }
            }
            Ok(Value::Object(chain))
        });
        self.globals.insert("Promise".into(), Value::Object(promise_obj));
    }
}

/// Create a new Pending Promise object.
pub fn new_promise() -> ObjectRef {
    Rc::new(RefCell::new(Object {
        proto: None,
        extensible: true,
        properties: HashMap::new(),
        internal_kind: InternalKind::Promise(PromiseState {
            status: PromiseStatus::Pending,
            value: Value::Undefined,
            fulfill_reactions: Vec::new(),
            reject_reactions: Vec::new(),
        }),
    }))
}

/// Resolve a Promise. If already settled, no-op. If pending, transition
/// to Fulfilled and drain fulfill_reactions as microtasks.
pub fn resolve_promise(rt: &mut Runtime, promise: &ObjectRef, value: Value) {
    let reactions = {
        let mut p = promise.borrow_mut();
        if let InternalKind::Promise(ps) = &mut p.internal_kind {
            if !matches!(ps.status, PromiseStatus::Pending) { return; }
            ps.status = PromiseStatus::Fulfilled;
            ps.value = value;
            std::mem::take(&mut ps.fulfill_reactions)
        } else { return; }
    };
    let value = match &promise.borrow().internal_kind {
        InternalKind::Promise(ps) => ps.value.clone(),
        _ => Value::Undefined,
    };
    for reaction in reactions {
        enqueue_reaction(rt, reaction.handler, value.clone(), reaction.chain, false);
    }
}

pub fn reject_promise(rt: &mut Runtime, promise: &ObjectRef, reason: Value) {
    let reactions = {
        let mut p = promise.borrow_mut();
        if let InternalKind::Promise(ps) = &mut p.internal_kind {
            if !matches!(ps.status, PromiseStatus::Pending) { return; }
            ps.status = PromiseStatus::Rejected;
            ps.value = reason;
            std::mem::take(&mut ps.reject_reactions)
        } else { return; }
    };
    let value = match &promise.borrow().internal_kind {
        InternalKind::Promise(ps) => ps.value.clone(),
        _ => Value::Undefined,
    };
    for reaction in reactions {
        enqueue_reaction(rt, reaction.handler, value.clone(), reaction.chain, true);
    }
}

/// Enqueue a microtask that runs a Promise reaction handler and resolves
/// the chained Promise with the result (or propagates rejection if the
/// handler throws).
fn enqueue_reaction(
    rt: &mut Runtime,
    handler: Option<Value>,
    value: Value,
    chain: ObjectRef,
    is_rejected: bool,
) {
    rt.enqueue_microtask("PromiseReactionJob", move |rt| {
        match handler {
            Some(h) => {
                match rt.call_function(h, Value::Undefined, vec![value]) {
                    Ok(result) => { resolve_promise(rt, &chain, result); }
                    Err(e) => {
                        // Per spec §27.2.2.1: if handler throws, the chain
                        // is rejected with the thrown value.
                        let thrown = match e {
                            RuntimeError::Thrown(v) => v,
                            other => Value::String(std::rc::Rc::new(format!("{:?}", other))),
                        };
                        reject_promise(rt, &chain, thrown);
                    }
                }
            }
            None => {
                // No handler — pass through. If rejected, propagate the
                // rejection; if fulfilled, propagate the value.
                if is_rejected {
                    reject_promise(rt, &chain, value);
                } else {
                    resolve_promise(rt, &chain, value);
                }
            }
        }
        Ok(())
    });
}

// ─────────── Local helpers ───────────

fn new_object() -> Rc<RefCell<Object>> {
    Rc::new(RefCell::new(Object::new_ordinary()))
}

fn register_method<F>(host: &Rc<RefCell<Object>>, name: &str, f: F)
where F: Fn(&mut Runtime, &[Value]) -> Result<Value, RuntimeError> + 'static {
    let native: NativeFn = Rc::new(f);
    let fn_obj = Object {
        proto: None,
        extensible: true,
        properties: HashMap::new(),
        internal_kind: InternalKind::Function(FunctionInternals {
            name: name.to_string(),
            native,
        }),
    };
    host.borrow_mut().set_own(name.into(), Value::Object(Rc::new(RefCell::new(fn_obj))));
}
