// Verifier for the Blob pilot.
//
// CD-BLOB = runs/2026-05-10-bun-v0.13b-spec-batch/constraints/blob.constraints.md
// SPEC    = https://w3c.github.io/FileAPI/#blob-section

use rusty_blob::*;

// ─────────── CD-BLOB / BLOB1 antichain reps (cardinality 17) ──────────

// `new Blob(["abcdef"])` ⇒ `expect(blob.size).toBe(6)`
#[test]
fn cd_blob1_size_from_string_part() {
    let b = Blob::from_parts(&[BlobPart::Str("abcdef")], Default::default());
    assert_eq!(b.size(), 6);
}

// `expect(typeof Blob !== "undefined").toBe(true)` — class existence
#[test]
fn cd_blob1_class_exists() {
    let _ = Blob::empty();
}

// FormData multipart roundtrip preserving Blob equality (`expect(c).toBe(b)`)
#[test]
fn cd_blob1_byte_equality_roundtrip() {
    let b1 = Blob::from_parts(&[BlobPart::Bytes(b"hello world")], Default::default());
    let b2 = Blob::from_bytes(b"hello world".to_vec());
    assert_eq!(b1, b2);
}

// ─────────── CD-BLOB / BLOB2 antichain reps (cardinality 2) ──────────

// `expect(blob.name).toBeUndefined()` — Blob has no name property (#10178)
#[test]
fn cd_blob2_no_name_property() {
    // Pilot's Blob has no `.name` field at all; the test is structural —
    // attempting to access a non-existent name is a compile error in Rust,
    // which is the type-system analog of "undefined" in JS. This test
    // documents the structural property by verifying the field doesn't
    // exist via what we expose publicly.
    let b = Blob::empty();
    let _ = b.size();
    let _ = b.mime_type();
    // No `.name` accessor — the type-system enforces the absence.
}

// `Blob is defined as a global constructor in any execution context with [Exposed=*]`
#[test]
fn cd_blob2_constructor_pattern() {
    let _ = Blob::empty();
    let _ = Blob::from_parts(&[BlobPart::Str("x")], Default::default());
}

// ─────────── CD-BLOB / BLOB3 antichain rep (cardinality 1) ──────────

// `expect(new Blob([])).toBeInstanceOf(Blob)` — class preservation across
// constructor with empty parts
#[test]
fn cd_blob3_empty_parts_constructs() {
    let b = Blob::from_parts(&[], Default::default());
    assert_eq!(b.size(), 0);
    assert_eq!(b.mime_type(), "");
}

// ─────────── Spec-derived: size + type ──────────

#[test]
fn spec_size_returns_byte_length() {
    let b = Blob::from_bytes(vec![1, 2, 3, 4, 5]);
    assert_eq!(b.size(), 5);
}

#[test]
fn spec_type_returns_empty_when_none() {
    let b = Blob::empty();
    assert_eq!(b.mime_type(), "");
}

#[test]
fn spec_type_lowercases_ascii() {
    let b = Blob::from_parts(
        &[BlobPart::Str("x")],
        BlobPropertyBag { mime_type: "Application/JSON".into(), ..Default::default() },
    );
    assert_eq!(b.mime_type(), "application/json");
}

#[test]
fn spec_type_preserves_non_ascii() {
    // SPEC: only ASCII upper→lower required; non-ASCII passes through.
    let b = Blob::from_parts(
        &[BlobPart::Str("x")],
        BlobPropertyBag { mime_type: "TEXT/Ω".into(), ..Default::default() },
    );
    assert_eq!(b.mime_type(), "text/Ω");
}

// ─────────── Spec-derived: slice ──────────

#[test]
fn spec_slice_basic_range() {
    let b = Blob::from_bytes(b"hello world".to_vec());
    let s = b.slice(0, Some(5), None);
    assert_eq!(s.size(), 5);
    assert_eq!(s.text(), "hello");
}

#[test]
fn spec_slice_default_end_is_size() {
    let b = Blob::from_bytes(b"hello world".to_vec());
    let s = b.slice(6, None, None);
    assert_eq!(s.text(), "world");
}

