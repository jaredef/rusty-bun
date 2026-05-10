// Verifier for the Bun.file pilot.
//
// CD = runs/2026-05-10-bun-v0.13b-spec-batch/constraints/bun.constraints.md
//      (Bun.file cluster: 470+ cross-corroborated clauses)
// REF = Bun docs at https://bun.sh/docs/api/file-io
//
// First pilot with real filesystem I/O. Tests use std::env::temp_dir() for
// isolation; each test creates and cleans up its own fixture file.

use rusty_bun_file::*;
use std::fs;
use std::io::Write;

fn make_fixture(name: &str, contents: &[u8]) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("rusty-bun-file-{}-{}", name, std::process::id()));
    let mut f = fs::File::create(&p).unwrap();
    f.write_all(contents).unwrap();
    p
}

fn cleanup(p: &std::path::Path) {
    let _ = fs::remove_file(p);
}

// ════════════════════ CONSTRUCTION ════════════════════

#[test]
fn cd_bun_file_construction_does_not_read() {
    // Constructing a BunFile against a nonexistent path is fine; only
    // reading touches the filesystem.
    let f = file("/nonexistent/path/somewhere");
    assert_eq!(f.name(), "/nonexistent/path/somewhere");
    assert!(!f.exists());
}

#[test]
fn cd_bun_file_open_alias() {
    let f = BunFile::open("/tmp/x");
    assert_eq!(f.name(), "/tmp/x");
}

// ════════════════════ NAME ════════════════════

// CD: `expect(file.name).toEqual(import.meta.filename)` — name returns the path
#[test]
fn cd_bun_file_name_returns_path() {
    let f = file("/etc/hosts");
    assert_eq!(f.name(), "/etc/hosts");
}

// ════════════════════ EXISTS ════════════════════

#[test]
fn spec_exists_true_for_real_file() {
    let p = make_fixture("exists-true", b"x");
    let f = BunFile::open(&p);
    assert!(f.exists());
    cleanup(&p);
}

#[test]
fn spec_exists_false_for_missing_file() {
    let f = file("/nonexistent/path/asdfqwer");
    assert!(!f.exists());
}

// ════════════════════ SIZE ════════════════════

// CD: `expect(bunStat.size).toBe(Buffer.byteLength(content))`
#[test]
fn cd_bun_file_size_matches_byte_length() {
    let p = make_fixture("size", b"hello world");
    let f = BunFile::open(&p);
    assert_eq!(f.size().unwrap(), 11);
    cleanup(&p);
}

#[test]
fn spec_size_zero_for_empty_file() {
    let p = make_fixture("size-empty", b"");
    let f = BunFile::open(&p);
    assert_eq!(f.size().unwrap(), 0);
    cleanup(&p);
}

#[test]
fn spec_size_unicode_byte_length_not_char_length() {
    // "héllo" is 5 chars but 6 utf-8 bytes
    let p = make_fixture("size-unicode", "héllo".as_bytes());
    let f = BunFile::open(&p);
    assert_eq!(f.size().unwrap(), 6);
    cleanup(&p);
}

// ════════════════════ TEXT ════════════════════

// CD: `await Bun.file(...).text()` — read full file as UTF-8
#[test]
fn cd_bun_file_text_reads_utf8() {
    let p = make_fixture("text", b"hello world\n");
    let f = BunFile::open(&p);
    assert_eq!(f.text().unwrap(), "hello world\n");
    cleanup(&p);
}

#[test]
fn spec_text_unicode() {
    let p = make_fixture("text-unicode", "héllo, мир! 🌍".as_bytes());
    let f = BunFile::open(&p);
    assert_eq!(f.text().unwrap(), "héllo, мир! 🌍");
    cleanup(&p);
}

#[test]
fn spec_text_returns_io_error_for_missing_file() {
    let f = file("/nonexistent/path/asdfqwer");
    assert!(f.text().is_err());
}

// ════════════════════ BYTES / ARRAY_BUFFER ════════════════════

#[test]
fn spec_bytes_reads_raw_bytes() {
    let p = make_fixture("bytes", &[0u8, 1, 2, 0xFF, 0xFE]);
    let f = BunFile::open(&p);
    assert_eq!(f.bytes().unwrap(), vec![0u8, 1, 2, 0xFF, 0xFE]);
    cleanup(&p);
}

