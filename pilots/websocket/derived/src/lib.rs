// rusty-websocket pilot — RFC 6455 frame codec + handshake key derivation.
//
// Π1.5 round of the Tier-Π parity trajectory. Composes on:
//   - rusty-web-crypto for SHA-1 (Sec-WebSocket-Accept derivation) and
//     get_random_values (Sec-WebSocket-Key generation).
//   - http-codec for the HTTP/1.1 Upgrade handshake wire format (used
//     at the host-integration tier, not here).
//
// Scope this round: frame codec (encode + decode for client and server
// directions), close-code parsing, Sec-WebSocket-Key generation +
// Sec-WebSocket-Accept derivation. The TCP/TLS transport binding and
// the JS-side WebSocket class wire up in the follow-on integration
// round once a consumer surfaces.
//
// Per Pin-Art Doc 707: the implementation surfaces several real-world
// invariants. (1) Client-to-server frames MUST be masked; server-to-
// client MUST NOT. (2) Control frames (close/ping/pong) must have
// payload <=125 bytes and MUST NOT be fragmented. (3) Close frames
// carry an optional 16-bit big-endian status code followed by optional
// UTF-8 reason text. (4) The Sec-WebSocket-Accept value is SHA-1 of
// the client key concatenated with the fixed magic UUID
// "258EAFA5-E914-47DA-95CA-C5AB0DC85B11", then base64-encoded.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl Opcode {
    pub fn from_u8(b: u8) -> Option<Opcode> {
        use Opcode::*;
        Some(match b & 0x0F {
            0x0 => Continuation, 0x1 => Text, 0x2 => Binary,
            0x8 => Close, 0x9 => Ping, 0xA => Pong,
            _ => return None,
        })
    }
    pub fn is_control(&self) -> bool { matches!(self, Opcode::Close | Opcode::Ping | Opcode::Pong) }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub fin: bool,
    pub opcode: Opcode,
    pub payload: Vec<u8>,
    /// Some(mask) if the frame is masked (client → server). None for
    /// server → client per RFC 6455 §5.1.
    pub mask: Option<[u8; 4]>,
}

#[derive(Debug, Clone)]
pub enum WsError {
    UnexpectedEnd,
    InvalidOpcode(u8),
    ControlTooLong,
    ControlFragmented,
    ReservedBitsSet,
    PayloadTooLong,
    Crypto(String),
}

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WsError::UnexpectedEnd => write!(f, "unexpected end of frame"),
            WsError::InvalidOpcode(b) => write!(f, "invalid opcode 0x{:x}", b),
            WsError::ControlTooLong => write!(f, "control frame payload >125 bytes"),
            WsError::ControlFragmented => write!(f, "control frame must not be fragmented (FIN=0)"),
            WsError::ReservedBitsSet => write!(f, "reserved bits set (extensions unsupported)"),
            WsError::PayloadTooLong => write!(f, "payload >2^31 bytes"),
            WsError::Crypto(s) => write!(f, "crypto: {}", s),
        }
    }
}

impl std::error::Error for WsError {}

// ─────────────────────────────────────────────────────────────────────
// Frame encoding
// ─────────────────────────────────────────────────────────────────────

/// Encode a single frame to wire bytes.
pub fn encode_frame(frame: &Frame) -> Result<Vec<u8>, WsError> {
    if frame.opcode.is_control() {
        if frame.payload.len() > 125 { return Err(WsError::ControlTooLong); }
        if !frame.fin { return Err(WsError::ControlFragmented); }
    }
    let mut out = Vec::with_capacity(2 + frame.payload.len());
    let b0 = if frame.fin { 0x80 } else { 0x00 } | (frame.opcode as u8);
    out.push(b0);
    let mask_bit: u8 = if frame.mask.is_some() { 0x80 } else { 0x00 };
    let len = frame.payload.len();
    if len <= 125 {
        out.push(mask_bit | (len as u8));
    } else if len <= 0xFFFF {
        out.push(mask_bit | 126);
        out.push((len >> 8) as u8);
        out.push((len & 0xFF) as u8);
    } else if len <= 0x7FFF_FFFF_FFFF_FFFF {
        out.push(mask_bit | 127);
        for shift in (0..8).rev() {
            out.push(((len >> (8 * shift)) & 0xFF) as u8);
        }
    } else {
        return Err(WsError::PayloadTooLong);
    }
    if let Some(mask) = frame.mask {
        out.extend_from_slice(&mask);
        for (i, b) in frame.payload.iter().enumerate() {
            out.push(b ^ mask[i % 4]);
        }
    } else {
        out.extend_from_slice(&frame.payload);
    }
    Ok(out)
}

