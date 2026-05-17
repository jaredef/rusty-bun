//! Ω.5.P37.E1.timers: setTimeout / setInterval / clearTimeout / clearInterval.
//!
//! Macrotask-driven timer substrate. Each timer registers a TimerEntry
//! in a thread_local TIMERS list; the existing PollIo hook (fs.rs)
//! consults `drain_due_timers` between fs work and watcher polling,
//! enqueues a callback-invocation macrotask per due entry, and reschedules
//! interval timers. Keeps the engine alive (PollIo returns true) while
//! any timer is pending; once all timers are cleared and no fs ops or
//! watchers remain, run_to_completion exits.
//!
//! Wires through the standard runtime globals path so consumers reach
//! these via plain `setTimeout(cb, ms)` without imports.

use crate::register::{make_callable, register_method};
use rusty_js_runtime::value::{Object, ObjectRef};
use rusty_js_runtime::{Runtime, Value};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Single registered timer entry.
struct TimerEntry {
    id: u64,
    callback: Value,
    args: Vec<Value>,
    due_at: Instant,
    /// Some(interval) for setInterval entries; None for one-shot setTimeout.
    repeat_ms: Option<u64>,
}

thread_local! {
    static TIMERS: RefCell<Vec<TimerEntry>> = RefCell::new(Vec::new());
    static NEXT_TIMER_ID: RefCell<u64> = RefCell::new(1);
}

fn next_id() -> u64 {
    NEXT_TIMER_ID.with(|c| { let mut c = c.borrow_mut(); let id = *c; *c += 1; id })
}

fn register(callback: Value, args: Vec<Value>, delay_ms: u64, repeat: bool) -> u64 {
    let id = next_id();
    let due_at = Instant::now() + Duration::from_millis(delay_ms);
    let repeat_ms = if repeat { Some(delay_ms.max(1)) } else { None };
    TIMERS.with(|t| {
        t.borrow_mut().push(TimerEntry { id, callback, args, due_at, repeat_ms });
    });
    id
}

fn cancel(id: u64) {
    TIMERS.with(|t| t.borrow_mut().retain(|e| e.id != id));
}

/// Return true if at least one timer is registered. Caller uses this
/// to decide whether to keep the event loop alive.
pub fn has_pending() -> bool {
    TIMERS.with(|t| !t.borrow().is_empty())
}

/// Milliseconds until the soonest-due timer fires. Returns None when
/// no timers are registered.
pub fn next_due_ms() -> Option<u64> {
    let now = Instant::now();
    TIMERS.with(|t| {
        t.borrow().iter().map(|e| {
            if e.due_at <= now { 0 } else { (e.due_at - now).as_millis() as u64 }
        }).min()
    })
}

/// Drain timers whose due_at has elapsed. For each due entry, append
/// (callback, args) to the return vector. Interval timers are
/// rescheduled with their next due_at; one-shot timers are removed.
/// Returns the list to fire so the caller can enqueue them as
/// macrotasks without holding the thread_local borrow.
pub fn drain_due_pairs() -> Vec<(Value, Vec<Value>)> {
    let now = Instant::now();
    let mut fired: Vec<(Value, Vec<Value>)> = Vec::new();
    TIMERS.with(|t| {
        let mut t = t.borrow_mut();
        let mut keep: Vec<TimerEntry> = Vec::with_capacity(t.len());
        for e in t.drain(..) {
            if e.due_at <= now {
                fired.push((e.callback.clone(), e.args.clone()));
                if let Some(ms) = e.repeat_ms {
                    keep.push(TimerEntry {
                        id: e.id,
                        callback: e.callback,
                        args: e.args,
                        due_at: now + Duration::from_millis(ms),
                        repeat_ms: e.repeat_ms,
                    });
                }
            } else {
                keep.push(e);
            }
        }
        *t = keep;
    });
    fired
}

