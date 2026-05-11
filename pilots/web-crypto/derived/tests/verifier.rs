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

// HMAC-SHA-256 RFC 4231 + ad-hoc canonical test vectors.

#[test]
fn hmac_sha256_short_key() {
    // RFC 4231 Test Case 1:
    //   Key  = 0x0b * 20
    //   Data = "Hi There"
    //   HMAC = b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7
    let key = [0x0bu8; 20];
    let data = b"Hi There";
    let out = rusty_web_crypto::hmac_sha256(&key, data);
    let mut hex = String::new();
    for b in &out { hex.push_str(&format!("{:02x}", b)); }
    assert_eq!(hex, "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7");
}

#[test]
fn hmac_sha256_oversize_key_gets_hashed() {
    // RFC 4231 Test Case 4:
    //   Key  = 0x0102030405060708090a0b0c0d0e0f10111213141516171819 (25 bytes)
    //   Data = 0xcd * 50
    //   HMAC = 82558a389a443c0ea4cc819899f2083a85f0faa3e578f8077a2e3ff46729665b
    let key: Vec<u8> = (1u8..=25).collect();
    let data = vec![0xcdu8; 50];
    let out = rusty_web_crypto::hmac_sha256(&key, &data);
    let mut hex = String::new();
    for b in &out { hex.push_str(&format!("{:02x}", b)); }
    assert_eq!(hex, "82558a389a443c0ea4cc819899f2083a85f0faa3e578f8077a2e3ff46729665b");
}

#[test]
fn hmac_sha256_long_key_truncated_via_hash() {
    // Key longer than block size (64 bytes) is first hashed to 32 bytes.
    // Verify by feeding 128-byte key and ensuring it equals HMAC with the
    // pre-hashed 32-byte key.
    let long_key = vec![0xaau8; 128];
    let data = b"test message";
    let direct = rusty_web_crypto::hmac_sha256(&long_key, data);
    let hashed_key = rusty_web_crypto::digest_sha256(&long_key);
    let via_hashed = rusty_web_crypto::hmac_sha256(&hashed_key, data);
    assert_eq!(direct, via_hashed);
}

#[test]
fn hmac_sha256_differs_for_one_bit_message_change() {
    let key = b"shared-secret-key";
    let a = rusty_web_crypto::hmac_sha256(key, b"hello");
    let b = rusty_web_crypto::hmac_sha256(key, b"hellp");  // one bit different
    assert_ne!(a, b);
}

// SHA-1 + HMAC-SHA-1 known-answer vectors.

#[test]
fn sha1_empty_string() {
    // FIPS 180-1 spec: SHA-1("") = da39a3ee5e6b4b0d3255bfef95601890afd80709
    let out = rusty_web_crypto::digest_sha1_hex(b"");
    assert_eq!(out, "da39a3ee5e6b4b0d3255bfef95601890afd80709");
}

#[test]
fn sha1_abc() {
    // FIPS 180-1: SHA-1("abc") = a9993e364706816aba3e25717850c26c9cd0d89d
    let out = rusty_web_crypto::digest_sha1_hex(b"abc");
    assert_eq!(out, "a9993e364706816aba3e25717850c26c9cd0d89d");
}

