// http-codec pilot — HTTP/1.1 wire-format codec.
//
// Inputs:
//   SPEC — RFC 7230 (HTTP/1.1 Message Syntax + Routing)
//          RFC 7231 (Semantics + Content)
//          RFC 7232 (Conditional Requests) — partial (E-Tag handling)
//   REF  — Bun.serve under the hood uses Bun's native HTTP parser; this
//          pilot models the wire format at byte fidelity for the Tier-G
//          deferred item "Bun.serve full (with sockets)" per seed §V.
//   M11  — DigestInfo and chunked-encoding sentinel bytes are hand-typed;
//          sanity-checked against curl-output for parse + Bun-emitted
//          bytes for serialize before commit.
//
// Scope:
//   - parse_request(bytes) → ParsedRequest | Err
//   - parse_response(bytes) → ParsedResponse | Err
//   - serialize_request(method, target, headers, body) → bytes
//   - serialize_response(status, reason, headers, body) → bytes
//   - chunked_encode(chunks) → bytes (single message build)
//   - chunked_decode(bytes) → Vec<u8> (assembled body) | Err
//   - HTTP/1.1 only; no upgrade, no WebSocket, no HTTP/2/3.
//
// Out of scope (deferred to follow-up rounds per M10 staging):
//   - Stateful streaming parser (this is a whole-message parser)
//   - Trailer headers in chunked encoding
//   - Compression (gzip / deflate / br Content-Encoding)
//   - Multipart bodies (form-data / mixed)

use std::str;

// ─────────────────────── Whole-message parsed shape ────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRequest {
    pub method: String,
    pub target: String,
    pub version: String,             // "HTTP/1.1"
    pub headers: Vec<(String, String)>,  // case as-received; lookup is case-insensitive
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedResponse {
    pub version: String,
    pub status: u16,
    pub reason: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

// ─────────────────────── Errors ────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    Truncated,                  // not enough bytes
    BadStartLine(String),
    BadHeader(String),
    BadVersion(String),
    BadStatus(String),
    BadChunkEncoding(String),
    ContentLengthMismatch,
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodecError::Truncated => write!(f, "http-codec: truncated message"),
            CodecError::BadStartLine(s) => write!(f, "http-codec: bad start line: {}", s),
            CodecError::BadHeader(s) => write!(f, "http-codec: bad header: {}", s),
            CodecError::BadVersion(s) => write!(f, "http-codec: bad version: {}", s),
            CodecError::BadStatus(s) => write!(f, "http-codec: bad status: {}", s),
            CodecError::BadChunkEncoding(s) => write!(f, "http-codec: bad chunk-encoding: {}", s),
            CodecError::ContentLengthMismatch => write!(f, "http-codec: content-length mismatch"),
        }
    }
}

// ─────────────────────── Helpers ───────────────────────────────────

// Find b"\r\n\r\n" — the end of the header section. Returns position
// where the body would start (after CRLFCRLF).
fn find_header_end(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < 4 { return None; }
    let needle = b"\r\n\r\n";
    for i in 0..=bytes.len().saturating_sub(4) {
        if &bytes[i..i + 4] == needle {
            return Some(i + 4);
        }
    }
    None
}

fn case_insensitive_get<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    let lower = name.to_ascii_lowercase();
    headers.iter().find(|(n, _)| n.to_ascii_lowercase() == lower).map(|(_, v)| v.as_str())
}

fn parse_headers(section: &[u8]) -> Result<Vec<(String, String)>, CodecError> {
    let s = str::from_utf8(section).map_err(|_| CodecError::BadHeader("non-UTF-8 header bytes".into()))?;
    let mut out = Vec::new();
    for line in s.split("\r\n") {
        if line.is_empty() { continue; }
        let colon = line.find(':').ok_or_else(|| CodecError::BadHeader(line.into()))?;
        let name = line[..colon].trim().to_string();
        if name.is_empty() {
            return Err(CodecError::BadHeader("empty header name".into()));
        }
        let value = line[colon + 1..].trim().to_string();
        out.push((name, value));
    }
    Ok(out)
}

