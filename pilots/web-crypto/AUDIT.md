# web-crypto pilot — coverage audit

**Sixteenth pilot. Tier-C #8 from the trajectory queue.** Web Crypto subset — SHA-256 digest + UUID v4 + getRandomValues. Real cryptographic primitives implemented from scratch (no external crates) to keep the apparatus' std-only pattern.

## Constraint inputs

`runs/2026-05-10-bun-v0.13b-spec-batch/constraints/crypto.constraints.md` — interface list includes `crypto.subtle.exportKey`, `crypto.subtle.generateKey`, `crypto.subtle.deriveBits`, `crypto.subtle.deriveKey`, `crypto.subtle.importKey`, `crypto.subtle.verify`, `crypto.timingSafeEqual`, `crypto.generatePrimeSync`, `crypto.getRandomValues`, `crypto.randomInt`, `crypto.sign`, `crypto.subtle`. Cross-corroboration via `specs/crypto-random.spec.md`.

## Pilot scope

A focused subset of Web Crypto + Node crypto:
- `crypto::random_uuid_v4()` → 36-char v4 UUID string per RFC 4122
- `crypto::get_random_values(buf: &mut [u8])` → fill with cryptographic random
- `crypto::subtle::digest_sha256(data: &[u8])` → 32-byte hash per FIPS 180-4
- `crypto::subtle::digest_sha1(data: &[u8])` → 20-byte hash (SHA-1)
- `crypto::timing_safe_equal(a: &[u8], b: &[u8])` → constant-time byte comparison

Out of scope:
- SHA-384/512 (same algorithm shape; deferred for LOC budget)
- HMAC (would compose with SHA, deferred)
- subtle.generateKey / deriveKey / importKey / exportKey (key material handling; substantial)
- RSA / ECDSA / Ed25519 (substantial; would need bignum arithmetic)
- AES (substantial)
- HKDF / PBKDF2 (deferred)

## Approach

Random sources via `/dev/urandom` direct read on Unix; no external crates. SHA-256 implemented per FIPS 180-4 in pure Rust. UUID v4 follows RFC 4122 §4.4 (random bits with version + variant fields).

## Ahead-of-time hypotheses

1. **Pilot is medium-sized in LOC.** SHA-256 implementation is ~150 LOC; UUID v4 ~30 LOC; random bytes ~20 LOC; timing-safe compare ~10 LOC; SHA-1 ~80 LOC. Total: ~290-320 code-only LOC.

2. **The SHA-256 implementation must match published test vectors exactly.** Verifier tests use NIST test vectors (empty string → e3b0c44...; "abc" → ba7816...; etc.).

3. **First-run derivation bug expected on SHA-256 padding or message-schedule.** SHA-256 has subtle bit-rotation semantics that LLM-derivations frequently get wrong. AOT prediction: at least one verifier-caught bug.

4. **getRandomValues uses /dev/urandom direct read on Unix.** Pilot won't support Windows for random-bytes; AUDIT documents this scope choice.

## Verifier strategy

~25 verifier tests with NIST SHA-256 vectors + UUID-format pattern matching + random-bytes statistical sanity check. Consumer regression: ~6-8 tests citing real consumer crypto patterns (JWT signing libraries, password hashing flows, content-addressed storage).
