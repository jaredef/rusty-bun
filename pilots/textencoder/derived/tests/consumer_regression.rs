// Consumer-regression suite for TextEncoder + TextDecoder.
//
// Each test encodes a documented behavioral expectation from a real npm
// consumer of TextEncoder/TextDecoder, with cited source. Per Doc 707
// (Pin-Art at the Behavioral Surface), each test is a bidirectional pin:
//   forward: derivation must satisfy the consumer's expectation
//   backward: cite reveals Bun's implicit commitment to that invariant

use rusty_textencoder::*;

// ─────────── undici / node-fetch — Response.text() round-trip ──────────
//
// Source: https://github.com/nodejs/undici/blob/main/lib/web/fetch/body.js
//   `bodyMixinMethods.text` → `consumeBody(this, "text", "Response.text")`
//   → byte sequence decoded via TextDecoder("utf-8") with stream:false.
// Consumer expectation: response bodies in UTF-8 round-trip exactly,
// regardless of source encoding (server is required to declare UTF-8 for
// Content-Type: text/* if it wants .text() to be lossless).

#[test]
fn consumer_undici_response_text_utf8_roundtrip() {
    let payload = "Hello, мир! 🌍";
    let bytes = TextEncoder::new().encode(Some(payload));
    let mut decoder = TextDecoder::new(None, Default::default()).unwrap();
    let decoded = decoder.decode(&bytes, Default::default()).unwrap();
    assert_eq!(decoded, payload);
}

#[test]
fn consumer_undici_response_text_empty_body() {
    // node-fetch test: zero-byte Response.text() returns empty string,
    // not throws. Source: node-fetch/test/main.js → "should handle empty body".
    let bytes = TextEncoder::new().encode(Some(""));
    assert_eq!(bytes.len(), 0);
    let mut decoder = TextDecoder::new(None, Default::default()).unwrap();
    assert_eq!(decoder.decode(&[], Default::default()).unwrap(), "");
}

// ─────────── whatwg-encoding (jsdom dependency) ──────────
//
// Source: https://github.com/jsdom/whatwg-encoding/blob/main/lib/whatwg-encoding.js
//   `decode(uint8Array, fallbackEncodingName)` → uses TextDecoder.
// Consumer expectation: TextDecoder.encoding returns the canonical name
// "utf-8" (lowercase, hyphenated), not "UTF-8" or "utf8".

#[test]
fn consumer_jsdom_whatwg_encoding_canonical_name() {
    // Different label inputs all canonicalize to "utf-8".
    for label in ["utf-8", "UTF-8", "utf8", "Unicode-1-1-UTF-8"] {
        let d = TextDecoder::new(Some(label), Default::default()).unwrap();
        assert_eq!(d.encoding(), "utf-8", "label {:?} should canonicalize", label);
    }
}

// ─────────── protobuf.js — string field encoding ──────────
//
// Source: https://github.com/protobufjs/protobuf.js/blob/master/src/util/utf8.js
//   `Utf8.write(string, buffer, offset)` — older versions used this; newer
//   versions use TextEncoder.encodeInto for performance.
// Consumer expectation: encodeInto returns {read, written} with read counting
// UTF-16 code units consumed and written counting UTF-8 bytes emitted.
// protobuf.js uses these to size buffers and advance offsets.

#[test]
fn consumer_protobufjs_encodeinto_returns_read_and_written() {
    let mut buf = [0u8; 32];
    let r = TextEncoder::new().encode_into("café", &mut buf);
    // "café" = c(1) + a(1) + f(1) + é(2 bytes UTF-8) = 5 bytes
    assert_eq!(r.written, 5);
    // "café" = 4 chars = 4 UTF-16 code units (all BMP, no surrogates)
    assert_eq!(r.read, 4);
}

#[test]
fn consumer_protobufjs_encodeinto_does_not_split_multibyte_at_boundary() {
    // protobuf.js: writing into a tight buffer must never produce a
    // truncated UTF-8 sequence; the decoder on the wire would fail.
    // Source: protobuf.js wire format reader rejects invalid UTF-8.
    let mut buf = [0u8; 3];
    let r = TextEncoder::new().encode_into("héllo", &mut buf);
    // "héllo" — h(1) é(2) = 3 bytes fits exactly.
    assert_eq!(r.written, 3);
    assert_eq!(&buf[..3], &[b'h', 0xC3, 0xA9]);
    // Verify no truncated multi-byte sequence: the bytes must be
    // valid UTF-8 when decoded.
    let mut d = TextDecoder::new(None, Default::default()).unwrap();
    assert_eq!(d.decode(&buf[..r.written], Default::default()).unwrap(), "hé");
}

