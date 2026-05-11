// web-crypto pilot — Web Crypto subset (SHA-256 digest, UUID v4, random,
// timing-safe equal).
//
// Inputs:
//   AUDIT — pilots/web-crypto/AUDIT.md
//   SPEC  — Web Crypto §10 (https://w3c.github.io/webcrypto/) +
//           NIST FIPS 180-4 (SHA-256) + RFC 4122 §4.4 (UUID v4)
//   CD    — runs/2026-05-10-bun-v0.13b-spec-batch/constraints/crypto.constraints.md
//
// Real cryptographic primitives implemented from scratch (no external
// crates) to maintain the apparatus' std-only pattern. Random source is
// /dev/urandom direct read on Unix (Windows deferred per AUDIT).

use std::fs::File;
use std::io::Read;

// ───────────────────────────── Random source ──────────────────────────

/// SPEC: crypto.getRandomValues(typedArray) fills the array with
/// cryptographic random bytes. Pilot uses /dev/urandom on Unix.
pub fn get_random_values(buf: &mut [u8]) -> std::io::Result<()> {
    let mut f = File::open("/dev/urandom")?;
    f.read_exact(buf)
}

// ───────────────────────────── UUID v4 ────────────────────────────────

/// SPEC: crypto.randomUUID() returns a v4 UUID per RFC 4122. Format:
/// xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx where y ∈ {8,9,a,b}.
pub fn random_uuid_v4() -> String {
    let mut bytes = [0u8; 16];
    get_random_values(&mut bytes).expect("random source");
    // RFC 4122 §4.4: set version (top nibble of byte 6) to 4; set variant
    // (top two bits of byte 8) to 10b.
    bytes[6] = (bytes[6] & 0x0F) | 0x40;
    bytes[8] = (bytes[8] & 0x3F) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    )
}

// ──────────────────────── Timing-safe equal ──────────────────────────

/// SPEC: crypto.timingSafeEqual(a, b) — compares byte arrays in constant
/// time wrt their length. Returns false immediately when lengths differ
/// (per Node spec; the constant-time guarantee applies only to equal-length
/// inputs).
pub fn timing_safe_equal(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut diff: u8 = 0;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

// ────────────────────────────── SHA-256 ──────────────────────────────
//
// FIPS 180-4 SHA-256.

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const SHA256_H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

pub fn digest_sha256(data: &[u8]) -> [u8; 32] {
    let mut h = SHA256_H0;
    let mut padded: Vec<u8> = data.to_vec();
    let bit_len = (data.len() as u64) * 8;
    padded.push(0x80);
    while padded.len() % 64 != 56 { padded.push(0); }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i*4], chunk[i*4+1], chunk[i*4+2], chunk[i*4+3]]);
        }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }

        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(SHA256_K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e; e = d.wrapping_add(t1);
            d = c; c = b; b = a; a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for i in 0..8 {
        out[i*4..i*4+4].copy_from_slice(&h[i].to_be_bytes());
    }
    out
}

/// Hex-encoded SHA-256 hash for verifier convenience.
pub fn digest_sha256_hex(data: &[u8]) -> String {
    let bytes = digest_sha256(data);
    let mut s = String::with_capacity(64);
    for b in &bytes { s.push_str(&format!("{:02x}", b)); }
    s
}

// ───────────────────────────── SHA-1 ──────────────────────────────────
// FIPS 180-4 reference implementation. SHA-1 is cryptographically broken
// for collision resistance (Shattered, 2017) but remains in scope because
// real consumer code still uses HMAC-SHA-1 (AWS SigV4 legacy, OAuth 1.0,
// some webhook signature schemes, git object identification). Pilot
// implementation here is for spec-correctness against existing usage,
// not endorsement.

const SHA1_H0: [u32; 5] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];

