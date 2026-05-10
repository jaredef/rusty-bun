// Verifier: each #[test] corresponds to one antichain representative from
// the auto-emitted constraint docs. Pass = derivation satisfies the
// constraint. Fail = derivation diverges. Skip (commented with reason) =
// constraint is unobservable in Rust's type system or out-of-pilot-scope.
//
// Constraint docs:
//   CD-TE = runs/2026-05-10-bun-derive-constraints/constraints/textencoder.constraints.md
//   CD-TD = runs/2026-05-10-deno-v0.11/constraints/textdecoder.constraints.md
// Spec: WHATWG Encoding Standard at https://encoding.spec.whatwg.org/

use rusty_textencoder::*;

// ────────── CD-TE / TEXT1 antichain rep #1 ──────────
// `expect(typeof TextEncoder !== "undefined").toBe(true)` — existence.
#[test]
fn cd_te_text1_existence() {
    let _ = TextEncoder::new();
}

// ────────── CD-TE / TEXT1 antichain rep #2 ──────────
// `expect(encoder.encode(undefined).length).toBe(0)`.
//
// Status: AUDIT.md row B (impl-vs-spec divergence). In Rust, "undefined" has
// no native representation. The closest semantic analog is None (absent
// argument). Per SPEC §9 default, absent argument → empty string → 0 bytes.
//
// The JS-side observable invariant is: an explicitly-passed JS undefined
// short-circuits to length 0. Per SPEC alone, this would coerce to "undefined"
// (9 bytes). The constraint doc captured Bun/V8/WPT's actual behavior, which
// is the deviation from spec. v0 derivation follows SPEC, so under a JS
// boundary this would emit 9 bytes for undefined. The verifier flags this.
#[test]
fn cd_te_text1_undefined_length_zero() {
    let encoder = TextEncoder::new();
    assert_eq!(encoder.encode(None).len(), 0,
        "absent-argument case (Rust None analog of JS absent) — passes per SPEC default");
    // The JS-undefined-explicit case is unobservable in Rust's type system.
    // Documented as VERIFIER-REPORT row R2.
}

// ────────── CD-TE / TEXT1 antichain rep #3 ──────────
// `assertEquals(encoder.toString(), "[object TextEncoder]")`.
#[test]
fn cd_te_text1_tostring_tag() {
    let encoder = TextEncoder::new();
    assert_eq!(format!("{}", encoder), "[object TextEncoder]");
}

// ────────── CD-TD / TEXT1 antichain rep #1 (Bun) ──────────
// `assertEquals(new TextDecoder().decode(stdout), "")` where stdout is empty.
#[test]
fn cd_td_text1_decode_empty() {
    let mut decoder = TextDecoder::new(None, Default::default()).unwrap();
    assert_eq!(decoder.decode(&[], Default::default()).unwrap(), "");
}

// ────────── CD-TD / TEXT1 antichain rep #2 ──────────
// `assertEquals(new TextDecoder().decode(stdout), "connected\n")`.
#[test]
fn cd_td_text1_decode_ascii_message() {
    let mut decoder = TextDecoder::new(None, Default::default()).unwrap();
    assert_eq!(
        decoder.decode(b"connected\n", Default::default()).unwrap(),
        "connected\n"
    );
}

// ────────── CD-TD / TEXT1 antichain rep #3 ──────────
// `assertEquals(new TextDecoder().decode(value), "hello world")`.
#[test]
fn cd_td_text1_decode_hello_world() {
    let mut decoder = TextDecoder::new(None, Default::default()).unwrap();
    assert_eq!(
        decoder.decode(b"hello world", Default::default()).unwrap(),
        "hello world"
    );
}

// ────────── CD-TD / TEXT2 antichain reps ──────────
// AUDIT.md identifies TEXT2 as classifier-noise: `assert(headerEnd > 0)` etc.
// are unrelated assertions; they don't constrain TextDecoder. Skip; the gap
// belongs to a separate cluster-phase fix.
#[test]
#[ignore = "AUDIT.md: TEXT2 antichain is classifier-noise; unrelated to TextDecoder"]
fn cd_td_text2_classifier_noise() {}

// ─────────────────────── SPEC-derived tests (rung-2) ──────────────────────
// These tests come from WHATWG spec, NOT from the auto-emitted constraint
// doc. They cover the AUDIT.md rows tagged "A = test-corpus coverage gap".
// A successful pass demonstrates the derivation handles spec-side semantics
// the constraint doc didn't witness.

#[test]
fn spec_te_encoding_is_utf8() {
    assert_eq!(TextEncoder::new().encoding(), "utf-8");
}

