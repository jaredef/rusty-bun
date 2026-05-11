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

// ─────────────────────── Big-integer arithmetic ───────────────────
//
// Minimal big-unsigned-integer impl for RSA. Little-endian Vec<u32>
// limb representation; all operations are constant-time-friendly only
// where it matters for security (modexp doesn't leak via timing here
// because we don't promise side-channel resistance — this is a
// reference implementation for correctness verification against Bun).
//
// Scope: enough for RSA-OAEP / RSA-PSS over 2048/3072/4096-bit keys.

#[derive(Clone, Debug)]
pub struct BigUInt(Vec<u32>);  // limbs[0] = least significant

impl BigUInt {
    pub fn zero() -> Self { BigUInt(vec![0]) }
    pub fn one() -> Self { BigUInt(vec![1]) }

    pub fn from_be_bytes(b: &[u8]) -> Self {
        // Strip leading zeros; not strictly necessary but keeps trim() cheap.
        let n_limbs = (b.len() + 3) / 4;
        let mut limbs = vec![0u32; n_limbs];
        for (i, byte) in b.iter().rev().enumerate() {
            limbs[i / 4] |= (*byte as u32) << ((i % 4) * 8);
        }
        let mut r = BigUInt(limbs);
        r.trim();
        r
    }

    pub fn to_be_bytes(&self, len: usize) -> Vec<u8> {
        let mut out = vec![0u8; len];
        for i in 0..len {
            let limb = self.0.get(i / 4).copied().unwrap_or(0);
            let byte = (limb >> ((i % 4) * 8)) & 0xff;
            out[len - 1 - i] = byte as u8;
        }
        out
    }

    fn trim(&mut self) {
        while self.0.len() > 1 && *self.0.last().unwrap() == 0 {
            self.0.pop();
        }
    }

    pub fn is_zero(&self) -> bool { self.0.iter().all(|&l| l == 0) }

    pub fn bit_len(&self) -> usize {
        for i in (0..self.0.len()).rev() {
            if self.0[i] != 0 {
                return i * 32 + (32 - self.0[i].leading_zeros() as usize);
            }
        }
        0
    }

    pub fn bit(&self, i: usize) -> bool {
        let limb = i / 32;
        let bit = i % 32;
        self.0.get(limb).copied().unwrap_or(0) & (1u32 << bit) != 0
    }

    pub fn cmp(&self, other: &BigUInt) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        // Compare limbs from most-significant down.
        let la = self.0.len();
        let lb = other.0.len();
        let la_eff = (0..la).rev().find(|&i| self.0[i] != 0).map(|i| i + 1).unwrap_or(0);
        let lb_eff = (0..lb).rev().find(|&i| other.0[i] != 0).map(|i| i + 1).unwrap_or(0);
        if la_eff != lb_eff { return la_eff.cmp(&lb_eff); }
        for i in (0..la_eff).rev() {
            match self.0[i].cmp(&other.0[i]) {
                Ordering::Equal => continue,
                ord => return ord,
            }
        }
        Ordering::Equal
    }

    pub fn add(&self, other: &BigUInt) -> BigUInt {
        let n = self.0.len().max(other.0.len()) + 1;
        let mut out = vec![0u32; n];
        let mut carry: u64 = 0;
        for i in 0..n {
            let a = self.0.get(i).copied().unwrap_or(0) as u64;
            let b = other.0.get(i).copied().unwrap_or(0) as u64;
            let sum = a + b + carry;
            out[i] = (sum & 0xffffffff) as u32;
            carry = sum >> 32;
        }
        let mut r = BigUInt(out);
        r.trim();
        r
    }

    /// Returns self - other. Caller must ensure self >= other.
    pub fn sub(&self, other: &BigUInt) -> BigUInt {
        let n = self.0.len();
        let mut out = vec![0u32; n];
        let mut borrow: i64 = 0;
        for i in 0..n {
            let a = self.0[i] as i64;
            let b = other.0.get(i).copied().unwrap_or(0) as i64;
            let diff = a - b - borrow;
            if diff < 0 {
                out[i] = (diff + (1i64 << 32)) as u32;
                borrow = 1;
            } else {
                out[i] = diff as u32;
                borrow = 0;
            }
        }
        let mut r = BigUInt(out);
        r.trim();
        r
    }

    pub fn mul(&self, other: &BigUInt) -> BigUInt {
        // Schoolbook O(n^2). For RSA-4096 this is ~16k u32-muls per
        // multiplication — fine for correctness-first verification.
        let n = self.0.len() + other.0.len();
        let mut out = vec![0u64; n];
        for i in 0..self.0.len() {
            for j in 0..other.0.len() {
                let p = (self.0[i] as u64) * (other.0[j] as u64);
                out[i + j] += p & 0xffffffff;
                out[i + j + 1] += p >> 32;
            }
        }
        // Propagate carries across the u64 accumulator into u32 limbs.
        let mut limbs = vec![0u32; n + 1];
        let mut carry: u64 = 0;
        for i in 0..n {
            let s = out[i] + carry;
            limbs[i] = (s & 0xffffffff) as u32;
            carry = s >> 32;
        }
        limbs[n] = carry as u32;
        let mut r = BigUInt(limbs);
        r.trim();
        r
    }

    fn shl1(&self) -> BigUInt {
        let mut out = vec![0u32; self.0.len() + 1];
        let mut carry: u32 = 0;
        for (i, &l) in self.0.iter().enumerate() {
            out[i] = (l << 1) | carry;
            carry = l >> 31;
        }
        out[self.0.len()] = carry;
        let mut r = BigUInt(out);
        r.trim();
        r
    }

    /// Returns (quotient, remainder) using binary long division.
    /// Slow but correct; adequate for the pilot's scope.
    pub fn divmod(&self, divisor: &BigUInt) -> (BigUInt, BigUInt) {
        use std::cmp::Ordering;
        assert!(!divisor.is_zero(), "BigUInt divmod by zero");
        let bits = self.bit_len();
        let mut q_limbs = vec![0u32; (bits + 31) / 32 + 1];
        let mut r = BigUInt::zero();
        for i in (0..bits).rev() {
            r = r.shl1();
            if self.bit(i) {
                // OR in 1.
                if r.0.is_empty() { r.0.push(0); }
                r.0[0] |= 1;
            }
            if r.cmp(divisor) != Ordering::Less {
                r = r.sub(divisor);
                q_limbs[i / 32] |= 1u32 << (i % 32);
            }
        }
        let mut q = BigUInt(q_limbs);
        q.trim();
        r.trim();
        (q, r)
    }

    pub fn modulo(&self, m: &BigUInt) -> BigUInt {
        self.divmod(m).1
    }

    /// Square-and-multiply modular exponentiation. Returns self^e mod m.
    /// Pilot scope: not constant-time; correctness only.
    pub fn mod_pow(&self, e: &BigUInt, m: &BigUInt) -> BigUInt {
        if m.cmp(&BigUInt::one()) == std::cmp::Ordering::Equal {
            return BigUInt::zero();
        }
        let mut result = BigUInt::one();
        let mut base = self.modulo(m);
        let bits = e.bit_len();
        for i in 0..bits {
            if e.bit(i) {
                result = result.mul(&base).modulo(m);
            }
            base = base.mul(&base).modulo(m);
        }
        result
    }
}

// ─────────────────────── RSA primitives (RFC 8017 §5) ──────────────
//
// RSAEP: c = m^e mod n  (encryption / verification)
// RSADP: m = c^d mod n  (decryption / signing)
//
// Plain RSA — no padding. OAEP / PSS padding wrap these. Pilot scope:
// public-key-with-(n,e) and private-key-with-(n,d). CRT-form private
// keys (n,e,d,p,q,dp,dq,qi) deferred to a follow-on round.

pub fn rsaep(n: &BigUInt, e: &BigUInt, m: &BigUInt) -> Result<BigUInt, String> {
    if m.cmp(n) != std::cmp::Ordering::Less {
        return Err("RSAEP: message representative out of range".to_string());
    }
    Ok(m.mod_pow(e, n))
}

pub fn rsadp(n: &BigUInt, d: &BigUInt, c: &BigUInt) -> Result<BigUInt, String> {
    if c.cmp(n) != std::cmp::Ordering::Less {
        return Err("RSADP: ciphertext representative out of range".to_string());
    }
    Ok(c.mod_pow(d, n))
}

// ───────────────────── RSASSA-PKCS1-v1_5 (RFC 8017 §8.2 + §9.2) ────
//
// Legacy RSA signature scheme — deterministic, no salt, no MGF.
// JWS JWT RS256/RS384/RS512, X.509 CA signatures, code-signing.
//
// EM = 0x00 || 0x01 || PS || 0x00 || T  where:
//   T = DigestInfo prefix || hash output (DER-encoded AlgorithmIdentifier + OCTET STRING)
//   PS = (k - 3 - tLen) bytes of 0xff
// Sign: m = OS2IP(EM); s = m^d mod n; S = I2OSP(s, k)
// Verify: recompute expected EM, compare in constant time.

fn digest_info_prefix(hash_name: &str) -> Result<&'static [u8], String> {
    // DER-encoded DigestInfo prefix per RFC 8017 §9.2 note 1.
    match hash_name {
        "SHA-1"   => Ok(&[0x30,0x21,0x30,0x09,0x06,0x05,0x2b,0x0e,0x03,0x02,0x1a,0x05,0x00,0x04,0x14]),
        "SHA-256" => Ok(&[0x30,0x31,0x30,0x0d,0x06,0x09,0x60,0x86,0x48,0x01,0x65,0x03,0x04,0x02,0x01,0x05,0x00,0x04,0x20]),
        "SHA-384" => Ok(&[0x30,0x41,0x30,0x0d,0x06,0x09,0x60,0x86,0x48,0x01,0x65,0x03,0x04,0x02,0x02,0x05,0x00,0x04,0x30]),
        "SHA-512" => Ok(&[0x30,0x51,0x30,0x0d,0x06,0x09,0x60,0x86,0x48,0x01,0x65,0x03,0x04,0x02,0x03,0x05,0x00,0x04,0x40]),
        other => Err(format!("PKCS1-v1_5: unsupported hash {}", other)),
    }
}