pub fn digest_sha1(data: &[u8]) -> [u8; 20] {
    let mut h = SHA1_H0;
    let mut padded: Vec<u8> = data.to_vec();
    let bit_len = (data.len() as u64) * 8;
    padded.push(0x80);
    while padded.len() % 64 != 56 { padded.push(0); }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i*4], chunk[i*4+1], chunk[i*4+2], chunk[i*4+3]]);
        }
        for i in 16..80 {
            w[i] = (w[i-3] ^ w[i-8] ^ w[i-14] ^ w[i-16]).rotate_left(1);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
        for i in 0..80 {
            let (f, k) = if i < 20 {
                ((b & c) | (!b & d), 0x5a827999_u32)
            } else if i < 40 {
                (b ^ c ^ d, 0x6ed9eba1_u32)
            } else if i < 60 {
                ((b & c) | (b & d) | (c & d), 0x8f1bbcdc_u32)
            } else {
                (b ^ c ^ d, 0xca62c1d6_u32)
            };
            let t = a.rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = t;
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }
    let mut out = [0u8; 20];
    for i in 0..5 {
        out[i*4..i*4+4].copy_from_slice(&h[i].to_be_bytes());
    }
    out
}

pub fn digest_sha1_hex(data: &[u8]) -> String {
    let bytes = digest_sha1(data);
    let mut s = String::with_capacity(40);
    for b in &bytes { s.push_str(&format!("{:02x}", b)); }
    s
}

/// HMAC-SHA-1 — RFC 2104 construction over SHA-1 with 64-byte block.
pub fn hmac_sha1(key: &[u8], message: &[u8]) -> [u8; 20] {
    const BLOCK: usize = 64;
    let mut key_pad = [0u8; BLOCK];
    if key.len() > BLOCK {
        let hashed = digest_sha1(key);
        key_pad[..20].copy_from_slice(&hashed);
    } else {
        key_pad[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0u8; BLOCK];
    let mut opad = [0u8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] = key_pad[i] ^ 0x36;
        opad[i] = key_pad[i] ^ 0x5C;
    }
    let mut inner_input = Vec::with_capacity(BLOCK + message.len());
    inner_input.extend_from_slice(&ipad);
    inner_input.extend_from_slice(message);
    let inner = digest_sha1(&inner_input);
    let mut outer_input = Vec::with_capacity(BLOCK + 20);
    outer_input.extend_from_slice(&opad);
    outer_input.extend_from_slice(&inner);
    digest_sha1(&outer_input)
}

/// HMAC-SHA-256(K, M). Standard RFC 2104 construction:
///   inner = SHA-256(K' XOR 0x36 || M)
///   tag   = SHA-256(K' XOR 0x5C || inner)
/// where K' = K padded to 64 bytes (block size), with K first hashed if longer.
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK: usize = 64;
    let mut key_pad = [0u8; BLOCK];
    if key.len() > BLOCK {
        let hashed = digest_sha256(key);
        key_pad[..32].copy_from_slice(&hashed);
    } else {
        key_pad[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0u8; BLOCK];
    let mut opad = [0u8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] = key_pad[i] ^ 0x36;
        opad[i] = key_pad[i] ^ 0x5C;
    }
    let mut inner_input = Vec::with_capacity(BLOCK + message.len());
    inner_input.extend_from_slice(&ipad);
    inner_input.extend_from_slice(message);
    let inner = digest_sha256(&inner_input);
    let mut outer_input = Vec::with_capacity(BLOCK + 32);
    outer_input.extend_from_slice(&opad);
    outer_input.extend_from_slice(&inner);
    digest_sha256(&outer_input)
}

// ───────────────────────── SHA-512 / SHA-384 ──────────────────────────
// FIPS 180-4 SHA-512 (64-bit words, 128-byte block, 80 rounds).
// SHA-384 reuses SHA-512's compression function with a different IV and
// truncates output to the first 48 bytes.

const SHA512_K: [u64; 80] = [
    0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc,
    0x3956c25bf348b538, 0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118,
    0xd807aa98a3030242, 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694,
    0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65,
    0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
    0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2, 0xd5a79147930aa725, 0x06ca6351e003826f, 0x142929670a0e6e70,
    0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
    0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b,
    0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30,
    0xd192e819d6ef5218, 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
    0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b,
    0xca273eceea26619c, 0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178,
    0x06f067aa72176fba, 0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
    0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817,
];

const SHA512_H0: [u64; 8] = [
    0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
    0x510e527fade682d1, 0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
];

const SHA384_H0: [u64; 8] = [
    0xcbbb9d5dc1059ed8, 0x629a292a367cd507, 0x9159015a3070dd17, 0x152fecd8f70e5939,
    0x67332667ffc00b31, 0x8eb44a8768581511, 0xdb0c2e0d64f98fa7, 0x47b5481dbefa4fa4,
];

