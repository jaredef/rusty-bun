// Simulated-derivation of Blob (W3C File API §3).
//
// Inputs:
//   AUDIT — pilots/blob/AUDIT.md
//   SPEC  — https://w3c.github.io/FileAPI/#blob-section
//   CD    — runs/2026-05-10-bun-v0.13b-spec-batch/constraints/blob.constraints.md
//
// Pilot is pure-bytes Blob (no lazy I/O / file-backing). stream() is
// out-of-scope per AUDIT. File extension is a separate pilot.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEndings {
    /// Default. Preserve the input's line endings as-is.
    Transparent,
    /// Convert all line endings to the platform native form.
    Native,
}

impl Default for LineEndings {
    fn default() -> Self { LineEndings::Transparent }
}

#[derive(Debug, Clone, Default)]
pub struct BlobPropertyBag {
    pub mime_type: String,
    pub endings: LineEndings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blob {
    bytes: Vec<u8>,
    mime_type: String,
}

/// Source-of-bytes union for Blob construction. Matches the spec's
/// (BufferSource | USVString | Blob) union in a pure-Rust analog.
pub enum BlobPart<'a> {
    Bytes(&'a [u8]),
    Str(&'a str),
    Blob(&'a Blob),
}

impl Blob {
    /// SPEC §3.constructor: empty Blob, no type, size 0.
    pub fn empty() -> Self {
        Self { bytes: Vec::new(), mime_type: String::new() }
    }

    /// SPEC §3.constructor with a single part and no options.
    /// Convenience shorthand around `from_parts`.
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self { bytes: bytes.into(), mime_type: String::new() }
    }

    /// SPEC §3.constructor.
    /// CD BLOB1 antichain rep: `new Blob(["abcdef"])` ⇒ `blob.size === 6`.
    pub fn from_parts(parts: &[BlobPart<'_>], options: BlobPropertyBag) -> Self {
        let mut bytes: Vec<u8> = Vec::new();
        for part in parts {
            match part {
                BlobPart::Bytes(b) => bytes.extend_from_slice(b),
                BlobPart::Str(s) => bytes.extend_from_slice(s.as_bytes()),
                BlobPart::Blob(other) => bytes.extend_from_slice(&other.bytes),
            }
        }
        if matches!(options.endings, LineEndings::Native) {
            bytes = normalize_native_line_endings(&bytes);
        }
        Self {
            bytes,
            mime_type: normalize_type(&options.mime_type),
        }
    }

    /// SPEC §3: byte-length of the blob.
    pub fn size(&self) -> usize { self.bytes.len() }

    /// SPEC §3: MIME type, lowercased ASCII; empty string when none provided.
    pub fn mime_type(&self) -> &str { &self.mime_type }

    /// SPEC §3.slice. start/end clamp to 0..=size; negative offsets become
    /// `size + offset` clamped to 0. Optional content-type override.
    pub fn slice(&self, start: i64, end: Option<i64>, content_type: Option<&str>) -> Self {
        let size = self.bytes.len() as i64;
        let resolve = |i: i64| -> usize {
            if i < 0 { (size + i).max(0) as usize }
            else { i.min(size) as usize }
        };
        let lo = resolve(start);
        let hi = match end {
            Some(e) => resolve(e),
            None => size as usize,
        };
        // SPEC §3.slice: "Let span be max(relativeEnd - relativeStart, 0)."
        // When hi < lo, the span is 0 — yields an empty slice, NOT a
        // swapped-endpoints slice. Pre-fix derivation had `(lo.min(hi),
        // hi.max(lo))` which swapped; that's a real bug the verifier
        // caught on the first run. Pilot 4 / surface 4 / verifier-found
        // bug 1 — recorded in RUN-NOTES.
        let hi = if hi < lo { lo } else { hi };
        Self {
            bytes: self.bytes[lo..hi].to_vec(),
            mime_type: content_type.map(normalize_type).unwrap_or_default(),
        }
    }

    /// SPEC §3.text: UTF-8 decode of full byte content (lossy on invalid).
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.bytes).into_owned()
    }

    /// SPEC §3.arrayBuffer: raw bytes (in JS, would be an ArrayBuffer
    /// resolving via Promise; in Rust, we expose the raw byte slice).
    pub fn array_buffer(&self) -> Vec<u8> { self.bytes.clone() }

    /// SPEC §3.bytes: alias of arrayBuffer in pilot scope.
    pub fn bytes(&self) -> Vec<u8> { self.bytes.clone() }
}

// ────────────────────────── Helpers ───────────────────────────────────

/// SPEC §3: type is ASCII-lowercased when set; preserved as the empty
/// string when no type is provided. Non-ASCII characters are passed
/// through unchanged (per spec — only ASCII upper→lower is required).
fn normalize_type(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii() { c.to_ascii_lowercase() } else { c })
        .collect()
}

/// SPEC §3: with `endings: "native"`, normalize CR, LF, CRLF to the
/// platform native sequence. Pilot uses LF (Unix). On a CRLF target,
/// this would emit b"\r\n" instead.
fn normalize_native_line_endings(input: &[u8]) -> Vec<u8> {
    let native: &[u8] = if cfg!(windows) { b"\r\n" } else { b"\n" };
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        match input[i] {
            b'\r' => {
                out.extend_from_slice(native);
                if i + 1 < input.len() && input[i + 1] == b'\n' {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            b'\n' => {
                out.extend_from_slice(native);
                i += 1;
            }
            other => {
                out.push(other);
                i += 1;
            }
        }
    }
    out
}
