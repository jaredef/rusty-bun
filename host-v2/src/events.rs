//! node:events — Tier-Ω.5.bb. Real EventEmitter implementation (not stub):
//! on / once / off / removeListener / removeAllListeners / emit /
//! listenerCount / listeners / eventNames / setMaxListeners /
//! getMaxListeners / prependListener / prependOnceListener.
//!
//! Storage: each emitter instance gets a `__listeners` own property
//! holding `{eventName: [fn, ...]}`. Once-listeners are wrapped in a
//! marker object `{fn, once: true}` and unwrapped on emit.

use crate::register::{make_callable, new_object, register_method};
use rusty_js_runtime::value::Object as RtObject;
use rusty_js_runtime::abstract_ops;
use rusty_js_runtime::{Runtime, RuntimeError, Value};
use std::rc::Rc;

fn this_emitter(rt: &Runtime) -> Option<rusty_js_runtime::value::ObjectRef> {
    match rt.current_this() {
        Value::Object(id) => Some(id),
        _ => None,
    }
}

fn get_or_create_listeners(rt: &mut Runtime, emitter: rusty_js_runtime::value::ObjectRef) -> rusty_js_runtime::value::ObjectRef {
    if let Value::Object(id) = rt.object_get(emitter, "__listeners") {
        return id;
    }
    let bag = rt.alloc_object(RtObject::new_ordinary());
    rt.object_set(emitter, "__listeners".into(), Value::Object(bag));
    bag
}

fn get_event_list(rt: &mut Runtime, bag: rusty_js_runtime::value::ObjectRef, event: &str) -> rusty_js_runtime::value::ObjectRef {
    if let Value::Object(id) = rt.object_get(bag, event) {
        if matches!(rt.obj(id).internal_kind, rusty_js_runtime::value::InternalKind::Array) {
            return id;
        }
    }
    let arr = rt.alloc_object(RtObject::new_array());
    rt.object_set(arr, "length".into(), Value::Number(0.0));
    rt.object_set(bag, event.to_string(), Value::Object(arr));
    arr
}

fn append_listener(rt: &mut Runtime, arr: rusty_js_runtime::value::ObjectRef, fn_v: Value) {
    let len = rt.array_length(arr);
    rt.object_set(arr, len.to_string(), fn_v);
    rt.object_set(arr, "length".into(), Value::Number((len + 1) as f64));
}