#[test]
fn sha1_quick_brown_fox() {
    let out = rusty_web_crypto::digest_sha1_hex(
        b"The quick brown fox jumps over the lazy dog"
    );
    assert_eq!(out, "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
}

#[test]
fn sha1_56_byte_boundary() {
    // 55 bytes — one byte short of the padding boundary, so 0x80 + zeros
    // fit in the same block.
    let s = "a".repeat(55);
    let out = rusty_web_crypto::digest_sha1_hex(s.as_bytes());
    // From OpenSSL: echo -n "aaaa...a" (55) | sha1sum
    assert_eq!(out, "c1c8bbdc22796e28c0e15163d20899b65621d65a");
}

#[test]
fn hmac_sha1_rfc2202_test1() {
    // RFC 2202 Test Case 1:
    //   Key  = 0x0b * 20
    //   Data = "Hi There"
    //   HMAC = b617318655057264e28bc0b6fb378c8ef146be00
    let key = [0x0bu8; 20];
    let data = b"Hi There";
    let out = rusty_web_crypto::hmac_sha1(&key, data);
    let mut hex = String::new();
    for b in &out { hex.push_str(&format!("{:02x}", b)); }
    assert_eq!(hex, "b617318655057264e28bc0b6fb378c8ef146be00");
}

#[test]
fn hmac_sha1_rfc2202_test2() {
    // RFC 2202 Test Case 2:
    //   Key  = "Jefe"
    //   Data = "what do ya want for nothing?"
    //   HMAC = effcdf6ae5eb2fa2d27416d5f184df9c259a7c79
    let out = rusty_web_crypto::hmac_sha1(b"Jefe", b"what do ya want for nothing?");
    let mut hex = String::new();
    for b in &out { hex.push_str(&format!("{:02x}", b)); }
    assert_eq!(hex, "effcdf6ae5eb2fa2d27416d5f184df9c259a7c79");
}

// SHA-512 / SHA-384 + HMAC variants — FIPS + RFC 4231 known-answer vectors.

#[test]
fn sha512_empty_string() {
    // FIPS: SHA-512("") = cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce
    //                     47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e
    let out = rusty_web_crypto::digest_sha512_hex(b"");
    assert_eq!(out, "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e");
}

#[test]
fn sha512_abc() {
    // FIPS: SHA-512("abc") = ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a
    //                        2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f
    let out = rusty_web_crypto::digest_sha512_hex(b"abc");
    assert_eq!(out, "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f");
}

#[test]
fn sha384_empty_string() {
    // FIPS: SHA-384("") = 38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da
    //                     274edebfe76f65fbd51ad2f14898b95b
    let out = rusty_web_crypto::digest_sha384_hex(b"");
    assert_eq!(out, "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b");
}

#[test]
fn sha384_abc() {
    // FIPS: SHA-384("abc") = cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed
    //                        8086072ba1e7cc2358baeca134c825a7
    let out = rusty_web_crypto::digest_sha384_hex(b"abc");
    assert_eq!(out, "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7");
}

#[test]
fn hmac_sha512_rfc4231_test1() {
    // RFC 4231 Test Case 1:
    //   Key  = 0x0b * 20
    //   Data = "Hi There"
    //   HMAC = 87aa7cdea5ef619d4ff0b4241a1d6cb02379f4e2ce4ec2787ad0b30545e17cde
    //          daa833b7d6b8a702038b274eaea3f4e4be9d914eeb61f1702e696c203a126854
    let key = [0x0bu8; 20];
    let data = b"Hi There";
    let out = rusty_web_crypto::hmac_sha512(&key, data);
    let mut hex = String::new();
    for b in &out { hex.push_str(&format!("{:02x}", b)); }
    assert_eq!(hex, "87aa7cdea5ef619d4ff0b4241a1d6cb02379f4e2ce4ec2787ad0b30545e17cdedaa833b7d6b8a702038b274eaea3f4e4be9d914eeb61f1702e696c203a126854");
}

#[test]
fn hmac_sha384_rfc4231_test1() {
    // RFC 4231 Test Case 1:
    //   Key  = 0x0b * 20
    //   Data = "Hi There"
    //   HMAC = afd03944d84895626b0825f4ab46907f15f9dadbe4101ec682aa034c7cebc59c
    //          faea9ea9076ede7f4af152e8b2fa9cb6
    let key = [0x0bu8; 20];
    let data = b"Hi There";
    let out = rusty_web_crypto::hmac_sha384(&key, data);
    let mut hex = String::new();
    for b in &out { hex.push_str(&format!("{:02x}", b)); }
    assert_eq!(hex, "afd03944d84895626b0825f4ab46907f15f9dadbe4101ec682aa034c7cebc59cfaea9ea9076ede7f4af152e8b2fa9cb6");
}

#[test]
fn hmac_sha512_rfc4231_test2() {
    // RFC 4231 Test Case 2:
    //   Key  = "Jefe"
    //   Data = "what do ya want for nothing?"
    //   HMAC = 164b7a7bfcf819e2e395fbe73b56e0a387bd64222e831fd610270cd7ea250554
    //          9758bf75c05a994a6d034f65f8f0e6fdcaeab1a34d4a6b4b636e070a38bce737
    let out = rusty_web_crypto::hmac_sha512(b"Jefe", b"what do ya want for nothing?");
    let mut hex = String::new();
    for b in &out { hex.push_str(&format!("{:02x}", b)); }
    assert_eq!(hex, "164b7a7bfcf819e2e395fbe73b56e0a387bd64222e831fd610270cd7ea2505549758bf75c05a994a6d034f65f8f0e6fdcaeab1a34d4a6b4b636e070a38bce737");
}

// PBKDF2 — RFC 6070 known-answer vectors.

#[test]
fn pbkdf2_hmac_sha1_rfc6070_test1() {
    // P = "password" (8 octets), S = "salt" (4 octets), c = 1, dkLen = 20
    // DK = 0c 60 c8 0f 96 1f 0e 71 f3 a9 b5 24 af 60 12 06 2f e0 37 a6
    let out = rusty_web_crypto::pbkdf2_hmac_sha1(b"password", b"salt", 1, 20);
    let mut hex = String::new();
    for b in &out { hex.push_str(&format!("{:02x}", b)); }
    assert_eq!(hex, "0c60c80f961f0e71f3a9b524af6012062fe037a6");
}

#[test]
fn pbkdf2_hmac_sha1_rfc6070_test2() {
    // P = "password", S = "salt", c = 2, dkLen = 20
    // DK = ea 6c 01 4d c7 2d 6f 8c cd 1e d9 2a ce 1d 41 f0 d8 de 89 57
    let out = rusty_web_crypto::pbkdf2_hmac_sha1(b"password", b"salt", 2, 20);
    let mut hex = String::new();
    for b in &out { hex.push_str(&format!("{:02x}", b)); }
    assert_eq!(hex, "ea6c014dc72d6f8ccd1ed92ace1d41f0d8de8957");
}

#[test]
fn pbkdf2_hmac_sha1_rfc6070_test3() {
    // P = "password", S = "salt", c = 4096, dkLen = 20
    // DK = 4b 00 79 01 b7 65 48 9a be ad 49 d9 26 f7 21 d0 65 a4 29 c1
    let out = rusty_web_crypto::pbkdf2_hmac_sha1(b"password", b"salt", 4096, 20);
    let mut hex = String::new();
    for b in &out { hex.push_str(&format!("{:02x}", b)); }
    assert_eq!(hex, "4b007901b765489abead49d926f721d065a429c1");
}

#[test]
fn pbkdf2_hmac_sha256_rfc7914_test_vector() {
    // RFC 7914 (scrypt context) shows PBKDF2-HMAC-SHA-256 known answer:
    // P = "passwd", S = "salt", c = 1, dkLen = 64
    // DK = 55 ac 04 6e 56 e3 08 9f ec 16 91 c2 25 44 b6 05
    //      f9 41 85 21 6d de 04 65 e6 8b 9d 57 c2 0d ac bc
    //      49 ca 9c cc f1 79 b6 45 99 16 64 b3 9d 77 ef 31
    //      7c 71 b8 45 b1 e3 0b d5 09 11 20 41 d3 a1 97 83
    let out = rusty_web_crypto::pbkdf2_hmac_sha256(b"passwd", b"salt", 1, 64);
    let mut hex = String::new();
    for b in &out { hex.push_str(&format!("{:02x}", b)); }
    assert_eq!(hex, "55ac046e56e3089fec1691c22544b605f94185216dde0465e68b9d57c20dacbc49ca9cccf179b645991664b39d77ef317c71b845b1e30bd509112041d3a19783");
}

#[test]
fn pbkdf2_dk_len_shorter_than_block() {
    // dkLen=16, SHA-256 hLen=32 — output is just the first 16 bytes of T_1.
    let full = rusty_web_crypto::pbkdf2_hmac_sha256(b"password", b"salt", 100, 32);
    let half = rusty_web_crypto::pbkdf2_hmac_sha256(b"password", b"salt", 100, 16);
    assert_eq!(&full[..16], &half[..]);
}

#[test]
fn pbkdf2_dk_len_spans_multiple_blocks() {
    // dkLen > hLen exercises the multi-block T_1 || T_2 || ... path.
    let out = rusty_web_crypto::pbkdf2_hmac_sha256(b"password", b"salt", 100, 100);
    assert_eq!(out.len(), 100);
}

// ─────────────── AES (FIPS 197) + AES-GCM (SP 800-38D) ─────────────

fn hex_decode(s: &str) -> Vec<u8> {
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    (0..s.len()).step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i+2], 16).unwrap())
        .collect()
}
fn hex_encode(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 2);
    for byte in b { s.push_str(&format!("{:02x}", byte)); }
    s
}

