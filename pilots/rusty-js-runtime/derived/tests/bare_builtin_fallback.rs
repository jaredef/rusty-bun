//! Tier-Ω.5.r: bare-specifier → `node:` synonym fallback. Real Node
//! treats `require("crypto")` as a synonym for `require("node:crypto")`
//! when no `node_modules/crypto` directory exists. The fallback runs
//! after the node_modules walk-up fails, so a real package wins.

use rusty_js_runtime::{Runtime, RuntimeError, Value};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static FIXTURE_SEQ: AtomicU32 = AtomicU32::new(0);

fn fixture_dir(tag: &str) -> PathBuf {
    let n = FIXTURE_SEQ.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let p = std::env::temp_dir().join(format!("rusty-js-bare-builtin-{}-{}-{}", pid, n, tag));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("create fixture dir");
    p
}

fn write_file(dir: &PathBuf, name: &str, contents: &str) {
    let p = dir.join(name);
    if let Some(parent) = p.parent() { let _ = fs::create_dir_all(parent); }
    fs::write(&p, contents).expect("write fixture");
}

fn entry_url(path: &PathBuf) -> String {
    format!("file://{}", path.canonicalize().expect("canonicalize").display())
}

fn fresh_runtime_with_host() -> Runtime {
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    rusty_bun_host_v2::install_bun_host(&mut rt, vec![]);
    rt
}

// ─── 1. require("path") resolves to node:path via fallback. ────────────
#[test]
fn t01_require_path_fallback() {
    let dir = fixture_dir("path");
    write_file(&dir, "main.cjs",
        "const p = require(\"path\");\nconst j = p.join(\"a\", \"b\");\nmodule.exports = { j };\n");
    let mut rt = fresh_runtime_with_host();
    let entry = dir.join("main.cjs");
    let url = entry_url(&entry);
    let src = fs::read_to_string(&entry).unwrap();
    rt.evaluate_cjs_module(&src, &url).expect("evaluate_cjs_module");
    let exports = rt.cjs_exports_of(&url).expect("cjs exports");
    let oid = match exports {
        Value::Object(o) => o,
        v => panic!("expected module.exports object, got {:?}", v),
    };
    match rt.object_get(oid, "j") {
        Value::String(s) => assert_eq!(s.as_str(), "a/b"),
        v => panic!("expected 'a/b', got {:?}", v),
    }
}

// ─── 2. require("crypto") resolves to node:crypto stub. ────────────────
#[test]
fn t02_require_crypto_fallback() {
    let dir = fixture_dir("crypto");
    write_file(&dir, "main.cjs",
        "const c = require(\"crypto\");\nconst u = c.randomUUID();\nmodule.exports = { u };\n");
    let mut rt = fresh_runtime_with_host();
    let entry = dir.join("main.cjs");
    let url = entry_url(&entry);
    let src = fs::read_to_string(&entry).unwrap();
    rt.evaluate_cjs_module(&src, &url).expect("evaluate_cjs_module");
    let exports = rt.cjs_exports_of(&url).expect("cjs exports");
    let oid = match exports {
        Value::Object(o) => o,
        v => panic!("expected module.exports object, got {:?}", v),
    };
    match rt.object_get(oid, "u") {
        Value::String(s) => {
            assert!(s.as_str().contains("-"),
                "randomUUID stub should return a dashed placeholder string, got {}", s);
        }
        v => panic!("expected uuid string, got {:?}", v),
    }
}

// ─── 3. Unresolvable bare specifier still errors with a clear message. ─
#[test]
fn t03_unresolvable_bare_specifier_errors() {
    let dir = fixture_dir("missing");
    write_file(&dir, "main.cjs",
        "const x = require(\"not-a-builtin-and-not-installed\");\nmodule.exports = { x };\n");
    let mut rt = fresh_runtime_with_host();
    let entry = dir.join("main.cjs");
    let url = entry_url(&entry);
    let src = fs::read_to_string(&entry).unwrap();
    let r = rt.evaluate_cjs_module(&src, &url);
    match r {
        Err(RuntimeError::TypeError(msg)) => {
            assert!(
                msg.contains("not-a-builtin-and-not-installed"),
                "error should name the unresolvable specifier, got: {}",
                msg
            );
        }
        other => panic!("expected TypeError for unresolvable bare spec, got {:?}", other),
    }
}
