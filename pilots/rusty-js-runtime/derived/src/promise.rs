//! Promise intrinsic + reaction routing through the JobQueue per
//! ECMA-262 §27.2 + the Doc 714 §VI Consequence 5 architectural shift.
//!
//! Round 3.e.d migrates Promise objects onto the managed heap. Promise
//! state + reactions are accessed through `rt.obj(id)` / `rt.obj_mut(id)`.

use crate::interp::{Runtime, RuntimeError};
use crate::value::{
    FunctionInternals, InternalKind, NativeFn, Object, ObjectRef, PromiseReaction,
    PromiseState, PromiseStatus, Value,
};
use std::collections::HashMap;
use std::rc::Rc;

impl Runtime {
    pub fn install_promise(&mut self) {
        // Tier-Ω.5.kkk: Promise installed as a real Function (InternalKind::Function),
        // not an ordinary object. Without this, `new Promise((r, j) => {...})`
        // hit "callee is not callable: Object(kind=ordinary)" — load-bearing
        // for p-defer, ky, jose, got, get-stream, and any package whose
        // module-init constructs a Promise (~5 packages in route-2 ERR).
        // The constructor takes the executor function, builds a fresh
        // pending Promise, invokes the executor with resolve/reject
        // callbacks, and returns the promise.
        let promise_ctor = crate::intrinsics::make_native("Promise", |rt, args| {
            let executor = match args.first() {
                Some(v @ Value::Object(_)) => v.clone(),
                _ => return Err(RuntimeError::TypeError("Promise constructor: executor must be a function".into())),
            };
            let p = new_promise(rt);
            let p_for_resolve = p;
            let p_for_reject = p;
            let resolve_fn = crate::intrinsics::make_native("<promise-resolve>", move |rt, args| {
                let v = args.first().cloned().unwrap_or(Value::Undefined);
                resolve_promise(rt, p_for_resolve, v);
                Ok(Value::Undefined)
            });
            let reject_fn = crate::intrinsics::make_native("<promise-reject>", move |rt, args| {
                let v = args.first().cloned().unwrap_or(Value::Undefined);
                reject_promise(rt, p_for_reject, v);
                Ok(Value::Undefined)
            });
            let resolve_id = rt.alloc_object(resolve_fn);
            let reject_id = rt.alloc_object(reject_fn);
            // Synchronously invoke executor(resolve, reject) per spec.
            let _ = rt.call_function(executor, Value::Undefined, vec![
                Value::Object(resolve_id),
                Value::Object(reject_id),
            ])?;
            Ok(Value::Object(p))
        });
        let promise_obj = self.alloc_object(promise_ctor);
        register_method(self, promise_obj, "resolve", |rt, args| {
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            let p = new_promise(rt);
            resolve_promise(rt, p, v);
            Ok(Value::Object(p))
        });
        register_method(self, promise_obj, "reject", |rt, args| {
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            let p = new_promise(rt);
            reject_promise(rt, p, v);
            Ok(Value::Object(p))
        });
        register_method(self, promise_obj, "then", |rt, args| {
            let source = match args.first() {
                Some(Value::Object(id)) => *id,
                _ => return Err(RuntimeError::TypeError("Promise.then: first arg must be a Promise".into())),
            };
            let on_fulfilled = args.get(1).cloned();
            let on_rejected = args.get(2).cloned();
            let chain = new_promise(rt);
            let (status, value) = {
                let s = rt.obj(source);
                if let InternalKind::Promise(ps) = &s.internal_kind {
                    (ps.status, ps.value.clone())
                } else {
                    return Err(RuntimeError::TypeError("Promise.then: first arg not a Promise object".into()));
                }
            };
            match status {
                PromiseStatus::Pending => {
                    let src = rt.obj_mut(source);
                    if let InternalKind::Promise(ps) = &mut src.internal_kind {
                        ps.fulfill_reactions.push(PromiseReaction {
                            handler: on_fulfilled.clone(),
                            chain,
                        });
                        ps.reject_reactions.push(PromiseReaction {
                            handler: on_rejected.clone(),
                            chain,
                        });
                    }
                }
                PromiseStatus::Fulfilled => {
                    enqueue_reaction(rt, on_fulfilled, value, chain, false);
                }
                PromiseStatus::Rejected => {
                    rt.pending_unhandled.remove(&source);
                    enqueue_reaction(rt, on_rejected, value, chain, true);
                }
            }
            Ok(Value::Object(chain))
        });
        register_method(self, promise_obj, "catch_", |rt, args| {
            let source = match args.first() {
                Some(Value::Object(id)) => *id,
                _ => return Err(RuntimeError::TypeError("Promise.catch_: first arg must be a Promise".into())),
            };
            let on_rejected = args.get(1).cloned();
            let chain = new_promise(rt);
            let (status, value) = {
                let s = rt.obj(source);
                if let InternalKind::Promise(ps) = &s.internal_kind {
                    (ps.status, ps.value.clone())
                } else { return Err(RuntimeError::TypeError("not a Promise".into())); }
            };
            match status {
                PromiseStatus::Pending => {
                    let src = rt.obj_mut(source);
                    if let InternalKind::Promise(ps) = &mut src.internal_kind {
                        ps.fulfill_reactions.push(PromiseReaction { handler: None, chain });
                        ps.reject_reactions.push(PromiseReaction { handler: on_rejected.clone(), chain });
                    }
                }
                PromiseStatus::Fulfilled => {
                    enqueue_reaction(rt, None, value, chain, false);
                }
                PromiseStatus::Rejected => {
                    rt.pending_unhandled.remove(&source);
                    enqueue_reaction(rt, on_rejected, value, chain, true);
                }
            }
            Ok(Value::Object(chain))
        });
        if let Some(proto) = self.promise_prototype {
            self.object_set(promise_obj, "prototype".into(), Value::Object(proto));
        }
        self.globals.insert("Promise".into(), Value::Object(promise_obj));
    }
}

