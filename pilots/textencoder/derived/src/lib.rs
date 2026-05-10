// Simulated-derivation v0 of TextEncoder + TextDecoder.
//
// Inputs declared at the head of each function:
//   AUDIT — pilots/textencoder/AUDIT.md (constraint coverage map)
//   SPEC  — https://encoding.spec.whatwg.org/ (WHATWG Encoding Standard)
//   CD    — auto-emitted constraint docs at runs/2026-05-10-bun-derive-constraints/
//           constraints/textencoder.constraints.md and runs/2026-05-10-deno-v0.11/
//           constraints/textdecoder.constraints.md
//
// v0 strategy: derive strictly from SPEC. Where SPEC and CD diverge, follow
// SPEC. The verifier should catch the divergences; v1 iterates from there.

use std::fmt;

// ─────────────────────────── TextEncoder ────────────────────────────────
//
// SPEC §9: always UTF-8. encode(USVString) → Uint8Array. The USVString
// conversion replaces invalid UTF-16 surrogates with U+FFFD before encoding.
// In Rust, &str is already valid UTF-8, so the USV conversion happens at the
// JS-Rust boundary (caller's responsibility); we treat the input as &str.

pub struct TextEncoder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodeIntoResult {
    pub read: usize,
    pub written: usize,
}

impl TextEncoder {
    pub fn new() -> Self { TextEncoder }

    /// SPEC §9.1: `readonly attribute DOMString encoding` always returns "utf-8".
    pub fn encoding(&self) -> &'static str { "utf-8" }

    /// SPEC §9.1.encode: `Uint8Array encode(optional USVString input = "")`.
    ///
    /// SPEC: absent argument defaults to empty string ⇒ 0 bytes.
    /// SPEC: `undefined` passed explicitly coerces via JS USVString conversion to
    ///       the string "undefined" ⇒ 9 bytes ("u","n","d","e","f","i","n","e","d").
    ///
    /// CD asserts: `encode(undefined).length === 0`.
    /// v0 follows SPEC; v0 expects to fail the CD assertion under the verifier
    /// at the JS boundary where `undefined` is the input. In Rust, undefined
    /// has no native representation, so the CD constraint is unobservable here
    /// — but Option<&str>::None is the closest analog to "absent argument",
    /// and that returns 0 bytes per SPEC default. (This already exposes a gap:
    /// Rust's type system makes the JS undefined-vs-absent distinction
    /// invisible. See VERIFIER-REPORT for the resolution.)
    pub fn encode(&self, input: Option<&str>) -> Vec<u8> {
        match input {
            None => Vec::new(),
            Some(s) => s.as_bytes().to_vec(),
        }
    }

    /// SPEC §9.1.encodeInto: write UTF-8 bytes of `source` into `destination`,
    /// never overflowing. Return `{read: USV_chars_consumed, written: bytes}`.
    ///
    /// SPEC: read counts UTF-16 code units consumed (the JS string's "length"
    /// units), not Rust chars. We approximate with `char.len_utf16()` for each
    /// char consumed.
    pub fn encode_into(&self, source: &str, destination: &mut [u8]) -> EncodeIntoResult {
        let mut written = 0usize;
        let mut read_utf16 = 0usize;
        for ch in source.chars() {
            let utf8_len = ch.len_utf8();
            if written + utf8_len > destination.len() { break; }
            ch.encode_utf8(&mut destination[written..]);
            written += utf8_len;
            read_utf16 += ch.len_utf16();
        }
        EncodeIntoResult { read: read_utf16, written }
    }
}

impl Default for TextEncoder { fn default() -> Self { Self::new() } }

impl fmt::Display for TextEncoder {
    /// CD asserts: `encoder.toString() === "[object TextEncoder]"`. JS's
    /// Object.prototype.toString uses the @@toStringTag well-known symbol;
    /// in Rust, Display is the closest analog.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[object TextEncoder]")
    }
}

// ─────────────────────────── TextDecoder ────────────────────────────────
//
// SPEC §10: TextDecoder(label, {fatal, ignoreBOM}). Pilot scope: UTF-8 only;
// other encoding labels throw RangeError. SPEC: full registry of 50+ encodings
// is out-of-scope per AUDIT.md "Pilot scope" §.

