// buffer pilot — Node's `Buffer` data type derivation.
//
// Inputs:
//   AUDIT — pilots/buffer/AUDIT.md
//   SPEC  — Node.js docs §Buffer (https://nodejs.org/api/buffer.html)
//   CD    — runs/2026-05-10-bun-v0.13b-spec-batch/constraints/
//             buffer.constraints.md (5 properties, 26 clauses)
//
// Tier-2 ecosystem-compat. Bun's tests + Node docs serve as authoritative
// reference. Reference target: Bun's node-fallbacks/buffer.js (2,035 LOC).
//
// Pilot scope: data-layer Buffer (factories, encodings, comparison,
// slice/subarray, index_of/includes, fill, copy). Numeric readers/writers
// (readUInt8, etc.) deferred per AUDIT.

use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Utf8,
    Utf16Le,
    Latin1,
    Ascii,
    Base64,
    Hex,
}

impl Encoding {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "utf-8" | "utf8" => Some(Encoding::Utf8),
            "utf-16le" | "utf16le" | "ucs-2" | "ucs2" => Some(Encoding::Utf16Le),
            "latin1" | "binary" => Some(Encoding::Latin1),
            "ascii" => Some(Encoding::Ascii),
            "base64" => Some(Encoding::Base64),
            "hex" => Some(Encoding::Hex),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    bytes: Vec<u8>,
}

impl Buffer {
    // ─────────── Static factories ────────────

    /// SPEC: Buffer.alloc(size, fill?). Zeroed by default.
    pub fn alloc(size: usize) -> Self {
        Self { bytes: vec![0; size] }
    }

    /// SPEC: Buffer.alloc(size, fill, encoding?). Fills with the given byte
    /// pattern; if fill is a string, encodes per encoding (utf-8 default).
    pub fn alloc_filled(size: usize, fill: &[u8]) -> Self {
        if fill.is_empty() { return Self::alloc(size); }
        let mut bytes = Vec::with_capacity(size);
        for i in 0..size { bytes.push(fill[i % fill.len()]); }
        Self { bytes }
    }

    /// SPEC: Buffer.allocUnsafe(size). Pilot zeros (Rust-safe analog).
    pub fn alloc_unsafe(size: usize) -> Self { Self::alloc(size) }

    /// SPEC: Buffer.from(string, encoding?).
    pub fn from_string(s: &str, encoding: Encoding) -> Self {
        Self { bytes: encode(s, encoding) }
    }

    /// SPEC: Buffer.from(arrayLike).
    pub fn from_bytes(bytes: &[u8]) -> Self { Self { bytes: bytes.to_vec() } }

    /// SPEC: Buffer.from(buffer). Pilot's Clone covers this.
    pub fn from_buffer(b: &Buffer) -> Self { b.clone() }

    /// SPEC: Buffer.byteLength(string, encoding?).
    pub fn byte_length(s: &str, encoding: Encoding) -> usize {
        encode(s, encoding).len()
    }

    /// SPEC: Buffer.compare(a, b). Returns -1/0/1.
    pub fn compare_bufs(a: &Buffer, b: &Buffer) -> i32 {
        match a.bytes.cmp(&b.bytes) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }

    /// SPEC: Buffer.concat(list, totalLength?).
    pub fn concat(list: &[Buffer], total_length: Option<usize>) -> Self {
        let total = total_length.unwrap_or_else(|| list.iter().map(|b| b.bytes.len()).sum());
        let mut bytes = Vec::with_capacity(total);
        for b in list {
            let remaining = total.saturating_sub(bytes.len());
            let take = remaining.min(b.bytes.len());
            bytes.extend_from_slice(&b.bytes[..take]);
            if bytes.len() >= total { break; }
        }
        // Pad with zeros if total exceeds combined length.
        bytes.resize(total, 0);
        Self { bytes }
    }

    /// SPEC: Buffer.isEncoding(name).
    pub fn is_encoding(name: &str) -> bool { Encoding::from_name(name).is_some() }

    // ─────────── Instance methods ────────────

    /// SPEC: buf.length.
    pub fn len(&self) -> usize { self.bytes.len() }
    pub fn is_empty(&self) -> bool { self.bytes.is_empty() }
    pub fn as_bytes(&self) -> &[u8] { &self.bytes }

