//! Native-function registration helpers. Each registers a closure as
//! a callable Value on a host object or globalThis.

use rusty_js_runtime::value::{FunctionInternals, InternalKind, NativeFn, Object};
use rusty_js_runtime::{Runtime, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn new_object() -> Rc<RefCell<Object>> {
    Rc::new(RefCell::new(Object::new_ordinary()))
}

pub fn register_method<F>(host: &Rc<RefCell<Object>>, name: &str, f: F)
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

pub fn set_constant(host: &Rc<RefCell<Object>>, name: &str, value: Value) {
    host.borrow_mut().set_own(name.into(), value);
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