fn sha512_compress(h: &mut [u64; 8], data: &[u8]) {
    // data must be 0-padded to a multiple of 128 with proper length encoding.
    let mut padded: Vec<u8> = data.to_vec();
    let bit_len_lo = (data.len() as u128) * 8;
    padded.push(0x80);
    while padded.len() % 128 != 112 { padded.push(0); }
    // 128-bit big-endian length.
    padded.extend_from_slice(&bit_len_lo.to_be_bytes());

    for chunk in padded.chunks_exact(128) {
        let mut w = [0u64; 80];
        for i in 0..16 {
            w[i] = u64::from_be_bytes([
                chunk[i*8], chunk[i*8+1], chunk[i*8+2], chunk[i*8+3],
                chunk[i*8+4], chunk[i*8+5], chunk[i*8+6], chunk[i*8+7],
            ]);
        }
        for i in 16..80 {
            let s0 = w[i-15].rotate_right(1) ^ w[i-15].rotate_right(8) ^ (w[i-15] >> 7);
            let s1 = w[i-2].rotate_right(19) ^ w[i-2].rotate_right(61) ^ (w[i-2] >> 6);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..80 {
            let s1 = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(SHA512_K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e; e = d.wrapping_add(t1);
            d = c; c = b; b = a; a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
}

pub fn digest_sha512(data: &[u8]) -> [u8; 64] {
    let mut h = SHA512_H0;
    sha512_compress(&mut h, data);
    let mut out = [0u8; 64];
    for i in 0..8 {
        out[i*8..i*8+8].copy_from_slice(&h[i].to_be_bytes());
    }
    out
}

pub fn digest_sha384(data: &[u8]) -> [u8; 48] {
    let mut h = SHA384_H0;
    sha512_compress(&mut h, data);
    let mut out = [0u8; 48];
    // SHA-384 truncates the SHA-512 state to the first 6 words (48 bytes).
    for i in 0..6 {
        out[i*8..i*8+8].copy_from_slice(&h[i].to_be_bytes());
    }
    out
}

pub fn digest_sha512_hex(data: &[u8]) -> String {
    let bytes = digest_sha512(data);
    let mut s = String::with_capacity(128);
    for b in &bytes { s.push_str(&format!("{:02x}", b)); }
    s
}

pub fn digest_sha384_hex(data: &[u8]) -> String {
    let bytes = digest_sha384(data);
    let mut s = String::with_capacity(96);
    for b in &bytes { s.push_str(&format!("{:02x}", b)); }
    s
}

/// HMAC-SHA-512(K, M). Per RFC 4231: 128-byte block (SHA-512 block size).
pub fn hmac_sha512(key: &[u8], message: &[u8]) -> [u8; 64] {
    const BLOCK: usize = 128;
    let mut key_pad = [0u8; BLOCK];
    if key.len() > BLOCK {
        let hashed = digest_sha512(key);
        key_pad[..64].copy_from_slice(&hashed);
    } else {
        key_pad[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0u8; BLOCK];
    let mut opad = [0u8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] = key_pad[i] ^ 0x36;
        opad[i] = key_pad[i] ^ 0x5C;
    }
    let mut inner_input = Vec::with_capacity(BLOCK + message.len());
    inner_input.extend_from_slice(&ipad);
    inner_input.extend_from_slice(message);
    let inner = digest_sha512(&inner_input);
    let mut outer_input = Vec::with_capacity(BLOCK + 64);
    outer_input.extend_from_slice(&opad);
    outer_input.extend_from_slice(&inner);
    digest_sha512(&outer_input)
}

/// HMAC-SHA-384(K, M). Per RFC 4231: 128-byte block (SHA-512 block size).
pub fn hmac_sha384(key: &[u8], message: &[u8]) -> [u8; 48] {
    const BLOCK: usize = 128;
    let mut key_pad = [0u8; BLOCK];
    if key.len() > BLOCK {
        let hashed = digest_sha384(key);
        key_pad[..48].copy_from_slice(&hashed);
    } else {
        key_pad[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0u8; BLOCK];
    let mut opad = [0u8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] = key_pad[i] ^ 0x36;
        opad[i] = key_pad[i] ^ 0x5C;
    }
    let mut inner_input = Vec::with_capacity(BLOCK + message.len());
    inner_input.extend_from_slice(&ipad);
    inner_input.extend_from_slice(message);
    let inner = digest_sha384(&inner_input);
    let mut outer_input = Vec::with_capacity(BLOCK + 48);
    outer_input.extend_from_slice(&opad);
    outer_input.extend_from_slice(&inner);
    digest_sha384(&outer_input)
}

// ───────────────────────── PBKDF2 ─────────────────────────────────────
// RFC 8018 / RFC 2898 §5.2. PBKDF2(P, S, c, dkLen) where PRF is HMAC.
//
//   T_1 = F(P, S, c, 1)
//   T_2 = F(P, S, c, 2)
//   ...
//   T_l = F(P, S, c, l)
//   F(P, S, c, i) = U_1 XOR U_2 XOR ... XOR U_c
//   U_1 = PRF(P, S || INT(i))      (INT(i) is i encoded as 32-bit big-endian)
//   U_j = PRF(P, U_{j-1})           for j > 1
//
// Output is the first dkLen bytes of T_1 || T_2 || ... || T_l where
// l = ceil(dkLen / hLen) and hLen is the HMAC output length.

fn pbkdf2_inner<F, const H: usize>(
    prf: F,
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    dk_len: usize,
) -> Vec<u8>
where
    F: Fn(&[u8], &[u8]) -> [u8; H],
{
    if iterations == 0 || dk_len == 0 { return Vec::new(); }
    let l = (dk_len + H - 1) / H;  // number of blocks
    let mut out = Vec::with_capacity(l * H);
    let mut salt_with_index = Vec::with_capacity(salt.len() + 4);
    for i in 1..=l {
        salt_with_index.clear();
        salt_with_index.extend_from_slice(salt);
        salt_with_index.extend_from_slice(&(i as u32).to_be_bytes());
        let mut u = prf(password, &salt_with_index);
        let mut t = u;
        for _ in 1..iterations {
            u = prf(password, &u);
            for k in 0..H { t[k] ^= u[k]; }
        }
        out.extend_from_slice(&t);
    }
    out.truncate(dk_len);
    out
}

pub fn pbkdf2_hmac_sha1(password: &[u8], salt: &[u8], iterations: u32, dk_len: usize) -> Vec<u8> {
    pbkdf2_inner::<_, 20>(hmac_sha1, password, salt, iterations, dk_len)
}

pub fn pbkdf2_hmac_sha256(password: &[u8], salt: &[u8], iterations: u32, dk_len: usize) -> Vec<u8> {
    pbkdf2_inner::<_, 32>(hmac_sha256, password, salt, iterations, dk_len)
}

pub fn pbkdf2_hmac_sha384(password: &[u8], salt: &[u8], iterations: u32, dk_len: usize) -> Vec<u8> {
    pbkdf2_inner::<_, 48>(hmac_sha384, password, salt, iterations, dk_len)
}

pub fn pbkdf2_hmac_sha512(password: &[u8], salt: &[u8], iterations: u32, dk_len: usize) -> Vec<u8> {
    pbkdf2_inner::<_, 64>(hmac_sha512, password, salt, iterations, dk_len)
}

// ─────────────────────── HKDF (RFC 5869) ────────────────────────────
//
// HMAC-based Extract-and-Expand Key Derivation Function. Reuses the
// HMAC family already in this pilot. Real consumer use: JOSE A*GCMKW
// content-encryption-key derivation, OAuth2 PoP, Noise Protocol.

fn hkdf_inner<F, const H: usize>(
    prf: F, ikm: &[u8], salt: &[u8], info: &[u8], length: usize,
) -> Result<Vec<u8>, String>
where F: Fn(&[u8], &[u8]) -> [u8; H],
{
    // L must be <= 255 * HashLen (RFC 5869 §2.3).
    if length > 255 * H {
        return Err(format!("HKDF: length {} exceeds 255 * HashLen ({})", length, 255 * H));
    }
    // Extract: PRK = HMAC(salt, IKM). If salt is empty, use HashLen zero bytes.
    let zero_salt = vec![0u8; H];
    let prk = if salt.is_empty() { prf(&zero_salt, ikm) } else { prf(salt, ikm) };
    // Expand: T(i) = HMAC(PRK, T(i-1) || info || i), concatenated until length bytes.
    let n = (length + H - 1) / H;
    let mut okm = Vec::with_capacity(n * H);
    let mut prev: Vec<u8> = Vec::new();
    for i in 1..=n {
        let mut buf = Vec::with_capacity(prev.len() + info.len() + 1);
        buf.extend_from_slice(&prev);
        buf.extend_from_slice(info);
        buf.push(i as u8);
        let t = prf(&prk, &buf);
        prev = t.to_vec();
        okm.extend_from_slice(&t);
    }
    okm.truncate(length);
    Ok(okm)
}

pub fn hkdf_sha1(ikm: &[u8], salt: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>, String> {
    hkdf_inner::<_, 20>(hmac_sha1, ikm, salt, info, length)
}
pub fn hkdf_sha256(ikm: &[u8], salt: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>, String> {
    hkdf_inner::<_, 32>(hmac_sha256, ikm, salt, info, length)
}
pub fn hkdf_sha384(ikm: &[u8], salt: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>, String> {
    hkdf_inner::<_, 48>(hmac_sha384, ikm, salt, info, length)
}
pub fn hkdf_sha512(ikm: &[u8], salt: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>, String> {
    hkdf_inner::<_, 64>(hmac_sha512, ikm, salt, info, length)
}

// ─────────────────────── AES (FIPS 197) ────────────────────────────
//
// AES-128 / AES-192 / AES-256 block cipher, encrypt-only path. GCM mode
// (below) only uses AES forward encryption; decrypt is not needed for
// authenticated encryption with associated data. Std-only reference impl
// — performance not a goal (apparatus-side correctness is).

const AES_SBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

const AES_RCON: [u8; 11] = [0x00, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];

fn aes_xtime(x: u8) -> u8 {
    (x << 1) ^ if x & 0x80 != 0 { 0x1b } else { 0x00 }
}

fn aes_sub_word(w: u32) -> u32 {
    let b = w.to_be_bytes();
    u32::from_be_bytes([AES_SBOX[b[0] as usize], AES_SBOX[b[1] as usize],
                        AES_SBOX[b[2] as usize], AES_SBOX[b[3] as usize]])
}

/// FIPS 197 §5.2 KeyExpansion. nk = 4/6/8 for AES-128/192/256.
/// Output length = 4 * (nr + 1) 32-bit words, where nr = nk + 6.
fn aes_key_expansion(key: &[u8]) -> Vec<u32> {
    let nk = key.len() / 4;
    let nr = nk + 6;
    let total = 4 * (nr + 1);
    let mut w = Vec::with_capacity(total);
    for i in 0..nk {
        w.push(u32::from_be_bytes([key[4*i], key[4*i+1], key[4*i+2], key[4*i+3]]));
    }
    for i in nk..total {
        let mut t = w[i - 1];
        if i % nk == 0 {
            t = aes_sub_word(t.rotate_left(8)) ^ ((AES_RCON[i / nk] as u32) << 24);
        } else if nk > 6 && i % nk == 4 {
            t = aes_sub_word(t);
        }
        w.push(w[i - nk] ^ t);
    }
    w
}

fn aes_add_round_key(state: &mut [u8; 16], w: &[u32]) {
    for c in 0..4 {
        let k = w[c].to_be_bytes();
        for r in 0..4 { state[r * 4 + c] ^= k[r]; }
    }
}

fn aes_sub_bytes(state: &mut [u8; 16]) {
    for b in state.iter_mut() { *b = AES_SBOX[*b as usize]; }
}

fn aes_shift_rows(state: &mut [u8; 16]) {
    // Row r is rotated left by r positions. State is row-major in the
    // conceptual 4×4 matrix; index = row*4 + col.
    let s = *state;
    for r in 1..4 {
        for c in 0..4 {
            state[r * 4 + c] = s[r * 4 + (c + r) % 4];
        }
    }
}

fn aes_mix_columns(state: &mut [u8; 16]) {
    for c in 0..4 {
        let s0 = state[c]; let s1 = state[4 + c];
        let s2 = state[8 + c]; let s3 = state[12 + c];
        let t = s0 ^ s1 ^ s2 ^ s3;
        state[c]      ^= t ^ aes_xtime(s0 ^ s1);
        state[4 + c]  ^= t ^ aes_xtime(s1 ^ s2);
        state[8 + c]  ^= t ^ aes_xtime(s2 ^ s3);
        state[12 + c] ^= t ^ aes_xtime(s3 ^ s0);
    }
}

/// FIPS 197 §5.1 Cipher. Single-block encryption. State layout matches
/// the spec: column-major when serialized as bytes (state[r*4+c] holds
/// the byte at row r, column c).
fn aes_encrypt_block(block: &[u8; 16], w: &[u32]) -> [u8; 16] {
    let nr = w.len() / 4 - 1;
    let mut state = [0u8; 16];
    for c in 0..4 {
        for r in 0..4 { state[r * 4 + c] = block[4 * c + r]; }
    }
    aes_add_round_key(&mut state, &w[0..4]);
    for round in 1..nr {
        aes_sub_bytes(&mut state);
        aes_shift_rows(&mut state);
        aes_mix_columns(&mut state);
        aes_add_round_key(&mut state, &w[4 * round .. 4 * round + 4]);
    }
    aes_sub_bytes(&mut state);
    aes_shift_rows(&mut state);
    aes_add_round_key(&mut state, &w[4 * nr .. 4 * nr + 4]);
    let mut out = [0u8; 16];
    for c in 0..4 {
        for r in 0..4 { out[4 * c + r] = state[r * 4 + c]; }
    }
    out
}

/// AES single-block encryption with key (128/192/256 bits).
pub fn aes_encrypt_block_with_key(key: &[u8], block: &[u8; 16]) -> [u8; 16] {
    assert!(key.len() == 16 || key.len() == 24 || key.len() == 32, "AES key must be 16/24/32 bytes");
    let w = aes_key_expansion(key);
    aes_encrypt_block(block, &w)
}

// ─────────────────────── AES-GCM (SP 800-38D) ───────────────────────
//
// Galois/Counter Mode authenticated encryption. Uses AES-CTR for the
// encryption stream and GHASH (multiplication in GF(2^128) under the
// reducing polynomial x^128 + x^7 + x^2 + x + 1) for authentication.
//
// Pilot scope: AES-128-GCM, AES-256-GCM, 12-byte IV (the dominant case;
// the WebCrypto AES-GCM algorithm specifies a 12-byte recommendation),
// 16-byte tag.

fn gf128_mul(x: [u8; 16], y: [u8; 16]) -> [u8; 16] {
    // SP 800-38D §6.3 multiplication in GF(2^128). Bits are treated as
    // a polynomial with the leftmost bit as the highest-order coefficient.
    let mut z = [0u8; 16];
    let mut v = y;
    for i in 0..128 {
        let bit = (x[i / 8] >> (7 - (i % 8))) & 1;
        if bit == 1 {
            for k in 0..16 { z[k] ^= v[k]; }
        }
        let lsb = v[15] & 1;
        // shift v right by 1 (in the spec's bit ordering this is the
        // rightward shift through bytes high-to-low).
        for k in (1..16).rev() {
            v[k] = (v[k] >> 1) | ((v[k - 1] & 1) << 7);
        }
        v[0] >>= 1;
        if lsb == 1 {
            v[0] ^= 0xe1;  // reducing polynomial high byte
        }
    }
    z
}

fn ghash(h: [u8; 16], aad: &[u8], ct: &[u8]) -> [u8; 16] {
    // SP 800-38D §6.4. GHASH_H(A || 0_pad || C || 0_pad || len(A)_64 || len(C)_64).
    let mut y = [0u8; 16];
    let mut absorb = |chunk: &[u8]| {
        for c in chunk.chunks(16) {
            let mut block = [0u8; 16];
            block[..c.len()].copy_from_slice(c);
            for i in 0..16 { y[i] ^= block[i]; }
            y = gf128_mul(y, h);
        }
    };
    absorb(aad);
    absorb(ct);
    let mut len_block = [0u8; 16];
    len_block[..8].copy_from_slice(&((aad.len() as u64) * 8).to_be_bytes());
    len_block[8..].copy_from_slice(&((ct.len() as u64) * 8).to_be_bytes());
    for i in 0..16 { y[i] ^= len_block[i]; }
    gf128_mul(y, h)
}

fn aes_ctr_xor(w: &[u32], counter0: [u8; 16], data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut counter = counter0;
    for chunk in data.chunks(16) {
        let ks = aes_encrypt_block(&counter, w);
        for (i, b) in chunk.iter().enumerate() {
            out.push(b ^ ks[i]);
        }
        // increment the last 32 bits (big-endian) per SP 800-38D §6.5.
        let inc = u32::from_be_bytes([counter[12], counter[13], counter[14], counter[15]])
            .wrapping_add(1);
        counter[12..16].copy_from_slice(&inc.to_be_bytes());
    }
    out
}

/// AES-GCM encrypt. Returns ciphertext || tag (WebCrypto layout).
/// Pilot scope: 12-byte IV, 16-byte tag.
pub fn aes_gcm_encrypt(key: &[u8], iv: &[u8], aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 16 && key.len() != 24 && key.len() != 32 {
        return Err(format!("AES-GCM: invalid key length {}", key.len()));
    }
    if iv.len() != 12 {
        return Err("AES-GCM pilot scope: IV must be 12 bytes".to_string());
    }
    let w = aes_key_expansion(key);
    let h = aes_encrypt_block(&[0u8; 16], &w);
    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(iv);
    j0[15] = 1;
    let mut counter1 = j0;
    let inc = u32::from_be_bytes([counter1[12], counter1[13], counter1[14], counter1[15]])
        .wrapping_add(1);
    counter1[12..16].copy_from_slice(&inc.to_be_bytes());
    let ciphertext = aes_ctr_xor(&w, counter1, plaintext);
    let s = ghash(h, aad, &ciphertext);
    let ej0 = aes_encrypt_block(&j0, &w);
    let mut tag = [0u8; 16];
    for i in 0..16 { tag[i] = s[i] ^ ej0[i]; }
    let mut out = ciphertext;
    out.extend_from_slice(&tag);
    Ok(out)
}

/// AES-GCM decrypt. Input is ciphertext || tag (WebCrypto layout).
/// Returns Err on authentication-tag mismatch.
pub fn aes_gcm_decrypt(key: &[u8], iv: &[u8], aad: &[u8], ct_and_tag: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 16 && key.len() != 24 && key.len() != 32 {
        return Err(format!("AES-GCM: invalid key length {}", key.len()));
    }
    if iv.len() != 12 {
        return Err("AES-GCM pilot scope: IV must be 12 bytes".to_string());
    }
    if ct_and_tag.len() < 16 {
        return Err("AES-GCM: input too short for tag".to_string());
    }
    let (ciphertext, tag) = ct_and_tag.split_at(ct_and_tag.len() - 16);
    let w = aes_key_expansion(key);
    let h = aes_encrypt_block(&[0u8; 16], &w);
    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(iv);
    j0[15] = 1;
    let s = ghash(h, aad, ciphertext);
    let ej0 = aes_encrypt_block(&j0, &w);
    let mut expected_tag = [0u8; 16];
    for i in 0..16 { expected_tag[i] = s[i] ^ ej0[i]; }
    if !timing_safe_equal(&expected_tag, tag) {
        return Err("AES-GCM: authentication tag mismatch".to_string());
    }
    let mut counter1 = j0;
    let inc = u32::from_be_bytes([counter1[12], counter1[13], counter1[14], counter1[15]])
        .wrapping_add(1);
    counter1[12..16].copy_from_slice(&inc.to_be_bytes());
    Ok(aes_ctr_xor(&w, counter1, ciphertext))
}

// ───────────────────────── crypto.subtle stub ─────────────────────────

pub mod subtle {
    use super::digest_sha256;

    /// SPEC: crypto.subtle.digest("SHA-256", data) → ArrayBuffer of 32 bytes.
    /// Pilot returns Vec<u8>. Algorithm name accepted in any of "SHA-256",
    /// "sha-256", "SHA256".
    pub fn digest(algorithm: &str, data: &[u8]) -> Result<Vec<u8>, String> {
        match algorithm.to_ascii_uppercase().replace("-", "").as_str() {
            "SHA256" => Ok(digest_sha256(data).to_vec()),
            other => Err(format!("unsupported algorithm: {}", other)),
        }
    }
}