#[test]
fn spec_te_encode_ascii() {
    assert_eq!(TextEncoder::new().encode(Some("hello")), b"hello".to_vec());
}

#[test]
fn spec_te_encode_unicode() {
    // U+1F600 GRINNING FACE — 4 UTF-8 bytes: F0 9F 98 80
    assert_eq!(TextEncoder::new().encode(Some("\u{1F600}")), vec![0xF0, 0x9F, 0x98, 0x80]);
}

#[test]
fn spec_te_encode_into() {
    let mut buf = [0u8; 16];
    let r = TextEncoder::new().encode_into("hello", &mut buf);
    assert_eq!(r.read, 5);
    assert_eq!(r.written, 5);
    assert_eq!(&buf[..5], b"hello");
}

#[test]
fn spec_te_encode_into_truncates_on_short_dest() {
    // Destination too small for the full source — never write past end, never
    // split a multi-byte UTF-8 sequence.
    let mut buf = [0u8; 3];
    // "héllo" — h(1) é(2) l(1) l(1) o(1) = 6 UTF-8 bytes
    let r = TextEncoder::new().encode_into("héllo", &mut buf);
    // Should fit h(1) + é(2) = 3 bytes; not split é.
    assert_eq!(r.written, 3);
    assert_eq!(r.read, 2); // 2 chars consumed, both = 1 UTF-16 code unit
    assert_eq!(&buf[..3], &[b'h', 0xC3, 0xA9]);
}

#[test]
fn spec_td_default_label_is_utf8() {
    let d = TextDecoder::new(None, Default::default()).unwrap();
    assert_eq!(d.encoding(), "utf-8");
}

#[test]
fn spec_td_label_aliases_resolve_to_utf8() {
    for label in ["UTF-8", "utf8", "Unicode-1-1-UTF-8", "  utf-8  "] {
        let d = TextDecoder::new(Some(label), Default::default()).unwrap();
        assert_eq!(d.encoding(), "utf-8", "label {:?} should resolve to utf-8", label);
    }
}

#[test]
fn spec_td_unknown_label_errors() {
    let r = TextDecoder::new(Some("not-a-real-encoding"), Default::default());
    assert!(matches!(r, Err(DecoderError::UnknownEncoding(_))));
}

#[test]
fn spec_td_unicode_decode() {
    let mut d = TextDecoder::new(None, Default::default()).unwrap();
    let bytes = [0xF0, 0x9F, 0x98, 0x80]; // U+1F600 GRINNING FACE
    assert_eq!(d.decode(&bytes, Default::default()).unwrap(), "\u{1F600}");
}

#[test]
fn spec_td_consumes_bom_by_default() {
    let mut d = TextDecoder::new(None, Default::default()).unwrap();
    assert_eq!(d.decode(b"\xEF\xBB\xBFhi", Default::default()).unwrap(), "hi");
}

#[test]
fn spec_td_ignore_bom_keeps_it() {
    let mut d = TextDecoder::new(None, TextDecoderOptions { fatal: false, ignore_bom: true }).unwrap();
    assert_eq!(d.decode(b"\xEF\xBB\xBFhi", Default::default()).unwrap(), "\u{FEFF}hi");
}

#[test]
fn spec_td_fatal_mode_rejects_invalid() {
    let mut d = TextDecoder::new(None, TextDecoderOptions { fatal: true, ignore_bom: false }).unwrap();
    let r = d.decode(&[0xFF, 0xFE], Default::default());
    assert_eq!(r, Err(DecoderError::InvalidSequence));
}

#[test]
fn spec_td_replacement_in_default_mode() {
    let mut d = TextDecoder::new(None, Default::default()).unwrap();
    assert_eq!(d.decode(&[0xFF], Default::default()).unwrap(), "\u{FFFD}");
}

#[test]
fn spec_td_streaming_partial_sequence() {
    let mut d = TextDecoder::new(None, Default::default()).unwrap();
    // Split U+1F600 (F0 9F 98 80) across two stream chunks.
    let s1 = d.decode(&[0xF0, 0x9F], TextDecodeOptions { stream: true }).unwrap();
    assert_eq!(s1, "");
    let s2 = d.decode(&[0x98, 0x80], TextDecodeOptions { stream: false }).unwrap();
    assert_eq!(s2, "\u{1F600}");
}

#[test]
fn spec_td_default_fatal_is_false() {
    let d = TextDecoder::new(None, Default::default()).unwrap();
    assert!(!d.fatal());
    assert!(!d.ignore_bom());
}