// ─────────────────────── Request parser ────────────────────────────

pub fn parse_request(bytes: &[u8]) -> Result<ParsedRequest, CodecError> {
    let header_end = find_header_end(bytes).ok_or(CodecError::Truncated)?;
    let header_section = &bytes[..header_end - 4];
    let body_section = &bytes[header_end..];

    // Find start-line. If header_section has no CRLF, the whole section is
    // the start line (request has no headers).
    let (start_line_bytes, headers_bytes): (&[u8], &[u8]) =
        match header_section.windows(2).position(|w| w == b"\r\n") {
            Some(crlf) => (&header_section[..crlf], &header_section[crlf + 2..]),
            None => (header_section, &[][..]),
        };
    let start_line = str::from_utf8(start_line_bytes)
        .map_err(|_| CodecError::BadStartLine("non-UTF-8 start line".into()))?;
    let parts: Vec<&str> = start_line.splitn(3, ' ').collect();
    if parts.len() != 3 {
        return Err(CodecError::BadStartLine(start_line.into()));
    }
    let method = parts[0].to_string();
    let target = parts[1].to_string();
    let version = parts[2].to_string();
    if !version.starts_with("HTTP/") {
        return Err(CodecError::BadVersion(version));
    }

    let headers = parse_headers(headers_bytes)?;
    let body = decode_body(&headers, body_section)?;
    Ok(ParsedRequest { method, target, version, headers, body })
}

// ─────────────────────── Response parser ───────────────────────────

pub fn parse_response(bytes: &[u8]) -> Result<ParsedResponse, CodecError> {
    let header_end = find_header_end(bytes).ok_or(CodecError::Truncated)?;
    let header_section = &bytes[..header_end - 4];
    let body_section = &bytes[header_end..];

    let (status_line_bytes, headers_bytes): (&[u8], &[u8]) =
        match header_section.windows(2).position(|w| w == b"\r\n") {
            Some(crlf) => (&header_section[..crlf], &header_section[crlf + 2..]),
            None => (header_section, &[][..]),
        };
    let status_line = str::from_utf8(status_line_bytes)
        .map_err(|_| CodecError::BadStartLine("non-UTF-8 status line".into()))?;
    let parts: Vec<&str> = status_line.splitn(3, ' ').collect();
    if parts.len() < 2 {
        return Err(CodecError::BadStartLine(status_line.into()));
    }
    let version = parts[0].to_string();
    if !version.starts_with("HTTP/") {
        return Err(CodecError::BadVersion(version));
    }
    let status: u16 = parts[1].parse()
        .map_err(|_| CodecError::BadStatus(parts[1].into()))?;
    let reason = if parts.len() == 3 { parts[2].to_string() } else { String::new() };

    let headers = parse_headers(headers_bytes)?;
    let body = decode_body(&headers, body_section)?;
    Ok(ParsedResponse { version, status, reason, headers, body })
}

// Decode body using Content-Length or Transfer-Encoding: chunked.
// If neither is present, body is taken as-is up to end of bytes.
fn decode_body(headers: &[(String, String)], body_bytes: &[u8]) -> Result<Vec<u8>, CodecError> {
    if let Some(te) = case_insensitive_get(headers, "transfer-encoding") {
        if te.to_ascii_lowercase().contains("chunked") {
            return chunked_decode(body_bytes);
        }
    }
    if let Some(cl) = case_insensitive_get(headers, "content-length") {
        let n: usize = cl.parse().map_err(|_| CodecError::BadHeader(format!("invalid content-length {}", cl)))?;
        if body_bytes.len() < n { return Err(CodecError::ContentLengthMismatch); }
        return Ok(body_bytes[..n].to_vec());
    }
    Ok(body_bytes.to_vec())
}

// ─────────────────────── Serializers ───────────────────────────────