/// Install setTimeout / setInterval / clearTimeout / clearInterval +
/// Node-style queueMicrotask + setImmediate. Returns Timeout-shaped
/// objects from setTimeout/setInterval so `clearTimeout(t)` works on
/// either the id-Number or the object form.
pub fn install(rt: &mut Runtime) {
    fn make_timeout_obj(rt: &mut Runtime, id: u64) -> ObjectRef {
        let o = rt.alloc_object(Object::new_ordinary());
        rt.object_set(o, "__timer_id".into(), Value::Number(id as f64));
        register_method(rt, o, "ref", |rt, _args| Ok(rt.current_this()));
        register_method(rt, o, "unref", |rt, _args| Ok(rt.current_this()));
        register_method(rt, o, "hasRef", |_rt, _args| Ok(Value::Boolean(true)));
        register_method(rt, o, "refresh", |rt, _args| Ok(rt.current_this()));
        // toPrimitive returns the numeric id so `+t === id`.
        let id_for_prim = id as f64;
        register_method(rt, o, "@@toPrimitive", move |_rt, _args| Ok(Value::Number(id_for_prim)));
        register_method(rt, o, "valueOf", move |_rt, _args| Ok(Value::Number(id_for_prim)));
        o
    }
    let set_timeout = make_callable(rt, "setTimeout", |rt, args| {
        let cb = args.first().cloned().unwrap_or(Value::Undefined);
        let delay = args.get(1).and_then(|v| if let Value::Number(n) = v { Some(*n as u64) } else { None }).unwrap_or(0);
        let cb_args: Vec<Value> = args.iter().skip(2).cloned().collect();
        let id = register(cb, cb_args, delay, false);
        Ok(Value::Object(make_timeout_obj(rt, id)))
    });
    rt.globals.insert("setTimeout".into(), Value::Object(set_timeout));

    let set_interval = make_callable(rt, "setInterval", |rt, args| {
        let cb = args.first().cloned().unwrap_or(Value::Undefined);
        let delay = args.get(1).and_then(|v| if let Value::Number(n) = v { Some(*n as u64) } else { None }).unwrap_or(0);
        let cb_args: Vec<Value> = args.iter().skip(2).cloned().collect();
        let id = register(cb, cb_args, delay, true);
        Ok(Value::Object(make_timeout_obj(rt, id)))
    });
    rt.globals.insert("setInterval".into(), Value::Object(set_interval));

    let clear_t = make_callable(rt, "clearTimeout", |rt, args| {
        let id = timer_id_from(rt, args.first().cloned().unwrap_or(Value::Undefined));
        if let Some(id) = id { cancel(id); }
        Ok(Value::Undefined)
    });
    rt.globals.insert("clearTimeout".into(), Value::Object(clear_t));
    let clear_i = make_callable(rt, "clearInterval", |rt, args| {
        let id = timer_id_from(rt, args.first().cloned().unwrap_or(Value::Undefined));
        if let Some(id) = id { cancel(id); }
        Ok(Value::Undefined)
    });
    rt.globals.insert("clearInterval".into(), Value::Object(clear_i));

    // queueMicrotask(cb) — direct microtask enqueue per HTML §8.1.5.6.
    let qmt = make_callable(rt, "queueMicrotask", |rt, args| {
        let cb = args.first().cloned().unwrap_or(Value::Undefined);
        rt.enqueue_microtask("queueMicrotask", move |rt| {
            let _ = rt.call_function(cb, Value::Undefined, Vec::new());
            Ok(())
        });
        Ok(Value::Undefined)
    });
    rt.globals.insert("queueMicrotask".into(), Value::Object(qmt));

    // setImmediate(cb, ...args) — Node-flavored macrotask scheduling.
    // Implemented as setTimeout with 0ms delay.
    let set_immediate = make_callable(rt, "setImmediate", |rt, args| {
        let cb = args.first().cloned().unwrap_or(Value::Undefined);
        let cb_args: Vec<Value> = args.iter().skip(1).cloned().collect();
        let id = register(cb, cb_args, 0, false);
        Ok(Value::Object(make_timeout_obj(rt, id)))
    });
    rt.globals.insert("setImmediate".into(), Value::Object(set_immediate));
    let clear_im = make_callable(rt, "clearImmediate", |rt, args| {
        let id = timer_id_from(rt, args.first().cloned().unwrap_or(Value::Undefined));
        if let Some(id) = id { cancel(id); }
        Ok(Value::Undefined)
    });
    rt.globals.insert("clearImmediate".into(), Value::Object(clear_im));
}

fn timer_id_from(rt: &Runtime, v: Value) -> Option<u64> {
    match v {
        Value::Number(n) => Some(n as u64),
        Value::Object(id) => match rt.object_get(id, "__timer_id") {
            Value::Number(n) => Some(n as u64),
            _ => None,
        },
        _ => None,
    }
}

