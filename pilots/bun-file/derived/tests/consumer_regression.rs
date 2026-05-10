// Consumer-regression suite for Bun.file.
//
// Bun.file is a Tier-2 ecosystem-only surface (no spec; Bun's tests are the
// spec). Consumer regressions cite Bun's own test corpus and Bun-using
// production code as authoritative.

use rusty_bun_file::*;
use std::fs;
use std::io::Write;

fn make_fixture(name: &str, contents: &[u8]) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("rusty-bun-file-cons-{}-{}", name, std::process::id()));
    let mut f = fs::File::create(&p).unwrap();
    f.write_all(contents).unwrap();
    p
}

fn cleanup(p: &std::path::Path) {
    let _ = fs::remove_file(p);
}

// ────────── Bun-internal — snapshot test pattern ──────────
//
// Source: Bun test `test/regression/issue/14029.test.ts:41` —
//   `expect(newSnapshot).toBe(await Bun.file(snapshotPath).text())`
// Consumer expectation: text() returns the exact UTF-8 contents byte-for-byte
// with no normalization.

#[test]
fn consumer_bun_snapshot_text_round_trip() {
    let snapshot = "// snapshot\nexports[`test 1`] = `value`;\nexports[`test 2`] = `another`;\n";
    let p = make_fixture("snapshot", snapshot.as_bytes());
    let f = BunFile::open(&p);
    assert_eq!(f.text().unwrap(), snapshot);
    cleanup(&p);
}

// ────────── Bun-internal — Buffer.byteLength size assertion ──────────
//
// Source: Bun test `test/regression/issue/26647.test.ts:40` —
//   `expect(bunStat.size).toBe(Buffer.byteLength(content))`
// Consumer expectation: BunFile.size matches the byte count downstream
// code computes via the Node Buffer surface.

#[test]
fn consumer_bun_size_matches_buffer_bytelength_pattern() {
    let content = "héllo, мир! 🌍";
    let byte_count = content.as_bytes().len();
    let p = make_fixture("size-match", content.as_bytes());
    let f = BunFile::open(&p);
    assert_eq!(f.size().unwrap() as usize, byte_count);
    cleanup(&p);
}

// ────────── Bun-internal — instanceof Blob assertion ──────────
//
// Source: Bun test `test/js/web/workers/message-channel.test.ts:283` —
//   `expect(file).toBeInstanceOf(Blob)`. Bun's BunFile extends Blob.
// Consumer expectation: BunFile coerces cleanly to a Blob view.

#[test]
fn consumer_bun_message_channel_file_as_blob() {
    let p = make_fixture("instance-of", b"contents");
    let f = BunFile::open_with_type(&p, "text/plain");
    let blob = f.as_blob().unwrap();
    assert_eq!(blob.size(), 8);
    assert_eq!(blob.mime_type(), "text/plain");
    cleanup(&p);
}

// ────────── HTTP server sample — Bun.serve(req → Bun.file(...)) ──────────
//
// Source: Bun docs at https://bun.sh/docs/api/http — the canonical pattern
//   `return new Response(Bun.file(path));` for static-file serving.
// Consumer expectation: a BunFile coerced to a Blob with the inferred MIME
// type sets the Response Content-Type correctly.

#[test]
fn consumer_static_file_response_content_type() {
    let p = std::env::temp_dir().join(format!("static-{}.html", std::process::id()));
    fs::write(&p, b"<html></html>").unwrap();
    let f = BunFile::open(&p);
    let blob = f.as_blob().unwrap();
    assert!(blob.mime_type().starts_with("text/html"));
    cleanup(&p);
}

// ────────── Bun-internal — exists predicate for conditional reads ──────────
//
// Source: Bun docs and many third-party Bun-using projects:
//   `if (await file.exists()) { ... }` is the canonical existence-then-read
//   guard pattern.

#[test]
fn consumer_exists_then_read_guard_pattern() {
    let p = make_fixture("guard", b"present");
    let f = BunFile::open(&p);
    if f.exists() {
        assert_eq!(f.text().unwrap(), "present");
    } else {
        panic!("file should exist after fixture creation");
    }
    cleanup(&p);

    let missing = file("/nonexistent/should-not-exist-anywhere");
    assert!(!missing.exists());
}

// ────────── Bun-internal — slice() returns Blob, not BunFile ──────────
//
// Source: Bun docs — sliced files lose file metadata. Production code that
// chunked-uploads a Bun.file must explicitly re-wrap each chunk if a File
// shape is needed downstream.

#[test]
fn consumer_chunked_upload_slice_returns_blob() {
    let p = make_fixture("chunked", b"large file content for chunking");
    let f = BunFile::open(&p);
    let chunk = f.slice(0, Some(5), None).unwrap();
    // chunk is a Blob, not a BunFile. Type system enforces.
    assert_eq!(chunk.text(), "large");
    cleanup(&p);
}

// ────────── Bundlers — file path identity for cache keys ──────────
//
// Source: many bundler projects use Bun.file(path).name as a cache key.
// Consumer expectation: name returns the exact string passed in.

#[test]
fn consumer_bundler_path_as_cache_key() {
    let path1 = "/project/src/index.ts";
    let path2 = "/project/src/index.ts";
    let f1 = file(path1);
    let f2 = file(path2);
    assert_eq!(f1.name(), f2.name());
    assert_eq!(f1.name(), path1);
}

// ────────── Bun-internal — explicit-type override for content negotiation ──
//
// Source: Bun.file(path, { type: "..." }) — pattern used when the consumer
// knows the content type but the path lacks an extension.

#[test]
fn consumer_explicit_type_overrides_extension() {
    let f = BunFile::open_with_type("data.bin", "application/x-protobuf");
    assert_eq!(f.mime_type(), "application/x-protobuf");
}
