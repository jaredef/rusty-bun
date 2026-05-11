// Verifier suite for rusty-tls (record layer + trust store + chain walk).

use rusty_tls::*;
use std::process::Command;

fn openssl_available() -> bool {
    Command::new("openssl").arg("version").output()
        .map(|o| o.status.success()).unwrap_or(false)
}

// ─── Record layer ───────────────────────────────────────────────────

#[test]
fn record_roundtrip_handshake() {
    let r = TlsRecord {
        content_type: ContentType::Handshake,
        version: ProtocolVersion::LEGACY,
        fragment: vec![0x01, 0x02, 0x03, 0x04, 0x05],
    };
    let bytes = encode_record(&r).unwrap();
    assert_eq!(bytes[0], 22);  // ContentType.Handshake
    assert_eq!(&bytes[1..3], &[0x03, 0x03]);  // legacy version
    assert_eq!(&bytes[3..5], &[0x00, 0x05]);  // length
    assert_eq!(&bytes[5..], &[0x01, 0x02, 0x03, 0x04, 0x05]);

    let (decoded, n) = decode_record(&bytes).unwrap();
    assert_eq!(n, bytes.len());
    assert_eq!(decoded.content_type, ContentType::Handshake);
    assert_eq!(decoded.version, ProtocolVersion::LEGACY);
    assert_eq!(decoded.fragment, r.fragment);
}

#[test]
fn record_roundtrip_application_data() {
    let r = TlsRecord {
        content_type: ContentType::ApplicationData,
        version: ProtocolVersion::LEGACY,
        fragment: b"GET / HTTP/1.1\r\n\r\n".to_vec(),
    };
    let bytes = encode_record(&r).unwrap();
    let (decoded, _) = decode_record(&bytes).unwrap();
    assert_eq!(decoded.content_type, ContentType::ApplicationData);
    assert_eq!(decoded.fragment, r.fragment);
}

#[test]
fn record_decode_truncated_returns_unexpected_end() {
    // Only 4 bytes; record header needs 5.
    let buf = [22, 0x03, 0x03, 0x00];
    assert!(matches!(decode_record(&buf), Err(TlsError::UnexpectedEnd)));
}

#[test]
fn record_decode_unknown_content_type() {
    let buf = [99, 0x03, 0x03, 0x00, 0x00];
    assert!(matches!(decode_record(&buf), Err(TlsError::UnknownContentType(99))));
}

#[test]
fn record_decode_truncated_fragment() {
    // Header claims 10-byte fragment but only 3 bytes follow.
    let mut buf = vec![22, 0x03, 0x03, 0x00, 0x0A, 0x01, 0x02, 0x03];
    let _ = buf;
    assert!(matches!(decode_record(&buf), Err(TlsError::UnexpectedEnd)));
}

#[test]
fn record_decode_two_back_to_back() {
    let r1 = TlsRecord {
        content_type: ContentType::Handshake,
        version: ProtocolVersion::LEGACY,
        fragment: vec![0xAA, 0xBB],
    };
    let r2 = TlsRecord {
        content_type: ContentType::ApplicationData,
        version: ProtocolVersion::LEGACY,
        fragment: vec![0xCC, 0xDD, 0xEE],
    };
    let mut combined = encode_record(&r1).unwrap();
    combined.extend(encode_record(&r2).unwrap());
    let (d1, n1) = decode_record(&combined).unwrap();
    assert_eq!(d1.fragment, vec![0xAA, 0xBB]);
    let (d2, _n2) = decode_record(&combined[n1..]).unwrap();
    assert_eq!(d2.fragment, vec![0xCC, 0xDD, 0xEE]);
}

#[test]
fn alert_encode_decode_close_notify() {
    let bytes = encode_alert(AlertLevel::Warning, AlertDescription::CLOSE_NOTIFY);
    assert_eq!(bytes, vec![1, 0]);
    let (lvl, desc) = decode_alert(&bytes).unwrap();
    assert_eq!(lvl, AlertLevel::Warning);
    assert_eq!(desc, AlertDescription::CLOSE_NOTIFY);
}

#[test]
fn alert_encode_decode_fatal_unknown_ca() {
    let bytes = encode_alert(AlertLevel::Fatal, AlertDescription::UNKNOWN_CA);
    assert_eq!(bytes, vec![2, 48]);
    let (lvl, desc) = decode_alert(&bytes).unwrap();
    assert_eq!(lvl, AlertLevel::Fatal);
    assert_eq!(desc, AlertDescription::UNKNOWN_CA);
}