/// Create a new Pending Promise object on the managed heap.
pub fn new_promise(rt: &mut Runtime) -> ObjectRef {
    rt.alloc_object(Object {
        proto: None,
        extensible: true,
        properties: indexmap::IndexMap::new(),
        internal_kind: InternalKind::Promise(PromiseState {
            status: PromiseStatus::Pending,
            value: Value::Undefined,
            fulfill_reactions: Vec::new(),
            reject_reactions: Vec::new(),
        }),
    })
}

pub fn resolve_promise(rt: &mut Runtime, promise: ObjectRef, value: Value) {
    let reactions = {
        let p = rt.obj_mut(promise);
        if let InternalKind::Promise(ps) = &mut p.internal_kind {
            if !matches!(ps.status, PromiseStatus::Pending) { return; }
            ps.status = PromiseStatus::Fulfilled;
            ps.value = value;
            std::mem::take(&mut ps.fulfill_reactions)
        } else { return; }
    };
    let value = match &rt.obj(promise).internal_kind {
        InternalKind::Promise(ps) => ps.value.clone(),
        _ => Value::Undefined,
    };
    for reaction in reactions {
        enqueue_reaction(rt, reaction.handler, value.clone(), reaction.chain, false);
    }
}

pub fn reject_promise(rt: &mut Runtime, promise: ObjectRef, reason: Value) {
    let reactions = {
        let p = rt.obj_mut(promise);
        if let InternalKind::Promise(ps) = &mut p.internal_kind {
            if !matches!(ps.status, PromiseStatus::Pending) { return; }
            ps.status = PromiseStatus::Rejected;
            ps.value = reason;
            std::mem::take(&mut ps.reject_reactions)
        } else { return; }
    };
    // Per §27.2.1.9 HostPromiseRejectionTracker: a rejection landing with
    // no reject reaction attached is a candidate unhandled rejection.
    // .then / .catch_ removes the entry if a handler attaches later (still
    // valid only because the source promise is already Rejected at that
    // point, so the spec-side "unhandledrejection" event timing collapses).
    if reactions.is_empty() {
        rt.pending_unhandled.insert(promise);
    }
    let value = match &rt.obj(promise).internal_kind {
        InternalKind::Promise(ps) => ps.value.clone(),
        _ => Value::Undefined,
    };
    for reaction in reactions {
        enqueue_reaction(rt, reaction.handler, value.clone(), reaction.chain, true);
    }
}

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
                    Ok(result) => { resolve_promise(rt, chain, result); }
                    Err(e) => {
                        let thrown = match e {
                            RuntimeError::Thrown(v) => v,
                            other => Value::String(std::rc::Rc::new(format!("{:?}", other))),
                        };
                        reject_promise(rt, chain, thrown);
                    }
                }
            }
            None => {
                if is_rejected {
                    reject_promise(rt, chain, value);
                } else {
                    resolve_promise(rt, chain, value);
                }
            }
        }
        Ok(())
    });
}

fn register_method<F>(rt: &mut Runtime, host: ObjectRef, name: &str, f: F)
where F: Fn(&mut Runtime, &[Value]) -> Result<Value, RuntimeError> + 'static {
    let native: NativeFn = Rc::new(f);
    let mut properties = indexmap::IndexMap::new();
    crate::value::install_function_meta_props(&mut properties, name, 0.0);
    let fn_obj = Object {
        proto: None,
        extensible: true,
        properties,
        internal_kind: InternalKind::Function(FunctionInternals {
            name: name.to_string(),
            length: 0,
            native,
        }),
    };
    let fn_id = rt.alloc_object(fn_obj);
    rt.object_set(host, name.into(), Value::Object(fn_id));
}
