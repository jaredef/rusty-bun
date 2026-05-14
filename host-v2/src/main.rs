//! rusty-bun-host-v2 entry point. Reads a JS source file, evaluates it
//! through the rusty-js-runtime engine, drives the event loop to
//! completion, exits.

use rusty_bun_host_v2::install_bun_host;
use rusty_js_runtime::Runtime;
use std::process::ExitCode;

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
            eprintln!("rusty-bun-host-v2: evaluation error: {:?}", e);
            return ExitCode::from(70); // EX_SOFTWARE
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
