//! Tier-Ω.5.v acceptance: `export default class …` lowering.
//!
//! The compiler synthesizes a class expression from the default-export
//! body and stores its constructor value into the module's default slot.
//! Importers see the class via `import X from "./mod.mjs"`.

use rusty_js_runtime::{Runtime, Value};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static FIXTURE_SEQ: AtomicU32 = AtomicU32::new(0);

fn fixture_dir(tag: &str) -> PathBuf {
    let n = FIXTURE_SEQ.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let p = std::env::temp_dir().join(format!("rusty-js-edc-{}-{}-{}", pid, n, tag));
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

fn fresh() -> Runtime {
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

// 1. Anonymous default-exported class.
#[test]
fn t01_anonymous_default_class() {
    let dir = fixture_dir("anon");
    write_file(&dir, "lib.mjs",
        "export default class { greet() { return \"hi\"; } }\n");
    write_file(&dir, "main.mjs", r#"
        import Lib from "./lib.mjs";
        const r = (new Lib()).greet();
        export { r };
    "#);
    let mut rt = fresh();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "r") {
        Value::String(s) => assert_eq!(&*s, "hi"),
        v => panic!("expected \"hi\", got {:?}", v),
    }
}

// 2. Named default-exported class — method works post-import.
#[test]
fn t02_named_default_class() {
    let dir = fixture_dir("named");
    write_file(&dir, "lib.mjs",
        "export default class Foo { foo() { return 1; } }\n");
    write_file(&dir, "main.mjs", r#"
        import Foo from "./lib.mjs";
        const r = (new Foo()).foo();
        export { r };
    "#);
    let mut rt = fresh();
    let ns = load(&mut rt, &dir, "main.mjs");
    assert_eq!(rt.object_get(ns, "r"), Value::Number(1.0));
}

// 3. Default-imported class is usable as the base of `extends`.
#[test]
fn t03_default_class_as_extends_base() {
    let dir = fixture_dir("extends");
    write_file(&dir, "base.mjs",
        "export default class Base { kind() { return \"base\"; } }\n");
    write_file(&dir, "main.mjs", r#"
        import Base from "./base.mjs";
        class Derived extends Base { extra() { return "extra"; } }
        const d = new Derived();
        const r = d.kind() + ":" + d.extra();
        export { r };
    "#);
    let mut rt = fresh();
    let ns = load(&mut rt, &dir, "main.mjs");
    match rt.object_get(ns, "r") {
        Value::String(s) => assert_eq!(&*s, "base:extra"),
        v => panic!("expected base:extra, got {:?}", v),
    }
}
