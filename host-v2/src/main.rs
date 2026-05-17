//! rusty-bun-host-v2 entry point. Reads a JS source file, evaluates it
//! through the rusty-js-runtime engine, drives the event loop to
//! completion, exits.

use rusty_bun_host_v2::install_bun_host;
use rusty_js_runtime::{Runtime, Value};
use std::process::ExitCode;

// Ω.5.P46.E1.napi-v1: reference the keepalive array so the linker
// retains every napi_* C symbol. Without this Rust dead-code-strips the
// `#[no_mangle] pub extern "C"` shims (they're not referenced by any
// other Rust code) and dlopen'd .node modules can't resolve them via
// dlsym. The array itself is declared `#[no_mangle] pub static`, so
// just reading its length here is enough to anchor every entry.
#[used]
static _NAPI_RETAIN: usize = rusty_js_runtime::napi::NAPI_KEEPALIVE.len();

fn format_thrown(rt: &Runtime, v: &Value) -> String {
    match v {
        Value::String(s) => format!("Thrown: {}", s),
        Value::Object(id) => {
            let name = match rt.object_get(*id, "name") { Value::String(s) => (*s).clone(), _ => String::new() };
            let message = match rt.object_get(*id, "message") { Value::String(s) => (*s).clone(), _ => String::new() };
            if !name.is_empty() && !message.is_empty() {
                format!("Thrown: {}: {}", name, message)
            } else if !message.is_empty() {
                format!("Thrown: {}", message)
            } else if !name.is_empty() {
                format!("Thrown: {}", name)
            } else {
                format!("Thrown: {:?}", v)
            }
        }
        _ => format!("Thrown: {:?}", v),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: {} <file.mjs>", args.get(0).map(|s| s.as_str()).unwrap_or("rusty-bun-host-v2"));
        return ExitCode::from(64); // EX_USAGE
    }
    let path = args[1].clone();
    let source = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("rusty-bun-host-v2: cannot read {}: {}", path, e);
            return ExitCode::from(66); // EX_NOINPUT
        }
    };

    let mut rt = Runtime::new();
    rt.install_intrinsics();
    install_bun_host(&mut rt, args);

    let url = format!("file://{}", path);
    match rt.evaluate_module(&source, &url) {
        Ok(_namespace) => {}
        Err(e) => {
            // Tier-Ω.5.hhhhh: stringify thrown Error objects via their
            // `message` / `name` properties rather than `[Object #NNN]`.
            // Doc 723 Layer-A: the surface message should at least name
            // what happened.
            let msg = match &e {
                rusty_js_runtime::RuntimeError::Thrown(v) => format_thrown(&rt, v),
                _ => format!("{:?}", e),
            };
            eprintln!("rusty-bun-host-v2: evaluation error: {}", msg);
            return ExitCode::from(70);
        }
    }

    if let Err(e) = rt.run_to_completion() {
        eprintln!("rusty-bun-host-v2: event-loop error: {:?}", e);
        return ExitCode::from(70);
    }

    let unhandled = rt.drain_unhandled_rejections();
    if !unhandled.is_empty() {
        for (_id, reason) in &unhandled {
            eprintln!("rusty-bun-host-v2: unhandled promise rejection: {:?}", reason);
        }
        return ExitCode::from(70);
    }

    ExitCode::SUCCESS
}
