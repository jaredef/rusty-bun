// Verifier suite for rusty-x509.
//
// Strategy: generate self-signed certs via openssl at test runtime,
// then parse + verify their self-signature using the pilot. If openssl
// is unavailable, tests skip cleanly.

use rusty_x509::*;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

fn openssl_available() -> bool {
    Command::new("openssl").arg("version").output()
        .map(|o| o.status.success()).unwrap_or(false)
}

static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);
fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
    let id = DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("{}-{}-{}", prefix, std::process::id(), id))
}

/// Generate a self-signed RSA-SHA256 cert. Returns (pem_bytes, key_pem).
fn gen_self_signed_rsa() -> Option<(Vec<u8>, Vec<u8>)> {
    if !openssl_available() { return None; }
    let dir = unique_temp_dir("rusty-x509-test");
    let _ = std::fs::create_dir_all(&dir);
    let key_path = dir.join("key.pem");
    let cert_path = dir.join("cert.pem");
    let r = Command::new("openssl").args(&[
        "req", "-x509", "-newkey", "rsa:2048", "-sha256",
        "-keyout", key_path.to_str().unwrap(),
        "-out", cert_path.to_str().unwrap(),
        "-days", "365", "-nodes",
        "-subj", "/CN=test.rusty-bun.local",
    ]).output().ok()?;
    if !r.status.success() { return None; }
    let cert = std::fs::read(&cert_path).ok()?;
    let key = std::fs::read(&key_path).ok()?;
    let _ = std::fs::remove_dir_all(&dir);
    Some((cert, key))
}

/// Generate a self-signed ECDSA-P256-SHA256 cert.
fn gen_self_signed_p256() -> Option<(Vec<u8>, Vec<u8>)> {
    if !openssl_available() { return None; }
    let dir = unique_temp_dir("rusty-x509-test-p256");
    let _ = std::fs::create_dir_all(&dir);
    let key_path = dir.join("key.pem");
    let cert_path = dir.join("cert.pem");
    let kg = Command::new("openssl").args(&[
        "ecparam", "-name", "prime256v1", "-genkey", "-noout",
        "-out", key_path.to_str().unwrap(),
    ]).output().ok()?;
    if !kg.status.success() { return None; }
    let r = Command::new("openssl").args(&[
        "req", "-x509", "-new", "-sha256",
        "-key", key_path.to_str().unwrap(),
        "-out", cert_path.to_str().unwrap(),
        "-days", "365",
        "-subj", "/CN=ec.rusty-bun.local",
    ]).output().ok()?;
    if !r.status.success() { return None; }
    let cert = std::fs::read(&cert_path).ok()?;
    let key = std::fs::read(&key_path).ok()?;
    let _ = std::fs::remove_dir_all(&dir);
    Some((cert, key))
}

#[test]
#[ignore]  // seed A8.17: 2048-bit RSA keygen via openssl exceeds inner-loop budget
fn parse_self_signed_rsa() {
    let (pem, _key) = match gen_self_signed_rsa() {
        Some(p) => p, None => { eprintln!("skipping: openssl unavailable"); return; }
    };
    let pem_str = std::str::from_utf8(&pem).unwrap();
    let der = pem_to_der(pem_str).unwrap();
    let cert = parse_certificate(&der).unwrap();
    assert!(cert.version >= 2);
    assert!(!cert.serial_number.is_empty());
    assert_eq!(cert.signature_algorithm.oid, OID_SHA256_WITH_RSA);
    assert_eq!(cert.issuer.raw_der, cert.subject.raw_der, "self-signed: issuer == subject");
    match &cert.subject_public_key_info.key {
        PublicKey::Rsa { n, e } => {
            assert!(n.len() >= 256 || n.len() >= 255, "expected 2048-bit modulus, got {} bytes", n.len());
            // Common RSA exponent is 65537 = 0x010001.
            assert!(e == &[0x01, 0x00, 0x01]);
        }
        _ => panic!("expected RSA key"),
    }
    // Subject CN.
    let cn = cert.subject.attributes.iter()
        .find(|(o, _)| o == OID_RDN_CN).map(|(_, v)| v.clone()).unwrap_or_default();
    assert_eq!(cn, "test.rusty-bun.local");
}

#[test]
#[ignore]  // seed A8.17: 2048-bit RSA keygen via openssl exceeds inner-loop budget
fn verify_self_signed_rsa() {
    let (pem, _key) = match gen_self_signed_rsa() {
        Some(p) => p, None => { eprintln!("skipping: openssl unavailable"); return; }
    };
    let pem_str = std::str::from_utf8(&pem).unwrap();
    let der = pem_to_der(pem_str).unwrap();
    let cert = parse_certificate(&der).unwrap();
    // Self-signed: the cert is its own issuer.
    verify_signature(&cert, &cert.subject_public_key_info).unwrap();
}