fn emsa_pkcs1_v1_5_encode(hash: &[u8], em_len: usize, hash_name: &str) -> Result<Vec<u8>, String> {
    let prefix = digest_info_prefix(hash_name)?;
    let t_len = prefix.len() + hash.len();
    if em_len < t_len + 11 {
        return Err("PKCS1-v1_5: intended encoded message length too short".into());
    }
    let ps_len = em_len - t_len - 3;
    let mut em = Vec::with_capacity(em_len);
    em.push(0x00);
    em.push(0x01);
    em.extend(std::iter::repeat(0xffu8).take(ps_len));
    em.push(0x00);
    em.extend_from_slice(prefix);
    em.extend_from_slice(hash);
    debug_assert_eq!(em.len(), em_len);
    Ok(em)
}

pub fn rsa_pkcs1_v15_sign(
    n_bytes: &[u8], d_bytes: &[u8], hash: &[u8], hash_name: &str,
) -> Result<Vec<u8>, String> {
    let k = n_bytes.len();
    let em = emsa_pkcs1_v1_5_encode(hash, k, hash_name)?;
    let n = BigUInt::from_be_bytes(n_bytes);
    let d = BigUInt::from_be_bytes(d_bytes);
    let m_int = BigUInt::from_be_bytes(&em);
    let s_int = rsadp(&n, &d, &m_int)?;
    Ok(s_int.to_be_bytes(k))
}

pub fn rsa_pkcs1_v15_verify(
    n_bytes: &[u8], e_bytes: &[u8], hash: &[u8], signature: &[u8], hash_name: &str,
) -> Result<(), String> {
    let k = n_bytes.len();
    if signature.len() != k { return Err("PKCS1-v1_5: signature length mismatch".into()); }
    let n = BigUInt::from_be_bytes(n_bytes);
    let e = BigUInt::from_be_bytes(e_bytes);
    let s_int = BigUInt::from_be_bytes(signature);
    let m_int = rsaep(&n, &e, &s_int)?;
    let em_recovered = m_int.to_be_bytes(k);
    let em_expected = emsa_pkcs1_v1_5_encode(hash, k, hash_name)?;
    if !timing_safe_equal(&em_recovered, &em_expected) {
        return Err("PKCS1-v1_5: signature verification failed".into());
    }
    Ok(())
}

// ─────────────────────── P-256 elliptic curve ──────────────────────
//
// NIST P-256 / secp256r1 / prime256v1. Short Weierstrass curve
// y² = x³ + ax + b (mod p) where a = -3 mod p.
// Parameters: FIPS 186-4 §D.1.2.3 / SEC 2 §2.4.2.
//
// Pilot scope: P-256 with SHA-256 only. Affine coordinates throughout
// (slow but correct — Jacobian projective is the production speedup).
// Reuses BigUInt above for the finite-field arithmetic.

fn p256_p() -> BigUInt {
    BigUInt::from_be_bytes(&[
        0xff,0xff,0xff,0xff,0x00,0x00,0x00,0x01,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
        0x00,0x00,0x00,0x00,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,
    ])
}
fn p256_n() -> BigUInt {
    BigUInt::from_be_bytes(&[
        0xff,0xff,0xff,0xff,0x00,0x00,0x00,0x00,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,
        0xbc,0xe6,0xfa,0xad,0xa7,0x17,0x9e,0x84,0xf3,0xb9,0xca,0xc2,0xfc,0x63,0x25,0x51,
    ])
}
fn p256_b() -> BigUInt {
    BigUInt::from_be_bytes(&[
        0x5a,0xc6,0x35,0xd8,0xaa,0x3a,0x93,0xe7,0xb3,0xeb,0xbd,0x55,0x76,0x98,0x86,0xbc,
        0x65,0x1d,0x06,0xb0,0xcc,0x53,0xb0,0xf6,0x3b,0xce,0x3c,0x3e,0x27,0xd2,0x60,0x4b,
    ])
}
pub fn p256_g() -> P256Point {
    P256Point::Affine {
        x: BigUInt::from_be_bytes(&[
            0x6b,0x17,0xd1,0xf2,0xe1,0x2c,0x42,0x47,0xf8,0xbc,0xe6,0xe5,0x63,0xa4,0x40,0xf2,
            0x77,0x03,0x7d,0x81,0x2d,0xeb,0x33,0xa0,0xf4,0xa1,0x39,0x45,0xd8,0x98,0xc2,0x96,
        ]),
        y: BigUInt::from_be_bytes(&[
            0x4f,0xe3,0x42,0xe2,0xfe,0x1a,0x7f,0x9b,0x8e,0xe7,0xeb,0x4a,0x7c,0x0f,0x9e,0x16,
            0x2b,0xce,0x33,0x57,0x6b,0x31,0x5e,0xce,0xcb,0xb6,0x40,0x68,0x37,0xbf,0x51,0xf5,
        ]),
    }
}

#[derive(Clone, Debug)]
pub enum P256Point {
    Identity,
    Affine { x: BigUInt, y: BigUInt },
}

fn mod_add(a: &BigUInt, b: &BigUInt, m: &BigUInt) -> BigUInt {
    a.add(b).modulo(m)
}
fn mod_sub(a: &BigUInt, b: &BigUInt, m: &BigUInt) -> BigUInt {
    use std::cmp::Ordering;
    if a.cmp(b) != Ordering::Less { a.sub(b) } else { m.add(a).sub(b).modulo(m) }
}
fn mod_mul(a: &BigUInt, b: &BigUInt, m: &BigUInt) -> BigUInt {
    a.mul(b).modulo(m)
}
fn mod_inv_fermat(a: &BigUInt, p: &BigUInt) -> BigUInt {
    // Fermat: a^(p-2) mod p for prime p.
    let two = BigUInt::from_be_bytes(&[2]);
    let p_minus_2 = p.sub(&two);
    a.mod_pow(&p_minus_2, p)
}

fn p256_double(pt: &P256Point) -> P256Point {
    let p = p256_p();
    let three = BigUInt::from_be_bytes(&[3]);
    let two = BigUInt::from_be_bytes(&[2]);
    match pt {
        P256Point::Identity => P256Point::Identity,
        P256Point::Affine { x, y } => {
            if y.is_zero() { return P256Point::Identity; }
            // λ = (3x² + a) / (2y); a = -3 mod p.
            let x2 = mod_mul(x, x, &p);
            let three_x2 = mod_mul(&three, &x2, &p);
            let three_x2_plus_a = mod_sub(&three_x2, &three, &p);  // a = -3 → +(-3) ≡ -3
            let two_y = mod_mul(&two, y, &p);
            let inv = mod_inv_fermat(&two_y, &p);
            let lambda = mod_mul(&three_x2_plus_a, &inv, &p);
            // x3 = λ² - 2x
            let lambda2 = mod_mul(&lambda, &lambda, &p);
            let two_x = mod_mul(&two, x, &p);
            let x3 = mod_sub(&lambda2, &two_x, &p);
            // y3 = λ(x - x3) - y
            let x_minus_x3 = mod_sub(x, &x3, &p);
            let lambda_diff = mod_mul(&lambda, &x_minus_x3, &p);
            let y3 = mod_sub(&lambda_diff, y, &p);
            P256Point::Affine { x: x3, y: y3 }
        }
    }
}

fn p256_add(p1: &P256Point, p2: &P256Point) -> P256Point {
    use std::cmp::Ordering;
    let p = p256_p();
    match (p1, p2) {
        (P256Point::Identity, q) | (q, P256Point::Identity) => q.clone(),
        (P256Point::Affine { x: x1, y: y1 }, P256Point::Affine { x: x2, y: y2 }) => {
            if x1.cmp(x2) == Ordering::Equal {
                // Same x: doubling or inverse (y1 = -y2 → identity).
                if y1.cmp(y2) == Ordering::Equal {
                    return p256_double(p1);
                }
                // y1 + y2 ≡ 0 mod p → identity.
                return P256Point::Identity;
            }
            // λ = (y2 - y1) / (x2 - x1)
            let dy = mod_sub(y2, y1, &p);
            let dx = mod_sub(x2, x1, &p);
            let inv = mod_inv_fermat(&dx, &p);
            let lambda = mod_mul(&dy, &inv, &p);
            // x3 = λ² - x1 - x2
            let lambda2 = mod_mul(&lambda, &lambda, &p);
            let x3 = mod_sub(&mod_sub(&lambda2, x1, &p), x2, &p);
            // y3 = λ(x1 - x3) - y1
            let x1_minus_x3 = mod_sub(x1, &x3, &p);
            let lambda_diff = mod_mul(&lambda, &x1_minus_x3, &p);
            let y3 = mod_sub(&lambda_diff, y1, &p);
            P256Point::Affine { x: x3, y: y3 }
        }
    }
}

/// Scalar multiplication via binary double-and-add (LSB-first).
pub fn p256_scalar_mul(k: &BigUInt, pt: &P256Point) -> P256Point {
    let mut result = P256Point::Identity;
    let mut addend = pt.clone();
    let bits = k.bit_len();
    for i in 0..bits {
        if k.bit(i) {
            result = p256_add(&result, &addend);
        }
        addend = p256_double(&addend);
    }
    result
}

/// ECDSA-P256 sign per FIPS 186-4 §6.4. `nonce_k` is the per-signature
/// random k in [1, n-1] — caller-supplied for testability. Hash output
/// is reduced mod n. Signature format: r ‖ s (P1363 / WebCrypto raw),
/// each 32 bytes big-endian.
pub fn ecdsa_p256_sha256_sign(
    d_bytes: &[u8], message: &[u8], nonce_k: &[u8],
) -> Result<Vec<u8>, String> {
    let n = p256_n();
    let d = BigUInt::from_be_bytes(d_bytes);
    let k = BigUInt::from_be_bytes(nonce_k);
    use std::cmp::Ordering;
    if k.is_zero() || k.cmp(&n) != Ordering::Less {
        return Err("ECDSA: nonce_k out of range".into());
    }
    if d.is_zero() || d.cmp(&n) != Ordering::Less {
        return Err("ECDSA: private key out of range".into());
    }
    let e_bytes = digest_sha256(message);
    let e = BigUInt::from_be_bytes(&e_bytes);
    // e mod n (P-256: hash output 256 bits == ⌈log2 n⌉, so this is just the reduction).
    let e_red = e.modulo(&n);
    let g = p256_g();
    let r_pt = p256_scalar_mul(&k, &g);
    let x1 = match &r_pt {
        P256Point::Affine { x, .. } => x.clone(),
        P256Point::Identity => return Err("ECDSA: k*G is identity".into()),
    };
    let r = x1.modulo(&n);
    if r.is_zero() { return Err("ECDSA: r=0 — retry with new k".into()); }
    let k_inv = mod_inv_fermat(&k, &n);
    let rd = mod_mul(&r, &d, &n);
    let e_plus_rd = mod_add(&e_red, &rd, &n);
    let s = mod_mul(&k_inv, &e_plus_rd, &n);
    if s.is_zero() { return Err("ECDSA: s=0 — retry with new k".into()); }
    let mut out = Vec::with_capacity(64);
    out.extend_from_slice(&r.to_be_bytes(32));
    out.extend_from_slice(&s.to_be_bytes(32));
    Ok(out)
}

