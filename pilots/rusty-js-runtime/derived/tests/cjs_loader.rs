//! Tier-Ω.5.j.cjs: CommonJS interop substrate. Each test builds a small
//! fixture under /tmp and drives an ESM entry (or direct CJS load) through
//! the runtime's evaluate_module / evaluate_cjs_module pipeline.
//!
//! Acceptance bar locked at 12 tests covering:
//!   - .cjs files (forced CJS),
//!   - .js with no package.json (defaults to CJS),
//!   - .js under package.json "type":"module" (forced ESM),
//!   - module.exports rebind vs exports.alias,
//!   - require() relative, require() of built-in (no node: prefix),
//!   - __dirname / __filename,
//!   - CJS→CJS chains, module-cache identity,
//!   - ESM importing CJS function as default.

use rusty_js_runtime::{Runtime, Value};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static FIXTURE_SEQ: AtomicU32 = AtomicU32::new(0);

fn fixture_dir(tag: &str) -> PathBuf {
    let n = FIXTURE_SEQ.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let p = std::env::temp_dir().join(format!("rusty-js-cjs-{}-{}-{}", pid, n, tag));
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

// ─── 1. Simple .cjs file imported by an ESM main. ──────────────────────
#[test]
fn t01_simple_cjs() {
    let dir = fixture_dir("simple-cjs");
    write_file(&dir, "lib.cjs",
        "module.exports = { add: function(a, b) { return a + b } };\n");
    write_file(&dir, "main.mjs", r#"
        import lib from "./lib.cjs";
        const result = lib.add(2, 3);
        export { result };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "result") {
        Value::Number(n) => assert_eq!(n, 5.0),
        v => panic!("expected 5, got {:?}", v),
    }
}

// ─── 2. CJS via .js extension (no package.json). ───────────────────────
#[test]
fn t02_js_no_pkg_is_cjs() {
    let dir = fixture_dir("js-no-pkg");
    write_file(&dir, "lib.js",
        "module.exports = { add: function(a, b) { return a + b } };\n");
    write_file(&dir, "main.mjs", r#"
        import lib from "./lib.js";
        const result = lib.add(4, 5);
        export { result };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "result") {
        Value::Number(n) => assert_eq!(n, 9.0),
        v => panic!("expected 9, got {:?}", v),
    }
}

// ─── 3. package.json "type":"module" forces ESM. ───────────────────────
#[test]
fn t03_pkg_type_module_is_esm() {
    let dir = fixture_dir("pkg-type-module");
    write_file(&dir, "pkg/package.json", "{ \"type\": \"module\" }");
    write_file(&dir, "pkg/lib.js",
        "function add(a, b) { return a + b }\nexport { add };\n");
    write_file(&dir, "main.mjs", r#"
        import { add } from "./pkg/lib.js";
        const result = add(6, 7);
        export { result };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "result") {
        Value::Number(n) => assert_eq!(n, 13.0),
        v => panic!("expected 13, got {:?}", v),
    }
}

// ─── 4. CJS named-property import. ─────────────────────────────────────
#[test]
fn t04_cjs_named_import() {
    let dir = fixture_dir("cjs-named");
    write_file(&dir, "lib.cjs",
        "module.exports.add = function(a, b) { return a + b };\n");
    write_file(&dir, "main.mjs", r#"
        import { add } from "./lib.cjs";
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

// ─── 5. module.exports rebind to a primitive. ──────────────────────────
#[test]
fn t05_module_exports_rebind() {
    let dir = fixture_dir("rebind");
    write_file(&dir, "lib.cjs", "module.exports = 42;\n");
    write_file(&dir, "main.mjs", r#"
        import x from "./lib.cjs";
        const v = x;
        export { v };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "v") {
        Value::Number(n) => assert_eq!(n, 42.0),
        v => panic!("expected 42, got {:?}", v),
    }
}

// ─── 6. `exports` alias works. ─────────────────────────────────────────
#[test]
fn t06_exports_alias() {
    let dir = fixture_dir("exports-alias");
    write_file(&dir, "lib.cjs", "exports.x = 1; exports.y = 2;\n");
    write_file(&dir, "main.mjs", r#"
        import { x, y } from "./lib.cjs";
        const sum = x + y;
        export { sum };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "sum") {
        Value::Number(n) => assert_eq!(n, 3.0),
        v => panic!("expected 3, got {:?}", v),
    }
}

// ─── 7. require() relative. ────────────────────────────────────────────
#[test]
fn t07_require_relative() {
    let dir = fixture_dir("require-rel");
    write_file(&dir, "b.cjs", "module.exports = { value: 41 };\n");
    write_file(&dir, "a.cjs",
        "var b = require(\"./b.cjs\"); module.exports = b.value + 1;\n");
    write_file(&dir, "main.mjs", r#"
        import a from "./a.cjs";
        const v = a;
        export { v };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "v") {
        Value::Number(n) => assert_eq!(n, 42.0),
        v => panic!("expected 42, got {:?}", v),
    }
}

// ─── 8. require() of built-in without node: prefix. ────────────────────
#[test]
fn t08_require_builtin_no_prefix() {
    let dir = fixture_dir("require-builtin");
    write_file(&dir, "lib.cjs",
        "var path = require(\"path\"); module.exports = path.sep;\n");
    write_file(&dir, "main.mjs", r#"
        import sep from "./lib.cjs";
        const s = sep;
        export { s };
    "#);
    let mut rt = fresh_runtime_with_host();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "s") {
        Value::String(s) => assert_eq!(s.as_str(), "/"),
        v => panic!("expected '/', got {:?}", v),
    }
}

// ─── 9. __dirname / __filename are non-empty strings. ─────────────────
#[test]
fn t09_filename_dirname() {
    let dir = fixture_dir("dirname");
    write_file(&dir, "lib.cjs",
        "module.exports = { dir: __dirname, file: __filename };\n");
    write_file(&dir, "main.mjs", r#"
        import lib from "./lib.cjs";
        const d = lib.dir;
        const f = lib.file;
        export { d, f };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    let d = rt.object_get(ns, "d");
    let f = rt.object_get(ns, "f");
    match (d, f) {
        (Value::String(d), Value::String(f)) => {
            assert!(!d.is_empty(), "__dirname empty");
            assert!(!f.is_empty(), "__filename empty");
            assert!(f.ends_with("lib.cjs"), "expected __filename to end with lib.cjs, got {:?}", f);
        }
        (d, f) => panic!("expected string dir+file, got {:?} {:?}", d, f),
    }
}

// ─── 10. CJS-requires-CJS chain a→b→c. ────────────────────────────────
#[test]
fn t10_cjs_chain() {
    let dir = fixture_dir("chain");
    write_file(&dir, "c.cjs", "module.exports = { val: 7 };\n");
    write_file(&dir, "b.cjs",
        "var c = require(\"./c.cjs\"); module.exports = { val: c.val + 1 };\n");
    write_file(&dir, "a.cjs",
        "var b = require(\"./b.cjs\"); module.exports = b.val + 1;\n");
    write_file(&dir, "main.mjs", r#"
        import a from "./a.cjs";
        const v = a;
        export { v };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "v") {
        Value::Number(n) => assert_eq!(n, 9.0),
        v => panic!("expected 9, got {:?}", v),
    }
}

// ─── 11. CJS module cache: same exports identity across two paths. ────
#[test]
fn t11_module_cache_identity() {
    let dir = fixture_dir("cache-id");
    // dep tracks a side-effect counter; if cache works, both views see 1.
    write_file(&dir, "dep.cjs", r#"
        if (!global_counter) { var global_counter = 0; }
        global_counter = (typeof global_counter === "number" ? global_counter : 0) + 1;
        module.exports = { tag: "once" };
    "#);
    write_file(&dir, "a.cjs",
        "var d = require(\"./dep.cjs\"); module.exports = d;\n");
    write_file(&dir, "b.cjs",
        "var d = require(\"./dep.cjs\"); module.exports = d;\n");
    write_file(&dir, "main.mjs", r#"
        import a from "./a.cjs";
        import b from "./b.cjs";
        const same = (a === b);
        const tag = a.tag;
        export { same, tag };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    assert!(matches!(rt.object_get(ns, "same"), Value::Boolean(true)),
        "a and b should be the same exports identity");
    match rt.object_get(ns, "tag") {
        Value::String(s) => assert_eq!(s.as_str(), "once"),
        v => panic!("expected 'once', got {:?}", v),
    }
}

// ─── 12. ESM importing CJS function as default. ────────────────────────
#[test]
fn t12_cjs_function_default() {
    let dir = fixture_dir("fn-default");
    write_file(&dir, "lib.cjs",
        "module.exports = function() { return 7 };\n");
    write_file(&dir, "main.mjs", r#"
        import f from "./lib.cjs";
        const v = f();
        export { v };
    "#);
    let mut rt = fresh_runtime();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "v") {
        Value::Number(n) => assert_eq!(n, 7.0),
        v => panic!("expected 7, got {:?}", v),
    }
}
