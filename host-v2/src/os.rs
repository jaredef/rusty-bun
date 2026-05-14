//! node:os intrinsic — minimal v1 surface.

use crate::register::{new_object, register_method, set_constant};
use rusty_js_runtime::{Runtime, Value};
use std::rc::Rc;

pub fn install(rt: &mut Runtime) {
    let os = new_object(rt);

    register_method(rt, os, "platform", |_rt, _args| {
        Ok(Value::String(Rc::new(detect_platform().to_string())))
    });
    register_method(rt, os, "arch", |_rt, _args| {
        Ok(Value::String(Rc::new(detect_arch().to_string())))
    });
    register_method(rt, os, "type", |_rt, _args| {
        Ok(Value::String(Rc::new(detect_os_type().to_string())))
    });
    register_method(rt, os, "release", |_rt, _args| {
        Ok(Value::String(Rc::new("0.0.0".to_string())))
    });
    register_method(rt, os, "hostname", |_rt, _args| {
        let h = std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string());
        Ok(Value::String(Rc::new(h)))
    });
    register_method(rt, os, "homedir", |_rt, _args| {
        let h = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        Ok(Value::String(Rc::new(h)))
    });
    register_method(rt, os, "tmpdir", |_rt, _args| {
        let t = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
        Ok(Value::String(Rc::new(t)))
    });
    register_method(rt, os, "endianness", |_rt, _args| {
        Ok(Value::String(Rc::new(if cfg!(target_endian = "little") { "LE".into() } else { "BE".into() })))
    });

    set_constant(rt, os, "EOL", Value::String(Rc::new("\n".to_string())));

    rt.globals.insert("os".into(), Value::Object(os));
}

fn detect_platform() -> &'static str {
    if cfg!(target_os = "linux") { "linux" }
    else if cfg!(target_os = "macos") { "darwin" }
    else if cfg!(target_os = "windows") { "win32" }
    else { "unknown" }
}

fn detect_arch() -> &'static str {
    if cfg!(target_arch = "x86_64") { "x64" }
    else if cfg!(target_arch = "aarch64") { "arm64" }
    else if cfg!(target_arch = "arm") { "arm" }
    else { "unknown" }
}

fn detect_os_type() -> &'static str {
    if cfg!(target_os = "linux") { "Linux" }
    else if cfg!(target_os = "macos") { "Darwin" }
    else if cfg!(target_os = "windows") { "Windows_NT" }
    else { "Unknown" }
}