/// Decode a single frame from a byte buffer. Returns the frame plus
/// bytes consumed.
pub fn decode_frame(buf: &[u8]) -> Result<(Frame, usize), WsError> {
    if buf.len() < 2 { return Err(WsError::UnexpectedEnd); }
    let b0 = buf[0];
    let fin = (b0 & 0x80) != 0;
    if (b0 & 0x70) != 0 { return Err(WsError::ReservedBitsSet); }
    let opcode = Opcode::from_u8(b0 & 0x0F).ok_or(WsError::InvalidOpcode(b0 & 0x0F))?;
    if opcode.is_control() && !fin { return Err(WsError::ControlFragmented); }
    let b1 = buf[1];
    let masked = (b1 & 0x80) != 0;
    let len7 = b1 & 0x7F;
    let mut pos = 2;
    let payload_len: usize = match len7 {
        0..=125 => len7 as usize,
        126 => {
            if buf.len() < pos + 2 { return Err(WsError::UnexpectedEnd); }
            let l = ((buf[pos] as usize) << 8) | (buf[pos + 1] as usize);
            pos += 2;
            l
        }
        127 => {
            if buf.len() < pos + 8 { return Err(WsError::UnexpectedEnd); }
            let mut l: u64 = 0;
            for i in 0..8 { l = (l << 8) | (buf[pos + i] as u64); }
            pos += 8;
            if l > 0x7FFF_FFFF_FFFF_FFFF { return Err(WsError::PayloadTooLong); }
            l as usize
        }
        _ => unreachable!(),
    };
    if opcode.is_control() && payload_len > 125 { return Err(WsError::ControlTooLong); }
    let mask: Option<[u8; 4]> = if masked {
        if buf.len() < pos + 4 { return Err(WsError::UnexpectedEnd); }
        let m = [buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]];
        pos += 4;
        Some(m)
    } else { None };
    if buf.len() < pos + payload_len { return Err(WsError::UnexpectedEnd); }
    let mut payload = buf[pos..pos + payload_len].to_vec();
    if let Some(m) = mask {
        for (i, b) in payload.iter_mut().enumerate() { *b ^= m[i % 4]; }
    }
    Ok((Frame { fin, opcode, payload, mask }, pos + payload_len))
}

// ─────────────────────────────────────────────────────────────────────
// Close frame helpers (RFC 6455 §5.5.1)
// ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CloseFrame {
    pub code: Option<u16>,
    pub reason: String,
}

pub fn encode_close(code: Option<u16>, reason: &str) -> Vec<u8> {
    let mut p = Vec::new();
    if let Some(c) = code {
        p.push((c >> 8) as u8);
        p.push((c & 0xFF) as u8);
        p.extend_from_slice(reason.as_bytes());
    }
    p
}

pub fn decode_close(payload: &[u8]) -> CloseFrame {
    if payload.is_empty() { return CloseFrame { code: None, reason: String::new() }; }
    if payload.len() < 2 { return CloseFrame { code: None, reason: String::new() }; }
    let code = ((payload[0] as u16) << 8) | (payload[1] as u16);
    let reason = String::from_utf8_lossy(&payload[2..]).into_owned();
    CloseFrame { code: Some(code), reason }
}

// ─────────────────────────────────────────────────────────────────────
// Handshake key derivation (RFC 6455 §4.2.2)
// ─────────────────────────────────────────────────────────────────────

const ACCEPT_MAGIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Generate a fresh client Sec-WebSocket-Key (base64 of 16 random bytes).
pub fn generate_key() -> Result<String, WsError> {
    let mut bytes = [0u8; 16];
    rusty_web_crypto::get_random_values(&mut bytes)
        .map_err(|e| WsError::Crypto(format!("RNG: {}", e)))?;
    Ok(base64_encode(&bytes))
}

/// Derive Sec-WebSocket-Accept from the client's Sec-WebSocket-Key.
/// accept = base64(SHA-1(key + ACCEPT_MAGIC))
pub fn derive_accept(client_key: &str) -> String {
    let concat = format!("{}{}", client_key, ACCEPT_MAGIC);
    let hash = rusty_web_crypto::digest_sha1(concat.as_bytes());
    base64_encode(&hash)
}

/// Verify that a server's Sec-WebSocket-Accept matches what we expect
/// for our client key.
pub fn verify_accept(client_key: &str, server_accept: &str) -> bool {
    derive_accept(client_key) == server_accept
}

// ─────────────────────────────────────────────────────────────────────
// Base64 encode (RFC 4648 §4)
// ─────────────────────────────────────────────────────────────────────

fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= input.len() {
        let a = input[i] as u32;
        let b = input[i + 1] as u32;
        let c = input[i + 2] as u32;
        let v = (a << 16) | (b << 8) | c;
        out.push(ALPHABET[((v >> 18) & 0x3F) as usize] as char);
        out.push(ALPHABET[((v >> 12) & 0x3F) as usize] as char);
        out.push(ALPHABET[((v >> 6) & 0x3F) as usize] as char);
        out.push(ALPHABET[(v & 0x3F) as usize] as char);
        i += 3;
    }
    let rem = input.len() - i;
    if rem == 1 {
        let v = (input[i] as u32) << 16;
        out.push(ALPHABET[((v >> 18) & 0x3F) as usize] as char);
        out.push(ALPHABET[((v >> 12) & 0x3F) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let v = ((input[i] as u32) << 16) | ((input[i + 1] as u32) << 8);
        out.push(ALPHABET[((v >> 18) & 0x3F) as usize] as char);
        out.push(ALPHABET[((v >> 12) & 0x3F) as usize] as char);
        out.push(ALPHABET[((v >> 6) & 0x3F) as usize] as char);
        out.push('=');
    }
    out
}
