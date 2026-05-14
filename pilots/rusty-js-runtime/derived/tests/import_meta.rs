//! Tier-Ω.5.r: `import.meta` lowering + frame-threading tests.
//!
//! The compiler lowers `import.meta` to Op::PushImportMeta. The runtime
//! populates the module frame's import_meta slot at evaluate_module
//! entry with `{ url, dir }`. Member access works naturally because the
//! parser emits Member{ MetaProperty, "url" } / Member{ MetaProperty,
//! "dir" } / Member{ MetaProperty, "unknown" } and the opcode pushes an
//! ordinary object whose properties get looked up by Op::GetProp.

use rusty_js_runtime::{Runtime, Value};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static FIXTURE_SEQ: AtomicU32 = AtomicU32::new(0);

fn fixture_dir(tag: &str) -> PathBuf {
    let n = FIXTURE_SEQ.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let p = std::env::temp_dir().join(format!("rusty-js-importmeta-{}-{}-{}", pid, n, tag));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("create fixture dir");
    p
}

fn write_file(dir: &PathBuf, name: &str, contents: &str) -> PathBuf {
    let p = dir.join(name);
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

fn load(rt: &mut Runtime, dir: &PathBuf, entry: &str) -> (rusty_js_runtime::ObjectRef, String) {
    let entry_path = dir.join(entry);
    let url = entry_url(&entry_path);
    let src = fs::read_to_string(&entry_path).expect("read entry");
    let ns = rt.evaluate_module(&src, &url).expect("evaluate_module");
    (ns, url)
}

// ─── 1. import.meta.url returns the module URL. ────────────────────────
#[test]
fn t01_import_meta_url() {
    let dir = fixture_dir("url");
    write_file(&dir, "main.mjs", "const u = import.meta.url; export { u };\n");
    let mut rt = fresh_runtime();
    let (ns, url) = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "u") {
        Value::String(s) => assert_eq!(s.as_str(), url),
        v => panic!("expected url string, got {:?}", v),
    }
}

// ─── 2. import.meta.dir returns the dirname (no file:// prefix). ───────
#[test]
fn t02_import_meta_dir() {
    let dir = fixture_dir("dir");
    write_file(&dir, "main.mjs", "const d = import.meta.dir; export { d };\n");
    let mut rt = fresh_runtime();
    let (ns, _url) = load(&mut rt, &dir, "main.mjs");
    let canonical_dir = dir.canonicalize().expect("canonicalize");
    match rt.object_get(ns, "d") {
        Value::String(s) => {
            // dir should be the parent directory of main.mjs, no file:// prefix.
            assert_eq!(s.as_str(), canonical_dir.display().to_string());
            assert!(!s.as_str().starts_with("file://"),
                "import.meta.dir should not carry the file:// prefix; got {}", s);
        }
        v => panic!("expected dir string, got {:?}", v),
    }
}

// ─── 3. Unknown property returns undefined. ────────────────────────────
#[test]
fn t03_import_meta_unknown_property() {
    let dir = fixture_dir("unknown");
    write_file(&dir, "main.mjs", "const x = import.meta.unknown_property; export { x };\n");
    let mut rt = fresh_runtime();
    let (ns, _url) = load(&mut rt, &dir, "main.mjs");
    assert!(matches!(rt.object_get(ns, "x"), Value::Undefined),
        "import.meta.unknown should be Undefined, got {:?}", rt.object_get(ns, "x"));
}

// ─── 4. import.meta as an object identity — bind, then read. ───────────
#[test]
fn t04_import_meta_object_binding() {
    let dir = fixture_dir("binding");
    write_file(&dir, "main.mjs",
        "const m = import.meta;\nconst u = m.url;\nexport { u };\n");
    let mut rt = fresh_runtime();
    let (ns, url) = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "u") {
        Value::String(s) => assert_eq!(s.as_str(), url),
        v => panic!("expected url via binding, got {:?}", v),
    }
}