/// ECDSA-P256 verify per FIPS 186-4 §6.4. Signature is P1363 r ‖ s.
pub fn ecdsa_p256_sha256_verify(
    qx_bytes: &[u8], qy_bytes: &[u8], message: &[u8], signature: &[u8],
) -> Result<(), String> {
    use std::cmp::Ordering;
    if signature.len() != 64 { return Err("ECDSA: signature must be 64 bytes".into()); }
    let n = p256_n();
    let one = BigUInt::one();
    let r = BigUInt::from_be_bytes(&signature[..32]);
    let s = BigUInt::from_be_bytes(&signature[32..]);
    if r.cmp(&one) == Ordering::Less || r.cmp(&n) != Ordering::Less {
        return Err("ECDSA: r out of range".into());
    }
    if s.cmp(&one) == Ordering::Less || s.cmp(&n) != Ordering::Less {
        return Err("ECDSA: s out of range".into());
    }
    let qx = BigUInt::from_be_bytes(qx_bytes);
    let qy = BigUInt::from_be_bytes(qy_bytes);
    // Validate Q is on curve: y² ≡ x³ + ax + b (mod p), a = -3.
    let p = p256_p();
    let three = BigUInt::from_be_bytes(&[3]);
    let lhs = mod_mul(&qy, &qy, &p);
    let x3 = mod_mul(&mod_mul(&qx, &qx, &p), &qx, &p);
    let neg3x = mod_mul(&three, &qx, &p);
    let rhs = mod_sub(&mod_add(&x3, &p256_b(), &p), &neg3x, &p);
    if lhs.cmp(&rhs) != Ordering::Equal {
        return Err("ECDSA: public key not on curve".into());
    }
    let q = P256Point::Affine { x: qx, y: qy };
    let e = BigUInt::from_be_bytes(&digest_sha256(message)).modulo(&n);
    let w = mod_inv_fermat(&s, &n);
    let u1 = mod_mul(&e, &w, &n);
    let u2 = mod_mul(&r, &w, &n);
    let p1 = p256_scalar_mul(&u1, &p256_g());
    let p2 = p256_scalar_mul(&u2, &q);
    let r_pt = p256_add(&p1, &p2);
    let x1 = match r_pt {
        P256Point::Affine { x, .. } => x,
        P256Point::Identity => return Err("ECDSA: u1*G + u2*Q is identity".into()),
    };
    if x1.modulo(&n).cmp(&r) == Ordering::Equal { Ok(()) }
    else { Err("ECDSA: signature mismatch".into()) }
}

// ─────────────────────── Curve-parameterized EC primitives ─────────
//
// Generalization of the P-256-specific code above. All NIST P-curves
// have a = -3, so we hardcode that and parameterize over (p, n, b, G,
// coord_bytes). P-384 + P-521 reuse this scaffold.

#[derive(Clone)]
pub struct Curve {
    pub p: BigUInt,
    pub n: BigUInt,
    pub b: BigUInt,
    pub g: P256Point,           // reuse the affine Point type — same shape
    pub coord_bytes: usize,     // 32 (P-256), 48 (P-384), 66 (P-521)
}

pub fn curve_p256() -> Curve {
    Curve {
        p: p256_p(),
        n: p256_n(),
        b: p256_b(),
        g: p256_g(),
        coord_bytes: 32,
    }
}

pub fn curve_p384() -> Curve {
    // SEC 2 §2.5.1 / FIPS 186-4 §D.1.2.4.
    let p = BigUInt::from_be_bytes(&hex_to_bytes(
        "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffff0000000000000000ffffffff"));
    let n = BigUInt::from_be_bytes(&hex_to_bytes(
        "ffffffffffffffffffffffffffffffffffffffffffffffffc7634d81f4372ddf581a0db248b0a77aecec196accc52973"));
    let b = BigUInt::from_be_bytes(&hex_to_bytes(
        "b3312fa7e23ee7e4988e056be3f82d19181d9c6efe8141120314088f5013875ac656398d8a2ed19d2a85c8edd3ec2aef"));
    let gx = BigUInt::from_be_bytes(&hex_to_bytes(
        "aa87ca22be8b05378eb1c71ef320ad746e1d3b628ba79b9859f741e082542a385502f25dbf55296c3a545e3872760ab7"));
    let gy = BigUInt::from_be_bytes(&hex_to_bytes(
        "3617de4a96262c6f5d9e98bf9292dc29f8f41dbd289a147ce9da3113b5f0b8c00a60b1ce1d7e819d7a431d7c90ea0e5f"));
    Curve { p, n, b, g: P256Point::Affine { x: gx, y: gy }, coord_bytes: 48 }
}

pub fn curve_p521() -> Curve {
    // SEC 2 §2.6.1 / FIPS 186-4 §D.1.2.5. Coordinates are 66 bytes
    // (521 bits, leading 7 bits are zero in the byte representation).
    let p = BigUInt::from_be_bytes(&hex_to_bytes(
        "01ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"));
    let n = BigUInt::from_be_bytes(&hex_to_bytes(
        "01fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa51868783bf2f966b7fcc0148f709a5d03bb5c9b8899c47aebb6fb71e91386409"));
    let b = BigUInt::from_be_bytes(&hex_to_bytes(
        "0051953eb9618e1c9a1f929a21a0b68540eea2da725b99b315f3b8b489918ef109e156193951ec7e937b1652c0bd3bb1bf073573df883d2c34f1ef451fd46b503f00"));
    let gx = BigUInt::from_be_bytes(&hex_to_bytes(
        "00c6858e06b70404e9cd9e3ecb662395b4429c648139053fb521f828af606b4d3dbaa14b5e77efe75928fe1dc127a2ffa8de3348b3c1856a429bf97e7e31c2e5bd66"));
    let gy = BigUInt::from_be_bytes(&hex_to_bytes(
        "011839296a789a3bc0045c8a5fb42c7d1bd998f54449579b446817afbd17273e662c97ee72995ef42640c550b9013fad0761353c7086a272c24088be94769fd16650"));
    Curve { p, n, b, g: P256Point::Affine { x: gx, y: gy }, coord_bytes: 66 }
}

fn hex_to_bytes(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i+2], 16).unwrap())
        .collect()
}

fn ec_double(c: &Curve, pt: &P256Point) -> P256Point {
    let p = &c.p;
    let three = BigUInt::from_be_bytes(&[3]);
    let two = BigUInt::from_be_bytes(&[2]);
    match pt {
        P256Point::Identity => P256Point::Identity,
        P256Point::Affine { x, y } => {
            if y.is_zero() { return P256Point::Identity; }
            let x2 = mod_mul(x, x, p);
            let three_x2 = mod_mul(&three, &x2, p);
            let three_x2_plus_a = mod_sub(&three_x2, &three, p);  // a = -3
            let two_y = mod_mul(&two, y, p);
            let inv = mod_inv_fermat(&two_y, p);
            let lambda = mod_mul(&three_x2_plus_a, &inv, p);
            let lambda2 = mod_mul(&lambda, &lambda, p);
            let two_x = mod_mul(&two, x, p);
            let x3 = mod_sub(&lambda2, &two_x, p);
            let x_minus_x3 = mod_sub(x, &x3, p);
            let lambda_diff = mod_mul(&lambda, &x_minus_x3, p);
            let y3 = mod_sub(&lambda_diff, y, p);
            P256Point::Affine { x: x3, y: y3 }
        }
    }
}

fn ec_add(c: &Curve, p1: &P256Point, p2: &P256Point) -> P256Point {
    use std::cmp::Ordering;
    let p = &c.p;
    match (p1, p2) {
        (P256Point::Identity, q) | (q, P256Point::Identity) => q.clone(),
        (P256Point::Affine { x: x1, y: y1 }, P256Point::Affine { x: x2, y: y2 }) => {
            if x1.cmp(x2) == Ordering::Equal {
                if y1.cmp(y2) == Ordering::Equal { return ec_double(c, p1); }
                return P256Point::Identity;
            }
            let dy = mod_sub(y2, y1, p);
            let dx = mod_sub(x2, x1, p);
            let inv = mod_inv_fermat(&dx, p);
            let lambda = mod_mul(&dy, &inv, p);
            let lambda2 = mod_mul(&lambda, &lambda, p);
            let x3 = mod_sub(&mod_sub(&lambda2, x1, p), x2, p);
            let x1_minus_x3 = mod_sub(x1, &x3, p);
            let lambda_diff = mod_mul(&lambda, &x1_minus_x3, p);
            let y3 = mod_sub(&lambda_diff, y1, p);
            P256Point::Affine { x: x3, y: y3 }
        }
    }
}

pub fn ec_scalar_mul(c: &Curve, k: &BigUInt, pt: &P256Point) -> P256Point {
    let mut result = P256Point::Identity;
    let mut addend = pt.clone();
    let bits = k.bit_len();
    for i in 0..bits {
        if k.bit(i) {
            result = ec_add(c, &result, &addend);
        }
        addend = ec_double(c, &addend);
    }
    result
}

fn on_curve(c: &Curve, x: &BigUInt, y: &BigUInt) -> bool {
    use std::cmp::Ordering;
    let three = BigUInt::from_be_bytes(&[3]);
    let p = &c.p;
    let lhs = mod_mul(y, y, p);
    let x3 = mod_mul(&mod_mul(x, x, p), x, p);
    let neg3x = mod_mul(&three, x, p);
    let rhs = mod_sub(&mod_add(&x3, &c.b, p), &neg3x, p);
    lhs.cmp(&rhs) == Ordering::Equal
}

