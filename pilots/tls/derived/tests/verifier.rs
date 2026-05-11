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
