// rusty-bun-host CLI: `rusty-bun-host <script>` runs a file under the
// rquickjs-embedded host with all wired pilots in globalThis.
//
// Two execution modes selected by file extension:
//   .mjs  →  eval_esm_module (top-level await + imports supported)
//   .js   →  ctx.eval script mode (legacy CJS-style scripts only)
//
// The parity-measure tool (host/tools/parity-measure.sh) depends on
// the .mjs path so it can probe `import pkg from "..."` shapes
// against real npm package trees.

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: rusty-bun-host <script.js|script.mjs>");
        process::exit(1);
    }
    let path = &args[1];
    if path.ends_with(".mjs") {
        match rusty_bun_host::eval_esm_module(path) {
            Ok(s) => {
                print!("{}", s);
                if !s.ends_with('\n') { println!(); }
                process::exit(0);
            }
            Err(e) => {
                eprintln!("eval error: {}", e);
                process::exit(1);
            }
        }
    }
    // .js (script mode)
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("read error: {}", e);
            process::exit(1);
        }
    };
    let (_runtime, context) = match rusty_bun_host::new_runtime() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("runtime init: {:?}", e);
            process::exit(1);
        }
    };
    let exit_code = context.with(|ctx| -> i32 {
        match ctx.eval::<rquickjs::Value, _>(source.as_bytes()) {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("eval error: {:?}", e);
                1
            }
        }
    });
    process::exit(exit_code);
}
