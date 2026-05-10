// Consumer-regression suite for File.
//
// Each test encodes a documented behavioral expectation from a real
// consumer of File. Per Doc 707, each test is a bidirectional pin.

use rusty_file::*;

// ─────────── multer — uploaded file metadata ──────────
//
// Source: https://github.com/expressjs/multer/blob/master/lib/file-appender.js
//   appendFile populates `originalname` (== file.name), `mimetype` (== file.type),
//   `size` (== file.size). consumer code reads these properties downstream.

#[test]
fn consumer_multer_file_name_preserved() {
    let f = File::new(&[BlobPart::Bytes(b"upload")], "report.pdf", Default::default());
    assert_eq!(f.name(), "report.pdf");
}

#[test]
fn consumer_multer_file_size_from_bytes() {
    let payload: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
    let f = File::new(&[BlobPart::Bytes(&payload)], "blob.bin", Default::default());
    assert_eq!(f.size(), 1024);
}

#[test]
fn consumer_multer_file_mime_type_lowercased() {
    let f = File::new(
        &[BlobPart::Str("data")],
        "data.json",
        FilePropertyBag {
            mime_type: "Application/JSON".into(),
            ..Default::default()
        },
    );
    assert_eq!(f.mime_type(), "application/json");
}

// ─────────── formidable — multipart form file parsing ──────────
//
// Source: https://github.com/node-formidable/formidable/blob/master/src/Formidable.js
//   constructs File objects from parsed multipart parts; downstream consumers
//   (form handlers) read `lastModified` for upload-time metadata.

#[test]
fn consumer_formidable_last_modified_preserved() {
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

// ─────────── HTML form submission — File via FormData ──────────
//
// Source: WHATWG HTML §form-submission §multipart-form-data — File entries
// in FormData submit with the file's `name` as the filename in the
// Content-Disposition header.

#[test]
fn consumer_html_form_submission_file_name_used_as_filename() {
    let f = File::new(&[BlobPart::Str("document")], "report-q3.pdf", Default::default());
    // The downstream form-data builder reads .name() to populate the
    // multipart filename parameter.
    assert_eq!(f.name(), "report-q3.pdf");
}

// ─────────── File-as-Blob — slice returns Blob, not File ──────────
//
// Source: WHATWG File API §File.prototype.slice — slicing a File returns
// a Blob, not a File. Consumers that re-wrap sliced bytes for upload must
// re-add the filename explicitly.
// https://github.com/transloadit/uppy/blob/main/packages/@uppy/utils/src/getFileType.ts
//   uppy slices Files for chunked upload and explicitly re-wraps each chunk
//   in a fresh File with chunk-specific name.

#[test]
fn consumer_uppy_chunked_upload_slice_returns_blob() {
    let f = File::new(&[BlobPart::Bytes(b"large file content")], "big.bin", Default::default());
    let chunk = f.slice(0, Some(5), None);
    // Chunk is a Blob; consumer must wrap it in a fresh File for upload.
    // The type system enforces this — chunk has no .name() method.
    assert_eq!(chunk.text(), "large");
    let _: &Blob = &chunk; // verifies type
}

// ─────────── WPT FileAPI/file/ test data ──────────

#[test]
fn wpt_file_constructor_minimum_args() {
    // WPT: `new File([], "name")` produces a 0-size File named "name".
    let f = File::new(&[], "name", Default::default());
    assert_eq!(f.name(), "name");
    assert_eq!(f.size(), 0);
    assert_eq!(f.mime_type(), "");
}

#[test]
fn wpt_file_extends_blob_via_as_blob() {
    // WPT: `new File([], "x") instanceof Blob === true`. Rust analog: a File
    // can be coerced to a Blob view.
    let f = File::new(&[BlobPart::Str("test")], "x", Default::default());
    let b: &Blob = f.as_blob();
    assert_eq!(b.size(), 4);
}