/// ECDSA over arbitrary NIST curve. `hash` is the message hash already
/// computed; caller selects the hash to match the curve. `nonce_k` is
/// the per-signature random k (caller-supplied).
pub fn ecdsa_sign(
    c: &Curve, d_bytes: &[u8], hash: &[u8], nonce_k: &[u8],
) -> Result<Vec<u8>, String> {
    use std::cmp::Ordering;
    let d = BigUInt::from_be_bytes(d_bytes);
    let k = BigUInt::from_be_bytes(nonce_k);
    if k.is_zero() || k.cmp(&c.n) != Ordering::Less {
        return Err("ECDSA: nonce k out of range".into());
    }
    if d.is_zero() || d.cmp(&c.n) != Ordering::Less {
        return Err("ECDSA: private key out of range".into());
    }
    // e = leftmost N_bits of hash, then mod n. For P-curves where the
    // hash length matches the field size, this is just OS2IP(hash) mod n.
    // For mismatched sizes WebCrypto still uses OS2IP(hash) mod n.
    let e = BigUInt::from_be_bytes(hash).modulo(&c.n);
    let r_pt = ec_scalar_mul(c, &k, &c.g);
    let x1 = match &r_pt {
        P256Point::Affine { x, .. } => x.clone(),
        P256Point::Identity => return Err("ECDSA: k*G is identity".into()),
    };
    let r = x1.modulo(&c.n);
    if r.is_zero() { return Err("ECDSA: r=0".into()); }
    let k_inv = mod_inv_fermat(&k, &c.n);
    let rd = mod_mul(&r, &d, &c.n);
    let e_plus_rd = mod_add(&e, &rd, &c.n);
    let s = mod_mul(&k_inv, &e_plus_rd, &c.n);
    if s.is_zero() { return Err("ECDSA: s=0".into()); }
    let mut out = Vec::with_capacity(2 * c.coord_bytes);
    out.extend_from_slice(&r.to_be_bytes(c.coord_bytes));
    out.extend_from_slice(&s.to_be_bytes(c.coord_bytes));
    Ok(out)
}

pub fn ecdsa_verify(
    c: &Curve, qx_bytes: &[u8], qy_bytes: &[u8], hash: &[u8], signature: &[u8],
) -> Result<(), String> {
    use std::cmp::Ordering;
    if signature.len() != 2 * c.coord_bytes {
        return Err("ECDSA: signature length mismatch".into());
    }
    let one = BigUInt::one();
    let r = BigUInt::from_be_bytes(&signature[..c.coord_bytes]);
    let s = BigUInt::from_be_bytes(&signature[c.coord_bytes..]);
    if r.cmp(&one) == Ordering::Less || r.cmp(&c.n) != Ordering::Less {
        return Err("ECDSA: r out of range".into());
    }
    if s.cmp(&one) == Ordering::Less || s.cmp(&c.n) != Ordering::Less {
        return Err("ECDSA: s out of range".into());
    }
    let qx = BigUInt::from_be_bytes(qx_bytes);
    let qy = BigUInt::from_be_bytes(qy_bytes);
    if !on_curve(c, &qx, &qy) {
        return Err("ECDSA: public key not on curve".into());
    }
    let q = P256Point::Affine { x: qx, y: qy };
    let e = BigUInt::from_be_bytes(hash).modulo(&c.n);
    let w = mod_inv_fermat(&s, &c.n);
    let u1 = mod_mul(&e, &w, &c.n);
    let u2 = mod_mul(&r, &w, &c.n);
    let p1 = ec_scalar_mul(c, &u1, &c.g);
    let p2 = ec_scalar_mul(c, &u2, &q);
    let r_pt = ec_add(c, &p1, &p2);
    let x1 = match r_pt {
        P256Point::Affine { x, .. } => x,
        P256Point::Identity => return Err("ECDSA: u1·G + u2·Q is identity".into()),
    };
    if x1.modulo(&c.n).cmp(&r) == Ordering::Equal { Ok(()) }
    else { Err("ECDSA: signature mismatch".into()) }
}

pub fn ecdh(c: &Curve, d_bytes: &[u8], qx_bytes: &[u8], qy_bytes: &[u8]) -> Result<Vec<u8>, String> {
    use std::cmp::Ordering;
    let d = BigUInt::from_be_bytes(d_bytes);
    if d.is_zero() || d.cmp(&c.n) != Ordering::Less {
        return Err("ECDH: private scalar out of range".into());
    }
    let qx = BigUInt::from_be_bytes(qx_bytes);
    let qy = BigUInt::from_be_bytes(qy_bytes);
    if !on_curve(c, &qx, &qy) {
        return Err("ECDH: peer public key not on curve".into());
    }
    let q = P256Point::Affine { x: qx, y: qy };
    let shared = ec_scalar_mul(c, &d, &q);
    match shared {
        P256Point::Identity => Err("ECDH: derived point is identity".into()),
        P256Point::Affine { x, .. } => Ok(x.to_be_bytes(c.coord_bytes)),
    }
}

// ─────────────────────── ECDH over P-256 (SEC 1 §3.3.1) ────────────
//
// Pure ECDH: derive a shared secret as the x-coordinate of (d_A · Q_B),
// returned as a 32-byte big-endian octet string. WebCrypto's
// deriveBits({name:"ECDH"}, priv, bitLen) returns the leftmost
// bitLen/8 bytes of this.

pub fn ecdh_p256(d_bytes: &[u8], qx_bytes: &[u8], qy_bytes: &[u8]) -> Result<Vec<u8>, String> {
    use std::cmp::Ordering;
    let n = p256_n();
    let d = BigUInt::from_be_bytes(d_bytes);
    if d.is_zero() || d.cmp(&n) != Ordering::Less {
        return Err("ECDH: private scalar out of range".into());
    }
    let p = p256_p();
    let three = BigUInt::from_be_bytes(&[3]);
    let qx = BigUInt::from_be_bytes(qx_bytes);
    let qy = BigUInt::from_be_bytes(qy_bytes);
    // Validate Q on curve.
    let lhs = mod_mul(&qy, &qy, &p);
    let x3 = mod_mul(&mod_mul(&qx, &qx, &p), &qx, &p);
    let neg3x = mod_mul(&three, &qx, &p);
    let rhs = mod_sub(&mod_add(&x3, &p256_b(), &p), &neg3x, &p);
    if lhs.cmp(&rhs) != Ordering::Equal {
        return Err("ECDH: peer public key not on curve".into());
    }
    let q = P256Point::Affine { x: qx, y: qy };
    let shared = p256_scalar_mul(&d, &q);
    match shared {
        P256Point::Identity => Err("ECDH: derived point is identity (peer key invalid)".into()),
        P256Point::Affine { x, .. } => Ok(x.to_be_bytes(32)),
    }
}

// ─────────────────────── MGF1 + RSA-OAEP (RFC 8017) ───────────────
//
// MGF1: mask generation function based on a hash. RFC 8017 §B.2.1.
// T = || H( mgfSeed || I2OSP(counter, 4) ) for counter in 0..ceil(maskLen/hLen)
// Output is T truncated to maskLen bytes.

pub fn mgf1<F>(mgf_seed: &[u8], mask_len: usize, hash_fn: F, hlen: usize) -> Vec<u8>
where F: Fn(&[u8]) -> Vec<u8>,
{
    let mut t = Vec::with_capacity(mask_len + hlen);
    let n_iters = (mask_len + hlen - 1) / hlen;
    for counter in 0..n_iters {
        let mut input = Vec::with_capacity(mgf_seed.len() + 4);
        input.extend_from_slice(mgf_seed);
        input.extend_from_slice(&(counter as u32).to_be_bytes());
        let h = hash_fn(&input);
        t.extend_from_slice(&h);
    }
    t.truncate(mask_len);
    t
}

/// RSAES-OAEP-ENCRYPT (RFC 8017 §7.1.1). `seed` must be `hlen` bytes
/// of randomness (caller-supplied for testability; production code
/// passes /dev/urandom output). Hash is parameterized via `hash_fn`.
pub fn rsa_oaep_encrypt<F: Fn(&[u8]) -> Vec<u8> + Copy>(
    n_bytes: &[u8], e_bytes: &[u8],
    message: &[u8], label: &[u8], seed: &[u8],
    hash_fn: F, hlen: usize,
) -> Result<Vec<u8>, String> {
    let n = BigUInt::from_be_bytes(n_bytes);
    let e = BigUInt::from_be_bytes(e_bytes);
    let k = n_bytes.len();  // octet length of n; assumes leading zeros are present in n_bytes if needed
    let k = if k == 0 { return Err("RSA-OAEP: empty modulus".into()) } else { k };
    // mLen check: mLen <= k - 2*hLen - 2.
    if message.len() > k.saturating_sub(2 * hlen + 2) {
        return Err("RSA-OAEP: message too long".into());
    }
    if seed.len() != hlen {
        return Err(format!("RSA-OAEP: seed length must be {}", hlen));
    }
    // lHash = Hash(L).
    let l_hash = hash_fn(label);
    // DB = lHash || PS || 0x01 || M  (length k - hLen - 1).
    let ps_len = k - message.len() - 2 * hlen - 2;
    let mut db = Vec::with_capacity(k - hlen - 1);
    db.extend_from_slice(&l_hash);
    db.extend(std::iter::repeat(0u8).take(ps_len));
    db.push(0x01);
    db.extend_from_slice(message);
    debug_assert_eq!(db.len(), k - hlen - 1);
    // dbMask = MGF1(seed, k - hLen - 1).
    let db_mask = mgf1(seed, k - hlen - 1, hash_fn, hlen);
    // maskedDB = DB ⊕ dbMask.
    let masked_db: Vec<u8> = db.iter().zip(db_mask.iter()).map(|(a, b)| a ^ b).collect();
    // seedMask = MGF1(maskedDB, hLen).
    let seed_mask = mgf1(&masked_db, hlen, hash_fn, hlen);
    // maskedSeed = seed ⊕ seedMask.
    let masked_seed: Vec<u8> = seed.iter().zip(seed_mask.iter()).map(|(a, b)| a ^ b).collect();
    // EM = 0x00 || maskedSeed || maskedDB.
    let mut em = Vec::with_capacity(k);
    em.push(0x00);
    em.extend_from_slice(&masked_seed);
    em.extend_from_slice(&masked_db);
    debug_assert_eq!(em.len(), k);
    // m = OS2IP(EM); c = m^e mod n; C = I2OSP(c, k).
    let m_int = BigUInt::from_be_bytes(&em);
    let c_int = rsaep(&n, &e, &m_int)?;
    Ok(c_int.to_be_bytes(k))
}