#[test]
fn aes128_fips_197_appendix_c1() {
    // FIPS 197 §C.1: AES-128 with key 000102...0f, input 00112233...ff.
    let key = hex_decode("000102030405060708090a0b0c0d0e0f");
    let pt: [u8; 16] = hex_decode("00112233445566778899aabbccddeeff").try_into().unwrap();
    let ct = rusty_web_crypto::aes_encrypt_block_with_key(&key, &pt);
    assert_eq!(hex_encode(&ct), "69c4e0d86a7b0430d8cdb78070b4c55a");
}

#[test]
fn aes256_fips_197_appendix_c3() {
    // FIPS 197 §C.3: AES-256.
    let key = hex_decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
    let pt: [u8; 16] = hex_decode("00112233445566778899aabbccddeeff").try_into().unwrap();
    let ct = rusty_web_crypto::aes_encrypt_block_with_key(&key, &pt);
    assert_eq!(hex_encode(&ct), "8ea2b7ca516745bfeafc49904b496089");
}

#[test]
fn aes_gcm_sp800_38d_test_case_2() {
    // SP 800-38D Appendix B Test Case 2: K=zero, IV=zero, P=16 zero bytes, A=empty.
    let key = hex_decode("00000000000000000000000000000000");
    let iv  = hex_decode("000000000000000000000000");
    let pt  = hex_decode("00000000000000000000000000000000");
    let out = rusty_web_crypto::aes_gcm_encrypt(&key, &iv, &[], &pt).unwrap();
    assert_eq!(hex_encode(&out),
        "0388dace60b6a392f328c2b971b2fe78ab6e47d42cec13bdf53a67b21257bddf");
}

