// rusty-compression pilot — hand-rolled DEFLATE + gzip decoder.
//
// Derived from RFC 1951 (DEFLATE) + RFC 1952 (gzip file format). Decode-only
// this round; encode deferred to a subsequent Π1.3.b round.
//
// Forward direction: this is the substrate-introduction round of Π1.3 per
// seed §III.A8.13 (substrate-amortization staging). The Huffman + LZ77
// decoder primitives derived here become the substrate for any future
// compression-related work (encode, brotli, zstd) by composition.
//
// Backward direction (Pin-Art per Doc 707): the implementation surfaces
// invariants about real-world gzip streams: (a) almost all gzipped HTTP
// responses use dynamic Huffman blocks, (b) CRC32 verification is
// load-bearing for corruption-detection; (c) the LZ77 32KB sliding-window
// bound is the load-bearing space constraint.
//
// Tier-3 implementation-contingent divergence per seed C1: this is a
// hand-rolled decoder, where Bun uses libdeflate (the fastest production
// DEFLATE library). Tier-1 (RFC 1951 + 1952 conformance) and Tier-2
// (consumer API shape) both hold; only internal performance diverges.

#[derive(Debug)]
pub enum DecodeError {
    UnexpectedEnd,
    InvalidBlockType,
    InvalidStoredLen,
    InvalidHuffmanCode,
    InvalidLengthCode,
    InvalidDistanceCode,
    DistanceTooFar,
    InvalidGzipMagic,
    UnsupportedGzipMethod,
    GzipReservedFlags,
    GzipCrcMismatch,
    GzipSizeMismatch,
    OutputTooLarge,
    Brotli(String),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::UnexpectedEnd => write!(f, "unexpected end of input"),
            DecodeError::InvalidBlockType => write!(f, "invalid DEFLATE block type"),
            DecodeError::InvalidStoredLen => write!(f, "stored block: LEN/NLEN mismatch"),
            DecodeError::InvalidHuffmanCode => write!(f, "invalid Huffman code"),
            DecodeError::InvalidLengthCode => write!(f, "invalid length code"),
            DecodeError::InvalidDistanceCode => write!(f, "invalid distance code"),
            DecodeError::DistanceTooFar => write!(f, "back-reference distance exceeds output"),
            DecodeError::InvalidGzipMagic => write!(f, "invalid gzip magic bytes"),
            DecodeError::UnsupportedGzipMethod => write!(f, "unsupported gzip compression method (only deflate=8)"),
            DecodeError::GzipReservedFlags => write!(f, "gzip reserved flags set"),
            DecodeError::GzipCrcMismatch => write!(f, "gzip CRC32 mismatch"),
            DecodeError::GzipSizeMismatch => write!(f, "gzip ISIZE mismatch"),
            DecodeError::OutputTooLarge => write!(f, "decoded output exceeds maximum size"),
            DecodeError::Brotli(s) => write!(f, "brotli: {}", s),
        }
    }
}

impl std::error::Error for DecodeError {}

const MAX_OUTPUT: usize = 256 * 1024 * 1024; // 256 MiB defensive cap

// ─────────────────────────────────────────────────────────────────────────
// Bit reader (LSB-first per RFC 1951 §3.1.1)
// ─────────────────────────────────────────────────────────────────────────

struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u32, // 0..=7, bit within the current byte (LSB-first)
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, byte_pos: 0, bit_pos: 0 }
    }

    fn read_bits(&mut self, n: u32) -> Result<u32, DecodeError> {
        let mut value: u32 = 0;
        for i in 0..n {
            if self.byte_pos >= self.data.len() {
                return Err(DecodeError::UnexpectedEnd);
            }
            let bit = (self.data[self.byte_pos] >> self.bit_pos) & 1;
            value |= (bit as u32) << i;
            self.bit_pos += 1;
            if self.bit_pos == 8 {
                self.bit_pos = 0;
                self.byte_pos += 1;
            }
        }
        Ok(value)
    }

    fn align_to_byte(&mut self) {
        if self.bit_pos != 0 {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }
    }

    fn read_aligned_u16_le(&mut self) -> Result<u16, DecodeError> {
        if self.byte_pos + 2 > self.data.len() {
            return Err(DecodeError::UnexpectedEnd);
        }
        let lo = self.data[self.byte_pos] as u16;
        let hi = self.data[self.byte_pos + 1] as u16;
        self.byte_pos += 2;
        Ok(lo | (hi << 8))
    }

    fn read_aligned_bytes(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        if self.byte_pos + n > self.data.len() {
            return Err(DecodeError::UnexpectedEnd);
        }
        let r = &self.data[self.byte_pos..self.byte_pos + n];
        self.byte_pos += n;
        Ok(r)
    }

    fn position(&self) -> usize {
        self.byte_pos
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Canonical Huffman decoder (RFC 1951 §3.2.2)
// ─────────────────────────────────────────────────────────────────────────

struct HuffmanTable {
    // For each code length L (1..=15), the list of symbols with that length,
    // ordered by symbol value (ascending). This is the canonical form.
    // We decode by reading bits MSB-first within the code value, accumulating
    // bit-by-bit and checking against the canonical numeric base for each L.
    counts: [u16; 16],   // counts[L] = number of symbols with code length L
    symbols: Vec<u16>,   // symbols in canonical order (length-major, ascending)
}

impl HuffmanTable {
    fn from_lengths(lengths: &[u8]) -> Result<Self, DecodeError> {
        let mut counts = [0u16; 16];
        for &l in lengths {
            if l as usize >= 16 { return Err(DecodeError::InvalidHuffmanCode); }
            counts[l as usize] += 1;
        }
        counts[0] = 0; // 0-length symbols don't participate
        // Symbols sorted by (length, symbol).
        let mut offsets = [0u16; 17];
        for i in 1..16 {
            offsets[i + 1] = offsets[i] + counts[i];
        }
        let total: usize = (1..16).map(|i| counts[i] as usize).sum();
        let mut symbols = vec![0u16; total];
        let mut next = offsets;
        for (sym, &l) in lengths.iter().enumerate() {
            if l != 0 {
                symbols[next[l as usize] as usize] = sym as u16;
                next[l as usize] += 1;
            }
        }
        Ok(HuffmanTable { counts, symbols })
    }

    /// Decode a single symbol from the bit reader. Reads MSB-first within
    /// the Huffman code itself, but the bit reader provides bits LSB-first
    /// in the stream — these are different conventions. RFC 1951 §3.1.1
    /// reads stream bits LSB-first; canonical Huffman codes are MSB-first.
    fn decode(&self, br: &mut BitReader) -> Result<u16, DecodeError> {
        let mut code: u32 = 0;
        let mut first: u32 = 0;   // first canonical code at this length
        let mut index: u32 = 0;   // index into self.symbols for this length
        for l in 1..16u32 {
            code = (code << 1) | br.read_bits(1)?;
            let count = self.counts[l as usize] as u32;
            if code < first + count {
                let sym_idx = index + (code - first);
                return Ok(self.symbols[sym_idx as usize]);
            }
            index += count;
            first = (first + count) << 1;
        }
        Err(DecodeError::InvalidHuffmanCode)
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Fixed Huffman tables (RFC 1951 §3.2.6)
// ─────────────────────────────────────────────────────────────────────────

fn fixed_literal_lengths() -> [u8; 288] {
    let mut l = [0u8; 288];
    // 0..=143 → 8 bits; 144..=255 → 9 bits; 256..=279 → 7 bits; 280..=287 → 8 bits.
    for i in 0..=143 { l[i] = 8; }
    for i in 144..=255 { l[i] = 9; }
    for i in 256..=279 { l[i] = 7; }
    for i in 280..=287 { l[i] = 8; }
    l
}

fn fixed_distance_lengths() -> [u8; 30] {
    [5u8; 30]
}

// ─────────────────────────────────────────────────────────────────────────
// Length & distance code tables (RFC 1951 §3.2.5)
// ─────────────────────────────────────────────────────────────────────────

const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31,
    35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258,
];
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2,
    3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DISTANCE_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193,
    257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193,
    12289, 16385, 24577,
];
const DISTANCE_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6,
    7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13,
];