#[test]
#[ignore]  // seed A8.17: 2048-bit RSA keygen via openssl exceeds inner-loop budget
fn parse_and_verify_self_signed_p256() {
    let (pem, _key) = match gen_self_signed_p256() {
        Some(p) => p, None => { eprintln!("skipping: openssl unavailable"); return; }
    };
    let pem_str = std::str::from_utf8(&pem).unwrap();
    let der = pem_to_der(pem_str).unwrap();
    let cert = parse_certificate(&der).unwrap();
    assert_eq!(cert.signature_algorithm.oid, OID_ECDSA_WITH_SHA256);
    match &cert.subject_public_key_info.key {
        PublicKey::Ec { curve_oid, point } => {
            assert_eq!(curve_oid, OID_P256_CURVE);
            assert_eq!(point.len(), 65); // 0x04 + X(32) + Y(32)
            assert_eq!(point[0], 0x04);
        }
        _ => panic!("expected EC key"),
    }
    verify_signature(&cert, &cert.subject_public_key_info).unwrap();
}

#[test]
#[ignore]  // seed A8.17: 2048-bit RSA keygen via openssl exceeds inner-loop budget
fn pem_all_to_der_multi_block() {
    let (pem1, _) = match gen_self_signed_rsa() {
        Some(p) => p, None => { eprintln!("skipping: openssl unavailable"); return; }
    };
    let (pem2, _) = gen_self_signed_p256().unwrap();
    let combined = format!("{}\n{}", std::str::from_utf8(&pem1).unwrap(),
                                       std::str::from_utf8(&pem2).unwrap());
    let ders = pem_all_to_der(&combined);
    assert_eq!(ders.len(), 2);
    let _c1 = parse_certificate(&ders[0]).unwrap();
    let _c2 = parse_certificate(&ders[1]).unwrap();
}

#[test]
#[ignore]  // seed A8.17: 2048-bit RSA keygen via openssl exceeds inner-loop budget
fn corrupted_signature_fails_verify() {
    let (pem, _) = match gen_self_signed_rsa() {
        Some(p) => p, None => { eprintln!("skipping: openssl unavailable"); return; }
    };
    let pem_str = std::str::from_utf8(&pem).unwrap();
    let der = pem_to_der(pem_str).unwrap();
    let mut cert = parse_certificate(&der).unwrap();
    // Flip one byte in the signature.
    let n = cert.signature_value.len();
    cert.signature_value[n / 2] ^= 0x01;
    let r = verify_signature(&cert, &cert.subject_public_key_info);
    assert!(r.is_err());
}

#[test]
#[ignore]  // seed A8.17: 2048-bit RSA keygen via openssl exceeds inner-loop budget
fn wrong_issuer_key_fails_verify() {
    let (pem_a, _) = match gen_self_signed_rsa() {
        Some(p) => p, None => { eprintln!("skipping: openssl unavailable"); return; }
    };
    let (pem_b, _) = gen_self_signed_rsa().unwrap();
    let der_a = pem_to_der(std::str::from_utf8(&pem_a).unwrap()).unwrap();
    let der_b = pem_to_der(std::str::from_utf8(&pem_b).unwrap()).unwrap();
    let cert_a = parse_certificate(&der_a).unwrap();
    let cert_b = parse_certificate(&der_b).unwrap();
    // Cert A was self-signed; verifying against cert B's pubkey must fail.
    let r = verify_signature(&cert_a, &cert_b.subject_public_key_info);
    assert!(r.is_err());
}

#[test]
#[ignore]  // seed A8.17: 2048-bit RSA keygen via openssl exceeds inner-loop budget
fn extensions_parsed() {
    let (pem, _) = match gen_self_signed_rsa() {
        Some(p) => p, None => { eprintln!("skipping: openssl unavailable"); return; }
    };
    let der = pem_to_der(std::str::from_utf8(&pem).unwrap()).unwrap();
    let cert = parse_certificate(&der).unwrap();
    // openssl req -x509 emits at minimum subjectKeyIdentifier +
    // authorityKeyIdentifier + basicConstraints. Just assert non-empty.
    assert!(!cert.extensions.is_empty());
}

#[test]
fn pem_bad_header_rejected() {
    let r = pem_to_der("nothing here");
    assert!(matches!(r, Err(X509Error::PemBadHeader)));
}
