// rusty-asn1-der pilot — Distinguished Encoding Rules (DER) reader.
//
// Π1.4.a substrate-introduction round of the TLS substrate-amortization
// sequence (seed §A8.13). DER is the strict deterministic subset of BER
// (ITU-T X.690) used by X.509 (RFC 5280), PKCS family (RFC 8017 / 5208 /
// 5915 / 7292), and SubjectPublicKeyInfo encodings the corpus's crypto
// pilots already consume in raw form.
//
// Forward direction (Pin-Art): this is the substrate for Π1.4.b
// (X.509 certificate parsing) and downstream Π1.4.c-e (TLS record +
// handshake). Forward only; encoding (DER writer) deferred until a
// consumer surfaces.
//
// Backward direction: the implementation surfaces specific X.509
// architectural invariants: SEQUENCE is the dominant container (cert
// structure, signature algorithm identifier, validity, issuer, subject,
// SubjectPublicKeyInfo, extensions). OID encoding is variable-length
// base-128 per X.690 §8.19. Context-specific tags carry version,
// extensions, and optional fields.
//
// Scope: read-only, no encoder. Long-form length is supported up to
// 4-byte length (sufficient for any practical certificate). The reader
// is zero-copy where possible: typed values return slices into the
// original buffer.

#[derive(Debug, Clone, PartialEq)]
pub enum DerError {
    UnexpectedEnd,
    InvalidLength,
    UnknownTag(u8),
    WrongTag { expected: u8, actual: u8 },
    NotConstructed,
    NotPrimitive,
    InvalidOid,
    InvalidInteger,
    TrailingData,
}

impl std::fmt::Display for DerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DerError::UnexpectedEnd => write!(f, "unexpected end of DER input"),
            DerError::InvalidLength => write!(f, "invalid DER length encoding"),
            DerError::UnknownTag(t) => write!(f, "unknown DER tag 0x{:02x}", t),
            DerError::WrongTag { expected, actual } => write!(f,
                "DER tag mismatch: expected 0x{:02x}, got 0x{:02x}", expected, actual),
            DerError::NotConstructed => write!(f, "DER value is not constructed (no inner)"),
            DerError::NotPrimitive => write!(f, "DER value is primitive (no constructed inner)"),
            DerError::InvalidOid => write!(f, "invalid OID encoding"),
            DerError::InvalidInteger => write!(f, "invalid INTEGER encoding"),
            DerError::TrailingData => write!(f, "unexpected trailing data after DER value"),
        }
    }
}

impl std::error::Error for DerError {}

// ─────────────────────────────────────────────────────────────────────
// Universal tags (X.690 §8.4)
// ─────────────────────────────────────────────────────────────────────
pub const TAG_BOOLEAN: u8 = 0x01;
pub const TAG_INTEGER: u8 = 0x02;
pub const TAG_BIT_STRING: u8 = 0x03;
pub const TAG_OCTET_STRING: u8 = 0x04;
pub const TAG_NULL: u8 = 0x05;
pub const TAG_OID: u8 = 0x06;
pub const TAG_UTF8_STRING: u8 = 0x0C;
pub const TAG_SEQUENCE: u8 = 0x30;  // 0x10 | constructed bit (0x20)
pub const TAG_SET: u8 = 0x31;       // 0x11 | constructed bit
pub const TAG_PRINTABLE_STRING: u8 = 0x13;
pub const TAG_TELETEX_STRING: u8 = 0x14;
pub const TAG_IA5_STRING: u8 = 0x16;
pub const TAG_UTC_TIME: u8 = 0x17;
pub const TAG_GENERALIZED_TIME: u8 = 0x18;

/// A parsed DER TLV. The slice points into the original input buffer.
#[derive(Debug, Clone)]
pub struct DerValue<'a> {
    pub tag: u8,
    pub content: &'a [u8],
}

impl<'a> DerValue<'a> {
    pub fn is_constructed(&self) -> bool { (self.tag & 0x20) != 0 }
    pub fn is_context_specific(&self) -> bool { (self.tag & 0xC0) == 0x80 }
    pub fn context_tag_number(&self) -> u8 { self.tag & 0x1F }
}

// ─────────────────────────────────────────────────────────────────────
// Reader
// ─────────────────────────────────────────────────────────────────────
pub struct DerReader<'a> {
    buf: &'a [u8],
}