// Code-length code ordering (RFC 1951 §3.2.7)
const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

// ─────────────────────────────────────────────────────────────────────────
// Dynamic-Huffman block header parsing (RFC 1951 §3.2.7)
// ─────────────────────────────────────────────────────────────────────────

fn read_dynamic_tables(br: &mut BitReader)
    -> Result<(HuffmanTable, HuffmanTable), DecodeError>
{
    let hlit = br.read_bits(5)? as usize + 257;
    let hdist = br.read_bits(5)? as usize + 1;
    let hclen = br.read_bits(4)? as usize + 4;

    let mut code_lengths = [0u8; 19];
    for i in 0..hclen {
        code_lengths[CODE_LENGTH_ORDER[i]] = br.read_bits(3)? as u8;
    }
    let cl_table = HuffmanTable::from_lengths(&code_lengths)?;

    let mut combined = vec![0u8; hlit + hdist];
    let mut i = 0;
    while i < combined.len() {
        let sym = cl_table.decode(br)?;
        match sym {
            0..=15 => { combined[i] = sym as u8; i += 1; }
            16 => {
                if i == 0 { return Err(DecodeError::InvalidHuffmanCode); }
                let prev = combined[i - 1];
                let repeat = br.read_bits(2)? as usize + 3;
                for _ in 0..repeat {
                    if i >= combined.len() { return Err(DecodeError::InvalidHuffmanCode); }
                    combined[i] = prev; i += 1;
                }
            }
            17 => {
                let repeat = br.read_bits(3)? as usize + 3;
                for _ in 0..repeat {
                    if i >= combined.len() { return Err(DecodeError::InvalidHuffmanCode); }
                    combined[i] = 0; i += 1;
                }
            }
            18 => {
                let repeat = br.read_bits(7)? as usize + 11;
                for _ in 0..repeat {
                    if i >= combined.len() { return Err(DecodeError::InvalidHuffmanCode); }
                    combined[i] = 0; i += 1;
                }
            }
            _ => return Err(DecodeError::InvalidHuffmanCode),
        }
    }

    let lit_table = HuffmanTable::from_lengths(&combined[..hlit])?;
    let dist_table = HuffmanTable::from_lengths(&combined[hlit..])?;
    Ok((lit_table, dist_table))
}

// ─────────────────────────────────────────────────────────────────────────
// Block decompression (RFC 1951 §3.2.3)
// ─────────────────────────────────────────────────────────────────────────

fn decompress_block(
    br: &mut BitReader,
    out: &mut Vec<u8>,
    lit: &HuffmanTable,
    dist: &HuffmanTable,
) -> Result<(), DecodeError> {
    loop {
        if out.len() > MAX_OUTPUT { return Err(DecodeError::OutputTooLarge); }
        let sym = lit.decode(br)?;
        if sym < 256 {
            out.push(sym as u8);
        } else if sym == 256 {
            return Ok(());
        } else {
            let code = (sym - 257) as usize;
            if code >= 29 { return Err(DecodeError::InvalidLengthCode); }
            let length = LENGTH_BASE[code] as usize
                + br.read_bits(LENGTH_EXTRA[code] as u32)? as usize;
            let dist_sym = dist.decode(br)? as usize;
            if dist_sym >= 30 { return Err(DecodeError::InvalidDistanceCode); }
            let distance = DISTANCE_BASE[dist_sym] as usize
                + br.read_bits(DISTANCE_EXTRA[dist_sym] as u32)? as usize;
            if distance > out.len() { return Err(DecodeError::DistanceTooFar); }
            let start = out.len() - distance;
            for i in 0..length {
                let b = out[start + i];
                out.push(b);
                if out.len() > MAX_OUTPUT { return Err(DecodeError::OutputTooLarge); }
            }
        }
    }
}

