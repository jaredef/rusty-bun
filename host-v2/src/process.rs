//! process intrinsic — minimal v1 surface.

use crate::register::{new_object, register_method, set_constant};
use rusty_js_runtime::{Runtime, Value};
use std::rc::Rc;

pub fn install(rt: &mut Runtime, argv: Vec<String>) {
    let process = new_object(rt);

    // argv: ["rusty-bun-host-v2", <script>, ...rest]
    let argv_array = rt.alloc_object(rusty_js_runtime::value::Object::new_array());
    for (i, s) in argv.iter().enumerate() {
        rt.object_set(argv_array, i.to_string(), Value::String(Rc::new(s.clone())));
    }
    set_constant(rt, process, "argv", Value::Object(argv_array));

    // env: snapshot of std::env::vars() at startup.
    let env_obj = new_object(rt);
    let vars: Vec<(String, String)> = std::env::vars().collect();
    for (k, v) in vars {
        rt.object_set(env_obj, k, Value::String(Rc::new(v)));
    }
    set_constant(rt, process, "env", Value::Object(env_obj));

    set_constant(rt, process, "platform", Value::String(Rc::new(
        if cfg!(target_os = "linux") { "linux" }
        else if cfg!(target_os = "macos") { "darwin" }
        else { "unknown" }.to_string()
    )));
    set_constant(rt, process, "arch", Value::String(Rc::new(
        if cfg!(target_arch = "x86_64") { "x64" }
        else if cfg!(target_arch = "aarch64") { "arm64" }
        else { "unknown" }.to_string()
    )));
    set_constant(rt, process, "version", Value::String(Rc::new("v0.1.0-rusty-bun".to_string())));
    // Tier-Ω.5.pppp: process.versions for fast-glob + many libs that gate
    // behavior on node major version.
    let versions = new_object(rt);
    rt.object_set(versions, "node".into(), Value::String(Rc::new("20.10.0".into())));
    rt.object_set(versions, "v8".into(), Value::String(Rc::new("11.3.244.8".into())));
    rt.object_set(versions, "uv".into(), Value::String(Rc::new("1.46.0".into())));
    rt.object_set(versions, "modules".into(), Value::String(Rc::new("115".into())));
    set_constant(rt, process, "versions", Value::Object(versions));
    set_constant(rt, process, "pid", Value::Number(std::process::id() as f64));
    // Tier-Ω.5.nnnnnn: process.stdout / stderr / stdin minimal shapes
    // — many libs check isTTY + fd at module-load to choose color/style.
    for (name, fd_num) in [("stdout", 1.0), ("stderr", 2.0), ("stdin", 0.0)] {
        let s = new_object(rt);
        rt.object_set(s, "isTTY".into(), Value::Boolean(false));
        rt.object_set(s, "fd".into(), Value::Number(fd_num));
        rt.object_set(s, "columns".into(), Value::Number(80.0));
        rt.object_set(s, "rows".into(), Value::Number(24.0));
        register_method(rt, s, "write", |_rt, args| {
            if let Some(Value::String(s)) = args.first() { eprint!("{}", s); }
            Ok(Value::Boolean(true))
        });
        register_method(rt, s, "on", |rt, _args| Ok(rt.current_this()));
        set_constant(rt, process, name, Value::Object(s));
    }

    register_method(rt, process, "cwd", |_rt, _args| {
        let cwd = std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "/".to_string());
        Ok(Value::String(Rc::new(cwd)))
    });

    register_method(rt, process, "exit", |_rt, args| {
        let code = args.first().map(|v| {
            rusty_js_runtime::abstract_ops::to_number(v) as i32
        }).unwrap_or(0);
        std::process::exit(code);
    });

    register_method(rt, process, "hrtime", |rt, _args| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        let arr = rt.alloc_object(rusty_js_runtime::value::Object::new_array());
        rt.object_set(arr, "0".into(), Value::Number(d.as_secs() as f64));
        rt.object_set(arr, "1".into(), Value::Number(d.subsec_nanos() as f64));
        Ok(Value::Object(arr))
    });
    // Tier-Ω.5.DDDDDDDD: process.hrtime.bigint() returns nanosecond BigInt
    // since the unix epoch. pino / pino-http call this at module-init for
    // their time-stamping closures.
    if let rusty_js_runtime::Value::Object(hrtime_id) = rt.object_get(process, "hrtime") {
        let bigint_fn: rusty_js_runtime::value::NativeFn = std::rc::Rc::new(|_rt, _args| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let ns = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as i64;
            Ok(Value::BigInt(std::rc::Rc::new(rusty_js_runtime::bigint::JsBigInt::from_i64(ns))))
        });
        let mut bigint_props = indexmap::IndexMap::new();
        rusty_js_runtime::value::install_function_meta_props(&mut bigint_props, "bigint", 0.0);
        let bigint_obj = rusty_js_runtime::value::Object {
            proto: None, extensible: true, properties: bigint_props,
            internal_kind: rusty_js_runtime::value::InternalKind::Function(
                rusty_js_runtime::value::FunctionInternals { name: "bigint".into(), length: 0, native: bigint_fn }
            ),
        };
        let bigint_id = rt.alloc_object(bigint_obj);
        rt.object_set(hrtime_id, "bigint".into(), Value::Object(bigint_id));
    }

    // Tier-Ω.5.DDDDDDDD: process.binding(name) — legacy internal API.
    // mock-fs and a handful of low-level packages probe it. Returns
    // empty namespace object so module-init reads pass; downstream
    // .fs / .uv methods return undefined.
    register_method(rt, process, "binding", |rt, _args| {
        let o = rt.alloc_object(rusty_js_runtime::value::Object::new_ordinary());
        Ok(Value::Object(o))
    });
    // process.report — Node's reporting surface. nx + others touch it.
    let report = rt.alloc_object(rusty_js_runtime::value::Object::new_ordinary());
    rt.object_set(report, "reportOnFatalError".into(), Value::Boolean(false));
    rt.object_set(report, "reportOnSignal".into(), Value::Boolean(false));
    rt.object_set(report, "reportOnUncaughtException".into(), Value::Boolean(false));
    rt.object_set(report, "directory".into(), Value::String(std::rc::Rc::new(String::new())));
    register_method(rt, report, "writeReport", |_rt, _a| Ok(Value::String(std::rc::Rc::new(String::new()))));
    register_method(rt, report, "getReport", |rt, _a| Ok(Value::Object(rt.alloc_object(rusty_js_runtime::value::Object::new_ordinary()))));
    rt.object_set(process, "report".into(), Value::Object(report));

    // Tier-Ω.5.cccc: process.nextTick(cb, ...args) — synchronous-ish
    // queuing of the callback. v1 deviation: invokes immediately since
    // we don't yet have a microtask integration at the JS-callable
    // surface. pump and many CJS streams rely on its existence and
    // single-callback shape.
    register_method(rt, process, "nextTick", |rt, args| {
        if let Some(cb) = args.first().cloned() {
            let rest: Vec<Value> = args.iter().skip(1).cloned().collect();
            let _ = rt.call_function(cb, Value::Undefined, rest);
        }
        Ok(Value::Undefined)
    });
    register_method(rt, process, "emit", |_rt, _args| Ok(Value::Boolean(false)));
    register_method(rt, process, "on", |rt, _args| Ok(rt.current_this()));
    register_method(rt, process, "off", |rt, _args| Ok(rt.current_this()));
    register_method(rt, process, "once", |rt, _args| Ok(rt.current_this()));
    register_method(rt, process, "removeListener", |rt, _args| Ok(rt.current_this()));

    // Tier-Ω.5.mmmm: process.getBuiltinModule(name) — Node 22+ API. ohash
    // calls it at module init to fetch node:crypto without going through
    // the loader.
    register_method(rt, process, "getBuiltinModule", |rt, args| {
        let name = match args.first() {
            Some(Value::String(s)) => s.as_str().to_string(),
            _ => return Ok(Value::Undefined),
        };
        let stripped = name.strip_prefix("node:").unwrap_or(&name);
        match rt.globals.get(stripped).cloned() {
            Some(v) => Ok(v),
            None => Ok(Value::Undefined),
        }
    });

    rt.globals.insert("process".into(), Value::Object(process));
}
