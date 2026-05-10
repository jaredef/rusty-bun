// Consumer-regression suite for web-crypto.

use rusty_web_crypto::*;

// ────────── JWT / JOSE — SHA-256 for HS256 base ──────────
//
// Source: https://github.com/panva/jose — JWS HS256 uses SHA-256 inside HMAC.
// Consumer expectation: SHA-256 produces NIST-spec-conforming output.

#[test]
fn consumer_jose_sha256_nist_conformance() {
    // Known JOSE-test-vector style: SHA-256("test") = 9f86d081...
    assert_eq!(
        digest_sha256_hex(b"test"),
        "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
    );
}

// ────────── Content-addressed storage (IPFS-style) ──────────
//
// Source: ipfs/js-ipfs and similar tools hash content for addressing.
// Consumer expects identical content → identical hash, deterministically.

#[test]
fn consumer_ipfs_content_addressed_determinism() {
    let payload = b"file contents";
    let h1 = digest_sha256(payload);
    let h2 = digest_sha256(payload);
    assert_eq!(h1, h2);
}

// ────────── Webhook signature verification — timing-safe equal ──────
//
// Source: Stripe/GitHub webhook verification reads signature from header,
// compares with HMAC of payload via timing-safe equal to prevent timing
// attacks.

#[test]
fn consumer_webhook_signature_timing_safe_compare() {
    let expected = digest_sha256(b"payload");
    let received = digest_sha256(b"payload");
    assert!(timing_safe_equal(&expected, &received));
    let attacker = digest_sha256(b"payload-modified");
    assert!(!timing_safe_equal(&expected, &attacker));
}

// ────────── Session ID generation — UUID v4 ──────────
//
// Source: express-session, fastify-session and similar libraries generate
// session IDs as v4 UUIDs. Consumer expects RFC 4122 format.

#[test]
fn consumer_session_id_format() {
    let id = random_uuid_v4();
    // Length and dash positions per RFC 4122
    assert_eq!(id.len(), 36);
    assert!(id.chars().nth(8).unwrap() == '-');
    assert!(id.chars().nth(13).unwrap() == '-');
    assert!(id.chars().nth(18).unwrap() == '-');
    assert!(id.chars().nth(23).unwrap() == '-');
    // All non-dash chars are lowercase hex
    for c in id.chars().filter(|&c| c != '-') {
        assert!(c.is_ascii_hexdigit() && (c.is_numeric() || c.is_lowercase()),
            "invalid char in UUID: {}", c);
    }
}

// ────────── CSRF token generation — getRandomValues ──────────
//
// Source: csurf, lusca, and similar middleware generate CSRF tokens via
// crypto.getRandomValues. Consumer expects each call to produce different
// random bytes.

#[test]
fn consumer_csrf_token_unique_per_call() {
    let mut t1 = [0u8; 32];
    let mut t2 = [0u8; 32];
    get_random_values(&mut t1).unwrap();
    get_random_values(&mut t2).unwrap();
    assert_ne!(t1, t2);
}

// ────────── Git object hashing — large file SHA-256 ──────────
//
// Source: git-lfs and content-versioning tools hash large files via SHA-256
// stream. Pilot doesn't have streaming SHA but verifies large-input
// correctness.

#[test]
fn consumer_git_lfs_large_file_hash_correctness() {
    let payload = vec![b'a'; 1_000_000];
    // Per FIPS 180-4 reference: SHA-256(1M 'a') = cdc76e5c9914...
    assert_eq!(
        digest_sha256_hex(&payload),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

// ────────── Password storage — never use SHA alone ──────────
//
// Source: anti-pattern documented in OWASP. Consumer code that previously
// used SHA-256 for passwords needs upgrade path. Pilot's SHA-256 is
// correct, but doc-comment recommends bcrypt/argon2 for password storage.

#[test]
fn consumer_password_hash_correctness_warning_aside() {
    // Document the spec-correctness; production code uses argon2/bcrypt.
    let pw_hash = digest_sha256(b"hunter2");
    assert_eq!(pw_hash.len(), 32);
}

// ────────── Bun.password / WebCrypto integration ──────────
//
// Source: subtle.digest is used by many Bun-using consumers as the
// canonical hash entry point. Consumer expects "SHA-256" string algorithm
// name to work.

#[test]
fn consumer_bun_subtle_digest_canonical_name() {
    let r = subtle::digest("SHA-256", b"hello").unwrap();
    let hex: String = r.iter().map(|b| format!("{:02x}", b)).collect();
    assert_eq!(hex,
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
}