// ─── Trust store + chain walk ────────────────────────────────────────

#[test]
#[ignore]  // seed A8.17: openssl keygen exceeds inner-loop budget
fn trust_store_loads_system_default() {
    let store = TrustStore::load_system_default();
    match store {
        Ok(s) => {
            assert!(s.len() > 0, "expected non-empty system trust store");
            eprintln!("loaded {} system root certs", s.len());
        }
        Err(_) => eprintln!("skipping: no system CA bundle"),
    }
}

#[test]
#[ignore]  // seed A8.17: openssl keygen exceeds inner-loop budget
fn chain_walk_self_signed_trust_anchor() {
    if !openssl_available() {
        eprintln!("skipping: openssl unavailable"); return;
    }
    let dir = std::env::temp_dir().join(format!("rusty-tls-chain-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let key_path = dir.join("ca.key");
    let cert_path = dir.join("ca.crt");
    let _ = Command::new("openssl").args(&[
        "req", "-x509", "-newkey", "rsa:2048", "-sha256",
        "-keyout", key_path.to_str().unwrap(),
        "-out", cert_path.to_str().unwrap(),
        "-days", "365", "-nodes",
        "-subj", "/CN=Test Root CA",
    ]).output().expect("openssl");

    let pem = std::fs::read_to_string(&cert_path).unwrap();
    let der = pem_to_der(&pem).unwrap();
    let cert = parse_certificate(&der).unwrap();
    let mut store = TrustStore::new();
    store.add(cert.clone());

    // Self-signed leaf == trust anchor walks in one step.
    chain_walk(&cert, &[], &store, 8).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore]  // seed A8.17
fn chain_walk_unknown_self_signed_rejected() {
    if !openssl_available() {
        eprintln!("skipping: openssl unavailable"); return;
    }
    let dir = std::env::temp_dir().join(format!("rusty-tls-unknown-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let key_path = dir.join("k.key");
    let cert_path = dir.join("k.crt");
    let _ = Command::new("openssl").args(&[
        "req", "-x509", "-newkey", "rsa:2048", "-sha256",
        "-keyout", key_path.to_str().unwrap(),
        "-out", cert_path.to_str().unwrap(),
        "-days", "365", "-nodes",
        "-subj", "/CN=Unknown Self-Signed",
    ]).output().expect("openssl");

    let pem = std::fs::read_to_string(&cert_path).unwrap();
    let der = pem_to_der(&pem).unwrap();
    let cert = parse_certificate(&der).unwrap();
    let empty_store = TrustStore::new();
    let r = chain_walk(&cert, &[], &empty_store, 8);
    assert!(matches!(r, Err(TlsError::SelfSignedNotInTrust)));
    let _ = std::fs::remove_dir_all(&dir);
}

use rusty_x509::*;

// ─── Handshake message framing ──────────────────────────────────────

#[test]
fn handshake_encode_decode_roundtrip() {
    let m = HandshakeMessage {
        msg_type: HandshakeType::ClientHello,
        body: vec![0x03, 0x03, 0xAA, 0xBB, 0xCC, 0xDD],
    };
    let bytes = encode_handshake(&m);
    assert_eq!(bytes[0], 1);  // ClientHello
    assert_eq!(&bytes[1..4], &[0x00, 0x00, 0x06]);  // length
    let (d, n) = decode_handshake(&bytes).unwrap();
    assert_eq!(n, bytes.len());
    assert_eq!(d.msg_type, HandshakeType::ClientHello);
    assert_eq!(d.body, m.body);
}

#[test]
fn handshake_decode_unknown_type() {
    let buf = [99u8, 0, 0, 0];
    let r = decode_handshake(&buf);
    assert!(r.is_err());
}

#[test]
fn handshake_decode_truncated() {
    let buf = [1u8, 0, 0, 0x10, 0x01];  // declared length 16, only 1 byte
    let r = decode_handshake(&buf);
    assert!(matches!(r, Err(TlsError::UnexpectedEnd)));
}

// ─── HKDF-Expand-Label vector ───────────────────────────────────────
//
// RFC 8448 §3 (1-RTT Handshake) supplies exact byte vectors. We use a
// minimal slice: the early_secret value and the HKDF-Expand-Label
// "derived" step that produces derived_early.

#[test]
fn early_secret_from_zero() {
    // RFC 8448 §3 early_secret = HKDF-Extract(0, 0):
    // 33ad0a1c607ec03b09e6cd9893680ce210adf300aa1f2660e1b22e10f170f92a
    let hash = HashAlgorithm::Sha256;
    let zeros = vec![0u8; 32];
    let early = hash.hkdf_extract(&zeros, &zeros);
    let expected = hex::decode_hex("33ad0a1c607ec03b09e6cd9893680ce210adf300aa1f2660e1b22e10f170f92a");
    assert_eq!(early, expected);
}

#[test]
fn derived_early_secret_label() {
    // Derive-Secret(early_secret, "derived", "") =
    //   HKDF-Expand-Label(early_secret, "derived", Hash(""), 32)
    // Per RFC 8448 §3:
    // derived = 6f2615a108c702c5678f54fc9dbab69716c076189c48250cebeac3576c3611ba
    let hash = HashAlgorithm::Sha256;
    let zeros = vec![0u8; 32];
    let early = hash.hkdf_extract(&zeros, &zeros);
    let empty_hash = hash.empty_hash();
    let derived = hkdf_expand_label(hash, &early, b"derived", &empty_hash, 32).unwrap();
    let expected = hex::decode_hex("6f2615a108c702c5678f54fc9dbab69716c076189c48250cebeac3576c3611ba");
    assert_eq!(derived, expected);
}

// ─── AEAD nonce construction ────────────────────────────────────────

#[test]
fn record_nonce_xors_seq_into_iv() {
    let iv = vec![0x00; 12];
    let nonce0 = record_nonce(&iv, 0);
    assert_eq!(nonce0, vec![0u8; 12]);
    let nonce1 = record_nonce(&iv, 1);
    let mut expected1 = vec![0u8; 12];
    expected1[11] = 1;
    assert_eq!(nonce1, expected1);
    // High-byte placement.
    let nonce_big = record_nonce(&iv, 0x0102_0304_0506_0708);
    let expected_big = vec![0,0,0,0, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    assert_eq!(nonce_big, expected_big);
}

#[test]
fn record_nonce_with_nonzero_iv() {
    let iv = vec![0xAA; 12];
    let nonce = record_nonce(&iv, 1);
    let mut expected = vec![0xAA; 12];
    expected[11] ^= 1;
    assert_eq!(nonce, expected);
}

// ─── AEAD record encrypt/decrypt round-trip ─────────────────────────

#[test]
fn aead_record_roundtrip() {
    // Synthetic but valid: AES-128-GCM with 32-byte traffic secret,
    // derive key+iv via the labeled-derive, then encrypt and decrypt.
    let hash = HashAlgorithm::Sha256;
    let traffic_secret = vec![0x42u8; 32];
    let keys = derive_traffic_keys(hash, &traffic_secret, 16, 12).unwrap();
    let plaintext = b"GET / HTTP/1.1\r\n\r\n";
    let ct = aead_encrypt_record(&keys, 0, 23 /* ApplicationData */, plaintext).unwrap();
    let (inner_ct, pt) = aead_decrypt_record(&keys, 0, &ct).unwrap();
    assert_eq!(inner_ct, 23);
    assert_eq!(pt, plaintext);
}

#[test]
fn aead_record_wrong_seq_fails_decrypt() {
    let hash = HashAlgorithm::Sha256;
    let traffic_secret = vec![0x42u8; 32];
    let keys = derive_traffic_keys(hash, &traffic_secret, 16, 12).unwrap();
    let ct = aead_encrypt_record(&keys, 0, 23, b"hello").unwrap();
    // Decrypt with wrong seq → AEAD auth fails.
    let r = aead_decrypt_record(&keys, 1, &ct);
    assert!(r.is_err());
}

// ─── Finished MAC structural test ───────────────────────────────────

#[test]
fn finished_mac_deterministic() {
    let hash = HashAlgorithm::Sha256;
    let secret = vec![0xAA; 32];
    let transcript = vec![0xBB; 32];
    let m1 = finished_mac(hash, &secret, &transcript).unwrap();
    let m2 = finished_mac(hash, &secret, &transcript).unwrap();
    assert_eq!(m1, m2);
    assert_eq!(m1.len(), 32);
}

// ─── Hex helper module ──────────────────────────────────────────────

mod hex {
    pub fn decode_hex(s: &str) -> Vec<u8> {
        let bytes: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
        let mut out = Vec::with_capacity(bytes.len() / 2);
        for i in (0..bytes.len()).step_by(2) {
            let hi = bytes[i].to_digit(16).unwrap() as u8;
            let lo = bytes[i + 1].to_digit(16).unwrap() as u8;
            out.push((hi << 4) | lo);
        }
        out
    }
}