    /// SPEC: buf.toString(encoding?, start?, end?).
    pub fn to_string(&self, encoding: Encoding, start: usize, end: Option<usize>) -> String {
        let end = end.unwrap_or(self.bytes.len()).min(self.bytes.len());
        let start = start.min(end);
        let slice = &self.bytes[start..end];
        decode(slice, encoding)
    }

    /// SPEC: buf.write(string, offset, length, encoding) — returns bytes written.
    pub fn write(&mut self, s: &str, offset: usize, length: Option<usize>, encoding: Encoding) -> usize {
        if offset >= self.bytes.len() { return 0; }
        let encoded = encode(s, encoding);
        let max_room = self.bytes.len() - offset;
        let cap = length.unwrap_or(max_room).min(max_room).min(encoded.len());
        self.bytes[offset..offset + cap].copy_from_slice(&encoded[..cap]);
        cap
    }

    /// SPEC: buf.fill(value, start?, end?, encoding?).
    pub fn fill_byte(&mut self, value: u8, start: usize, end: Option<usize>) {
        let end = end.unwrap_or(self.bytes.len()).min(self.bytes.len());
        if start >= end { return; }
        for b in &mut self.bytes[start..end] { *b = value; }
    }

    pub fn fill_bytes(&mut self, pattern: &[u8], start: usize, end: Option<usize>) {
        if pattern.is_empty() { return; }
        let end = end.unwrap_or(self.bytes.len()).min(self.bytes.len());
        if start >= end { return; }
        for (i, b) in self.bytes[start..end].iter_mut().enumerate() {
            *b = pattern[i % pattern.len()];
        }
    }

    /// SPEC: buf.equals(other).
    pub fn equals(&self, other: &Buffer) -> bool { self.bytes == other.bytes }

    /// SPEC: buf.compare(other, ...). Returns -1/0/1 over the relevant ranges.
    pub fn compare(
        &self, other: &Buffer,
        target_start: usize, target_end: Option<usize>,
        source_start: usize, source_end: Option<usize>,
    ) -> i32 {
        let te = target_end.unwrap_or(other.bytes.len()).min(other.bytes.len());
        let ts = target_start.min(te);
        let se = source_end.unwrap_or(self.bytes.len()).min(self.bytes.len());
        let ss = source_start.min(se);
        match self.bytes[ss..se].cmp(&other.bytes[ts..te]) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }

    /// SPEC: buf.subarray(start?, end?). Returns a view-equivalent (copy in
    /// pilot's value-semantic model; the spec mandates a view, but consumers
    /// who don't mutate the source observe identical behavior).
    pub fn subarray(&self, start: usize, end: Option<usize>) -> Self {
        let end = end.unwrap_or(self.bytes.len()).min(self.bytes.len());
        let start = start.min(end);
        Self { bytes: self.bytes[start..end].to_vec() }
    }

    /// SPEC: buf.slice(start?, end?). Identical to subarray since v8+.
    pub fn slice(&self, start: usize, end: Option<usize>) -> Self {
        self.subarray(start, end)
    }

    /// SPEC: buf.indexOf(value, byteOffset?, encoding?). Returns isize: -1
    /// when not found.
    pub fn index_of_bytes(&self, needle: &[u8], byte_offset: usize) -> isize {
        if needle.is_empty() { return byte_offset as isize; }
        if byte_offset >= self.bytes.len() { return -1; }
        for i in byte_offset..=self.bytes.len().saturating_sub(needle.len()) {
            if self.bytes[i..i + needle.len()] == *needle { return i as isize; }
        }
        -1
    }

    pub fn last_index_of_bytes(&self, needle: &[u8]) -> isize {
        if needle.is_empty() { return self.bytes.len() as isize; }
        if needle.len() > self.bytes.len() { return -1; }
        let upper = self.bytes.len() - needle.len();
        for i in (0..=upper).rev() {
            if self.bytes[i..i + needle.len()] == *needle { return i as isize; }
        }
        -1
    }

    pub fn includes_bytes(&self, needle: &[u8]) -> bool {
        self.index_of_bytes(needle, 0) >= 0
    }

