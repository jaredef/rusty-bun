// Consumer-regression suite for Blob.
//
// Each test encodes a documented behavioral expectation from a real
// consumer of Blob with cited source. Per Doc 707, each test is a
// bidirectional pin.

use rusty_blob::*;

// ─────────── multer — multipart/form-data file body access ──────────
//
// Source: https://github.com/expressjs/multer/blob/master/lib/make-middleware.js
//   files received as Buffer/stream are eventually exposed as Blob-shaped
//   objects (size, type, name). multer's tests assert .size matches the
//   uploaded byte length and .type matches the Content-Type header.

#[test]
fn consumer_multer_file_size_matches_byte_length() {
    let payload: Vec<u8> = (0..4096).map(|i| (i % 256) as u8).collect();
    let b = Blob::from_bytes(payload.clone());
    assert_eq!(b.size(), payload.len());
}

#[test]
fn consumer_multer_blob_type_lowercased_per_spec() {
    let b = Blob::from_parts(
        &[BlobPart::Bytes(b"<svg></svg>")],
        BlobPropertyBag { mime_type: "Image/SVG+XML".into(), ..Default::default() },
    );
    // multer normalizes Content-Type to lowercase per Blob spec; consumers
    // dispatch on `.type` and expect lowercase.
    assert_eq!(b.mime_type(), "image/svg+xml");
}

// ─────────── formdata-polyfill — Blob slicing and concat ──────────
//
// Source: https://github.com/jimmywarting/FormData/blob/master/lib/formdata.mjs
//   the polyfill builds multipart bodies by concatenating Blobs of header
//   bytes, separators, and entry bodies. relies on Blob constructor accepting
//   sequences of Blobs, and the resulting Blob's size summing the parts'.

#[test]
fn consumer_formdata_polyfill_concatenates_blob_parts() {
    let part1 = Blob::from_bytes(b"--boundary\r\n".to_vec());
    let part2 = Blob::from_bytes(b"Content-Disposition: form-data; name=\"x\"\r\n\r\n".to_vec());
    let part3 = Blob::from_bytes(b"value\r\n".to_vec());
    let combined = Blob::from_parts(
        &[BlobPart::Blob(&part1), BlobPart::Blob(&part2), BlobPart::Blob(&part3)],
        Default::default(),
    );
    assert_eq!(combined.size(), part1.size() + part2.size() + part3.size());
}

// ─────────── busboy — multipart parser slicing ──────────
//
// Source: https://github.com/mscdex/busboy/blob/master/lib/types/multipart.js
//   busboy slices incoming chunks by boundary offsets, expecting
//   Blob.slice(start, end) to produce a contiguous byte view with the given
//   byte range.

#[test]
fn consumer_busboy_slice_extracts_byte_range() {
    let body = Blob::from_bytes(b"prefix__payload__suffix".to_vec());
    let payload = body.slice(8, Some(15), None);
    assert_eq!(payload.text(), "payload");
}

#[test]
fn consumer_busboy_slice_clamps_at_end() {
    // busboy frequently passes end-of-buffer offsets that may exceed size
    // when the parser miscounts; slice must clamp rather than panic.
    let body = Blob::from_bytes(b"hi".to_vec());
    let s = body.slice(0, Some(1_000_000), None);
    assert_eq!(s.text(), "hi");
}

// ─────────── @azure/storage-blob — Blob.text() for object body fetch ──────
//
// Source: https://github.com/Azure/azure-sdk-for-js/blob/main/sdk/storage/
//   storage-blob — fetches blob bodies via Response.blob() then calls
//   .text() for text-typed blobs. Azure sometimes uploads with Windows
//   line endings; .text() must NOT normalize them.

#[test]
fn consumer_azure_storage_text_does_not_normalize_line_endings() {
    let b = Blob::from_parts(
        &[BlobPart::Bytes(b"line1\r\nline2\r\nline3")],
        Default::default(),
    );
    let s = b.text();
    assert!(s.contains("\r\n"), "Blob.text() must preserve \\r\\n, not normalize");
}

// ─────────── papa-parse — CSV parser BOM & encoding handling ──────────
//
// Source: https://github.com/mholt/PapaParse — accepts Blob input and
// requests .text() for CSV rows. A Blob constructed with leading BOM bytes
// must surface them through .text() so the parser can detect and strip.

#[test]
fn consumer_papa_parse_blob_text_passes_bom_through() {
    let b = Blob::from_bytes(b"\xEF\xBB\xBFname,age".to_vec());
    let s = b.text();
    assert!(s.starts_with('\u{FEFF}'), "Blob.text() must NOT consume BOM (TextDecoder may, but Blob.text returns the byte sequence as-is)");
}

// ─────────── WPT File API test data ──────────
//
// Source: web-platform-tests/wpt/FileAPI/blob/

#[test]
fn wpt_blob_constructor_empty_array_yields_empty_blob() {
    // WPT: `new Blob([])` produces size-0, empty-type Blob.
    let b = Blob::from_parts(&[], Default::default());
    assert_eq!(b.size(), 0);
    assert_eq!(b.mime_type(), "");
}

#[test]
fn wpt_blob_slice_negative_offsets_clamp_to_size() {
    // WPT: `b.slice(-3)` of "abcdef" produces "def".
    let b = Blob::from_bytes(b"abcdef".to_vec());
    let s = b.slice(-3, None, None);
    assert_eq!(s.text(), "def");
}

#[test]
fn wpt_blob_slice_strips_type_when_none_given() {
    // WPT: `b.slice(0, 5)` of typed Blob produces empty-type Blob unless
    // contentType arg is supplied.
    let b = Blob::from_parts(
        &[BlobPart::Bytes(b"content")],
        BlobPropertyBag { mime_type: "text/plain".into(), ..Default::default() },
    );
    let s = b.slice(0, Some(3), None);
    assert_eq!(s.mime_type(), "");
}