impl<'a> DerReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        DerReader { buf }
    }

    pub fn is_empty(&self) -> bool { self.buf.is_empty() }
    pub fn remaining(&self) -> &'a [u8] { self.buf }

    /// Read one TLV. Advances the reader past the value.
    pub fn read_tlv(&mut self) -> Result<DerValue<'a>, DerError> {
        if self.buf.is_empty() { return Err(DerError::UnexpectedEnd); }
        let tag = self.buf[0];
        let (length, header_len) = parse_length(&self.buf[1..])?;
        let total = 1 + header_len + length;
        if total > self.buf.len() { return Err(DerError::UnexpectedEnd); }
        let content = &self.buf[1 + header_len .. 1 + header_len + length];
        self.buf = &self.buf[total..];
        Ok(DerValue { tag, content })
    }

    /// Read a TLV and assert its tag matches.
    pub fn read_tag(&mut self, expected: u8) -> Result<DerValue<'a>, DerError> {
        let v = self.read_tlv()?;
        if v.tag != expected {
            return Err(DerError::WrongTag { expected, actual: v.tag });
        }
        Ok(v)
    }

    /// Peek at the next tag without consuming.
    pub fn peek_tag(&self) -> Option<u8> {
        self.buf.first().copied()
    }
}

/// Parse a DER length field. Returns (length, bytes_consumed).
fn parse_length(buf: &[u8]) -> Result<(usize, usize), DerError> {
    if buf.is_empty() { return Err(DerError::UnexpectedEnd); }
    let first = buf[0];
    if first < 0x80 {
        // Short form: length 0..=127 in a single byte.
        Ok((first as usize, 1))
    } else {
        // Long form: low 7 bits = number of length-bytes.
        let n = (first & 0x7F) as usize;
        if n == 0 || n > 4 {
            // n == 0 is BER indefinite (forbidden in DER).
            // n > 4 is impractical for any real certificate.
            return Err(DerError::InvalidLength);
        }
        if buf.len() < 1 + n { return Err(DerError::UnexpectedEnd); }
        let mut length: usize = 0;
        for i in 0..n {
            length = (length << 8) | (buf[1 + i] as usize);
        }
        // DER requires minimal length encoding: if n=1 and length < 128,
        // short form should have been used. Reject as ill-formed.
        if n == 1 && length < 128 { return Err(DerError::InvalidLength); }
        Ok((length, 1 + n))
    }
}

// ─────────────────────────────────────────────────────────────────────
// Typed accessors
// ─────────────────────────────────────────────────────────────────────