#[test]
fn spec_array_buffer_alias_of_bytes() {
    let p = make_fixture("array-buffer", b"abc");
    let f = BunFile::open(&p);
    assert_eq!(f.array_buffer().unwrap(), f.bytes().unwrap());
    cleanup(&p);
}

// ════════════════════ MIME TYPE ════════════════════

#[test]
fn spec_mime_type_inferred_from_extension() {
    assert_eq!(BunFile::open("test.html").mime_type(), "text/html;charset=utf-8");
    assert_eq!(BunFile::open("test.json").mime_type(), "application/json;charset=utf-8");
    assert_eq!(BunFile::open("test.png").mime_type(), "image/png");
    assert_eq!(BunFile::open("test.svg").mime_type(), "image/svg+xml");
}

#[test]
fn spec_mime_type_octet_stream_for_unknown_extension() {
    assert_eq!(BunFile::open("test.xyzunknown").mime_type(), "application/octet-stream");
}

#[test]
fn spec_mime_type_empty_for_no_extension() {
    assert_eq!(BunFile::open("README").mime_type(), "");
}

#[test]
fn spec_mime_type_explicit_overrides_inferred() {
    let f = BunFile::open_with_type("data.json", "text/plain");
    assert_eq!(f.mime_type(), "text/plain");
}

// ════════════════════ LAST MODIFIED ════════════════════

#[test]
fn spec_last_modified_returns_ms_since_epoch() {
    let p = make_fixture("lastmod", b"x");
    let f = BunFile::open(&p);
    let ms = f.last_modified().unwrap();
    assert!(ms > 1_700_000_000_000, "expected modern timestamp, got {}", ms);
    cleanup(&p);
}

// ════════════════════ SLICE ════════════════════

// SPEC: slice() returns Blob, not BunFile (per File API §4)
#[test]
fn spec_slice_returns_blob_not_bunfile() {
    let p = make_fixture("slice", b"hello world");
    let f = BunFile::open(&p);
    let slice: Blob = f.slice(0, Some(5), None).unwrap();
    assert_eq!(slice.text(), "hello");
    cleanup(&p);
}

#[test]
fn spec_slice_negative_offset_clamps() {
    let p = make_fixture("slice-neg", b"hello world");
    let f = BunFile::open(&p);
    let slice = f.slice(-5, None, None).unwrap();
    assert_eq!(slice.text(), "world");
    cleanup(&p);
}

#[test]
fn spec_slice_content_type_override() {
    let p = make_fixture("slice-type", b"abc");
    let f = BunFile::open(&p);
    let slice = f.slice(0, None, Some("application/json")).unwrap();
    assert_eq!(slice.mime_type(), "application/json");
    cleanup(&p);
}

// ════════════════════ AS_BLOB ════════════════════

// CD: `expect(file).toBeInstanceOf(Blob)` — coerce to Blob view
#[test]
fn cd_bun_file_as_blob_preserves_size_and_type() {
    let p = make_fixture("as-blob", b"contents");
    let f = BunFile::open_with_type(&p, "text/markdown");
    let b = f.as_blob().unwrap();
    assert_eq!(b.size(), 8);
    assert_eq!(b.mime_type(), "text/markdown");
    cleanup(&p);
}

#[test]
fn spec_as_blob_uses_inferred_type_when_no_explicit() {
    let p = make_fixture("inferred-type", b"<html></html>");
    let path = p.with_extension("html");
    fs::rename(&p, &path).unwrap();
    let f = BunFile::open(&path);
    let b = f.as_blob().unwrap();
    assert!(b.mime_type().starts_with("text/html"));
    cleanup(&path);
}

// ════════════════════ ROUND-TRIP ════════════════════

#[test]
fn integration_round_trip_text_through_bunfile() {
    let original = "fixture content with newlines\n\nand utf-8 char é";
    let p = make_fixture("roundtrip", original.as_bytes());
    let f = BunFile::open(&p);
    assert_eq!(f.text().unwrap(), original);
    assert_eq!(f.size().unwrap() as usize, original.as_bytes().len());
    cleanup(&p);
}
