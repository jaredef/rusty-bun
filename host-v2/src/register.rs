//! Native-function registration helpers. Each registers a closure as
//! a callable Value on a host object or globalThis.

use rusty_js_runtime::value::{FunctionInternals, InternalKind, NativeFn, Object, ObjectRef};
use rusty_js_runtime::{Runtime, RuntimeError, Value};
use std::collections::HashMap;
use std::rc::Rc;

pub fn new_object(rt: &mut Runtime) -> ObjectRef {
    rt.alloc_object(Object::new_ordinary())
}

pub fn register_method<F>(rt: &mut Runtime, host: ObjectRef, name: &str, f: F)
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
    let fn_id = rt.alloc_object(fn_obj);
    rt.object_set(host, name.into(), Value::Object(fn_id));
}

pub fn set_constant(rt: &mut Runtime, host: ObjectRef, name: &str, value: Value) {
    rt.object_set(host, name.into(), value);
}

/// Allocate a callable Function object directly. Used for constructor
/// surfaces like `new EventEmitter()` where the global is the function
/// rather than a wrapper object. Returns the object id.
pub fn make_callable<F>(rt: &mut Runtime, name: &str, f: F) -> ObjectRef
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
    rt.alloc_object(fn_obj)
}

pub fn arg_string(args: &[Value], i: usize) -> String {
    use rusty_js_runtime::abstract_ops;
    args.get(i)
        .map(|v| abstract_ops::to_string(v).as_str().to_string())
        .unwrap_or_default()
}

pub fn arg_number(args: &[Value], i: usize) -> f64 {
    use rusty_js_runtime::abstract_ops;
    args.get(i).map(abstract_ops::to_number).unwrap_or(f64::NAN)
}