/// Decompress a raw DEFLATE stream (no gzip/zlib wrapper).
pub fn inflate(data: &[u8]) -> Result<Vec<u8>, DecodeError> {
    let mut br = BitReader::new(data);
    let mut out = Vec::new();
    loop {
        let bfinal = br.read_bits(1)?;
        let btype = br.read_bits(2)?;
        match btype {
            0 => {
                br.align_to_byte();
                let len = br.read_aligned_u16_le()?;
                let nlen = br.read_aligned_u16_le()?;
                if len ^ nlen != 0xFFFF { return Err(DecodeError::InvalidStoredLen); }
                let bytes = br.read_aligned_bytes(len as usize)?;
                out.extend_from_slice(bytes);
                if out.len() > MAX_OUTPUT { return Err(DecodeError::OutputTooLarge); }
            }
            1 => {
                let lit = HuffmanTable::from_lengths(&fixed_literal_lengths())?;
                let dist = HuffmanTable::from_lengths(&fixed_distance_lengths())?;
                decompress_block(&mut br, &mut out, &lit, &dist)?;
            }
            2 => {
                let (lit, dist) = read_dynamic_tables(&mut br)?;
                decompress_block(&mut br, &mut out, &lit, &dist)?;
            }
            _ => return Err(DecodeError::InvalidBlockType),
        }
        if bfinal != 0 { break; }
    }
    Ok(out)
}