#[derive(Debug, Clone)]
pub struct TextDecoder {
    encoding: &'static str,
    fatal: bool,
    ignore_bom: bool,
    /// Streaming-mode partial-sequence buffer. Holds bytes from a prior
    /// `.decode(bytes, {stream: true})` call that did not complete a UTF-8
    /// sequence; flushed on the next decode.
    pending: Vec<u8>,
    /// Whether we've consumed the leading BOM yet for this decoder instance.
    /// Per SPEC, BOM consumption is a one-shot at the start of the byte stream.
    bom_consumed: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct TextDecoderOptions {
    pub fatal: bool,
    pub ignore_bom: bool,
}

impl Default for TextDecoderOptions {
    fn default() -> Self { Self { fatal: false, ignore_bom: false } }
}

#[derive(Debug, Clone, Copy)]
pub struct TextDecodeOptions {
    pub stream: bool,
}

impl Default for TextDecodeOptions {
    fn default() -> Self { Self { stream: false } }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecoderError {
    /// Constructor: unknown encoding label.
    UnknownEncoding(String),
    /// `decode` with `fatal: true`: invalid byte sequence.
    InvalidSequence,
}

impl TextDecoder {
    /// SPEC §10.1.constructor.
    /// label: optional, defaults to "utf-8". Resolved via the encoding-label
    /// table; unknown labels throw RangeError.
    pub fn new(label: Option<&str>, options: TextDecoderOptions) -> Result<Self, DecoderError> {
        let resolved = resolve_label(label.unwrap_or("utf-8"))?;
        Ok(TextDecoder {
            encoding: resolved,
            fatal: options.fatal,
            ignore_bom: options.ignore_bom,
            pending: Vec::new(),
            bom_consumed: false,
        })
    }

    pub fn encoding(&self) -> &'static str { self.encoding }
    pub fn fatal(&self) -> bool { self.fatal }
    pub fn ignore_bom(&self) -> bool { self.ignore_bom }

    /// SPEC §10.1.decode.
    pub fn decode(&mut self, input: &[u8], options: TextDecodeOptions) -> Result<String, DecoderError> {
        // Combine any pending stream-mode bytes with the new input.
        let mut buf: Vec<u8> = Vec::with_capacity(self.pending.len() + input.len());
        buf.extend_from_slice(&self.pending);
        buf.extend_from_slice(input);
        self.pending.clear();

        // SPEC: BOM handling — consume leading EF BB BF if encoding is utf-8 and
        // ignore_bom is false and we haven't yet consumed a BOM for this decoder.
        let mut start = 0;
        if !self.bom_consumed && !self.ignore_bom && self.encoding == "utf-8" {
            if buf.len() >= 3 && &buf[..3] == [0xEF, 0xBB, 0xBF] {
                start = 3;
            }
            self.bom_consumed = true;
        }
        let body = &buf[start..];

        // UTF-8 decode. In streaming mode, retain incomplete trailing bytes.
        let (decoded, retained) = utf8_decode(body, self.fatal, options.stream)?;
        if options.stream {
            self.pending = retained;
        } else if !retained.is_empty() {
            // Non-streaming end-of-input with incomplete trailing bytes.
            if self.fatal { return Err(DecoderError::InvalidSequence); }
            // Otherwise: replace each incomplete byte with U+FFFD per SPEC's
            // "end-of-stream" handler in the UTF-8 decoder.
            let mut s = decoded;
            for _ in &retained { s.push('\u{FFFD}'); }
            return Ok(s);
        }
        Ok(decoded)
    }
}

/// SPEC §4.2 — get an encoding from a label. Pilot supports UTF-8 labels only.
/// All other labels resolve to UnknownEncoding.
fn resolve_label(label: &str) -> Result<&'static str, DecoderError> {
    let l = label.trim().to_ascii_lowercase();
    match l.as_str() {
        "utf-8" | "utf8" | "unicode-1-1-utf-8" | "unicode11utf8" | "unicode20utf8" | "x-unicode20utf8" => {
            Ok("utf-8")
        }
        _ => Err(DecoderError::UnknownEncoding(label.to_string())),
    }
}

/// UTF-8 decoder per SPEC §4.4.4. Returns (decoded_string, retained_partial_bytes).
/// In non-streaming, retained_partial_bytes is the trailing bytes that did not
/// form a complete sequence — the caller decides whether to U+FFFD-pad or error.
fn utf8_decode(bytes: &[u8], fatal: bool, _stream: bool) -> Result<(String, Vec<u8>), DecoderError> {
    let mut out = String::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        let need = if b < 0x80 { 1 }
            else if b & 0xE0 == 0xC0 { 2 }
            else if b & 0xF0 == 0xE0 { 3 }
            else if b & 0xF8 == 0xF0 { 4 }
            else {
                if fatal { return Err(DecoderError::InvalidSequence); }
                out.push('\u{FFFD}');
                i += 1;
                continue;
            };
        if i + need > bytes.len() {
            // Incomplete trailing sequence. Defer to caller.
            let retained = bytes[i..].to_vec();
            return Ok((out, retained));
        }
        let seq = &bytes[i..i + need];
        match std::str::from_utf8(seq) {
            Ok(s) => out.push_str(s),
            Err(_) => {
                if fatal { return Err(DecoderError::InvalidSequence); }
                out.push('\u{FFFD}');
            }
        }
        i += need;
    }
    Ok((out, Vec::new()))
}

impl fmt::Display for TextDecoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[object TextDecoder]")
    }
}