    /// SPEC: buf.copy(target, targetStart?, sourceStart?, sourceEnd?). Returns
    /// bytes copied.
    pub fn copy(
        &self, target: &mut Buffer,
        target_start: usize,
        source_start: usize, source_end: Option<usize>,
    ) -> usize {
        let se = source_end.unwrap_or(self.bytes.len()).min(self.bytes.len());
        let ss = source_start.min(se);
        if target_start >= target.bytes.len() { return 0; }
        let target_room = target.bytes.len() - target_start;
        let source_avail = se - ss;
        let n = source_avail.min(target_room);
        target.bytes[target_start..target_start + n]
            .copy_from_slice(&self.bytes[ss..ss + n]);
        n
    }
}

// ─────────────────────── Encoding codecs ────────────────────────────────

fn encode(s: &str, encoding: Encoding) -> Vec<u8> {
    match encoding {
        Encoding::Utf8 => s.as_bytes().to_vec(),
        Encoding::Utf16Le => {
            let mut out = Vec::with_capacity(s.len() * 2);
            for u in s.encode_utf16() {
                out.push(u as u8);
                out.push((u >> 8) as u8);
            }
            out
        }
        Encoding::Latin1 => s.chars().map(|c| c as u8).collect(),
        Encoding::Ascii => s.chars().map(|c| (c as u32 & 0x7F) as u8).collect(),
        Encoding::Base64 => base64_decode(s),
        Encoding::Hex => hex_decode(s),
    }
}

fn decode(bytes: &[u8], encoding: Encoding) -> String {
    match encoding {
        Encoding::Utf8 => String::from_utf8_lossy(bytes).into_owned(),
        Encoding::Utf16Le => {
            let mut units: Vec<u16> = Vec::with_capacity(bytes.len() / 2);
            for chunk in bytes.chunks_exact(2) {
                units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
            }
            String::from_utf16_lossy(&units)
        }
        Encoding::Latin1 => bytes.iter().map(|&b| b as char).collect(),
        Encoding::Ascii => bytes.iter().map(|&b| (b & 0x7F) as char).collect(),
        Encoding::Base64 => base64_encode(bytes),
        Encoding::Hex => hex_encode(bytes),
    }
}

// ─────────── base64 (RFC 4648) ───────────────────────────────────────

const B64_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };
        out.push(B64_ALPHABET[((b0 >> 2) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[(((b0 << 4) | (b1 >> 4)) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(B64_ALPHABET[(((b1 << 2) | (b2 >> 6)) & 0x3F) as usize] as char);
        } else { out.push('='); }
        if chunk.len() > 2 {
            out.push(B64_ALPHABET[(b2 & 0x3F) as usize] as char);
        } else { out.push('='); }
    }
    out
}

fn base64_decode(s: &str) -> Vec<u8> {
    fn idx(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let cleaned: Vec<u8> = s.bytes()
        .filter(|&b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b'='))
        .collect();
    let mut out = Vec::with_capacity(cleaned.len() * 3 / 4);
    for chunk in cleaned.chunks(4) {
        let v0 = idx(chunk[0]).unwrap_or(0) as u32;
        let v1 = if chunk.len() > 1 { idx(chunk[1]).unwrap_or(0) as u32 } else { 0 };
        let v2 = if chunk.len() > 2 { idx(chunk[2]).unwrap_or(0) as u32 } else { 0 };
        let v3 = if chunk.len() > 3 { idx(chunk[3]).unwrap_or(0) as u32 } else { 0 };
        let combined = (v0 << 18) | (v1 << 12) | (v2 << 6) | v3;
        out.push((combined >> 16) as u8);
        if chunk.len() > 2 { out.push(((combined >> 8) & 0xFF) as u8); }
        if chunk.len() > 3 { out.push((combined & 0xFF) as u8); }
    }
    out
}

// ─────────── hex ─────────────────────────────────────────────────────

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

fn hex_decode(s: &str) -> Vec<u8> {
    fn nyb(c: u8) -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    }
    let bytes: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i + 1 < bytes.len() {
        let hi = match nyb(bytes[i]) { Some(v) => v, None => break };
        let lo = match nyb(bytes[i + 1]) { Some(v) => v, None => break };
        out.push((hi << 4) | lo);
        i += 2;
    }
    out
}