/// RSAES-OAEP-DECRYPT (RFC 8017 §7.1.2).
pub fn rsa_oaep_decrypt<F: Fn(&[u8]) -> Vec<u8> + Copy>(
    n_bytes: &[u8], d_bytes: &[u8],
    ciphertext: &[u8], label: &[u8],
    hash_fn: F, hlen: usize,
) -> Result<Vec<u8>, String> {
    let n = BigUInt::from_be_bytes(n_bytes);
    let d = BigUInt::from_be_bytes(d_bytes);
    let k = n_bytes.len();
    if ciphertext.len() != k {
        return Err("RSA-OAEP: ciphertext length mismatch".into());
    }
    if k < 2 * hlen + 2 {
        return Err("RSA-OAEP: modulus too small for hash".into());
    }
    // c = OS2IP(C); m = c^d mod n; EM = I2OSP(m, k).
    let c_int = BigUInt::from_be_bytes(ciphertext);
    let m_int = rsadp(&n, &d, &c_int)?;
    let em = m_int.to_be_bytes(k);
    // lHash = Hash(L).
    let l_hash = hash_fn(label);
    // Split EM: Y (1 byte) || maskedSeed (hlen) || maskedDB (k - hlen - 1).
    let y = em[0];
    let masked_seed = &em[1 .. 1 + hlen];
    let masked_db = &em[1 + hlen ..];
    // seedMask = MGF1(maskedDB, hlen). seed = maskedSeed ⊕ seedMask.
    let seed_mask = mgf1(masked_db, hlen, hash_fn, hlen);
    let seed: Vec<u8> = masked_seed.iter().zip(seed_mask.iter()).map(|(a, b)| a ^ b).collect();
    // dbMask = MGF1(seed, k - hlen - 1). DB = maskedDB ⊕ dbMask.
    let db_mask = mgf1(&seed, k - hlen - 1, hash_fn, hlen);
    let db: Vec<u8> = masked_db.iter().zip(db_mask.iter()).map(|(a, b)| a ^ b).collect();
    // Verify structure: DB = lHash' || PS || 0x01 || M, with lHash' == lHash,
    // PS all-zeros, separator 0x01. Constant-time comparison of these checks
    // is the spec recommendation; we use a single boolean accumulator to
    // avoid leaking which check failed first.
    let l_hash_prime = &db[..hlen];
    let rest = &db[hlen..];
    let mut sep_idx: Option<usize> = None;
    for (i, &b) in rest.iter().enumerate() {
        if b == 0x01 && sep_idx.is_none() {
            sep_idx = Some(i);
            break;
        } else if b != 0x00 {
            // First non-zero byte before 0x01 — invalid padding.
            return Err("RSA-OAEP: decryption error".into());
        }
    }
    let sep = sep_idx.ok_or_else(|| "RSA-OAEP: decryption error".to_string())?;
    let ok = y == 0x00 && timing_safe_equal(l_hash_prime, &l_hash);
    if !ok {
        return Err("RSA-OAEP: decryption error".into());
    }
    Ok(rest[sep + 1 ..].to_vec())
}

// ─────────────────────── RSA-PSS (RFC 8017 §8.1 + §9.1) ──────────
//
// Probabilistic Signature Scheme. RSASSA-PSS-SIGN wraps EMSA-PSS-ENCODE
// + RSADP; RSASSA-PSS-VERIFY wraps RSAEP + EMSA-PSS-VERIFY.

fn emsa_pss_encode<F: Fn(&[u8]) -> Vec<u8> + Copy>(
    message: &[u8], em_bits: usize, salt: &[u8], hash_fn: F, hlen: usize,
) -> Result<Vec<u8>, String> {
    let em_len = (em_bits + 7) / 8;
    let s_len = salt.len();
    if em_len < hlen + s_len + 2 {
        return Err("EMSA-PSS-ENCODE: encoding length too short".into());
    }
    let m_hash = hash_fn(message);
    // M'' = (0x00)^8 || mHash || salt
    let mut m_prime = Vec::with_capacity(8 + hlen + s_len);
    m_prime.extend_from_slice(&[0u8; 8]);
    m_prime.extend_from_slice(&m_hash);
    m_prime.extend_from_slice(salt);
    let h = hash_fn(&m_prime);
    // DB = PS || 0x01 || salt
    let mut db = Vec::with_capacity(em_len - hlen - 1);
    db.extend(std::iter::repeat(0u8).take(em_len - s_len - hlen - 2));
    db.push(0x01);
    db.extend_from_slice(salt);
    let db_mask = mgf1(&h, em_len - hlen - 1, hash_fn, hlen);
    let mut masked_db: Vec<u8> = db.iter().zip(db_mask.iter()).map(|(a, b)| a ^ b).collect();
    // Zero leftmost (8*emLen - emBits) bits of maskedDB.
    let unused_bits = 8 * em_len - em_bits;
    if unused_bits > 0 {
        masked_db[0] &= 0xff >> unused_bits;
    }
    // EM = maskedDB || H || 0xbc
    let mut em = Vec::with_capacity(em_len);
    em.extend_from_slice(&masked_db);
    em.extend_from_slice(&h);
    em.push(0xbc);
    Ok(em)
}

fn emsa_pss_verify<F: Fn(&[u8]) -> Vec<u8> + Copy>(
    message: &[u8], em: &[u8], em_bits: usize, s_len: usize, hash_fn: F, hlen: usize,
) -> Result<(), String> {
    let em_len = (em_bits + 7) / 8;
    if em.len() != em_len { return Err("EMSA-PSS-VERIFY: EM length mismatch".into()); }
    if em_len < hlen + s_len + 2 { return Err("EMSA-PSS-VERIFY: inconsistent".into()); }
    if *em.last().unwrap() != 0xbc { return Err("EMSA-PSS-VERIFY: missing 0xbc trailer".into()); }
    let masked_db = &em[..em_len - hlen - 1];
    let h = &em[em_len - hlen - 1 .. em_len - 1];
    let unused_bits = 8 * em_len - em_bits;
    if unused_bits > 0 {
        let mask: u8 = (0xff_u16 << (8 - unused_bits)) as u8;
        if masked_db[0] & mask != 0 {
            return Err("EMSA-PSS-VERIFY: non-zero leftmost bits".into());
        }
    }
    let db_mask = mgf1(h, em_len - hlen - 1, hash_fn, hlen);
    let mut db: Vec<u8> = masked_db.iter().zip(db_mask.iter()).map(|(a, b)| a ^ b).collect();
    if unused_bits > 0 {
        db[0] &= 0xff >> unused_bits;
    }
    // First emLen - hLen - sLen - 2 bytes must be 0x00, then 0x01.
    let ps_len = em_len - hlen - s_len - 2;
    for &b in &db[..ps_len] {
        if b != 0 { return Err("EMSA-PSS-VERIFY: non-zero PS".into()); }
    }
    if db[ps_len] != 0x01 {
        return Err("EMSA-PSS-VERIFY: missing 0x01 separator".into());
    }
    let salt = &db[ps_len + 1 ..];
    let m_hash = hash_fn(message);
    let mut m_prime = Vec::with_capacity(8 + hlen + salt.len());
    m_prime.extend_from_slice(&[0u8; 8]);
    m_prime.extend_from_slice(&m_hash);
    m_prime.extend_from_slice(salt);
    let h_prime = hash_fn(&m_prime);
    if !timing_safe_equal(h, &h_prime) {
        return Err("EMSA-PSS-VERIFY: H mismatch".into());
    }
    Ok(())
}

/// RSASSA-PSS-SIGN (RFC 8017 §8.1.1). `salt` must be `sLen` bytes of
/// randomness (caller-supplied for testability).
pub fn rsa_pss_sign<F: Fn(&[u8]) -> Vec<u8> + Copy>(
    n_bytes: &[u8], d_bytes: &[u8], message: &[u8], salt: &[u8],
    hash_fn: F, hlen: usize,
) -> Result<Vec<u8>, String> {
    let k = n_bytes.len();
    let mod_bits = BigUInt::from_be_bytes(n_bytes).bit_len();
    let em = emsa_pss_encode(message, mod_bits - 1, salt, hash_fn, hlen)?;
    let n = BigUInt::from_be_bytes(n_bytes);
    let d = BigUInt::from_be_bytes(d_bytes);
    let m_int = BigUInt::from_be_bytes(&em);
    let s_int = rsadp(&n, &d, &m_int)?;
    Ok(s_int.to_be_bytes(k))
}

/// RSASSA-PSS-VERIFY (RFC 8017 §8.1.2).
pub fn rsa_pss_verify<F: Fn(&[u8]) -> Vec<u8> + Copy>(
    n_bytes: &[u8], e_bytes: &[u8], message: &[u8], signature: &[u8],
    s_len: usize, hash_fn: F, hlen: usize,
) -> Result<(), String> {
    let k = n_bytes.len();
    if signature.len() != k { return Err("RSA-PSS-VERIFY: signature length mismatch".into()); }
    let n = BigUInt::from_be_bytes(n_bytes);
    let e = BigUInt::from_be_bytes(e_bytes);
    let mod_bits = n.bit_len();
    let s_int = BigUInt::from_be_bytes(signature);
    let m_int = rsaep(&n, &e, &s_int)?;
    let em_len = (mod_bits - 1 + 7) / 8;
    let em = m_int.to_be_bytes(em_len);
    emsa_pss_verify(message, &em, mod_bits - 1, s_len, hash_fn, hlen)
}

// ─────────────────────── AES inverse cipher (FIPS 197 §5.3) ─────────

