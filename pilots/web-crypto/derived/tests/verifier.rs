// Verifier for the web-crypto pilot.

use rusty_web_crypto::*;

// ════════════════════ SHA-256 ════════════════════

// FIPS 180-4 / NIST test vectors

#[test]
fn cd_sha256_empty_string() {
    // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    assert_eq!(
        digest_sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn cd_sha256_abc() {
    // SHA-256("abc") = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
    assert_eq!(
        digest_sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn cd_sha256_long_message() {
    // SHA-256("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq")
    // = 248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1
    assert_eq!(
        digest_sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

#[test]
fn cd_sha256_one_million_a() {
    // SHA-256(1,000,000 'a' bytes) = cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0
    let data = vec![b'a'; 1_000_000];
    assert_eq!(
        digest_sha256_hex(&data),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn spec_sha256_returns_32_bytes() {
    let h = digest_sha256(b"test");
    assert_eq!(h.len(), 32);
}

#[test]
fn spec_sha256_deterministic() {
    let a = digest_sha256(b"hello world");
    let b = digest_sha256(b"hello world");
    assert_eq!(a, b);
}

#[test]
fn spec_sha256_avalanche() {
    // Tiny input change must cascade to many output bits.
    let a = digest_sha256(b"hello world");
    let b = digest_sha256(b"hello worle");  // changed one byte
    let differing_bytes = a.iter().zip(b.iter()).filter(|(x, y)| x != y).count();
    // Expect roughly half the bytes to differ
    assert!(differing_bytes > 16, "avalanche too weak: {} bytes differ", differing_bytes);
}

// ════════════════════ subtle.digest ════════════════════

#[test]
fn cd_subtle_digest_sha256_basic() {
    let r = subtle::digest("SHA-256", b"abc").unwrap();
    let hex: String = r.iter().map(|b| format!("{:02x}", b)).collect();
    assert_eq!(hex, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}

#[test]
fn spec_subtle_digest_algorithm_case_insensitive() {
    for alg in ["SHA-256", "sha-256", "SHA256", "sha256"] {
        let r = subtle::digest(alg, b"x");
        assert!(r.is_ok(), "{} should be accepted", alg);
    }
}

#[test]
fn spec_subtle_digest_unknown_algorithm_errors() {
    let r = subtle::digest("MD5", b"x");
    assert!(r.is_err());
}

// ════════════════════ UUID v4 ════════════════════

#[test]
fn cd_uuid_v4_format() {
    let u = random_uuid_v4();
    assert_eq!(u.len(), 36);
    let parts: Vec<&str> = u.split('-').collect();
    assert_eq!(parts.len(), 5);
    assert_eq!(parts[0].len(), 8);
    assert_eq!(parts[1].len(), 4);
    assert_eq!(parts[2].len(), 4);
    assert_eq!(parts[3].len(), 4);
    assert_eq!(parts[4].len(), 12);
}

#[test]
fn spec_uuid_v4_version_field() {
    // Per RFC 4122 §4.4: version digit (first char of group 3) is "4"
    let u = random_uuid_v4();
    let parts: Vec<&str> = u.split('-').collect();
    assert_eq!(&parts[2][0..1], "4");
}

#[test]
fn spec_uuid_v4_variant_field() {
    // Per RFC 4122 §4.4: variant high bits are 10xx, so first hex digit of
    // group 4 is one of {8, 9, a, b}.
    let u = random_uuid_v4();
    let parts: Vec<&str> = u.split('-').collect();
    let v = parts[3].chars().next().unwrap();
    assert!(matches!(v, '8' | '9' | 'a' | 'b'),
        "variant should be 8/9/a/b, got {}", v);
}

#[test]
fn spec_uuid_v4_unique_with_high_probability() {
    let a = random_uuid_v4();
    let b = random_uuid_v4();
    assert_ne!(a, b);
}

// ════════════════════ getRandomValues ════════════════════

#[test]
fn cd_get_random_values_fills_buffer() {
    let mut buf = [0u8; 32];
    get_random_values(&mut buf).unwrap();
    // Statistical sanity: not all zeros after fill.
    assert!(buf.iter().any(|&b| b != 0));
}

#[test]
fn spec_get_random_values_independent_calls() {
    let mut a = [0u8; 32];
    let mut b = [0u8; 32];
    get_random_values(&mut a).unwrap();
    get_random_values(&mut b).unwrap();
    assert_ne!(a, b);
}

#[test]
fn spec_get_random_values_empty_buffer() {
    let mut buf: [u8; 0] = [];
    // Should succeed with no-op.
    assert!(get_random_values(&mut buf).is_ok());
}

// ════════════════════ timing-safe equal ════════════════════

#[test]
fn cd_timing_safe_equal_match() {
    assert!(timing_safe_equal(b"hello", b"hello"));
}

#[test]
fn cd_timing_safe_equal_no_match() {
    assert!(!timing_safe_equal(b"hello", b"world"));
}

#[test]
fn spec_timing_safe_equal_different_lengths() {
    assert!(!timing_safe_equal(b"a", b"ab"));
    assert!(!timing_safe_equal(b"abc", b"ab"));
}

#[test]
fn spec_timing_safe_equal_empty() {
    assert!(timing_safe_equal(b"", b""));
}

#[test]
fn spec_timing_safe_equal_one_bit_diff() {
    assert!(!timing_safe_equal(&[0x00], &[0x01]));
    assert!(!timing_safe_equal(&[0xFF, 0xFF], &[0xFF, 0xFE]));
}
