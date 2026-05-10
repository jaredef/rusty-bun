// Verifier for the File pilot.
//
// CD-FILE = runs/2026-05-10-bun-v0.13b-spec-batch/constraints/file.constraints.md
// SPEC    = https://w3c.github.io/FileAPI/#file-section

use rusty_file::*;

// ─────────── CD-FILE / FILE1 antichain reps (cardinality 22) ──────────

// `expect(file.name).toBe("example.txt")` — name preservation
#[test]
fn cd_file1_name_preserved() {
    let f = File::new(&[BlobPart::Str("data")], "example.txt", Default::default());
    assert_eq!(f.name(), "example.txt");
}

// `expect(typeof File !== "undefined").toBe(true)` — class existence
#[test]
fn cd_file1_class_exists() {
    let _ = File::new(&[], "x", Default::default());
}

// `new File(new Uint8Array(), "file.txt")` ⇒ `expect(blob.name).toBe("file.txt")`
#[test]
fn cd_file1_constructed_from_bytes_with_name() {
    let f = File::new(&[BlobPart::Bytes(&[1, 2, 3])], "file.txt", Default::default());
    assert_eq!(f.name(), "file.txt");
    assert_eq!(f.size(), 3);
}

// File extends Blob — `instanceof Blob` test analog
#[test]
fn cd_file1_extends_blob() {
    let f = File::new(&[BlobPart::Str("x")], "x.txt", Default::default());
    let b = f.as_blob();
    assert_eq!(b.size(), 1);
}

// ─────────── CD-FILE / FILE2 antichain rep ──────────

// `File is defined as a global constructor in any execution context with [Exposed=*]`
#[test]
fn cd_file2_constructor_pattern() {
    let _ = File::new(&[], "name", Default::default());
}

// ─────────── Spec-derived: name ──────────

#[test]
fn spec_name_is_required_constructor_arg() {
    let f = File::new(&[BlobPart::Str("body")], "report.pdf", Default::default());
    assert_eq!(f.name(), "report.pdf");
}

// ─────────── Spec-derived: lastModified ──────────

#[test]
fn spec_last_modified_default_is_zero_when_unspecified() {
    let f = File::new(&[], "x", Default::default());
    // Pilot's default is 0 (epoch); the spec says "current time", but the
    // pilot is platform-agnostic. AUDIT documents this scope choice.
    assert_eq!(f.last_modified(), 0);
}

#[test]
fn spec_last_modified_uses_provided_value() {
    let f = File::new(
        &[],
        "x",
        FilePropertyBag {
            last_modified: Some(1_700_000_000_000),
            ..Default::default()
        },
    );
    assert_eq!(f.last_modified(), 1_700_000_000_000);
}

// ─────────── Spec-derived: webkitRelativePath ──────────

#[test]
fn spec_webkit_relative_path_default_empty() {
    let f = File::new(&[], "x", Default::default());
    assert_eq!(f.webkit_relative_path(), "");
}

// ─────────── Spec-derived: Blob delegation surface ──────────

#[test]
fn spec_size_delegates_to_blob() {
    let f = File::new(&[BlobPart::Str("hello world")], "x", Default::default());
    assert_eq!(f.size(), 11);
}

#[test]
fn spec_type_delegates_to_blob_with_normalization() {
    let f = File::new(
        &[],
        "x.json",
        FilePropertyBag {
            mime_type: "Application/JSON".into(),
            ..Default::default()
        },
    );
    assert_eq!(f.mime_type(), "application/json");
}

#[test]
fn spec_slice_delegates_to_blob_returning_blob_not_file() {
    // SPEC: slice() on a File returns a Blob, not a File. The spec is
    // explicit — slicing strips the File-specific metadata.
    let f = File::new(&[BlobPart::Str("hello world")], "x.txt", Default::default());
    let s = f.slice(6, None, None);
    assert_eq!(s.text(), "world");
    // The slice has no name; it's a Blob, not a File. Type system enforces.
}

#[test]
fn spec_text_delegates_to_blob() {
    let f = File::new(&[BlobPart::Str("héllo")], "x", Default::default());
    assert_eq!(f.text(), "héllo");
}

#[test]
fn spec_array_buffer_delegates_to_blob() {
    let f = File::new(&[BlobPart::Bytes(&[1, 2, 3, 4])], "x", Default::default());
    assert_eq!(f.array_buffer(), vec![1, 2, 3, 4]);
}

#[test]
fn spec_bytes_delegates_to_blob() {
    let f = File::new(&[BlobPart::Bytes(&[10, 20])], "x", Default::default());
    assert_eq!(f.bytes(), vec![10, 20]);
}

// ─────────── Composition-as-inheritance: structural ──────────

#[test]
fn structural_file_can_be_used_where_blob_expected() {
    fn takes_blob(b: &Blob) -> usize { b.size() }
    let f = File::new(&[BlobPart::Str("test")], "x", Default::default());
    let n = takes_blob(f.as_blob());
    assert_eq!(n, 4);
}