// ─────────── WPT encoding test corpus ──────────
//
// Source: https://github.com/web-platform-tests/wpt/blob/master/encoding/
//   `textencoder-utf8.any.js` — basic TextEncoder UTF-8 conformance.
//   `textdecoder-utf8.any.js` — basic TextDecoder UTF-8 conformance.

#[test]
fn wpt_textencoder_utf8_basic() {
    // WPT entry: encoding "z\u{1F600}" produces specific bytes.
    let r = TextEncoder::new().encode(Some("z\u{1F600}"));
    // 'z' = 0x7A; U+1F600 = F0 9F 98 80 (4 bytes)
    assert_eq!(r, vec![0x7A, 0xF0, 0x9F, 0x98, 0x80]);
}

#[test]
fn wpt_textdecoder_utf8_replacement_on_invalid() {
    // WPT: invalid UTF-8 produces U+FFFD without throwing (default fatal: false).
    let mut d = TextDecoder::new(None, Default::default()).unwrap();
    let result = d.decode(&[0xFF, 0xFE, 0x41], Default::default()).unwrap();
    assert!(result.contains('\u{FFFD}'));
    assert!(result.contains('A'));
}

#[test]
fn wpt_textdecoder_utf8_streaming_split_codepoint() {
    // WPT: splitting a multi-byte UTF-8 sequence across stream chunks
    // must NOT emit replacement chars; the decoder retains state.
    let mut d = TextDecoder::new(None, Default::default()).unwrap();
    let s1 = d.decode(&[0xF0, 0x9F], TextDecodeOptions { stream: true }).unwrap();
    assert_eq!(s1, "");
    let s2 = d.decode(&[0x98, 0x80], TextDecodeOptions { stream: false }).unwrap();
    assert_eq!(s2, "\u{1F600}");
}

// ─────────── BOM handling — IE-era CSV/text-file consumers ──────────
//
// Source: many JS CSV parsers (papaparse, csv-parse) deal with UTF-8 BOM
// from Windows-exported CSVs. They expect TextDecoder to consume the BOM
// when ignoreBOM is false (default).
// https://github.com/mholt/PapaParse/blob/master/papaparse.js
//   handles BOM stripping BEFORE TextDecoder; relies on TextDecoder NOT
//   double-stripping if the BOM was already removed.

#[test]
fn consumer_csv_parser_bom_consumed_by_default() {
    let mut d = TextDecoder::new(None, Default::default()).unwrap();
    let bytes = b"\xEF\xBB\xBFname,age\nAlice,30\n";
    let s = d.decode(bytes, Default::default()).unwrap();
    assert_eq!(s, "name,age\nAlice,30\n");
    assert!(!s.starts_with('\u{FEFF}'));
}

#[test]
fn consumer_csv_parser_bom_preserved_with_ignore_bom() {
    let mut d = TextDecoder::new(
        None,
        TextDecoderOptions { fatal: false, ignore_bom: true },
    ).unwrap();
    let bytes = b"\xEF\xBB\xBFhello";
    let s = d.decode(bytes, Default::default()).unwrap();
    assert_eq!(s, "\u{FEFF}hello");
}

// ─────────── Fatal mode — strict-validating consumers ──────────
//
// Source: https://github.com/sidorares/node-mysql2/blob/master/lib/parsers/
//   `MySQL` text-protocol parsers use TextDecoder with fatal:true to detect
//   protocol corruption immediately rather than emit garbled strings.

#[test]
fn consumer_strict_decoder_fatal_throws_on_invalid() {
    let mut d = TextDecoder::new(
        None,
        TextDecoderOptions { fatal: true, ignore_bom: false },
    ).unwrap();
    let r = d.decode(&[0xFF, 0xFE], Default::default());
    assert_eq!(r, Err(DecoderError::InvalidSequence));
}