#[test]
fn aes_gcm_sp800_38d_test_case_3() {
    // SP 800-38D Appendix B Test Case 3.
    let key = hex_decode("feffe9928665731c6d6a8f9467308308");
    let iv  = hex_decode("cafebabefacedbaddecaf888");
    let pt  = hex_decode("d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a721c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b391aafd255");
    let out = rusty_web_crypto::aes_gcm_encrypt(&key, &iv, &[], &pt).unwrap();
    let expect_ct = "42831ec2217774244b7221b784d0d49ce3aa212f2c02a4e035c17e2329aca12e21d514b25466931c7d8f6a5aac84aa051ba30b396a0aac973d58e091473f5985";
    let expect_tag = "4d5c2af327cd64a62cf35abd2ba6fab4";
    assert_eq!(hex_encode(&out), format!("{}{}", expect_ct, expect_tag));
}

#[test]
fn aes_gcm_sp800_38d_test_case_4() {
    // SP 800-38D Appendix B Test Case 4 — with AAD + truncated plaintext.
    let key = hex_decode("feffe9928665731c6d6a8f9467308308");
    let iv  = hex_decode("cafebabefacedbaddecaf888");
    let pt  = hex_decode("d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a721c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b39");
    let aad = hex_decode("feedfacedeadbeeffeedfacedeadbeefabaddad2");
    let out = rusty_web_crypto::aes_gcm_encrypt(&key, &iv, &aad, &pt).unwrap();
    let expect_ct = "42831ec2217774244b7221b784d0d49ce3aa212f2c02a4e035c17e2329aca12e21d514b25466931c7d8f6a5aac84aa051ba30b396a0aac973d58e091";
    let expect_tag = "5bc94fbc3221a5db94fae95ae7121a47";
    assert_eq!(hex_encode(&out), format!("{}{}", expect_ct, expect_tag));
}

#[test]
fn aes_gcm_roundtrip() {
    let key = hex_decode("feffe9928665731c6d6a8f9467308308");
    let iv  = hex_decode("cafebabefacedbaddecaf888");
    let pt  = b"hello world, this is a roundtrip test of arbitrary length";
    let aad = b"associated-data";
    let ct = rusty_web_crypto::aes_gcm_encrypt(&key, &iv, aad, pt).unwrap();
    let dec = rusty_web_crypto::aes_gcm_decrypt(&key, &iv, aad, &ct).unwrap();
    assert_eq!(&dec, pt);
}

#[test]
fn aes_gcm_tag_mismatch_rejected() {
    let key = hex_decode("feffe9928665731c6d6a8f9467308308");
    let iv  = hex_decode("cafebabefacedbaddecaf888");
    let pt  = b"some plaintext";
    let mut ct = rusty_web_crypto::aes_gcm_encrypt(&key, &iv, &[], pt).unwrap();
    let n = ct.len();
    ct[n - 1] ^= 0x01;
    assert!(rusty_web_crypto::aes_gcm_decrypt(&key, &iv, &[], &ct).is_err());
}

#[test]
fn aes_gcm_aad_tampered_rejected() {
    let key = hex_decode("feffe9928665731c6d6a8f9467308308");
    let iv  = hex_decode("cafebabefacedbaddecaf888");
    let pt  = b"some plaintext";
    let aad = b"original-aad";
    let ct = rusty_web_crypto::aes_gcm_encrypt(&key, &iv, aad, pt).unwrap();
    assert!(rusty_web_crypto::aes_gcm_decrypt(&key, &iv, b"tampered-aad", &ct).is_err());
}

