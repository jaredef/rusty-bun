// Integration tests for rusty-bun-host. These tests run JS code through
// the rquickjs-embedded host with all pilots wired into globalThis,
// validating that the integration layer works end-to-end.

use rusty_bun_host::{eval_bool, eval_i64, eval_string};

// ════════════════════ atob / btoa ════════════════════

#[test]
fn js_atob_roundtrip() {
    // btoa("hello") should encode; atob() should decode back.
    let r = eval_string(r#"atob(btoa("hello"))"#).unwrap();
    assert_eq!(r, "hello");
}

#[test]
fn js_btoa_known_value() {
    let r = eval_string(r#"btoa("hello")"#).unwrap();
    assert_eq!(r, "aGVsbG8=");
}

#[test]
fn js_atob_known_value() {
    let r = eval_string(r#"atob("aGVsbG8=")"#).unwrap();
    assert_eq!(r, "hello");
}

// ════════════════════ path ════════════════════

#[test]
fn js_path_basename() {
    let r = eval_string(r#"path.basename("/foo/bar/baz.html")"#).unwrap();
    assert_eq!(r, "baz.html");
}

#[test]
fn js_path_basename_with_ext() {
    let r = eval_string(r#"path.basename("/foo/bar/baz.html", ".html")"#).unwrap();
    assert_eq!(r, "baz");
}

#[test]
fn js_path_dirname() {
    let r = eval_string(r#"path.dirname("/foo/bar/baz")"#).unwrap();
    assert_eq!(r, "/foo/bar");
}

#[test]
fn js_path_extname() {
    let r = eval_string(r#"path.extname("file.tar.gz")"#).unwrap();
    assert_eq!(r, ".gz");
}

#[test]
fn js_path_normalize() {
    let r = eval_string(r#"path.normalize("/foo/bar//baz/asdf/quux/..")"#).unwrap();
    assert_eq!(r, "/foo/bar/baz/asdf");
}

#[test]
fn js_path_is_absolute() {
    let r = eval_bool(r#"path.isAbsolute("/foo")"#).unwrap();
    assert!(r);
    let r = eval_bool(r#"path.isAbsolute("foo")"#).unwrap();
    assert!(!r);
}

#[test]
fn js_path_sep_constant() {
    let r = eval_string(r#"path.sep"#).unwrap();
    assert_eq!(r, "/");
}

// ════════════════════ crypto.randomUUID ════════════════════

#[test]
fn js_crypto_random_uuid_format() {
    // 36-char string with v4 format
    let r = eval_string(r#"crypto.randomUUID()"#).unwrap();
    assert_eq!(r.len(), 36);
    let parts: Vec<&str> = r.split('-').collect();
    assert_eq!(parts.len(), 5);
    // Version field is "4"
    assert_eq!(&parts[2][0..1], "4");
}

#[test]
fn js_crypto_random_uuid_unique() {
    // Two calls produce different values with overwhelming probability.
    let a = eval_string(r#"crypto.randomUUID()"#).unwrap();
    let b = eval_string(r#"crypto.randomUUID()"#).unwrap();
    assert_ne!(a, b);
}

// ════════════════════ Composition: pilots used together from JS ════════════════════

#[test]
fn js_composition_atob_path_combined() {
    // Decode a base64-encoded path, then split via path.basename.
    // btoa("/usr/local/bin/node") = "L3Vzci9sb2NhbC9iaW4vbm9kZQ=="
    // → atob → "/usr/local/bin/node" → basename → "node"
    let r = eval_string(r#"
        const encoded = btoa("/usr/local/bin/node");
        const decoded = atob(encoded);
        path.basename(decoded)
    "#).unwrap();
    assert_eq!(r, "node");
}

// ════════════════════ JS evaluation works at all ════════════════════

#[test]
fn js_pure_javascript_works() {
    let r = eval_i64("1 + 2 + 3").unwrap();
    assert_eq!(r, 6);
}

#[test]
fn js_basic_arithmetic_with_string() {
    let r = eval_string(r#"["a", "b", "c"].join("-")"#).unwrap();
    assert_eq!(r, "a-b-c");
}