// ─────────────────────────────────────────────────────────────────────────
// CRC32 (RFC 1952 §8 / IEEE 802.3)
// ─────────────────────────────────────────────────────────────────────────

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for n in 0..256u32 {
        let mut c = n;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB88320 ^ (c >> 1) } else { c >> 1 };
        }
        table[n as usize] = c;
    }
    let mut c = 0xFFFFFFFFu32;
    for &b in data {
        c = table[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFFFFFF
}

// ─────────────────────────────────────────────────────────────────────────
// gzip framing (RFC 1952 §2.3)
// ─────────────────────────────────────────────────────────────────────────

/// Decompress a gzip-wrapped DEFLATE stream and verify CRC32 + ISIZE.
pub fn gunzip(data: &[u8]) -> Result<Vec<u8>, DecodeError> {
    if data.len() < 18 { return Err(DecodeError::UnexpectedEnd); }
    if data[0] != 0x1f || data[1] != 0x8b { return Err(DecodeError::InvalidGzipMagic); }
    if data[2] != 8 { return Err(DecodeError::UnsupportedGzipMethod); }
    let flg = data[3];
    if flg & 0xE0 != 0 { return Err(DecodeError::GzipReservedFlags); }
    // 4..8: MTIME (ignored). 8: XFL (ignored). 9: OS (ignored).
    let mut p: usize = 10;
    // FEXTRA bit 2
    if flg & 0x04 != 0 {
        if p + 2 > data.len() { return Err(DecodeError::UnexpectedEnd); }
        let xlen = (data[p] as usize) | ((data[p + 1] as usize) << 8);
        p += 2 + xlen;
        if p > data.len() { return Err(DecodeError::UnexpectedEnd); }
    }
    // FNAME bit 3 — zero-terminated string
    if flg & 0x08 != 0 {
        while p < data.len() && data[p] != 0 { p += 1; }
        if p >= data.len() { return Err(DecodeError::UnexpectedEnd); }
        p += 1;
    }
    // FCOMMENT bit 4 — zero-terminated
    if flg & 0x10 != 0 {
        while p < data.len() && data[p] != 0 { p += 1; }
        if p >= data.len() { return Err(DecodeError::UnexpectedEnd); }
        p += 1;
    }
    // FHCRC bit 1 — 2-byte header CRC (we skip verification)
    if flg & 0x02 != 0 {
        if p + 2 > data.len() { return Err(DecodeError::UnexpectedEnd); }
        p += 2;
    }
    // The trailer is the last 8 bytes; DEFLATE payload sits between p and len-8.
    if data.len() < p + 8 { return Err(DecodeError::UnexpectedEnd); }
    let payload = &data[p..data.len() - 8];
    let trailer = &data[data.len() - 8..];

    let out = inflate(payload)?;

    let crc_expected = (trailer[0] as u32)
        | ((trailer[1] as u32) << 8)
        | ((trailer[2] as u32) << 16)
        | ((trailer[3] as u32) << 24);
    let isize_expected = (trailer[4] as u32)
        | ((trailer[5] as u32) << 8)
        | ((trailer[6] as u32) << 16)
        | ((trailer[7] as u32) << 24);
    if crc32(&out) != crc_expected { return Err(DecodeError::GzipCrcMismatch); }
    if (out.len() as u32) != isize_expected { return Err(DecodeError::GzipSizeMismatch); }
    Ok(out)
}

// ─────────────────────────────────────────────────────────────────────────
// zlib framing (RFC 1950) — for Content-Encoding: deflate
// ─────────────────────────────────────────────────────────────────────────
//
// Content-Encoding: deflate in HTTP is, despite the name, zlib-wrapped
// DEFLATE per RFC 1950 (most user-agents send/expect this form, not raw
// DEFLATE). Header is 2 bytes (CMF + FLG); trailer is 4-byte Adler-32.

fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &x in data {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// Decompress a zlib-wrapped DEFLATE stream (Content-Encoding: deflate).
pub fn zlib_inflate(data: &[u8]) -> Result<Vec<u8>, DecodeError> {
    if data.len() < 6 { return Err(DecodeError::UnexpectedEnd); }
    let cmf = data[0];
    let flg = data[1];
    if (cmf & 0x0F) != 8 { return Err(DecodeError::UnsupportedGzipMethod); }
    if ((cmf as u16) << 8 | flg as u16) % 31 != 0 {
        return Err(DecodeError::InvalidHuffmanCode);
    }
    if flg & 0x20 != 0 {
        // Preset dictionary not supported.
        return Err(DecodeError::GzipReservedFlags);
    }
    let payload = &data[2..data.len() - 4];
    let trailer = &data[data.len() - 4..];

    let out = inflate(payload)?;

    let adler_expected = ((trailer[0] as u32) << 24)
        | ((trailer[1] as u32) << 16)
        | ((trailer[2] as u32) << 8)
        | (trailer[3] as u32);
    if adler32(&out) != adler_expected {
        return Err(DecodeError::GzipCrcMismatch);
    }
    Ok(out)
}

/// HTTP Content-Encoding: deflate is ambiguous in practice — some servers
/// send zlib-wrapped (RFC 1950), some send raw DEFLATE (RFC 1951). Try
/// zlib first; on header-mismatch, fall back to raw.
pub fn http_deflate_inflate(data: &[u8]) -> Result<Vec<u8>, DecodeError> {
    match zlib_inflate(data) {
        Ok(v) => Ok(v),
        Err(_) => inflate(data),
    }
}

// ──────────────────── Stored-block encoders (Π1.3.b) ────────────────────
//
// Real DEFLATE encoders use LZ77 + dynamic Huffman, which is a significant
// implementation lift. For wire-format parity with Bun.deflateSync /
// Bun.gzipSync we only need to produce *valid* output that any conforming
// decoder accepts — including our own inflate(), node's zlib, and Bun's
// libdeflate. RFC 1951 §3.2.4 defines a "stored" block type: literal
// uncompressed bytes with a 5-byte header per block. Each stored block
// can carry up to 65535 bytes (0xFFFF); larger inputs split across blocks.
//
// This trades compression ratio (1.0) for full format compatibility with
// any consumer that decodes deflate/gzip/zlib. The LZ77 path is tracked
// for follow-on once a real-world bottleneck demands it.

const STORED_MAX: usize = 0xFFFF;

/// Encode `data` as a sequence of DEFLATE stored blocks (RFC 1951 §3.2.4).
pub fn deflate_stored(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + 5 * (data.len() / STORED_MAX + 1));
    if data.is_empty() {
        // Empty input: one final stored block of length 0.
        out.push(0x01); // BFINAL=1, BTYPE=00
        out.extend_from_slice(&[0x00, 0x00, 0xFF, 0xFF]);
        return out;
    }
    let mut i = 0;
    while i < data.len() {
        let chunk_len = std::cmp::min(STORED_MAX, data.len() - i);
        let is_final = i + chunk_len == data.len();
        // Stored-block header is byte-aligned. First byte: BFINAL(1) | BTYPE(00).
        out.push(if is_final { 0x01 } else { 0x00 });
        let len = chunk_len as u16;
        out.push((len & 0xFF) as u8);
        out.push((len >> 8) as u8);
        out.push((!len & 0xFF) as u8);
        out.push((!len >> 8) as u8);
        out.extend_from_slice(&data[i..i + chunk_len]);
        i += chunk_len;
    }
    out
}

/// zlib wrapper (RFC 1950) around a stored-block deflate stream.
pub fn zlib_deflate_stored(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + 11);
    // CMF: CM=8 (deflate), CINFO=7 (32K window).
    out.push(0x78);
    // FLG: FLEVEL=0 (fastest), no FDICT. FCHECK chosen so that
    // (CMF*256 + FLG) % 31 == 0. 0x78 * 256 = 30720; (30720 + FLG) % 31 == 0.
    // FLG with FLEVEL=0 and FDICT=0: FCHECK = 31 - (30720 % 31) = 31 - 30 = 1.
    out.push(0x01);
    out.extend_from_slice(&deflate_stored(data));
    let a = adler32(data);
    out.push((a >> 24) as u8);
    out.push((a >> 16) as u8);
    out.push((a >>  8) as u8);
    out.push((a >>  0) as u8);
    out
}

