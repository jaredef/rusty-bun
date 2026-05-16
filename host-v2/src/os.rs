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
    // Tier-Ω.5.DDDDDDDD: os.availableParallelism() per Node 20+ API.
    // piscina and other worker-thread packages read this at module-init
    // to size their thread pool. Defaults to 4 (a plausible commodity
    // core count); real impl would query the OS.
    register_method(rt, os, "availableParallelism", |_rt, _args| {
        Ok(Value::Number(4.0))
    });
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

    // Tier-Ω.5.zzzzzz: os.constants.signals per Node convention. human-signals /
    // signal-exit / execa / clipboardy / shelljs read this as a name→number map
    // and `Object.entries(...).map(...)` over it at module-init. Without it,
    // `const { signals } = require('os').constants` destructures undefined.
    let constants = new_object(rt);
    let signals = new_object(rt);
    // Linux signal numbers — Node returns these on Linux platforms; macOS/BSD
    // differ on a few (SIGSTKFLT/SIGPWR are Linux-only). The consumer set
    // only reads from this map; no semantic dependency on the host kernel.
    for (name, num) in &[
        ("SIGHUP", 1), ("SIGINT", 2), ("SIGQUIT", 3), ("SIGILL", 4),
        ("SIGTRAP", 5), ("SIGABRT", 6), ("SIGIOT", 6), ("SIGBUS", 7),
        ("SIGFPE", 8), ("SIGKILL", 9), ("SIGUSR1", 10), ("SIGSEGV", 11),
        ("SIGUSR2", 12), ("SIGPIPE", 13), ("SIGALRM", 14), ("SIGTERM", 15),
        ("SIGSTKFLT", 16), ("SIGCHLD", 17), ("SIGCONT", 18), ("SIGSTOP", 19),
        ("SIGTSTP", 20), ("SIGTTIN", 21), ("SIGTTOU", 22), ("SIGURG", 23),
        ("SIGXCPU", 24), ("SIGXFSZ", 25), ("SIGVTALRM", 26), ("SIGPROF", 27),
        ("SIGWINCH", 28), ("SIGIO", 29), ("SIGPOLL", 29), ("SIGPWR", 30),
        ("SIGSYS", 31), ("SIGUNUSED", 31),
    ] {
        set_constant(rt, signals, name, Value::Number(*num as f64));
    }
    set_constant(rt, constants, "signals", Value::Object(signals));
    set_constant(rt, os, "constants", Value::Object(constants));

    // EOL — child_process / readline writers ask for it.
    set_constant(rt, os, "EOL", Value::String(Rc::new("\n".into())));

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