pub fn serialize_request(
    method: &str, target: &str, headers: &[(String, String)], body: &[u8],
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(method.as_bytes());
    out.push(b' ');
    out.extend_from_slice(target.as_bytes());
    out.extend_from_slice(b" HTTP/1.1\r\n");
    write_headers(&mut out, headers, body.len());
    out.extend_from_slice(b"\r\n");
    out.extend_from_slice(body);
    out
}

pub fn serialize_response(
    status: u16, reason: &str, headers: &[(String, String)], body: &[u8],
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"HTTP/1.1 ");
    out.extend_from_slice(status.to_string().as_bytes());
    out.push(b' ');
    out.extend_from_slice(reason.as_bytes());
    out.extend_from_slice(b"\r\n");
    write_headers(&mut out, headers, body.len());
    out.extend_from_slice(b"\r\n");
    out.extend_from_slice(body);
    out
}

fn write_headers(out: &mut Vec<u8>, headers: &[(String, String)], body_len: usize) {
    let mut has_cl = false;
    let mut has_te = false;
    for (n, v) in headers {
        let lower = n.to_ascii_lowercase();
        if lower == "content-length" { has_cl = true; }
        if lower == "transfer-encoding" { has_te = true; }
        out.extend_from_slice(n.as_bytes());
        out.extend_from_slice(b": ");
        out.extend_from_slice(v.as_bytes());
        out.extend_from_slice(b"\r\n");
    }
    // Auto-set Content-Length unless Transfer-Encoding was specified.
    if !has_cl && !has_te {
        out.extend_from_slice(b"Content-Length: ");
        out.extend_from_slice(body_len.to_string().as_bytes());
        out.extend_from_slice(b"\r\n");
    }
}

// ─────────────────────── Chunked transfer-encoding ─────────────────
// RFC 7230 §4.1: chunk-size = 1*HEXDIG; chunks terminated by "0\r\n\r\n".

pub fn chunked_encode(chunks: &[&[u8]]) -> Vec<u8> {
    let mut out = Vec::new();
    for c in chunks {
        out.extend_from_slice(format!("{:X}\r\n", c.len()).as_bytes());
        out.extend_from_slice(c);
        out.extend_from_slice(b"\r\n");
    }
    out.extend_from_slice(b"0\r\n\r\n");
    out
}

pub fn chunked_decode(bytes: &[u8]) -> Result<Vec<u8>, CodecError> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        // Read chunk-size line.
        let line_end = bytes[i..].windows(2).position(|w| w == b"\r\n")
            .ok_or_else(|| CodecError::BadChunkEncoding("missing chunk-size CRLF".into()))? + i;
        let size_str = str::from_utf8(&bytes[i..line_end])
            .map_err(|_| CodecError::BadChunkEncoding("non-UTF-8 chunk size".into()))?;
        // Strip optional chunk-ext after ';'.
        let size_hex = size_str.split(';').next().unwrap().trim();
        let size = usize::from_str_radix(size_hex, 16)
            .map_err(|_| CodecError::BadChunkEncoding(format!("bad chunk size {}", size_hex)))?;
        i = line_end + 2;
        if size == 0 {
            // Last chunk: optional trailers + final CRLF.
            // Pilot scope: skip trailers; require final CRLF.
            if !bytes[i..].starts_with(b"\r\n") {
                // Could have trailers; for pilot, accept any bytes followed
                // by "\r\n" as terminator.
                let term = bytes[i..].windows(2).position(|w| w == b"\r\n")
                    .ok_or_else(|| CodecError::BadChunkEncoding("missing terminator".into()))?;
                i += term + 2;
            } else {
                i += 2;
            }
            return Ok(out);
        }
        if i + size > bytes.len() {
            return Err(CodecError::BadChunkEncoding("chunk size exceeds remaining bytes".into()));
        }
        out.extend_from_slice(&bytes[i..i + size]);
        i += size;
        // Each chunk ends with CRLF.
        if &bytes[i..i + 2] != b"\r\n" {
            return Err(CodecError::BadChunkEncoding("chunk not followed by CRLF".into()));
        }
        i += 2;
    }
    Err(CodecError::BadChunkEncoding("no zero-chunk terminator".into()))
}
