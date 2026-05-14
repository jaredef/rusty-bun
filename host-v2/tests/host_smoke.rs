//! Smoke tests for rusty-bun-host-v2. Each test drives the host's API
//! programmatically (without going through the binary's argv path).

use rusty_bun_host_v2::install_bun_host;
use rusty_js_runtime::{Runtime, Value};

fn run(src: &str) -> Runtime {
    let module = rusty_js_bytecode::compile_module(src).expect("compile");
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    install_bun_host(&mut rt, vec!["rusty-bun-host-v2".to_string()]);
    rt.run_module(&module).expect("run module");
    rt.run_to_completion().expect("run to completion");
    rt
}

fn recorded(rt: &Runtime) -> Option<Value> {
    rt.globals.get("__last_recorded").cloned()
}

// ─────────── Engine basics still work ───────────

#[test]
fn engine_intrinsics_available() {
    let rt = run("__record(Math.sqrt(16));");
    if let Some(Value::Number(n)) = recorded(&rt) {
        assert_eq!(n, 4.0);
    } else { panic!(); }
}

#[test]
fn json_roundtrip_works() {
    let rt = run(r#"
        let s = JSON.stringify({x: 42});
        let o = JSON.parse(s);
        __record(o.x);
    "#);
    if let Some(Value::Number(n)) = recorded(&rt) {
        assert_eq!(n, 42.0);
    } else { panic!(); }
}

// ─────────── path intrinsic ───────────

#[test]
fn path_basename() {
    let rt = run("__record(path.basename('/foo/bar/baz.js'));");
    if let Some(Value::String(s)) = recorded(&rt) {
        assert_eq!(s.as_str(), "baz.js");
    } else { panic!(); }
}

#[test]
fn path_basename_with_ext() {
    let rt = run("__record(path.basename('/foo/bar/baz.js', '.js'));");
    if let Some(Value::String(s)) = recorded(&rt) {
        assert_eq!(s.as_str(), "baz");
    } else { panic!(); }
}

#[test]
fn path_dirname() {
    let rt = run("__record(path.dirname('/foo/bar/baz.js'));");
    if let Some(Value::String(s)) = recorded(&rt) {
        assert_eq!(s.as_str(), "/foo/bar");
    } else { panic!(); }
}

#[test]
fn path_extname() {
    let rt = run("__record(path.extname('/foo/bar/baz.tar.gz'));");
    if let Some(Value::String(s)) = recorded(&rt) {
        assert_eq!(s.as_str(), ".gz");
    } else { panic!(); }
}

#[test]
fn path_join() {
    let rt = run("__record(path.join('foo', 'bar', 'baz'));");
    if let Some(Value::String(s)) = recorded(&rt) {
        assert_eq!(s.as_str(), "foo/bar/baz");
    } else { panic!(); }
}

#[test]
fn path_normalize_dots() {
    let rt = run("__record(path.normalize('/foo/./bar/../baz'));");
    if let Some(Value::String(s)) = recorded(&rt) {
        assert_eq!(s.as_str(), "/foo/baz");
    } else { panic!(); }
}

#[test]
fn path_is_absolute() {
    let rt = run("__record(path.isAbsolute('/foo'));");
    assert!(matches!(recorded(&rt), Some(Value::Boolean(true))));
    let rt = run("__record(path.isAbsolute('foo'));");
    assert!(matches!(recorded(&rt), Some(Value::Boolean(false))));
}

#[test]
fn path_sep_constant() {
    let rt = run("__record(path.sep);");
    if let Some(Value::String(s)) = recorded(&rt) {
        assert_eq!(s.as_str(), "/");
    } else { panic!(); }
}

// ─────────── os intrinsic ───────────

#[test]
fn os_platform() {
    let rt = run("__record(os.platform());");
    if let Some(Value::String(s)) = recorded(&rt) {
        assert!(matches!(s.as_str(), "linux" | "darwin" | "win32" | "unknown"));
    } else { panic!(); }
}

#[test]
fn os_arch() {
    let rt = run("__record(os.arch());");
    if let Some(Value::String(s)) = recorded(&rt) {
        assert!(matches!(s.as_str(), "x64" | "arm64" | "arm" | "unknown"));
    } else { panic!(); }
}

#[test]
fn os_eol_constant() {
    let rt = run("__record(os.EOL);");
    if let Some(Value::String(s)) = recorded(&rt) {
        assert_eq!(s.as_str(), "\n");
    } else { panic!(); }
}

// ─────────── process intrinsic ───────────

#[test]
fn process_platform() {
    let rt = run("__record(process.platform);");
    if let Some(Value::String(s)) = recorded(&rt) {
        assert!(matches!(s.as_str(), "linux" | "darwin" | "unknown"));
    } else { panic!(); }
}

#[test]
fn process_argv() {
    let rt = run("__record(process.argv[0]);");
    if let Some(Value::String(s)) = recorded(&rt) {
        assert_eq!(s.as_str(), "rusty-bun-host-v2");
    } else { panic!(); }
}

#[test]
fn process_env_present() {
    // process.env should be an object (we can't reliably assert keys
    // since env varies by host, but the global should exist).
    let rt = run("__record(typeof process.env);");
    if let Some(Value::String(s)) = recorded(&rt) {
        assert_eq!(s.as_str(), "object");
    } else { panic!(); }
}

#[test]
fn process_cwd() {
    let rt = run("__record(typeof process.cwd());");
    if let Some(Value::String(s)) = recorded(&rt) {
        assert_eq!(s.as_str(), "string");
    } else { panic!(); }
}

// ─────────── Composed: host + engine intrinsics together ───────────

#[test]
fn composed_path_and_promise() {
    // Promise reaction uses path.basename — exercises the full stack.
    let rt = run(r#"
        Promise.then(Promise.resolve('/foo/bar.js'), function(p) {
            __record(path.basename(p));
            return p;
        });
    "#);
    if let Some(Value::String(s)) = recorded(&rt) {
        assert_eq!(s.as_str(), "bar.js");
    } else { panic!(); }
}
