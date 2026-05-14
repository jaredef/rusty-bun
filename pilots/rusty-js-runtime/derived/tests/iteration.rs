//! Tier-Ω.5.c Stage 2 acceptance: iterator protocol + for-of, plus
//! Stage 3 statics (Object.keys/values/entries/assign, Array.from/isArray,
//! Symbol.iterator).
//!
//! Deviations from spec, recorded here:
//! - Symbol.iterator is the string `"@@iterator"`, not a primitive Symbol.
//!   `obj[Symbol.iterator]()` resolves through normal property lookup
//!   against that string key.
//! - String iteration is by char (Unicode scalar value), not by UTF-16
//!   code unit. Matches ECMA-262 for BMP characters.

use rusty_js_runtime::{run_module, Runtime, Value};

fn run(src: &str) -> Value {
    run_module(src).unwrap_or_else(|e| panic!("run failed for {:?}: {:?}", src, e))
}

fn run_rt(src: &str) -> Runtime {
    let module = rusty_js_bytecode::compile_module(src)
        .unwrap_or_else(|e| panic!("compile: {:?}", e));
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    rt.run_module(&module).unwrap_or_else(|e| panic!("run: {:?}", e));
    rt
}

// ─────────── 1. for-of over array — basic sum ───────────
#[test]
fn t01_forof_array_sum() {
    let rt = run_rt("let s = 0; for (const x of [1,2,3]) { s = s + x; } __record(s);");
    assert_eq!(rt.globals.get("__last_recorded").cloned().unwrap(), Value::Number(6.0));
}