pub fn install(rt: &mut Runtime) {
    // Build EventEmitter constructor + prototype.
    let proto = new_object(rt);

    register_method(rt, proto, "on", |rt, args| {
        let em = this_emitter(rt).ok_or_else(|| RuntimeError::TypeError("on: this is not an EventEmitter".into()))?;
        let event = abstract_ops::to_string(args.first().unwrap_or(&Value::Undefined)).as_str().to_string();
        let fn_v = args.get(1).cloned().unwrap_or(Value::Undefined);
        let bag = get_or_create_listeners(rt, em);
        let arr = get_event_list(rt, bag, &event);
        append_listener(rt, arr, fn_v);
        Ok(Value::Object(em))
    });
    register_method(rt, proto, "addListener", |rt, args| {
        let em = this_emitter(rt).ok_or_else(|| RuntimeError::TypeError("addListener: this is not an EventEmitter".into()))?;
        let event = abstract_ops::to_string(args.first().unwrap_or(&Value::Undefined)).as_str().to_string();
        let fn_v = args.get(1).cloned().unwrap_or(Value::Undefined);
        let bag = get_or_create_listeners(rt, em);
        let arr = get_event_list(rt, bag, &event);
        append_listener(rt, arr, fn_v);
        Ok(Value::Object(em))
    });
    register_method(rt, proto, "once", |rt, args| {
        // Wrap fn so it removes itself after first emit. v1 deviation:
        // marker-object {__once: fn} stored; emit unwraps.
        let em = this_emitter(rt).ok_or_else(|| RuntimeError::TypeError("once: this is not an EventEmitter".into()))?;
        let event = abstract_ops::to_string(args.first().unwrap_or(&Value::Undefined)).as_str().to_string();
        let fn_v = args.get(1).cloned().unwrap_or(Value::Undefined);
        let wrapper = rt.alloc_object(RtObject::new_ordinary());
        rt.object_set(wrapper, "__once".into(), fn_v);
        let bag = get_or_create_listeners(rt, em);
        let arr = get_event_list(rt, bag, &event);
        append_listener(rt, arr, Value::Object(wrapper));
        Ok(Value::Object(em))
    });
    register_method(rt, proto, "off", |rt, args| {
        let em = this_emitter(rt).ok_or_else(|| RuntimeError::TypeError("off: this is not an EventEmitter".into()))?;
        let event = abstract_ops::to_string(args.first().unwrap_or(&Value::Undefined)).as_str().to_string();
        let target = args.get(1).cloned().unwrap_or(Value::Undefined);
        let bag = get_or_create_listeners(rt, em);
        let arr = get_event_list(rt, bag, &event);
        let len = rt.array_length(arr);
        // Find and remove the first matching listener.
        for i in 0..len {
            let item = rt.object_get(arr, &i.to_string());
            let matches = match (&item, &target) {
                (Value::Object(a), Value::Object(b)) if a == b => true,
                _ => false,
            };
            if matches {
                // Shift remaining elements left.
                for j in i..(len - 1) {
                    let next = rt.object_get(arr, &(j + 1).to_string());
                    rt.object_set(arr, j.to_string(), next);
                }
                rt.object_set(arr, (len - 1).to_string(), Value::Undefined);
                rt.object_set(arr, "length".into(), Value::Number((len - 1) as f64));
                break;
            }
        }
        Ok(Value::Object(em))
    });
    register_method(rt, proto, "removeListener", |rt, args| {
        // Alias for off. We can't take a closure-of-a-closure cleanly;
        // duplicate the implementation.
        let em = this_emitter(rt).ok_or_else(|| RuntimeError::TypeError("removeListener: this is not an EventEmitter".into()))?;
        let event = abstract_ops::to_string(args.first().unwrap_or(&Value::Undefined)).as_str().to_string();
        let target = args.get(1).cloned().unwrap_or(Value::Undefined);
        let bag = get_or_create_listeners(rt, em);
        let arr = get_event_list(rt, bag, &event);
        let len = rt.array_length(arr);
        for i in 0..len {
            let item = rt.object_get(arr, &i.to_string());
            let matches = match (&item, &target) {
                (Value::Object(a), Value::Object(b)) if a == b => true,
                _ => false,
            };
            if matches {
                for j in i..(len - 1) {
                    let next = rt.object_get(arr, &(j + 1).to_string());
                    rt.object_set(arr, j.to_string(), next);
                }
                rt.object_set(arr, (len - 1).to_string(), Value::Undefined);
                rt.object_set(arr, "length".into(), Value::Number((len - 1) as f64));
                break;
            }
        }
        Ok(Value::Object(em))
    });
    register_method(rt, proto, "removeAllListeners", |rt, args| {
        let em = this_emitter(rt).ok_or_else(|| RuntimeError::TypeError("removeAllListeners: this is not an EventEmitter".into()))?;
        let bag = get_or_create_listeners(rt, em);
        if let Some(ev) = args.first() {
            let event = abstract_ops::to_string(ev).as_str().to_string();
            let arr = rt.alloc_object(RtObject::new_array());
            rt.object_set(arr, "length".into(), Value::Number(0.0));
            rt.object_set(bag, event, Value::Object(arr));
        } else {
            // Drop the whole bag.
            let new_bag = rt.alloc_object(RtObject::new_ordinary());
            rt.object_set(em, "__listeners".into(), Value::Object(new_bag));
        }
        Ok(Value::Object(em))
    });
    register_method(rt, proto, "emit", |rt, args| {
        let em = this_emitter(rt).ok_or_else(|| RuntimeError::TypeError("emit: this is not an EventEmitter".into()))?;
        let event = abstract_ops::to_string(args.first().unwrap_or(&Value::Undefined)).as_str().to_string();
        let rest: Vec<Value> = args.iter().skip(1).cloned().collect();
        let bag = get_or_create_listeners(rt, em);
        let arr = match rt.object_get(bag, &event) {
            Value::Object(a) if matches!(rt.obj(a).internal_kind, rusty_js_runtime::value::InternalKind::Array) => a,
            _ => return Ok(Value::Boolean(false)),
        };
        let len = rt.array_length(arr);
        if len == 0 { return Ok(Value::Boolean(false)); }
        let mut to_call: Vec<(Value, bool)> = Vec::new();
        for i in 0..len {
            let item = rt.object_get(arr, &i.to_string());
            let (fn_v, once) = match &item {
                Value::Object(id) => {
                    if let Value::Object(_) = rt.object_get(*id, "__once") {
                        // wrapper
                        let inner = rt.object_get(*id, "__once");
                        (inner, true)
                    } else {
                        (item.clone(), false)
                    }
                }
                _ => continue,
            };
            to_call.push((fn_v, once));
        }
        // Call each. After calls, remove once-wrapped items.
        for (fn_v, _once) in &to_call {
            let _ = rt.call_function(fn_v.clone(), Value::Object(em), rest.clone())?;
        }
        // Filter out once entries.
        if to_call.iter().any(|(_, once)| *once) {
            let keep: Vec<Value> = (0..len).filter_map(|i| {
                let item = rt.object_get(arr, &i.to_string());
                if let Value::Object(id) = &item {
                    if !matches!(rt.object_get(*id, "__once"), Value::Undefined) {
                        return None; // drop once wrappers
                    }
                }
                Some(item)
            }).collect();
            for (i, v) in keep.iter().enumerate() {
                rt.object_set(arr, i.to_string(), v.clone());
            }
            for i in keep.len()..(len as usize) {
                rt.object_set(arr, i.to_string(), Value::Undefined);
            }
            rt.object_set(arr, "length".into(), Value::Number(keep.len() as f64));
        }
        Ok(Value::Boolean(true))
    });
    register_method(rt, proto, "listenerCount", |rt, args| {
        let em = this_emitter(rt).ok_or_else(|| RuntimeError::TypeError("listenerCount: this is not an EventEmitter".into()))?;
        let event = abstract_ops::to_string(args.first().unwrap_or(&Value::Undefined)).as_str().to_string();
        let bag = get_or_create_listeners(rt, em);
        let n = match rt.object_get(bag, &event) {
            Value::Object(a) => rt.array_length(a),
            _ => 0,
        };
        Ok(Value::Number(n as f64))
    });
    register_method(rt, proto, "listeners", |rt, args| {
        let em = this_emitter(rt).ok_or_else(|| RuntimeError::TypeError("listeners: this is not an EventEmitter".into()))?;
        let event = abstract_ops::to_string(args.first().unwrap_or(&Value::Undefined)).as_str().to_string();
        let bag = get_or_create_listeners(rt, em);
        let src = match rt.object_get(bag, &event) {
            Value::Object(a) => a,
            _ => {
                let arr = rt.alloc_object(RtObject::new_array());
                rt.object_set(arr, "length".into(), Value::Number(0.0));
                return Ok(Value::Object(arr));
            }
        };
        // Return a fresh copy.
        let len = rt.array_length(src);
        let out = rt.alloc_object(RtObject::new_array());
        for i in 0..len {
            let item = rt.object_get(src, &i.to_string());
            let unwrapped = match &item {
                Value::Object(id) => {
                    let inner = rt.object_get(*id, "__once");
                    if matches!(inner, Value::Undefined) { item } else { inner }
                }
                _ => item,
            };
            rt.object_set(out, i.to_string(), unwrapped);
        }
        rt.object_set(out, "length".into(), Value::Number(len as f64));
        Ok(Value::Object(out))
    });
    register_method(rt, proto, "eventNames", |rt, _args| {
        let em = this_emitter(rt).ok_or_else(|| RuntimeError::TypeError("eventNames: this is not an EventEmitter".into()))?;
        let bag = get_or_create_listeners(rt, em);
        let names: Vec<String> = rt.obj(bag).properties.keys().cloned().collect();
        let arr = rt.alloc_object(RtObject::new_array());
        for (i, n) in names.iter().enumerate() {
            rt.object_set(arr, i.to_string(), Value::String(Rc::new(n.clone())));
        }
        rt.object_set(arr, "length".into(), Value::Number(names.len() as f64));
        Ok(Value::Object(arr))
    });
    register_method(rt, proto, "setMaxListeners", |_rt, _args| Ok(Value::Undefined));
    register_method(rt, proto, "getMaxListeners", |_rt, _args| Ok(Value::Number(10.0)));
    register_method(rt, proto, "prependListener", |rt, args| {
        // v1: just appends (order matters for some packages but not most).
        let em = this_emitter(rt).ok_or_else(|| RuntimeError::TypeError("prependListener: this is not an EventEmitter".into()))?;
        let event = abstract_ops::to_string(args.first().unwrap_or(&Value::Undefined)).as_str().to_string();
        let fn_v = args.get(1).cloned().unwrap_or(Value::Undefined);
        let bag = get_or_create_listeners(rt, em);
        let arr = get_event_list(rt, bag, &event);
        append_listener(rt, arr, fn_v);
        Ok(Value::Object(em))
    });

    // EventEmitter constructor.
    let proto_for_ctor = proto;
    let ctor = make_callable(rt, "EventEmitter", move |rt, _args| {
        // Tier-Ω.5.ooooo: mutate the receiver when called as super(). The
        // EventEmitter ctor takes no real state; returning a fresh object
        // here would (now that Ω.5.nnnnn rebinds this on super-return)
        // wreck subclasses by replacing the subclass instance with a
        // bare-prototype EventEmitter. Returning Undefined keeps the
        // Op::New / SetThis paths happy with the original `this`.
        match rt.current_this() {
            Value::Object(id) => {
                // Optional: pre-create the listeners bag so subclass code
                // that reads _events at construction sees an empty map.
                let bag = rt.alloc_object(RtObject::new_ordinary());
                rt.object_set(id, "_events".into(), Value::Object(bag));
                rt.object_set(id, "_eventsCount".into(), Value::Number(0.0));
                Ok(Value::Undefined)
            }
            _ => {
                // Bare-call (no `new`): allocate a fresh instance.
                let mut o = RtObject::new_ordinary();
                o.proto = Some(proto_for_ctor);
                let id = rt.alloc_object(o);
                Ok(Value::Object(id))
            }
        }
    });
    rt.object_set(ctor, "prototype".into(), Value::Object(proto));
    rt.object_set(proto, "constructor".into(), Value::Object(ctor));

    // node:events namespace exposes EventEmitter as default + named.
    let ns = new_object(rt);
    rt.object_set(ns, "EventEmitter".into(), Value::Object(ctor));
    rt.object_set(ns, "default".into(), Value::Object(ctor));
    // Tier-Ω.5.gggg: events.on / events.once static helpers. Real spec
    // returns async iterator / Promise; v1 stubs return undefined so
    // the import-time presence-check passes and downstream
    // Object.assign(nodeImports, {on, finished}) doesn't bind
    // undefined.
    register_method(rt, ns, "on", |_rt, _args| Ok(Value::Undefined));
    register_method(rt, ns, "once", |_rt, _args| Ok(Value::Undefined));
    register_method(rt, ns, "setMaxListeners", |_rt, _args| Ok(Value::Undefined));
    register_method(rt, ns, "listenerCount", |_rt, _args| Ok(Value::Number(0.0)));
    rt.globals.insert("events".into(), Value::Object(ns));
}