impl<'a> DerValue<'a> {
    /// Treat content as a SEQUENCE (or SET) and return a reader over inner.
    pub fn into_reader(self) -> Result<DerReader<'a>, DerError> {
        if !self.is_constructed() { return Err(DerError::NotConstructed); }
        Ok(DerReader::new(self.content))
    }

    /// Return content as a slice (for OCTET STRING, raw bytes, etc.).
    pub fn as_bytes(&self) -> &'a [u8] { self.content }

    /// Parse content as INTEGER. Returns the raw two's-complement bytes.
    /// Caller can convert to i64 / u64 / BigInt as needed.
    pub fn as_integer_bytes(&self) -> Result<&'a [u8], DerError> {
        if self.tag != TAG_INTEGER { return Err(DerError::WrongTag {
            expected: TAG_INTEGER, actual: self.tag }); }
        if self.content.is_empty() { return Err(DerError::InvalidInteger); }
        // DER requires minimal encoding (X.690 §8.3.2).
        if self.content.len() > 1 {
            let b0 = self.content[0];
            let b1 = self.content[1];
            if (b0 == 0x00 && (b1 & 0x80) == 0) ||
               (b0 == 0xFF && (b1 & 0x80) != 0) {
                return Err(DerError::InvalidInteger);
            }
        }
        Ok(self.content)
    }

    /// Parse content as a non-negative INTEGER, returning the unsigned
    /// magnitude bytes (stripping the leading 0x00 padding if any).
    pub fn as_unsigned_integer(&self) -> Result<&'a [u8], DerError> {
        let bytes = self.as_integer_bytes()?;
        // Sign bit set → negative; reject for unsigned context.
        if !bytes.is_empty() && (bytes[0] & 0x80) != 0 {
            return Err(DerError::InvalidInteger);
        }
        // Strip leading 0x00 inserted to keep value non-negative.
        if bytes.len() > 1 && bytes[0] == 0x00 {
            Ok(&bytes[1..])
        } else {
            Ok(bytes)
        }
    }

    /// Parse content as a small INTEGER. Returns i64 if it fits.
    pub fn as_i64(&self) -> Result<i64, DerError> {
        let bytes = self.as_integer_bytes()?;
        if bytes.len() > 8 { return Err(DerError::InvalidInteger); }
        let mut v: i64 = if (bytes[0] & 0x80) != 0 { -1 } else { 0 };
        for &b in bytes { v = (v << 8) | (b as i64 & 0xff); }
        Ok(v)
    }

    /// Parse content as BIT STRING. Returns (unused_bits, bytes).
    /// The first content byte is the count of unused bits in the final
    /// byte; the rest are the bit-string content (typically a DER blob
    /// for SubjectPublicKey / Signature wrappers).
    pub fn as_bit_string(&self) -> Result<(u8, &'a [u8]), DerError> {
        if self.tag != TAG_BIT_STRING { return Err(DerError::WrongTag {
            expected: TAG_BIT_STRING, actual: self.tag }); }
        if self.content.is_empty() { return Err(DerError::InvalidLength); }
        let unused = self.content[0];
        if unused > 7 { return Err(DerError::InvalidLength); }
        Ok((unused, &self.content[1..]))
    }

    /// Parse content as OBJECT IDENTIFIER. Returns the decoded arc list.
    /// X.690 §8.19: first two arcs encoded as 40 * first + second.
    pub fn as_oid(&self) -> Result<Vec<u64>, DerError> {
        if self.tag != TAG_OID { return Err(DerError::WrongTag {
            expected: TAG_OID, actual: self.tag }); }
        if self.content.is_empty() { return Err(DerError::InvalidOid); }
        let mut out = Vec::new();
        let first_byte = self.content[0];
        out.push((first_byte / 40) as u64);
        out.push((first_byte % 40) as u64);
        let mut value: u64 = 0;
        for &b in &self.content[1..] {
            // Reject value-too-large that would overflow u64.
            if value > (u64::MAX >> 7) { return Err(DerError::InvalidOid); }
            value = (value << 7) | ((b & 0x7F) as u64);
            if (b & 0x80) == 0 {
                out.push(value);
                value = 0;
            }
        }
        if value != 0 {
            // Last byte must have MSB cleared.
            return Err(DerError::InvalidOid);
        }
        Ok(out)
    }

    /// Parse content as a string (UTF8String, PrintableString, IA5String,
    /// or TeletexString). The byte slice is returned without re-encoding;
    /// caller can validate UTF-8 explicitly if needed.
    pub fn as_string(&self) -> Result<&'a str, DerError> {
        match self.tag {
            TAG_UTF8_STRING | TAG_PRINTABLE_STRING | TAG_IA5_STRING
            | TAG_TELETEX_STRING => {
                std::str::from_utf8(self.content)
                    .map_err(|_| DerError::InvalidLength)
            }
            _ => Err(DerError::WrongTag {
                expected: TAG_UTF8_STRING, actual: self.tag,
            }),
        }
    }

    /// Parse UTCTime (YYMMDDHHMMSSZ or YYMMDDHHMMZ). Returns the raw bytes.
    /// Caller composes the year-decode using the 50-year window per
    /// RFC 5280 §4.1.2.5.1.
    pub fn as_utc_time(&self) -> Result<&'a [u8], DerError> {
        if self.tag != TAG_UTC_TIME { return Err(DerError::WrongTag {
            expected: TAG_UTC_TIME, actual: self.tag }); }
        Ok(self.content)
    }

    /// Parse GeneralizedTime (YYYYMMDDHHMMSSZ). Returns the raw bytes.
    pub fn as_generalized_time(&self) -> Result<&'a [u8], DerError> {
        if self.tag != TAG_GENERALIZED_TIME { return Err(DerError::WrongTag {
            expected: TAG_GENERALIZED_TIME, actual: self.tag }); }
        Ok(self.content)
    }

    /// Parse a BOOLEAN. DER requires content == [0x00] (FALSE) or [0xFF] (TRUE).
    pub fn as_bool(&self) -> Result<bool, DerError> {
        if self.tag != TAG_BOOLEAN { return Err(DerError::WrongTag {
            expected: TAG_BOOLEAN, actual: self.tag }); }
        if self.content.len() != 1 { return Err(DerError::InvalidLength); }
        match self.content[0] {
            0x00 => Ok(false),
            0xFF => Ok(true),
            _ => Err(DerError::InvalidLength),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────

/// Format an OID as a dot-separated string (1.2.840.113549.1.1.1).
pub fn oid_to_string(arcs: &[u64]) -> String {
    let mut s = String::new();
    for (i, a) in arcs.iter().enumerate() {
        if i > 0 { s.push('.'); }
        s.push_str(&a.to_string());
    }
    s
}

/// Parse a single top-level DER value from a complete buffer.
/// Errors if the buffer contains trailing bytes after the value.
pub fn parse_single<'a>(buf: &'a [u8]) -> Result<DerValue<'a>, DerError> {
    let mut r = DerReader::new(buf);
    let v = r.read_tlv()?;
    if !r.is_empty() { return Err(DerError::TrailingData); }
    Ok(v)
}