const AES_INV_SBOX: [u8; 256] = [
    0x52, 0x09, 0x6a, 0xd5, 0x30, 0x36, 0xa5, 0x38, 0xbf, 0x40, 0xa3, 0x9e, 0x81, 0xf3, 0xd7, 0xfb,
    0x7c, 0xe3, 0x39, 0x82, 0x9b, 0x2f, 0xff, 0x87, 0x34, 0x8e, 0x43, 0x44, 0xc4, 0xde, 0xe9, 0xcb,
    0x54, 0x7b, 0x94, 0x32, 0xa6, 0xc2, 0x23, 0x3d, 0xee, 0x4c, 0x95, 0x0b, 0x42, 0xfa, 0xc3, 0x4e,
    0x08, 0x2e, 0xa1, 0x66, 0x28, 0xd9, 0x24, 0xb2, 0x76, 0x5b, 0xa2, 0x49, 0x6d, 0x8b, 0xd1, 0x25,
    0x72, 0xf8, 0xf6, 0x64, 0x86, 0x68, 0x98, 0x16, 0xd4, 0xa4, 0x5c, 0xcc, 0x5d, 0x65, 0xb6, 0x92,
    0x6c, 0x70, 0x48, 0x50, 0xfd, 0xed, 0xb9, 0xda, 0x5e, 0x15, 0x46, 0x57, 0xa7, 0x8d, 0x9d, 0x84,
    0x90, 0xd8, 0xab, 0x00, 0x8c, 0xbc, 0xd3, 0x0a, 0xf7, 0xe4, 0x58, 0x05, 0xb8, 0xb3, 0x45, 0x06,
    0xd0, 0x2c, 0x1e, 0x8f, 0xca, 0x3f, 0x0f, 0x02, 0xc1, 0xaf, 0xbd, 0x03, 0x01, 0x13, 0x8a, 0x6b,
    0x3a, 0x91, 0x11, 0x41, 0x4f, 0x67, 0xdc, 0xea, 0x97, 0xf2, 0xcf, 0xce, 0xf0, 0xb4, 0xe6, 0x73,
    0x96, 0xac, 0x74, 0x22, 0xe7, 0xad, 0x35, 0x85, 0xe2, 0xf9, 0x37, 0xe8, 0x1c, 0x75, 0xdf, 0x6e,
    0x47, 0xf1, 0x1a, 0x71, 0x1d, 0x29, 0xc5, 0x89, 0x6f, 0xb7, 0x62, 0x0e, 0xaa, 0x18, 0xbe, 0x1b,
    0xfc, 0x56, 0x3e, 0x4b, 0xc6, 0xd2, 0x79, 0x20, 0x9a, 0xdb, 0xc0, 0xfe, 0x78, 0xcd, 0x5a, 0xf4,
    0x1f, 0xdd, 0xa8, 0x33, 0x88, 0x07, 0xc7, 0x31, 0xb1, 0x12, 0x10, 0x59, 0x27, 0x80, 0xec, 0x5f,
    0x60, 0x51, 0x7f, 0xa9, 0x19, 0xb5, 0x4a, 0x0d, 0x2d, 0xe5, 0x7a, 0x9f, 0x93, 0xc9, 0x9c, 0xef,
    0xa0, 0xe0, 0x3b, 0x4d, 0xae, 0x2a, 0xf5, 0xb0, 0xc8, 0xeb, 0xbb, 0x3c, 0x83, 0x53, 0x99, 0x61,
    0x17, 0x2b, 0x04, 0x7e, 0xba, 0x77, 0xd6, 0x26, 0xe1, 0x69, 0x14, 0x63, 0x55, 0x21, 0x0c, 0x7d,
];

fn aes_inv_sub_bytes(state: &mut [u8; 16]) {
    for b in state.iter_mut() { *b = AES_INV_SBOX[*b as usize]; }
}

fn aes_inv_shift_rows(state: &mut [u8; 16]) {
    let s = *state;
    for r in 1..4 {
        for c in 0..4 {
            // inverse: column c gets value from column (c + 4 - r) % 4
            state[r * 4 + c] = s[r * 4 + (c + 4 - r) % 4];
        }
    }
}

fn gf_mul(mut a: u8, mut b: u8) -> u8 {
    let mut p = 0u8;
    for _ in 0..8 {
        if b & 1 != 0 { p ^= a; }
        let hi = a & 0x80;
        a <<= 1;
        if hi != 0 { a ^= 0x1b; }
        b >>= 1;
    }
    p
}

fn aes_inv_mix_columns(state: &mut [u8; 16]) {
    for c in 0..4 {
        let s0 = state[c]; let s1 = state[4 + c];
        let s2 = state[8 + c]; let s3 = state[12 + c];
        state[c]      = gf_mul(0x0e, s0) ^ gf_mul(0x0b, s1) ^ gf_mul(0x0d, s2) ^ gf_mul(0x09, s3);
        state[4 + c]  = gf_mul(0x09, s0) ^ gf_mul(0x0e, s1) ^ gf_mul(0x0b, s2) ^ gf_mul(0x0d, s3);
        state[8 + c]  = gf_mul(0x0d, s0) ^ gf_mul(0x09, s1) ^ gf_mul(0x0e, s2) ^ gf_mul(0x0b, s3);
        state[12 + c] = gf_mul(0x0b, s0) ^ gf_mul(0x0d, s1) ^ gf_mul(0x09, s2) ^ gf_mul(0x0e, s3);
    }
}

fn aes_decrypt_block(block: &[u8; 16], w: &[u32]) -> [u8; 16] {
    let nr = w.len() / 4 - 1;
    let mut state = [0u8; 16];
    for c in 0..4 {
        for r in 0..4 { state[r * 4 + c] = block[4 * c + r]; }
    }
    aes_add_round_key(&mut state, &w[4 * nr .. 4 * nr + 4]);
    for round in (1..nr).rev() {
        aes_inv_shift_rows(&mut state);
        aes_inv_sub_bytes(&mut state);
        aes_add_round_key(&mut state, &w[4 * round .. 4 * round + 4]);
        aes_inv_mix_columns(&mut state);
    }
    aes_inv_shift_rows(&mut state);
    aes_inv_sub_bytes(&mut state);
    aes_add_round_key(&mut state, &w[0..4]);
    let mut out = [0u8; 16];
    for c in 0..4 {
        for r in 0..4 { out[4 * c + r] = state[r * 4 + c]; }
    }
    out
}

pub fn aes_decrypt_block_with_key(key: &[u8], block: &[u8; 16]) -> [u8; 16] {
    let w = aes_key_expansion(key);
    aes_decrypt_block(block, &w)
}

// ─────────────────────── AES-CBC (SP 800-38A §6.2) ──────────────────
//
// PKCS#7 padding per RFC 5652 §6.3 — matches WebCrypto AES-CBC.

pub fn aes_cbc_encrypt(key: &[u8], iv: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 16 && key.len() != 24 && key.len() != 32 {
        return Err(format!("AES-CBC: invalid key length {}", key.len()));
    }
    if iv.len() != 16 { return Err("AES-CBC: IV must be 16 bytes".to_string()); }
    let w = aes_key_expansion(key);
    let pad = 16 - (plaintext.len() % 16);
    let mut padded = plaintext.to_vec();
    padded.extend(std::iter::repeat(pad as u8).take(pad));
    let mut prev = [0u8; 16];
    prev.copy_from_slice(iv);
    let mut out = Vec::with_capacity(padded.len());
    for chunk in padded.chunks(16) {
        let mut block = [0u8; 16];
        for i in 0..16 { block[i] = chunk[i] ^ prev[i]; }
        let c = aes_encrypt_block(&block, &w);
        out.extend_from_slice(&c);
        prev = c;
    }
    Ok(out)
}

pub fn aes_cbc_decrypt(key: &[u8], iv: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 16 && key.len() != 24 && key.len() != 32 {
        return Err(format!("AES-CBC: invalid key length {}", key.len()));
    }
    if iv.len() != 16 { return Err("AES-CBC: IV must be 16 bytes".to_string()); }
    if ciphertext.is_empty() || ciphertext.len() % 16 != 0 {
        return Err("AES-CBC: ciphertext must be a positive multiple of 16 bytes".to_string());
    }
    let w = aes_key_expansion(key);
    let mut prev = [0u8; 16];
    prev.copy_from_slice(iv);
    let mut out = Vec::with_capacity(ciphertext.len());
    for chunk in ciphertext.chunks(16) {
        let mut block = [0u8; 16];
        block.copy_from_slice(chunk);
        let d = aes_decrypt_block(&block, &w);
        let mut plain = [0u8; 16];
        for i in 0..16 { plain[i] = d[i] ^ prev[i]; }
        out.extend_from_slice(&plain);
        prev = block;
    }
    // PKCS#7 unpad.
    let pad = *out.last().ok_or("AES-CBC: empty output")? as usize;
    if pad == 0 || pad > 16 { return Err("AES-CBC: bad padding".to_string()); }
    if out.len() < pad { return Err("AES-CBC: bad padding".to_string()); }
    let n = out.len();
    for &b in &out[n - pad ..] {
        if b as usize != pad { return Err("AES-CBC: bad padding".to_string()); }
    }
    out.truncate(n - pad);
    Ok(out)
}

// ─────────────────────── AES-CTR (SP 800-38A §6.5) ──────────────────
//
// WebCrypto AES-CTR uses a 16-byte counter block where the last
// `length` bits are the counter (incremented per block) and the rest
// is the nonce prefix.

pub fn aes_ctr_xor_with_key(key: &[u8], counter0: &[u8], counter_bits: u32, data: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 16 && key.len() != 24 && key.len() != 32 {
        return Err(format!("AES-CTR: invalid key length {}", key.len()));
    }
    if counter0.len() != 16 { return Err("AES-CTR: counter must be 16 bytes".to_string()); }
    if counter_bits == 0 || counter_bits > 128 {
        return Err("AES-CTR: length must be in 1..=128".to_string());
    }
    let w = aes_key_expansion(key);
    let mut counter = [0u8; 16];
    counter.copy_from_slice(counter0);
    let mut out = Vec::with_capacity(data.len());
    let total_blocks = (data.len() + 15) / 16;
    let mut block_idx = 0u64;
    for chunk in data.chunks(16) {
        let ks = aes_encrypt_block(&counter, &w);
        for (i, b) in chunk.iter().enumerate() {
            out.push(b ^ ks[i]);
        }
        block_idx += 1;
        if block_idx as usize == total_blocks { break; }
        // Increment the low `counter_bits` of the counter block per
        // SP 800-38A §B.1. Modulo 2^counter_bits, wrap allowed.
        counter_inc(&mut counter, counter_bits as usize);
    }
    Ok(out)
}

fn counter_inc(counter: &mut [u8; 16], bits: usize) {
    // counter occupies the low `bits` bits of the 128-bit block (the
    // tail end of the byte array). Increment modulo 2^bits.
    let mut remaining = bits;
    let mut idx = 15;
    let mut carry: u16 = 1;
    while remaining > 0 && carry != 0 {
        let take = remaining.min(8);
        let mask: u16 = if take == 8 { 0xff } else { (1u16 << take) - 1 };
        let low = (counter[idx] as u16) & mask;
        let high = (counter[idx] as u16) & !mask;
        let sum = low + carry;
        let new_low = sum & mask;
        counter[idx] = (high | new_low) as u8;
        carry = sum >> take;
        remaining -= take;
        if idx == 0 { break; }
        idx -= 1;
    }
}