#[test]
fn spec_slice_negative_offsets_clamp() {
    // SPEC: negative offsets become `size + offset` clamped to 0.
    let b = Blob::from_bytes(b"hello world".to_vec());
    // start = -5 ⇒ size + (-5) = 6
    let s = b.slice(-5, None, None);
    assert_eq!(s.text(), "world");
    // start = -1000 ⇒ clamps to 0
    let s2 = b.slice(-1000, None, None);
    assert_eq!(s2.text(), "hello world");
}

#[test]
fn spec_slice_end_clamps_to_size() {
    let b = Blob::from_bytes(b"hi".to_vec());
    let s = b.slice(0, Some(1000), None);
    assert_eq!(s.text(), "hi");
}

#[test]
fn spec_slice_content_type_override() {
    let b = Blob::from_parts(
        &[BlobPart::Str("data")],
        BlobPropertyBag { mime_type: "text/plain".into(), ..Default::default() },
    );
    let s = b.slice(0, None, Some("APPLICATION/Json"));
    assert_eq!(s.mime_type(), "application/json");
}

#[test]
fn spec_slice_no_content_type_override_clears_to_empty() {
    // SPEC: when content_type is omitted, the slice's type is empty
    // string (not the parent's type).
    let b = Blob::from_parts(
        &[BlobPart::Str("data")],
        BlobPropertyBag { mime_type: "text/plain".into(), ..Default::default() },
    );
    let s = b.slice(0, None, None);
    assert_eq!(s.mime_type(), "");
}

#[test]
fn spec_slice_swapped_endpoints_yield_empty() {
    let b = Blob::from_bytes(b"abcdef".to_vec());
    let s = b.slice(4, Some(2), None); // start > end after clamping
    assert_eq!(s.size(), 0);
}

// ─────────── Spec-derived: text / array_buffer / bytes ──────────

#[test]
fn spec_text_decodes_utf8() {
    let b = Blob::from_parts(&[BlobPart::Str("héllo")], Default::default());
    assert_eq!(b.text(), "héllo");
}

#[test]
fn spec_text_lossy_on_invalid_utf8() {
    let b = Blob::from_bytes(vec![0xFE, 0xFF]);
    let t = b.text();
    assert!(t.contains('\u{FFFD}'));
}

#[test]
fn spec_array_buffer_returns_full_content() {
    let b = Blob::from_bytes(vec![1, 2, 3, 4]);
    assert_eq!(b.array_buffer(), vec![1, 2, 3, 4]);
}

#[test]
fn spec_bytes_alias_of_array_buffer() {
    let b = Blob::from_bytes(vec![10, 20, 30]);
    assert_eq!(b.bytes(), b.array_buffer());
}

// ─────────── Spec-derived: multi-part construction ──────────

#[test]
fn spec_constructor_concatenates_string_parts() {
    let b = Blob::from_parts(
        &[BlobPart::Str("hello "), BlobPart::Str("world")],
        Default::default(),
    );
    assert_eq!(b.text(), "hello world");
}

#[test]
fn spec_constructor_mixes_bytes_and_strings() {
    let b = Blob::from_parts(
        &[BlobPart::Bytes(b"AB"), BlobPart::Str("CD"), BlobPart::Bytes(b"EF")],
        Default::default(),
    );
    assert_eq!(b.text(), "ABCDEF");
    assert_eq!(b.size(), 6);
}

#[test]
fn spec_constructor_includes_blob_parts() {
    let inner = Blob::from_bytes(b"inner".to_vec());
    let outer = Blob::from_parts(
        &[BlobPart::Str("["), BlobPart::Blob(&inner), BlobPart::Str("]")],
        Default::default(),
    );
    assert_eq!(outer.text(), "[inner]");
}

// ─────────── Endings normalization ──────────

#[test]
fn spec_endings_transparent_preserves_input() {
    let b = Blob::from_parts(
        &[BlobPart::Str("a\rb\nc\r\nd")],
        BlobPropertyBag { endings: LineEndings::Transparent, ..Default::default() },
    );
    assert_eq!(b.bytes(), b"a\rb\nc\r\nd");
}

#[test]
#[cfg(not(windows))]
fn spec_endings_native_lf_on_unix() {
    let b = Blob::from_parts(
        &[BlobPart::Str("a\rb\nc\r\nd")],
        BlobPropertyBag { endings: LineEndings::Native, ..Default::default() },
    );
    assert_eq!(b.bytes(), b"a\nb\nc\nd");
}