#[test]
fn aes256_gcm_roundtrip() {
    let key = hex_decode("0000000000000000000000000000000000000000000000000000000000000000");
    let iv  = hex_decode("000000000000000000000000");
    let pt  = b"AES-256-GCM smoke test payload of arbitrary length";
    let aad = b"some-aad";
    let ct = rusty_web_crypto::aes_gcm_encrypt(&key, &iv, aad, pt).unwrap();
    let dec = rusty_web_crypto::aes_gcm_decrypt(&key, &iv, aad, &ct).unwrap();
    assert_eq!(&dec, pt);
}

// ─────────────── HKDF (RFC 5869) ───────────────────────────────────

#[test]
fn hkdf_sha256_rfc5869_test1() {
    // RFC 5869 Appendix A Test Case 1: basic test case with SHA-256.
    let ikm = hex_decode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b");
    let salt = hex_decode("000102030405060708090a0b0c");
    let info = hex_decode("f0f1f2f3f4f5f6f7f8f9");
    let okm = rusty_web_crypto::hkdf_sha256(&ikm, &salt, &info, 42).unwrap();
    assert_eq!(hex_encode(&okm),
        "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865");
}

#[test]
fn hkdf_sha256_rfc5869_test2() {
    // RFC 5869 A.2: longer inputs/outputs with SHA-256.
    let ikm = hex_decode(
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f\
         202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f\
         404142434445464748494a4b4c4d4e4f");
    let salt = hex_decode(
        "606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f\
         808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f\
         a0a1a2a3a4a5a6a7a8a9aaabacadaeaf");
    let info = hex_decode(
        "b0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0c1c2c3c4c5c6c7c8c9cacbcccdcecf\
         d0d1d2d3d4d5d6d7d8d9dadbdcdddedfe0e1e2e3e4e5e6e7e8e9eaebecedeeef\
         f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff");
    let okm = rusty_web_crypto::hkdf_sha256(&ikm, &salt, &info, 82).unwrap();
    assert_eq!(hex_encode(&okm),
        "b11e398dc80327a1c8e7f78c596a49344f012eda2d4efad8a050cc4c19afa97c\
         59045a99cac7827271cb41c65e590e09da3275600c2f09b8367793a9aca3db71\
         cc30c58179ec3e87c14c01d5c1f3434f1d87".replace("\n", "").replace(" ", ""));
}

#[test]
fn hkdf_sha256_rfc5869_test3_empty_salt_and_info() {
    // RFC 5869 A.3: SHA-256 with zero-length salt + info.
    let ikm = hex_decode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b");
    let okm = rusty_web_crypto::hkdf_sha256(&ikm, &[], &[], 42).unwrap();
    assert_eq!(hex_encode(&okm),
        "8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d9d201395faa4b61a96c8");
}

#[test]
fn hkdf_sha1_rfc5869_test4() {
    // RFC 5869 A.4: basic test case with SHA-1.
    let ikm = hex_decode("0b0b0b0b0b0b0b0b0b0b0b");
    let salt = hex_decode("000102030405060708090a0b0c");
    let info = hex_decode("f0f1f2f3f4f5f6f7f8f9");
    let okm = rusty_web_crypto::hkdf_sha1(&ikm, &salt, &info, 42).unwrap();
    assert_eq!(hex_encode(&okm),
        "085a01ea1b10f36933068b56efa5ad81a4f14b822f5b091568a9cdd4f155fda2c22e422478d305f3f896");
}

#[test]
fn hkdf_length_exceeds_max_errors() {
    // L > 255 * HashLen must error per RFC 5869 §2.3.
    let r = rusty_web_crypto::hkdf_sha256(b"ikm", b"salt", b"info", 255 * 32 + 1);
    assert!(r.is_err());
}

#[test]
fn hkdf_sha512_roundtrip_smoke() {
    // No standardized RFC 5869 SHA-512 vector — check structural properties.
    let okm = rusty_web_crypto::hkdf_sha512(b"ikm", b"salt", b"info", 128).unwrap();
    assert_eq!(okm.len(), 128);
    // Deterministic: same inputs → same output.
    let okm2 = rusty_web_crypto::hkdf_sha512(b"ikm", b"salt", b"info", 128).unwrap();
    assert_eq!(okm, okm2);
    // Different info → different output.
    let okm3 = rusty_web_crypto::hkdf_sha512(b"ikm", b"salt", b"info2", 128).unwrap();
    assert_ne!(okm, okm3);
}
