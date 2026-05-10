// rusty-bun-host CLI: `rusty-bun-host <script.js>` runs the file under the
// rquickjs-embedded host with all wired pilots in globalThis.

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: rusty-bun-host <script.js>");
        process::exit(1);
    }
    let source = match fs::read_to_string(&args[1]) {
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
