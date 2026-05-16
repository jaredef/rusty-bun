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

    // Tier-Ω.5.eeeee: cpus / totalmem / freemem / loadavg / uptime /
    // networkInterfaces / userInfo. fast-glob reads os.cpus().length to
    // pick its default worker count; many libs gate behavior on these.
    register_method(rt, os, "cpus", |rt, _args| {
        // Read from /proc/cpuinfo on Linux; otherwise return a 1-entry stub.
        let count = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
        let arr = rt.alloc_object(rusty_js_runtime::value::Object::new_array());
        for i in 0..count {
            let cpu = rt.alloc_object(rusty_js_runtime::value::Object::new_ordinary());
            rt.object_set(cpu, "model".into(), Value::String(Rc::new("Unknown CPU".into())));
            rt.object_set(cpu, "speed".into(), Value::Number(2400.0));
            let times = rt.alloc_object(rusty_js_runtime::value::Object::new_ordinary());
            rt.object_set(times, "user".into(), Value::Number(0.0));
            rt.object_set(times, "nice".into(), Value::Number(0.0));
            rt.object_set(times, "sys".into(), Value::Number(0.0));
            rt.object_set(times, "idle".into(), Value::Number(0.0));
            rt.object_set(times, "irq".into(), Value::Number(0.0));
            rt.object_set(cpu, "times".into(), Value::Object(times));
            rt.object_set(arr, i.to_string(), Value::Object(cpu));
        }
        rt.object_set(arr, "length".into(), Value::Number(count as f64));
        Ok(Value::Object(arr))
    });
    register_method(rt, os, "totalmem", |_rt, _args| Ok(Value::Number(8.0 * 1024.0 * 1024.0 * 1024.0)));
    register_method(rt, os, "freemem", |_rt, _args| Ok(Value::Number(4.0 * 1024.0 * 1024.0 * 1024.0)));
    register_method(rt, os, "uptime", |_rt, _args| Ok(Value::Number(0.0)));
    register_method(rt, os, "loadavg", |rt, _args| {
        let arr = rt.alloc_object(rusty_js_runtime::value::Object::new_array());
        for i in 0..3 { rt.object_set(arr, i.to_string(), Value::Number(0.0)); }
        rt.object_set(arr, "length".into(), Value::Number(3.0));
        Ok(Value::Object(arr))
    });
    register_method(rt, os, "networkInterfaces", |rt, _args| {
        Ok(Value::Object(rt.alloc_object(rusty_js_runtime::value::Object::new_ordinary())))
    });
    register_method(rt, os, "userInfo", |rt, _args| {
        let o = rt.alloc_object(rusty_js_runtime::value::Object::new_ordinary());
        let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
        rt.object_set(o, "username".into(), Value::String(Rc::new(user)));
        rt.object_set(o, "uid".into(), Value::Number(1000.0));
        rt.object_set(o, "gid".into(), Value::Number(1000.0));
        rt.object_set(o, "shell".into(), Value::String(Rc::new("/bin/sh".into())));
        rt.object_set(o, "homedir".into(), Value::String(Rc::new(std::env::var("HOME").unwrap_or_else(|_| "/".into()))));
        Ok(Value::Object(o))
    });

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
