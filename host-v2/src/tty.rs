//! node:tty stub — Tier-Ω.5.y. Import-time + shape probe only.

use crate::register::{new_object, register_method};
use rusty_js_runtime::{Runtime, RuntimeError, Value};
use std::rc::Rc;

fn stub(name: &'static str) -> impl Fn(&mut Runtime, &[Value]) -> Result<Value, RuntimeError> {
    move |_rt, _args| {
        Err(RuntimeError::Thrown(Value::String(Rc::new(format!(
            "TypeError: node:tty.{name} not yet implemented (Tier-Ω.5.y stub)"
        )))))
    }
}

pub fn install(rt: &mut Runtime) {
    let t = new_object(rt);
    register_method(rt, t, "isatty", |_rt, _args| Ok(Value::Boolean(false)));
    register_method(rt, t, "ReadStream", stub("ReadStream"));
    register_method(rt, t, "WriteStream", stub("WriteStream"));
    rt.globals.insert("tty".into(), Value::Object(t));
}
