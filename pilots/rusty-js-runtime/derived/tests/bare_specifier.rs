//! Tier-Ω.5.q: bare-specifier resolution + node_modules walk-up +
//! package.json `exports` / `main` / `module` / `index.js`. Each test
//! builds a small fixture under /tmp/rusty-js-bare-<pid>-<n>/, lays out
//! `node_modules/<pkg>/...`, then evaluates an entry module that
//! imports the bare specifier.
//!
//! Acceptance bar: 12 tests covering bare + scoped + subpath +
//! wildcard + walk-up + cache + CJS-via-require + ESM-importing-CJS +
//! not-installed.

use rusty_js_runtime::{Runtime, RuntimeError, Value};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static FIXTURE_SEQ: AtomicU32 = AtomicU32::new(0);

fn fixture_dir(tag: &str) -> PathBuf {
    let n = FIXTURE_SEQ.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let p = std::env::temp_dir().join(format!("rusty-js-bare-{}-{}-{}", pid, n, tag));
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

/// Evaluate an entry module's source under the given URL; return the
/// value bound to the entry's `result` export (or Undefined).
fn run_entry(rt: &mut Runtime, source: &str, url: &str) -> Result<Value, RuntimeError> {
    let ns = rt.evaluate_module(source, url)?;
    Ok(rt.object_get(ns, "result"))
}

// ─── 1. Bare specifier, simple package, ESM ─────────────────────────
#[test]
fn t01_bare_simple_esm_main() {
    let dir = fixture_dir("t01");
    write_file(
        &dir,
        "node_modules/foo/package.json",
        r#"{"name":"foo","main":"index.js","type":"module"}"#,
    );
    write_file(&dir, "node_modules/foo/index.js", "const x = 1; export { x };");
    let entry = write_file(
        &dir,
        "entry.mjs",
        "import { x } from \"foo\"; const result = x; export { result };",
    );
    let mut rt = fresh_runtime();
    let v = run_entry(&mut rt, &fs::read_to_string(&entry).unwrap(), &entry_url(&entry)).unwrap();
    assert!(matches!(v, Value::Number(n) if (n - 1.0).abs() < 1e-9), "got {:?}", v);
}

// ─── 2. `module` field preferred for ESM ────────────────────────────
#[test]
fn t02_module_field_esm_preferred() {
    let dir = fixture_dir("t02");
    write_file(
        &dir,
        "node_modules/foo/package.json",
        r#"{"name":"foo","main":"index.cjs","module":"index.mjs"}"#,
    );
    write_file(&dir, "node_modules/foo/index.mjs", "const x = 42; export { x };");
    write_file(
        &dir,
        "node_modules/foo/index.cjs",
        "module.exports = { x: 0 };",
    );
    let entry = write_file(
        &dir,
        "entry.mjs",
        "import { x } from \"foo\"; const result = x; export { result };",
    );
    let mut rt = fresh_runtime();
    let v = run_entry(&mut rt, &fs::read_to_string(&entry).unwrap(), &entry_url(&entry)).unwrap();
    assert!(matches!(v, Value::Number(n) if (n - 42.0).abs() < 1e-9));
}

// ─── 3. exports.".".import condition ────────────────────────────────
#[test]
fn t03_exports_dot_import_condition() {
    let dir = fixture_dir("t03");
    write_file(
        &dir,
        "node_modules/foo/package.json",
        r#"{"name":"foo","exports":{".":{"import":"./esm.js","require":"./cjs.js"}},"type":"module"}"#,
    );
    write_file(&dir, "node_modules/foo/esm.js", "const x = 10; export { x };");
    write_file(
        &dir,
        "node_modules/foo/cjs.js",
        "module.exports = { x: -1 };",
    );
    let entry = write_file(
        &dir,
        "entry.mjs",
        "import { x } from \"foo\"; const result = x; export { result };",
    );
    let mut rt = fresh_runtime();
    let v = run_entry(&mut rt, &fs::read_to_string(&entry).unwrap(), &entry_url(&entry)).unwrap();
    assert!(matches!(v, Value::Number(n) if (n - 10.0).abs() < 1e-9));
}

// ─── 4. exports.".".default fallback ────────────────────────────────
#[test]
fn t04_exports_dot_default_fallback() {
    let dir = fixture_dir("t04");
    write_file(
        &dir,
        "node_modules/foo/package.json",
        r#"{"name":"foo","exports":{".":{"default":"./entry.js"}},"type":"module"}"#,
    );
    write_file(&dir, "node_modules/foo/entry.js", "const x = 7; export { x };");
    let entry = write_file(
        &dir,
        "entry.mjs",
        "import { x } from \"foo\"; const result = x; export { result };",
    );
    let mut rt = fresh_runtime();
    let v = run_entry(&mut rt, &fs::read_to_string(&entry).unwrap(), &entry_url(&entry)).unwrap();
    assert!(matches!(v, Value::Number(n) if (n - 7.0).abs() < 1e-9));
}

// ─── 5. Subpath import ──────────────────────────────────────────────
#[test]
fn t05_subpath_import_via_exports() {
    let dir = fixture_dir("t05");
    write_file(
        &dir,
        "node_modules/foo/package.json",
        r#"{"name":"foo","exports":{"./sub":"./lib/sub.js"},"type":"module"}"#,
    );
    write_file(&dir, "node_modules/foo/lib/sub.js", "const x = 99; export { x };");
    let entry = write_file(
        &dir,
        "entry.mjs",
        "import { x } from \"foo/sub\"; const result = x; export { result };",
    );
    let mut rt = fresh_runtime();
    let v = run_entry(&mut rt, &fs::read_to_string(&entry).unwrap(), &entry_url(&entry)).unwrap();
    assert!(matches!(v, Value::Number(n) if (n - 99.0).abs() < 1e-9));
}

// ─── 6. Wildcard subpath pattern ────────────────────────────────────
#[test]
fn t06_wildcard_subpath() {
    let dir = fixture_dir("t06");
    write_file(
        &dir,
        "node_modules/foo/package.json",
        r#"{"name":"foo","exports":{"./fp/*":"./dist/fp/*.js"},"type":"module"}"#,
    );
    write_file(&dir, "node_modules/foo/dist/fp/get.js", "const x = 123; export { x };");
    let entry = write_file(
        &dir,
        "entry.mjs",
        "import { x } from \"foo/fp/get\"; const result = x; export { result };",
    );
    let mut rt = fresh_runtime();
    let v = run_entry(&mut rt, &fs::read_to_string(&entry).unwrap(), &entry_url(&entry)).unwrap();
    assert!(matches!(v, Value::Number(n) if (n - 123.0).abs() < 1e-9));
}

// ─── 7. Scoped package ──────────────────────────────────────────────
#[test]
fn t07_scoped_package() {
    let dir = fixture_dir("t07");
    write_file(
        &dir,
        "node_modules/@org/pkg/package.json",
        r#"{"name":"@org/pkg","main":"index.js","type":"module"}"#,
    );
    write_file(
        &dir,
        "node_modules/@org/pkg/index.js",
        "const x = 55; export { x };",
    );
    let entry = write_file(
        &dir,
        "entry.mjs",
        "import { x } from \"@org/pkg\"; const result = x; export { result };",
    );
    let mut rt = fresh_runtime();
    let v = run_entry(&mut rt, &fs::read_to_string(&entry).unwrap(), &entry_url(&entry)).unwrap();
    assert!(matches!(v, Value::Number(n) if (n - 55.0).abs() < 1e-9));
}

// ─── 8. Walk-up resolution ──────────────────────────────────────────
#[test]
fn t08_walk_up_two_levels() {
    let dir = fixture_dir("t08");
    write_file(
        &dir,
        "node_modules/foo/package.json",
        r#"{"name":"foo","main":"index.js","type":"module"}"#,
    );
    write_file(&dir, "node_modules/foo/index.js", "const x = 8; export { x };");
    let entry = write_file(
        &dir,
        "src/deep/entry.mjs",
        "import { x } from \"foo\"; const result = x; export { result };",
    );
    let mut rt = fresh_runtime();
    let v = run_entry(&mut rt, &fs::read_to_string(&entry).unwrap(), &entry_url(&entry)).unwrap();
    assert!(matches!(v, Value::Number(n) if (n - 8.0).abs() < 1e-9));
}

// ─── 9. package.json cache populated ────────────────────────────────
#[test]
fn t09_package_json_cached() {
    let dir = fixture_dir("t09");
    write_file(
        &dir,
        "node_modules/foo/package.json",
        r#"{"name":"foo","main":"index.js","type":"module"}"#,
    );
    write_file(&dir, "node_modules/foo/index.js", "const x = 2; export { x };");
    let entry = write_file(
        &dir,
        "entry.mjs",
        "import { x } from \"foo\"; const result = x; export { result };",
    );
    let mut rt = fresh_runtime();
    let _ = run_entry(&mut rt, &fs::read_to_string(&entry).unwrap(), &entry_url(&entry)).unwrap();
    // Cache must contain at least the package's package.json.
    let pkg_json_path = dir.join("node_modules/foo/package.json").canonicalize().unwrap();
    assert!(
        rt.pkg_json_cache.contains_key(&pkg_json_path)
            || rt.pkg_json_cache.keys().any(|k| k.ends_with("node_modules/foo/package.json")),
        "pkg_json_cache should have an entry; got keys = {:?}",
        rt.pkg_json_cache.keys().collect::<Vec<_>>()
    );
}

// ─── 10. CJS require of a bare package via `require` condition ─────
#[test]
fn t10_cjs_require_bare_via_require_condition() {
    let dir = fixture_dir("t10");
    write_file(
        &dir,
        "node_modules/foo/package.json",
        r#"{"name":"foo","exports":{".":{"import":"./esm.js","require":"./cjs.js"}}}"#,
    );
    write_file(&dir, "node_modules/foo/esm.js", "const x = -1; export { x };");
    write_file(
        &dir,
        "node_modules/foo/cjs.js",
        "module.exports = { x: 33 };",
    );
    // Entry is a .cjs file requiring "foo".
    let entry = write_file(
        &dir,
        "entry.cjs",
        "const foo = require('foo'); module.exports = { result: foo.x };",
    );
    let mut rt = fresh_runtime();
    let ns = rt.evaluate_cjs_module(
        &fs::read_to_string(&entry).unwrap(),
        &entry_url(&entry),
    ).unwrap();
    let result = rt.object_get(ns, "result");
    assert!(matches!(result, Value::Number(n) if (n - 33.0).abs() < 1e-9));
}

// ─── 11. ESM importing CJS bare package ─────────────────────────────
#[test]
fn t11_esm_imports_cjs_bare_package() {
    let dir = fixture_dir("t11");
    // Bare package whose entry is .cjs and module.exports has an `x`.
    write_file(
        &dir,
        "node_modules/foo/package.json",
        r#"{"name":"foo","main":"index.cjs"}"#,
    );
    write_file(
        &dir,
        "node_modules/foo/index.cjs",
        "module.exports = { x: 17 };",
    );
    let entry = write_file(
        &dir,
        "entry.mjs",
        "import foo from \"foo\"; const result = foo.x; export { result };",
    );
    let mut rt = fresh_runtime();
    let v = run_entry(&mut rt, &fs::read_to_string(&entry).unwrap(), &entry_url(&entry)).unwrap();
    assert!(matches!(v, Value::Number(n) if (n - 17.0).abs() < 1e-9));
}

// ─── 12. Bare spec not installed → clear error ──────────────────────
#[test]
fn t12_bare_spec_not_installed_clear_error() {
    let dir = fixture_dir("t12");
    let entry = write_file(
        &dir,
        "entry.mjs",
        "import x from \"nonexistent-pkg\"; const result = x; export { result };",
    );
    let mut rt = fresh_runtime();
    let err = run_entry(&mut rt, &fs::read_to_string(&entry).unwrap(), &entry_url(&entry))
        .err()
        .expect("expected TypeError for unresolved bare specifier");
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("nonexistent-pkg"),
        "error should name the unresolvable specifier; got {}",
        msg
    );
    assert!(
        matches!(err, RuntimeError::TypeError(_)),
        "expected TypeError, got {:?}",
        err
    );
}