// ─────────────────────── AES-KW (RFC 3394) ──────────────────────────
//
// AES Key Wrap — the symmetric KEK-wrapping algorithm used in JWE
// A128KW / A256KW. Requires the wrapped key length to be a positive
// multiple of 8 bytes (64 bits) and ≥ 8 bytes.

const AES_KW_IV: [u8; 8] = [0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0xa6];

pub fn aes_kw_wrap(kek: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    if kek.len() != 16 && kek.len() != 24 && kek.len() != 32 {
        return Err(format!("AES-KW: invalid KEK length {}", kek.len()));
    }
    if plaintext.len() % 8 != 0 || plaintext.is_empty() {
        return Err("AES-KW: plaintext must be a positive multiple of 8 bytes".to_string());
    }
    let n = plaintext.len() / 8;
    let w = aes_key_expansion(kek);
    let mut a = AES_KW_IV;
    let mut r: Vec<[u8; 8]> = (0..n)
        .map(|i| {
            let mut b = [0u8; 8];
            b.copy_from_slice(&plaintext[i * 8 .. (i + 1) * 8]);
            b
        })
        .collect();
    for j in 0..6 {
        for i in 0..n {
            let mut b = [0u8; 16];
            b[..8].copy_from_slice(&a);
            b[8..].copy_from_slice(&r[i]);
            let enc = aes_encrypt_block(&b, &w);
            a.copy_from_slice(&enc[..8]);
            let t = ((n * j) + i + 1) as u64;
            let t_be = t.to_be_bytes();
            for k in 0..8 { a[k] ^= t_be[k]; }
            r[i].copy_from_slice(&enc[8..]);
        }
    }
    let mut out = Vec::with_capacity(8 * (n + 1));
    out.extend_from_slice(&a);
    for block in &r { out.extend_from_slice(block); }
    Ok(out)
}