/// gzip wrapper (RFC 1952) around a stored-block deflate stream.
pub fn gzip_deflate_stored(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + 18);
    out.extend_from_slice(&[
        0x1f, 0x8b,       // ID1, ID2
        0x08,             // CM = deflate
        0x00,             // FLG: no fields
        0x00, 0x00, 0x00, 0x00, // MTIME = 0
        0x00,             // XFL
        0xff,             // OS = unknown
    ]);
    out.extend_from_slice(&deflate_stored(data));
    let c = crc32(data);
    out.push((c >>  0) as u8);
    out.push((c >>  8) as u8);
    out.push((c >> 16) as u8);
    out.push((c >> 24) as u8);
    let isize_le = (data.len() as u32) & 0xFFFFFFFF;
    out.push((isize_le >>  0) as u8);
    out.push((isize_le >>  8) as u8);
    out.push((isize_le >> 16) as u8);
    out.push((isize_le >> 24) as u8);
    out
}

/// Brotli decode per RFC 7932. Borrowed substrate via brotli-decompressor
/// crate; same policy as rusty-tls borrowing std::net::TcpStream. The
/// algorithm IS canonical (RFC 7932); re-deriving it would add no
/// apparatus value and would require ~1500 LOC + 122KB static dictionary.
pub fn brotli_decode(data: &[u8]) -> Result<Vec<u8>, DecodeError> {
    use std::io::Read;
    let mut decoder = brotli_decompressor::Decompressor::new(data, 4096);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)
        .map_err(|e| DecodeError::Brotli(format!("{}", e)))?;
    Ok(out)
}

#[cfg(test)]
mod brotli_tests {
    use super::*;

    #[test]
    fn brotli_decode_empty() {
        // RFC 7932 §10: shortest valid brotli stream — single empty
        // ISLAST meta-block. WBITS=0(b1)=16, ISLAST=1, ISLASTEMPTY=1,
        // padded with zero bits. Encoded: 0x06 0x00 (2 bytes).
        let empty_stream = [0x06];
        let r = brotli_decode(&empty_stream);
        assert!(r.is_ok(), "brotli empty: {:?}", r);
        assert_eq!(r.unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn brotli_decode_hello_roundtrip() {
        // Encoded "Hello, World!" via Python brotli.compress (default level).
        // Bytes hand-verified against `python3 -c "import brotli; print(brotli.compress(b'Hello, World!').hex())"`.
        let encoded = [
            0x0b, 0x06, 0x80, 0x48, 0x65, 0x6c, 0x6c, 0x6f,
            0x2c, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21, 0x03,
        ];
        let r = brotli_decode(&encoded).expect("brotli decode");
        assert_eq!(r, b"Hello, World!");
    }
}
