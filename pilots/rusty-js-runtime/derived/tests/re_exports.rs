//! Tier-Ω.5.h: ESM re-export forms. Eight acceptance tests covering:
//!   1. Named re-export (`export { x } from`).
//!   2. Named re-export with rename (`export { x as y } from`).
//!   3. Star re-export (`export * from`).
//!   4. Star re-export skips default (per ECMA-262 §16.2.3.7).
//!   5. Star-as re-export (`export * as ns from`).
//!   6. Default re-export (`export { default } from`).
//!   7. Default-to-named conversion (`export { default as x } from`).
//!   8. Named-to-default conversion (`export { x as default } from`).
//!
//! Snapshot semantics (v1 deviation from spec live-bindings) — source
//! modules are loaded eagerly during the link phase so namespaces are
//! populated when the namespace-build phase reads them.

use rusty_js_runtime::{Runtime, Value};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static FIXTURE_SEQ: AtomicU32 = AtomicU32::new(0);

fn fixture_dir(tag: &str) -> PathBuf {
    let n = FIXTURE_SEQ.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let p = std::env::temp_dir().join(format!("rusty-js-reexports-{}-{}-{}", pid, n, tag));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("create fixture dir");
    p
}

fn write_file(dir: &PathBuf, name: &str, contents: &str) -> PathBuf {
    let p = dir.join(name);
    if let Some(parent) = p.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(&p, contents).expect("write fixture");
    p
}

fn entry_url(path: &PathBuf) -> String {
    format!("file://{}", path.canonicalize().expect("canonicalize").display())
}

fn fresh_runtime() -> Runtime {
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    rt
}

fn load(rt: &mut Runtime, dir: &PathBuf, entry: &str) -> rusty_js_runtime::ObjectRef {
    let entry_path = dir.join(entry);
    let url = entry_url(&entry_path);
    let src = fs::read_to_string(&entry_path).expect("read entry");
    rt.evaluate_module(&src, &url).expect("evaluate_module")
}

fn expect_num(rt: &Runtime, ns: rusty_js_runtime::ObjectRef, key: &str, expected: f64) {
    match rt.object_get(ns, key) {
        Value::Number(n) => assert_eq!(n, expected, "key {}", key),
        v => panic!("expected number for {}, got {:?}", key, v),
    }
}

// ─── 1. Named re-export. ───────────────────────────────────────────────
#[test]
fn t01_named_reexport() {
    let dir = fixture_dir("named");
    write_file(&dir, "mod.mjs",
        "function add() { return 1 }\nexport { add };\n");
    write_file(&dir, "index.mjs",
        "export { add } from \"./mod.mjs\";\n");
    write_file(&dir, "main.mjs", r#"
        import { add } from "./index.mjs";
        const result = add();
        export { result };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    expect_num(&rt, ns, "result", 1.0);
}

// ─── 2. Named re-export with rename. ───────────────────────────────────
#[test]
fn t02_named_reexport_rename() {
    let dir = fixture_dir("rename");
    write_file(&dir, "mod.mjs",
        "function add() { return 7 }\nexport { add };\n");
    write_file(&dir, "index.mjs",
        "export { add as sum } from \"./mod.mjs\";\n");
    write_file(&dir, "main.mjs", r#"
        import { sum } from "./index.mjs";
        const result = sum();
        export { result };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    expect_num(&rt, ns, "result", 7.0);
}

// ─── 3. Star re-export. ────────────────────────────────────────────────
#[test]
fn t03_star_reexport() {
    let dir = fixture_dir("star");
    write_file(&dir, "mod.mjs", r#"
        function a() { return 10 }
        function b() { return 20 }
        function c() { return 30 }
        export { a, b, c };
    "#);
    write_file(&dir, "index.mjs",
        "export * from \"./mod.mjs\";\n");
    write_file(&dir, "main.mjs", r#"
        import { a, b, c } from "./index.mjs";
        const ra = a();
        const rb = b();
        const rc = c();
        export { ra, rb, rc };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    expect_num(&rt, ns, "ra", 10.0);
    expect_num(&rt, ns, "rb", 20.0);
    expect_num(&rt, ns, "rc", 30.0);
}

// ─── 4. Star re-export skips default. ──────────────────────────────────
#[test]
fn t04_star_reexport_skips_default() {
    let dir = fixture_dir("star-no-default");
    write_file(&dir, "mod.mjs", r#"
        function named() { return 42 }
        export { named };
        export default function() { return 99 }
    "#);
    write_file(&dir, "index.mjs",
        "export * from \"./mod.mjs\";\n");
    write_file(&dir, "main.mjs", r#"
        import * as ns from "./index.mjs";
        const named_v = ns.named();
        const default_kind = typeof ns.default;
        export { named_v, default_kind };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    expect_num(&rt, ns, "named_v", 42.0);
    match rt.object_get(ns, "default_kind") {
        Value::String(s) => assert_eq!(s.as_str(), "undefined"),
        v => panic!("expected 'undefined' string for default_kind, got {:?}", v),
    }
}

// ─── 5. Star-as re-export. ─────────────────────────────────────────────
#[test]
fn t05_star_as_reexport() {
    let dir = fixture_dir("star-as");
    write_file(&dir, "mod.mjs", r#"
        function add(a, b) { return a + b }
        export { add };
    "#);
    write_file(&dir, "index.mjs",
        "export * as utils from \"./mod.mjs\";\n");
    write_file(&dir, "main.mjs", r#"
        import { utils } from "./index.mjs";
        const result = utils.add(2, 3);
        export { result };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    expect_num(&rt, ns, "result", 5.0);
}

// ─── 6. Default re-export. ─────────────────────────────────────────────
#[test]
fn t06_default_reexport() {
    let dir = fixture_dir("default");
    write_file(&dir, "mod.mjs",
        "export default function() { return 42 }\n");
    write_file(&dir, "index.mjs",
        "export { default } from \"./mod.mjs\";\n");
    write_file(&dir, "main.mjs", r#"
        import f from "./index.mjs";
        const result = f();
        export { result };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    expect_num(&rt, ns, "result", 42.0);
}

// ─── 7. Default-to-named conversion. ───────────────────────────────────
#[test]
fn t07_default_as_named() {
    let dir = fixture_dir("default-as-named");
    write_file(&dir, "mod.mjs",
        "export default function() { return 8 }\n");
    write_file(&dir, "index.mjs",
        "export { default as add } from \"./mod.mjs\";\n");
    write_file(&dir, "main.mjs", r#"
        import { add } from "./index.mjs";
        const result = add();
        export { result };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    expect_num(&rt, ns, "result", 8.0);
}

// ─── 8. Named-to-default conversion. ───────────────────────────────────
#[test]
fn t08_named_as_default() {
    let dir = fixture_dir("named-as-default");
    write_file(&dir, "mod.mjs", r#"
        function add() { return 13 }
        export { add };
    "#);
    write_file(&dir, "index.mjs",
        "export { add as default } from \"./mod.mjs\";\n");
    write_file(&dir, "main.mjs", r#"
        import f from "./index.mjs";
        const result = f();
        export { result };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    expect_num(&rt, ns, "result", 13.0);
}
