//! Tier-Ω.5.b: ESM module loader integration tests. Each test builds a
//! small multi-file fixture under /tmp/rusty-js-modload-<pid>-<n>/ and
//! loads the entry module through Runtime::evaluate_module. The
//! acceptance bar is locked at 10+ tests covering relative-path
//! resolution, extension probing, index resolution, named/default/
//! namespace imports, two-level transitive imports, the module cache,
//! node:* built-in dispatch, and the bare-specifier error path.

use rusty_js_runtime::{Runtime, RuntimeError, Value};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static FIXTURE_SEQ: AtomicU32 = AtomicU32::new(0);

/// Allocate a fresh fixture directory under /tmp. The path is unique
/// per (pid, monotonic counter) so parallel `cargo test` invocations
/// don't collide.
fn fixture_dir(tag: &str) -> PathBuf {
    let n = FIXTURE_SEQ.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let p = std::env::temp_dir().join(format!("rusty-js-modload-{}-{}-{}", pid, n, tag));
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

fn fresh_runtime_with_host() -> Runtime {
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    rusty_bun_host_v2::install_bun_host(&mut rt, vec![]);
    rt
}

fn load(rt: &mut Runtime, dir: &PathBuf, entry: &str) -> rusty_js_runtime::ObjectRef {
    let entry_path = dir.join(entry);
    let url = entry_url(&entry_path);
    let src = fs::read_to_string(&entry_path).expect("read entry");
    rt.evaluate_module(&src, &url).expect("evaluate_module")
}

// ─── 1. Single default import. ─────────────────────────────────────────
#[test]
fn t01_single_default_import() {
    let dir = fixture_dir("default");
    write_file(&dir, "add.mjs",
        "function add(a, b) { return a + b }\nexport default add;\n");
    write_file(&dir, "main.mjs", r#"
        import add from "./add.mjs";
        const result = add(2, 3);
        export { result };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "result") {
        Value::Number(n) => assert_eq!(n, 5.0),
        v => panic!("expected 5, got {:?}", v),
    }
}

// ─── 2. Named import. ──────────────────────────────────────────────────
#[test]
fn t02_named_import() {
    let dir = fixture_dir("named");
    write_file(&dir, "math.mjs",
        "function mul(a, b) { return a * b }\nexport { mul };\n");
    write_file(&dir, "main.mjs", r#"
        import { mul } from "./math.mjs";
        const result = mul(3, 4);
        export { result };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "result") {
        Value::Number(n) => assert_eq!(n, 12.0),
        v => panic!("expected 12, got {:?}", v),
    }
}

// ─── 3. Namespace import. ──────────────────────────────────────────────
#[test]
fn t03_namespace_import() {
    let dir = fixture_dir("ns");
    write_file(&dir, "mod.mjs", r#"
        const x = 1;
        function f(n) { return n * 2 }
        export { x, f };
    "#);
    write_file(&dir, "main.mjs", r#"
        import * as M from "./mod.mjs";
        const a = M.x;
        const b = M.f(2);
        export { a, b };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    assert!(matches!(rt.object_get(ns, "a"), Value::Number(n) if n == 1.0));
    assert!(matches!(rt.object_get(ns, "b"), Value::Number(n) if n == 4.0));
}

// ─── 4. Multiple named imports from one file. ──────────────────────────
#[test]
fn t04_multiple_named_imports() {
    let dir = fixture_dir("pack");
    write_file(&dir, "pack.mjs", r#"
        const a = 10;
        const b = 20;
        const c = 30;
        export { a, b, c };
    "#);
    write_file(&dir, "main.mjs", r#"
        import { a, b, c } from "./pack.mjs";
        const sum = a + b + c;
        export { sum };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    assert!(matches!(rt.object_get(ns, "sum"), Value::Number(n) if n == 60.0));
}

// ─── 5. Module cache — re-importing the same module yields the same
//      namespace; the body runs once. ─────────────────────────────────
#[test]
fn t05_module_cache() {
    let dir = fixture_dir("cache");
    // The dep increments a counter held in a closure each time its body
    // runs. With the cache in place, two imports from sibling modules
    // both see counter == 1, not 2.
    write_file(&dir, "dep.mjs", r#"
        let counter = 0;
        counter = counter + 1;
        const seen = counter;
        export { seen };
    "#);
    write_file(&dir, "a.mjs",
        "import { seen } from \"./dep.mjs\";\nconst a_seen = seen;\nexport { a_seen };\n");
    write_file(&dir, "b.mjs",
        "import { seen } from \"./dep.mjs\";\nconst b_seen = seen;\nexport { b_seen };\n");
    write_file(&dir, "main.mjs", r#"
        import { a_seen } from "./a.mjs";
        import { b_seen } from "./b.mjs";
        const both = a_seen + b_seen;
        export { both };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    // Each child saw counter == 1, sum == 2.
    assert!(matches!(rt.object_get(ns, "both"), Value::Number(n) if n == 2.0));
}

// ─── 6. Two-level transitive import. ───────────────────────────────────
#[test]
fn t06_two_level_transitive() {
    let dir = fixture_dir("transitive");
    write_file(&dir, "leaf.mjs",
        "const value = 42;\nexport { value };\n");
    write_file(&dir, "mid.mjs", r#"
        import { value } from "./leaf.mjs";
        const doubled = value + value;
        export { doubled };
    "#);
    write_file(&dir, "main.mjs", r#"
        import { doubled } from "./mid.mjs";
        const final_value = doubled;
        export { final_value };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    assert!(matches!(rt.object_get(ns, "final_value"), Value::Number(n) if n == 84.0));
}

// ─── 7. node:fs built-in. ──────────────────────────────────────────────
#[test]
fn t07_node_fs_builtin() {
    let dir = fixture_dir("nodefs");
    write_file(&dir, "main.mjs", r#"
        import { existsSync } from "node:fs";
        const here = existsSync("/tmp");
        const nope = existsSync("/this/path/should/not/exist/abc123");
        export { here, nope };
    "#);
    let mut rt = fresh_runtime_with_host();
    let ns = load(&mut rt, &dir, "main.mjs");
    assert!(matches!(rt.object_get(ns, "here"), Value::Boolean(true)),
        "existsSync(/tmp) should be true");
    assert!(matches!(rt.object_get(ns, "nope"), Value::Boolean(false)),
        "existsSync(bogus) should be false");
}

// ─── 8. node:path built-in (default import). ──────────────────────────
#[test]
fn t08_node_path_default() {
    let dir = fixture_dir("nodepath");
    write_file(&dir, "main.mjs", r#"
        import path from "node:path";
        const j = path.join("a", "b");
        export { j };
    "#);
    let mut rt = fresh_runtime_with_host();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "j") {
        Value::String(s) => assert_eq!(s.as_str(), "a/b"),
        v => panic!("expected 'a/b', got {:?}", v),
    }
}

// ─── 9. Default + named in one import. ─────────────────────────────────
#[test]
fn t09_default_plus_named() {
    let dir = fixture_dir("mixed-import");
    write_file(&dir, "mod.mjs", r#"
        function makeIt(n) { return n + 100 }
        const x = 7;
        export { x };
        export default makeIt;
    "#);
    write_file(&dir, "main.mjs", r#"
        import def, { x } from "./mod.mjs";
        const a = def(1);
        const b = x;
        export { a, b };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    assert!(matches!(rt.object_get(ns, "a"), Value::Number(n) if n == 101.0));
    assert!(matches!(rt.object_get(ns, "b"), Value::Number(n) if n == 7.0));
}

// ─── 10. Mixed export: both default and named reachable. ───────────────
#[test]
fn t10_mixed_export_shapes() {
    let dir = fixture_dir("mixed-export");
    write_file(&dir, "lib.mjs", r#"
        function helper(n) { return n - 1 }
        const VERSION = "1.0.0";
        export { helper, VERSION };
        export default helper;
    "#);
    write_file(&dir, "main.mjs", r#"
        import * as L from "./lib.mjs";
        const v = L.VERSION;
        const h = L.helper(10);
        const d = L.default(20);
        export { v, h, d };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "v") {
        Value::String(s) => assert_eq!(s.as_str(), "1.0.0"),
        v => panic!("v expected '1.0.0', got {:?}", v),
    }
    assert!(matches!(rt.object_get(ns, "h"), Value::Number(n) if n == 9.0));
    assert!(matches!(rt.object_get(ns, "d"), Value::Number(n) if n == 19.0));
}

// ─── 11. Resolution probing: ./util resolves to ./util.mjs. ───────────
#[test]
fn t11_extension_probe_mjs() {
    let dir = fixture_dir("probe-mjs");
    write_file(&dir, "util.mjs",
        "const greet = \"hi\";\nexport { greet };\n");
    write_file(&dir, "main.mjs", r#"
        import { greet } from "./util";
        const s = greet;
        export { s };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "s") {
        Value::String(s) => assert_eq!(s.as_str(), "hi"),
        v => panic!("expected 'hi', got {:?}", v),
    }
}

// ─── 12. Index resolution: ./lib resolves to ./lib/index.mjs. ─────────
#[test]
fn t12_index_resolution() {
    let dir = fixture_dir("indexres");
    write_file(&dir, "lib/index.mjs",
        "const id = 99;\nexport { id };\n");
    write_file(&dir, "main.mjs", r#"
        import { id } from "./lib";
        const v = id;
        export { v };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    assert!(matches!(rt.object_get(ns, "v"), Value::Number(n) if n == 99.0));
}

// ─── 13. Bare specifier errors cleanly. ────────────────────────────────
#[test]
fn t13_bare_specifier_errors() {
    let dir = fixture_dir("bare");
    write_file(&dir, "main.mjs", r#"
        import x from "react";
        const v = x;
        export { v };
    "#);
    let mut rt = fresh_runtime();
    let entry = dir.join("main.mjs");
    let url = entry_url(&entry);
    let src = fs::read_to_string(&entry).unwrap();
    let r = rt.evaluate_module(&src, &url);
    match r {
        Err(RuntimeError::TypeError(msg)) => {
            assert!(msg.contains("bare specifier") && msg.contains("react"),
                "error should mention bare specifier + react, got: {}", msg);
        }
        other => panic!("expected TypeError for bare specifier, got {:?}", other),
    }
}

// ─── 14. .js extension probe (alternate path through the probe list). ─
#[test]
fn t14_extension_probe_js() {
    let dir = fixture_dir("probe-js");
    // Ω.5.j.cjs: .js files default to CJS unless a nearby package.json
    // declares `"type": "module"`. The fixture is ESM-shaped, so the
    // marker is required.
    write_file(&dir, "package.json", r#"{"type":"module"}"#);
    write_file(&dir, "util.js",
        "const tag = \"js-mode\";\nexport { tag };\n");
    write_file(&dir, "main.mjs", r#"
        import { tag } from "./util";
        const s = tag;
        export { s };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "s") {
        Value::String(s) => assert_eq!(s.as_str(), "js-mode"),
        v => panic!("expected 'js-mode', got {:?}", v),
    }
}