pub fn aes_kw_unwrap(kek: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    if kek.len() != 16 && kek.len() != 24 && kek.len() != 32 {
        return Err(format!("AES-KW: invalid KEK length {}", kek.len()));
    }
    if ciphertext.len() % 8 != 0 || ciphertext.len() < 16 {
        return Err("AES-KW: ciphertext must be a multiple of 8 bytes ≥ 16".to_string());
    }
    let n = ciphertext.len() / 8 - 1;
    let w = aes_key_expansion(kek);
    let mut a = [0u8; 8];
    a.copy_from_slice(&ciphertext[..8]);
    let mut r: Vec<[u8; 8]> = (0..n)
        .map(|i| {
            let mut b = [0u8; 8];
            b.copy_from_slice(&ciphertext[8 + i * 8 .. 8 + (i + 1) * 8]);
            b
        })
        .collect();
    for j in (0..6).rev() {
        for i in (0..n).rev() {
            let t = ((n * j) + i + 1) as u64;
            let t_be = t.to_be_bytes();
            let mut b = [0u8; 16];
            for k in 0..8 { b[k] = a[k] ^ t_be[k]; }
            b[8..].copy_from_slice(&r[i]);
            let dec = aes_decrypt_block(&b, &w);
            a.copy_from_slice(&dec[..8]);
            r[i].copy_from_slice(&dec[8..]);
        }
    }
    if !timing_safe_equal(&a, &AES_KW_IV) {
        return Err("AES-KW: integrity check failed".to_string());
    }
    let mut out = Vec::with_capacity(8 * n);
    for block in &r { out.extend_from_slice(block); }
    Ok(out)
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

// ──────────────────────── Blake2b (RFC 7693) ────────────────────────
//
// 64-bit BLAKE2b: produces 1-64-byte hashes, optional 0-64-byte key.
// Block size 128 bytes; 12 rounds of mixing per compression call.
// Used by Argon2id (RFC 9106) for password hashing.

const BLAKE2B_IV: [u64; 8] = [
    0x6a09e667f3bcc908, 0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
    0x510e527fade682d1, 0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
];

// SIGMA permutation table (RFC 7693 §2.7).
const BLAKE2B_SIGMA: [[usize; 16]; 12] = [
    [ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [ 7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [ 9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [ 2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [ 6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
    [ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
];

fn blake2b_mix(v: &mut [u64; 16], a: usize, b: usize, c: usize, d: usize, x: u64, y: u64) {
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
    v[d] = (v[d] ^ v[a]).rotate_right(32);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(24);
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
    v[d] = (v[d] ^ v[a]).rotate_right(16);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(63);
}

fn blake2b_compress(h: &mut [u64; 8], block: &[u8; 128], t: u128, last: bool) {
    let mut v: [u64; 16] = [0; 16];
    v[..8].copy_from_slice(h);
    v[8..].copy_from_slice(&BLAKE2B_IV);
    v[12] ^= t as u64;
    v[13] ^= (t >> 64) as u64;
    if last { v[14] = !v[14]; }
    let mut m: [u64; 16] = [0; 16];
    for i in 0..16 {
        let off = i * 8;
        m[i] = u64::from_le_bytes([
            block[off], block[off+1], block[off+2], block[off+3],
            block[off+4], block[off+5], block[off+6], block[off+7],
        ]);
    }
    for round in 0..12 {
        let s = &BLAKE2B_SIGMA[round];
        blake2b_mix(&mut v, 0, 4,  8, 12, m[s[ 0]], m[s[ 1]]);
        blake2b_mix(&mut v, 1, 5,  9, 13, m[s[ 2]], m[s[ 3]]);
        blake2b_mix(&mut v, 2, 6, 10, 14, m[s[ 4]], m[s[ 5]]);
        blake2b_mix(&mut v, 3, 7, 11, 15, m[s[ 6]], m[s[ 7]]);
        blake2b_mix(&mut v, 0, 5, 10, 15, m[s[ 8]], m[s[ 9]]);
        blake2b_mix(&mut v, 1, 6, 11, 12, m[s[10]], m[s[11]]);
        blake2b_mix(&mut v, 2, 7,  8, 13, m[s[12]], m[s[13]]);
        blake2b_mix(&mut v, 3, 4,  9, 14, m[s[14]], m[s[15]]);
    }
    for i in 0..8 { h[i] ^= v[i] ^ v[i + 8]; }
}

/// Blake2b hash with variable output length (1..=64) and optional key (0..=64).
pub fn blake2b(input: &[u8], key: &[u8], out_len: usize) -> Result<Vec<u8>, String> {
    if out_len == 0 || out_len > 64 { return Err("blake2b: out_len must be 1..=64".into()); }
    if key.len() > 64 { return Err("blake2b: key length must be 0..=64".into()); }
    let mut h = BLAKE2B_IV;
    // Parameter block (RFC 7693 §2.5): low byte order is
    //   digest_length || key_length || fanout || depth.
    h[0] ^= 0x01010000 | ((key.len() as u64) << 8) | (out_len as u64);
    // Prepend the key padded to a full block (RFC 7693 §3.3).
    let mut buf: Vec<u8> = Vec::new();
    if !key.is_empty() {
        let mut padded = [0u8; 128];
        padded[..key.len()].copy_from_slice(key);
        buf.extend_from_slice(&padded);
    }
    buf.extend_from_slice(input);
    // If buf is empty (no key AND no input), the final block is all-zero with t=0 + last=true.
    // Process all but the final block as non-last.
    let mut t: u128 = 0;
    let mut i = 0;
    while i + 128 < buf.len() {
        let mut block = [0u8; 128];
        block.copy_from_slice(&buf[i..i + 128]);
        t = t.wrapping_add(128);
        blake2b_compress(&mut h, &block, t, false);
        i += 128;
    }
    // Final block (may be partial, padded with zeros).
    let remaining = buf.len() - i;
    let mut last_block = [0u8; 128];
    last_block[..remaining].copy_from_slice(&buf[i..]);
    t = t.wrapping_add(remaining as u128);
    blake2b_compress(&mut h, &last_block, t, true);
    // Output the first out_len bytes (LE).
    let mut out = Vec::with_capacity(out_len);
    for word in h.iter().take((out_len + 7) / 8) {
        for b in word.to_le_bytes() {
            if out.len() < out_len { out.push(b); }
        }
    }
    out.truncate(out_len);
    Ok(out)
}


// ──────────────────────── Argon2id (RFC 9106) ──────────────────────
// Single-lane (p=1) implementation. Hybrid indexing: Argon2i for pass 0
// slices 0 and 1; Argon2d after. Composes on the Blake2b primitive above.

const ARGON2_VERSION: u32 = 0x13;
const ARGON2ID_TYPE: u32 = 2;
const ARGON2_BLOCK_SIZE: usize = 1024;
const ARGON2_QWORDS: usize = 128;
const ARGON2_SYNC_POINTS: usize = 4;

#[derive(Debug, Clone)]
pub struct Argon2idParams {
    pub t_cost: u32,      // iterations
    pub m_kib: u32,       // memory in KiB (each block = 1 KiB)
    pub parallelism: u32, // must be 1 here
    pub tau: u32,         // output length in bytes
}

#[derive(Debug)]
pub enum Argon2Error { InvalidParam(&'static str), Crypto(String) }
impl std::fmt::Display for Argon2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Argon2Error::InvalidParam(s) => write!(f, "argon2: {}", s),
            Argon2Error::Crypto(s) => write!(f, "argon2 crypto: {}", s),
        }
    }
}
impl std::error::Error for Argon2Error {}

type Block = [u64; ARGON2_QWORDS];
#[inline] fn block_zero() -> Block { [0u64; ARGON2_QWORDS] }
fn block_from_bytes(b: &[u8]) -> Block {
    let mut r = block_zero();
    for (i, c) in b.chunks_exact(8).enumerate().take(ARGON2_QWORDS) {
        r[i] = u64::from_le_bytes(c.try_into().unwrap());
    }
    r
}
fn block_to_bytes(b: &Block) -> Vec<u8> {
    let mut o = Vec::with_capacity(ARGON2_BLOCK_SIZE);
    for &w in b { o.extend_from_slice(&w.to_le_bytes()); }
    o
}
fn block_xor(a: &Block, b: &Block) -> Block {
    let mut r = block_zero();
    for i in 0..ARGON2_QWORDS { r[i] = a[i] ^ b[i]; }
    r
}

/// Variable-length BLAKE2b "H' " per RFC 9106 §3.3.
pub fn argon2_h_prime(input: &[u8], tau: u32) -> Result<Vec<u8>, Argon2Error> {
    let mut tagged = Vec::with_capacity(4 + input.len());
    tagged.extend_from_slice(&tau.to_le_bytes());
    tagged.extend_from_slice(input);
    if tau <= 64 {
        return blake2b(&tagged, &[], tau as usize).map_err(Argon2Error::Crypto);
    }
    let r = ((tau + 31) / 32) as usize - 2;
    let mut out = Vec::with_capacity(tau as usize);
    let mut v = blake2b(&tagged, &[], 64).map_err(Argon2Error::Crypto)?;
    out.extend_from_slice(&v[..32]);
    for _ in 1..r {
        v = blake2b(&v, &[], 64).map_err(Argon2Error::Crypto)?;
        out.extend_from_slice(&v[..32]);
    }
    let final_len = (tau as usize) - 32 * r;
    let vf = blake2b(&v, &[], final_len).map_err(Argon2Error::Crypto)?;
    out.extend_from_slice(&vf);
    Ok(out)
}

/// BLAKE2b-style G with multiplication step (RFC 9106 §3.5).
#[inline]
fn gb(v: &mut [u64; 16], a: usize, b: usize, c: usize, d: usize) {
    let add = |x: u64, y: u64| {
        let lx = x & 0xFFFFFFFF; let ly = y & 0xFFFFFFFF;
        x.wrapping_add(y).wrapping_add(2u64.wrapping_mul(lx).wrapping_mul(ly))
    };
    v[a] = add(v[a], v[b]); v[d] = (v[d] ^ v[a]).rotate_right(32);
    v[c] = add(v[c], v[d]); v[b] = (v[b] ^ v[c]).rotate_right(24);
    v[a] = add(v[a], v[b]); v[d] = (v[d] ^ v[a]).rotate_right(16);
    v[c] = add(v[c], v[d]); v[b] = (v[b] ^ v[c]).rotate_right(63);
}

/// Permutation P (RFC 9106 §3.5) over 16 u64s.
fn permute_p(v: &mut [u64; 16]) {
    gb(v, 0, 4,  8, 12);
    gb(v, 1, 5,  9, 13);
    gb(v, 2, 6, 10, 14);
    gb(v, 3, 7, 11, 15);
    gb(v, 0, 5, 10, 15);
    gb(v, 1, 6, 11, 12);
    gb(v, 2, 7,  8, 13);
    gb(v, 3, 4,  9, 14);
}

/// G(X, Y) compression. R = X^Y, apply P to 8 rows then 8 columns of R,
/// final = result ^ R.
fn compress_g(x: &Block, y: &Block) -> Block {
    let r = block_xor(x, y);
    let mut z = r;
    // 8 rows of 16 u64s each.
    for row in 0..8 {
        let off = row * 16;
        let mut v = [0u64; 16];
        v.copy_from_slice(&z[off..off + 16]);
        permute_p(&mut v);
        z[off..off + 16].copy_from_slice(&v);
    }
    // 8 columns: column c spans 16 u64s, taking 2 consecutive words from each row.
    for col in 0..8 {
        let mut v = [0u64; 16];
        for row in 0..8 {
            v[row * 2] = z[row * 16 + col * 2];
            v[row * 2 + 1] = z[row * 16 + col * 2 + 1];
        }
        permute_p(&mut v);
        for row in 0..8 {
            z[row * 16 + col * 2] = v[row * 2];
            z[row * 16 + col * 2 + 1] = v[row * 2 + 1];
        }
    }
    block_xor(&z, &r)
}

/// Build an Argon2i address block (RFC 9106 §3.4.1.2). The block at
/// `counter` provides 128 (J1, J2) pseudo-random pairs.
fn argon2i_address_block(
    pass: u64, lane: u64, slice: u64, total_blocks: u64,
    total_passes: u64, ty: u64, counter: u64,
) -> Block {
    let zero = block_zero();
    let mut input = block_zero();
    input[0] = pass;
    input[1] = lane;
    input[2] = slice;
    input[3] = total_blocks;
    input[4] = total_passes;
    input[5] = ty;
    input[6] = counter;
    // First G with zero||input, then G(zero, first) to randomize.
    let first = compress_g(&zero, &input);
    compress_g(&zero, &first)
}

/// Map a 32-bit pseudo-random J1 onto [0, ref_area) using the
/// non-uniform mapping of RFC 9106 §3.4.
fn map_index(j1: u32, ref_area_size: usize) -> usize {
    let x = ((j1 as u64).wrapping_mul(j1 as u64)) >> 32;
    let y = ((ref_area_size as u64).wrapping_mul(x)) >> 32;
    (ref_area_size as u64).wrapping_sub(1).wrapping_sub(y) as usize
}

/// Argon2id KDF per RFC 9106 — single lane.
pub fn argon2id_hash(
    password: &[u8], salt: &[u8], params: &Argon2idParams,
) -> Result<Vec<u8>, Argon2Error> {
    if params.parallelism != 1 { return Err(Argon2Error::InvalidParam("p=1 only")); }
    if params.t_cost < 1 { return Err(Argon2Error::InvalidParam("t >= 1")); }
    if params.tau < 4 { return Err(Argon2Error::InvalidParam("tau >= 4")); }
    if salt.len() < 8 { return Err(Argon2Error::InvalidParam("salt >= 8")); }
    if params.m_kib < 8 { return Err(Argon2Error::InvalidParam("m >= 8")); }

    let p = params.parallelism;
    let tau = params.tau;
    let t = params.t_cost;
    // m' = 4 * p * floor(m / (4*p))
    let m_prime = 4 * p * (params.m_kib / (4 * p));
    let q = (m_prime / p) as usize;
    let segment_length = q / ARGON2_SYNC_POINTS;

    // H₀
    let mut h0_in = Vec::new();
    h0_in.extend_from_slice(&p.to_le_bytes());
    h0_in.extend_from_slice(&tau.to_le_bytes());
    h0_in.extend_from_slice(&params.m_kib.to_le_bytes());
    h0_in.extend_from_slice(&t.to_le_bytes());
    h0_in.extend_from_slice(&ARGON2_VERSION.to_le_bytes());
    h0_in.extend_from_slice(&ARGON2ID_TYPE.to_le_bytes());
    h0_in.extend_from_slice(&(password.len() as u32).to_le_bytes());
    h0_in.extend_from_slice(password);
    h0_in.extend_from_slice(&(salt.len() as u32).to_le_bytes());
    h0_in.extend_from_slice(salt);
    h0_in.extend_from_slice(&0u32.to_le_bytes()); // |K|
    h0_in.extend_from_slice(&0u32.to_le_bytes()); // |X|
    let h0 = blake2b(&h0_in, &[], 64).map_err(Argon2Error::Crypto)?;

    // Memory buffer.
    let mut mem: Vec<Block> = vec![block_zero(); q];

    // Initial two blocks of the lane.
    let mut ext = h0.clone();
    ext.extend_from_slice(&0u32.to_le_bytes());
    ext.extend_from_slice(&0u32.to_le_bytes());
    mem[0] = block_from_bytes(&argon2_h_prime(&ext, ARGON2_BLOCK_SIZE as u32)?);
    let mut ext = h0.clone();
    ext.extend_from_slice(&1u32.to_le_bytes());
    ext.extend_from_slice(&0u32.to_le_bytes());
    mem[1] = block_from_bytes(&argon2_h_prime(&ext, ARGON2_BLOCK_SIZE as u32)?);

    for pass in 0..t as u64 {
        for slice in 0..ARGON2_SYNC_POINTS as u64 {
            // Argon2id: Argon2i indexing iff (pass == 0 && slice < 2).
            let use_argon2i = pass == 0 && slice < 2;
            let seg_start = (slice as usize) * segment_length;

            // For Argon2i, prepare a rolling address block; refresh every 128 blocks.
            let mut addr_block = block_zero();
            let mut addr_counter: u64 = 0;
            if use_argon2i {
                addr_counter = 1;
                addr_block = argon2i_address_block(
                    pass, 0, slice, q as u64, t as u64, ARGON2ID_TYPE as u64, addr_counter,
                );
            }

            let start_in_seg = if pass == 0 && slice == 0 { 2 } else { 0 };
            for idx_in_seg in start_in_seg..segment_length {
                let j_abs = seg_start + idx_in_seg;
                // Refresh Argon2i address block every 128 entries.
                if use_argon2i && idx_in_seg > 0 && idx_in_seg % ARGON2_QWORDS == 0 {
                    addr_counter += 1;
                    addr_block = argon2i_address_block(
                        pass, 0, slice, q as u64, t as u64, ARGON2ID_TYPE as u64, addr_counter,
                    );
                }
                let prev_idx = if j_abs == 0 { q - 1 } else { j_abs - 1 };
                let prev_block = mem[prev_idx];

                // Pseudo-random word.
                let pseudo = if use_argon2i {
                    addr_block[idx_in_seg % ARGON2_QWORDS]
                } else {
                    prev_block[0]
                };
                let j1 = (pseudo & 0xFFFFFFFF) as u32;
                // J2 (lane selector) ignored for p=1.

                // Reference area: all blocks already computed in the lane,
                // excluding the previous block.
                let ref_area_size: usize = if pass == 0 {
                    j_abs - 1
                } else {
                    // Whole lane minus current segment, plus already-computed
                    // blocks in current segment, minus 1.
                    q - segment_length + idx_in_seg - 1
                };
                if ref_area_size == 0 { continue; }

                let rel = map_index(j1, ref_area_size);
                let ref_index = if pass == 0 {
                    rel
                } else {
                    let start = ((slice as usize + 1) * segment_length) % q;
                    (start + rel) % q
                };
                let ref_block = mem[ref_index];

                let new_block = compress_g(&prev_block, &ref_block);
                if pass == 0 {
                    mem[j_abs] = new_block;
                } else {
                    mem[j_abs] = block_xor(&mem[j_abs], &new_block);
                }
            }
        }
    }

    // Final = B[lane=0][q-1]; tag = H'(C, tau).
    let c_bytes = block_to_bytes(&mem[q - 1]);
    argon2_h_prime(&c_bytes, tau)
}
