//! process intrinsic — minimal v1 surface (argv / env / cwd / platform /
//! version / exit). The pre-Ω host's process module also wired event
//! emitter + signal handlers; those land in Ω.4.e alongside the mio
//! reactor migration.

use crate::register::{new_object, register_method, set_constant};
use rusty_js_runtime::{Runtime, Value};
use std::cell::RefCell;
use std::rc::Rc;

pub fn install(rt: &mut Runtime, argv: Vec<String>) {
    let process = new_object();

    // argv: ["rusty-bun-host-v2", <script>, ...rest]
    let argv_array = Rc::new(RefCell::new(rusty_js_runtime::value::Object::new_array()));
    for (i, s) in argv.iter().enumerate() {
        argv_array.borrow_mut().set_own(i.to_string(), Value::String(Rc::new(s.clone())));
    }
    set_constant(&process, "argv", Value::Object(argv_array));

    // env: snapshot of std::env::vars() at startup.
    let env_obj = new_object();
    for (k, v) in std::env::vars() {
        env_obj.borrow_mut().set_own(k, Value::String(Rc::new(v)));
    }
    set_constant(&process, "env", Value::Object(env_obj));

    set_constant(&process, "platform", Value::String(Rc::new(
        if cfg!(target_os = "linux") { "linux" }
        else if cfg!(target_os = "macos") { "darwin" }
        else { "unknown" }.to_string()
    )));
    set_constant(&process, "arch", Value::String(Rc::new(
        if cfg!(target_arch = "x86_64") { "x64" }
        else if cfg!(target_arch = "aarch64") { "arm64" }
        else { "unknown" }.to_string()
    )));
    set_constant(&process, "version", Value::String(Rc::new("v0.1.0-rusty-bun".to_string())));
    set_constant(&process, "pid", Value::Number(std::process::id() as f64));

    register_method(&process, "cwd", |_rt, _args| {
        let cwd = std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "/".to_string());
        Ok(Value::String(Rc::new(cwd)))
    });

    register_method(&process, "exit", |_rt, args| {
        let code = args.first().map(|v| {
            rusty_js_runtime::abstract_ops::to_number(v) as i32
        }).unwrap_or(0);
        std::process::exit(code);
    });

    register_method(&process, "hrtime", |_rt, _args| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        // Returns [seconds, nanoseconds] as an array.
        let arr = Rc::new(RefCell::new(rusty_js_runtime::value::Object::new_array()));
        arr.borrow_mut().set_own("0".into(), Value::Number(d.as_secs() as f64));
        arr.borrow_mut().set_own("1".into(), Value::Number(d.subsec_nanos() as f64));
        Ok(Value::Object(arr))
    });

    rt.globals.insert("process".into(), Value::Object(process));
}