// ─────────── 2. for-of over string ───────────
#[test]
fn t02_forof_string() {
    let rt = run_rt(r#"let out = ""; for (const c of "abc") { out = out + c + "-"; } __record(out);"#);
    if let Some(Value::String(s)) = rt.globals.get("__last_recorded") {
        assert_eq!(s.as_str(), "a-b-c-");
    } else { panic!(); }
}

// ─────────── 3. nested for-of ───────────
#[test]
fn t03_nested_forof() {
    let rt = run_rt(r#"
        let sum = 0;
        for (const row of [[1,2],[3,4]]) {
            for (const x of row) { sum = sum + x; }
        }
        __record(sum);
    "#);
    assert_eq!(rt.globals.get("__last_recorded").cloned().unwrap(), Value::Number(10.0));
}

// ─────────── 4. Symbol.iterator is the well-known key ───────────
#[test]
fn t04_symbol_iterator_key() {
    if let Value::String(s) = run("return Symbol.iterator;") {
        assert_eq!(s.as_str(), "@@iterator");
    } else { panic!(); }
}

// ─────────── 5. for-of using existing identifier binding ───────────
#[test]
fn t05_forof_identifier_binding() {
    let rt = run_rt("let x = 0; let sum = 0; for (x of [10, 20, 30]) { sum = sum + x; } __record(sum);");
    assert_eq!(rt.globals.get("__last_recorded").cloned().unwrap(), Value::Number(60.0));
}

// ─────────── 6. for-of over empty array ───────────
#[test]
fn t06_forof_empty() {
    let rt = run_rt("let n = 0; for (const x of []) { n = n + 1; } __record(n);");
    assert_eq!(rt.globals.get("__last_recorded").cloned().unwrap(), Value::Number(0.0));
}

// ─────────── 7. Object.keys ───────────
#[test]
fn t07_object_keys() {
    if let Value::String(s) = run(r#"return Object.keys({a:1, b:2, c:3}).join(",");"#) {
        let mut parts: Vec<&str> = s.split(',').collect();
        parts.sort();
        assert_eq!(parts, vec!["a", "b", "c"]);
    } else { panic!(); }
}

// ─────────── 8. Object.values ───────────
#[test]
fn t08_object_values() {
    if let Value::String(s) = run(r#"return Object.values({a:1, b:2}).join(",");"#) {
        let mut parts: Vec<&str> = s.split(',').collect();
        parts.sort();
        assert_eq!(parts, vec!["1", "2"]);
    } else { panic!(); }
}

// ─────────── 9. Object.entries ───────────
#[test]
fn t09_object_entries() {
    if let Value::String(s) = run(r#"
        let e = Object.entries({a:1});
        return e[0][0] + "=" + e[0][1];
    "#) {
        assert_eq!(s.as_str(), "a=1");
    } else { panic!(); }
}

// ─────────── 10. Object.assign ───────────
#[test]
fn t10_object_assign() {
    let rt = run_rt(r#"
        let t = {a:1};
        Object.assign(t, {b:2}, {c:3});
        __record(t.a + t.b + t.c);
    "#);
    assert_eq!(rt.globals.get("__last_recorded").cloned().unwrap(), Value::Number(6.0));
}

// ─────────── 11. Array.from(iterable) ───────────
#[test]
fn t11_array_from_iterable() {
    if let Value::String(s) = run(r#"return Array.from([1,2,3]).join(",");"#) {
        assert_eq!(s.as_str(), "1,2,3");
    } else { panic!(); }
}

// ─────────── 12. Array.from(string) ───────────
#[test]
fn t12_array_from_string() {
    if let Value::String(s) = run(r#"return Array.from("xyz").join("-");"#) {
        assert_eq!(s.as_str(), "x-y-z");
    } else { panic!(); }
}

// ─────────── 13. Array.from with map fn ───────────
#[test]
fn t13_array_from_with_mapfn() {
    if let Value::String(s) = run(r#"return Array.from([1,2,3], x => x * 10).join(",");"#) {
        assert_eq!(s.as_str(), "10,20,30");
    } else { panic!(); }
}

// ─────────── 14. Array.isArray ───────────
#[test]
fn t14_array_is_array() {
    assert_eq!(run("return Array.isArray([1,2,3]);"), Value::Boolean(true));
    assert_eq!(run("return Array.isArray({length:3});"), Value::Boolean(false));
    assert_eq!(run("return Array.isArray(\"abc\");"), Value::Boolean(false));
}

// ─────────── 15. Array.of ───────────
#[test]
fn t15_array_of() {
    if let Value::String(s) = run("return Array.of(1, 2, 3).join(\",\");") {
        assert_eq!(s.as_str(), "1,2,3");
    } else { panic!(); }
}

// ─────────── 16. Object.freeze + isFrozen ───────────
#[test]
fn t16_freeze() {
    assert_eq!(run("let o = {a:1}; Object.freeze(o); return Object.isFrozen(o);"),
        Value::Boolean(true));
    assert_eq!(run("return Object.isFrozen({});"), Value::Boolean(false));
}

// ─────────── 17. Object.fromEntries ───────────
#[test]
fn t17_from_entries() {
    let rt = run_rt(r#"
        let o = Object.fromEntries([["a", 1], ["b", 2]]);
        __record(o.a + o.b);
    "#);
    assert_eq!(rt.globals.get("__last_recorded").cloned().unwrap(), Value::Number(3.0));
}

// ─────────── 18. for-of break stops iteration ───────────
#[test]
fn t18_forof_break() {
    let rt = run_rt(r#"
        let n = 0;
        for (const x of [1,2,3,4,5]) {
            if (x === 3) { break; }
            n = n + 1;
        }
        __record(n);
    "#);
    assert_eq!(rt.globals.get("__last_recorded").cloned().unwrap(), Value::Number(2.0));
}

// ─────────── 19. Explicit @@iterator-protocol consumption ───────────
#[test]
fn t19_manual_iterator_protocol() {
    let rt = run_rt(r#"
        let it = [10, 20][Symbol.iterator]();
        let first = it.next();
        __record(first.value);
    "#);
    assert_eq!(rt.globals.get("__last_recorded").cloned().unwrap(), Value::Number(10.0));
}

// ─────────── 20. Object.keys on array yields numeric-string indices ───────────
#[test]
fn t20_object_keys_array() {
    if let Value::String(s) = run(r#"return Object.keys(["a","b","c"]).join(",");"#) {
        assert_eq!(s.as_str(), "0,1,2");
    } else { panic!(); }
}
