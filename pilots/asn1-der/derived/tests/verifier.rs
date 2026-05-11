// Verifier suite for rusty-asn1-der. Real DER test vectors:
// - INTEGER encodings from RFC 5280 examples
// - OID encodings for rsaEncryption + sha256WithRSAEncryption
// - SEQUENCE structures
// - A minimal AlgorithmIdentifier SEQUENCE per RFC 5280

use rusty_asn1_der::*;

#[test]
fn integer_small_positive() {
    // INTEGER 1 = 02 01 01
    let buf = [0x02, 0x01, 0x01];
    let v = parse_single(&buf).unwrap();
    assert_eq!(v.tag, TAG_INTEGER);
    assert_eq!(v.as_i64().unwrap(), 1);
}

#[test]
fn integer_zero() {
    let buf = [0x02, 0x01, 0x00];
    let v = parse_single(&buf).unwrap();
    assert_eq!(v.as_i64().unwrap(), 0);
}

#[test]
fn integer_negative() {
    // INTEGER -1 = 02 01 FF
    let buf = [0x02, 0x01, 0xFF];
    let v = parse_single(&buf).unwrap();
    assert_eq!(v.as_i64().unwrap(), -1);
}

#[test]
fn integer_large_unsigned() {
    // INTEGER 65537 = 02 03 01 00 01 (leading 0 to keep sign positive)
    let buf = [0x02, 0x03, 0x01, 0x00, 0x01];
    let v = parse_single(&buf).unwrap();
    assert_eq!(v.as_i64().unwrap(), 65537);
    let mag = v.as_unsigned_integer().unwrap();
    assert_eq!(mag, &[0x01, 0x00, 0x01]);
}

#[test]
fn integer_non_minimal_rejected() {
    // 02 02 00 01 — leading zero with positive value; non-minimal per DER.
    let buf = [0x02, 0x02, 0x00, 0x01];
    let v = parse_single(&buf).unwrap();
    assert!(v.as_i64().is_err());
}

#[test]
fn null_value() {
    let buf = [0x05, 0x00];
    let v = parse_single(&buf).unwrap();
    assert_eq!(v.tag, TAG_NULL);
    assert!(v.content.is_empty());
}

#[test]
fn boolean_true_false() {
    let t = parse_single(&[0x01, 0x01, 0xFF]).unwrap();
    assert_eq!(t.as_bool().unwrap(), true);
    let f = parse_single(&[0x01, 0x01, 0x00]).unwrap();
    assert_eq!(f.as_bool().unwrap(), false);
    let bad = parse_single(&[0x01, 0x01, 0x42]).unwrap();
    assert!(bad.as_bool().is_err());
}

#[test]
fn oid_rsa_encryption() {
    // 1.2.840.113549.1.1.1 (rsaEncryption) DER encoding:
    // 06 09 2A 86 48 86 F7 0D 01 01 01
    let buf = [0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01, 0x01];
    let v = parse_single(&buf).unwrap();
    assert_eq!(v.tag, TAG_OID);
    let arcs = v.as_oid().unwrap();
    assert_eq!(arcs, vec![1, 2, 840, 113549, 1, 1, 1]);
    assert_eq!(oid_to_string(&arcs), "1.2.840.113549.1.1.1");
}

#[test]
fn oid_sha256_with_rsa() {
    // 1.2.840.113549.1.1.11 (sha256WithRSAEncryption):
    // 06 09 2A 86 48 86 F7 0D 01 01 0B
    let buf = [0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01, 0x0B];
    let v = parse_single(&buf).unwrap();
    let arcs = v.as_oid().unwrap();
    assert_eq!(oid_to_string(&arcs), "1.2.840.113549.1.1.11");
}

#[test]
fn octet_string_basic() {
    // OCTET STRING with three bytes 0x01 0x02 0x03
    let buf = [0x04, 0x03, 0x01, 0x02, 0x03];
    let v = parse_single(&buf).unwrap();
    assert_eq!(v.tag, TAG_OCTET_STRING);
    assert_eq!(v.as_bytes(), &[0x01, 0x02, 0x03]);
}

#[test]
fn bit_string_basic() {
    // BIT STRING: unused-bits=0, content [0xAA, 0xBB]
    let buf = [0x03, 0x03, 0x00, 0xAA, 0xBB];
    let v = parse_single(&buf).unwrap();
    assert_eq!(v.tag, TAG_BIT_STRING);
    let (unused, bytes) = v.as_bit_string().unwrap();
    assert_eq!(unused, 0);
    assert_eq!(bytes, &[0xAA, 0xBB]);
}

#[test]
fn sequence_with_inner() {
    // SEQUENCE { INTEGER 1, NULL }
    let buf = [0x30, 0x05, 0x02, 0x01, 0x01, 0x05, 0x00];
    let v = parse_single(&buf).unwrap();
    assert_eq!(v.tag, TAG_SEQUENCE);
    assert!(v.is_constructed());
    let mut inner = v.into_reader().unwrap();
    let n = inner.read_tag(TAG_INTEGER).unwrap();
    assert_eq!(n.as_i64().unwrap(), 1);
    let nul = inner.read_tag(TAG_NULL).unwrap();
    assert!(nul.content.is_empty());
    assert!(inner.is_empty());
}

#[test]
fn algorithm_identifier_rsa_encryption() {
    // AlgorithmIdentifier ::= SEQUENCE { OID 1.2.840.113549.1.1.1, NULL }
    // 30 0D 06 09 2A 86 48 86 F7 0D 01 01 01 05 00
    let buf = [
        0x30, 0x0D,
        0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01, 0x01,
        0x05, 0x00,
    ];
    let v = parse_single(&buf).unwrap();
    let mut inner = v.into_reader().unwrap();
    let oid = inner.read_tag(TAG_OID).unwrap();
    let arcs = oid.as_oid().unwrap();
    assert_eq!(oid_to_string(&arcs), "1.2.840.113549.1.1.1");
    let _null = inner.read_tag(TAG_NULL).unwrap();
    assert!(inner.is_empty());
}

#[test]
fn long_form_length() {
    // OCTET STRING with 300 bytes of 0x42.
    let mut buf = vec![0x04, 0x82, 0x01, 0x2C];
    buf.extend_from_slice(&[0x42; 300]);
    let v = parse_single(&buf).unwrap();
    assert_eq!(v.tag, TAG_OCTET_STRING);
    assert_eq!(v.content.len(), 300);
    assert_eq!(v.content[0], 0x42);
    assert_eq!(v.content[299], 0x42);
}

#[test]
fn long_form_one_byte_non_minimal_rejected() {
    // 81 7F should have been 7F (short form). Reject as non-minimal.
    let buf = [0x04, 0x81, 0x7F, 0x00];
    let r = parse_single(&buf);
    assert!(matches!(r, Err(DerError::InvalidLength)));
}

#[test]
fn utf8_string_round_trip() {
    // UTF8String "hello"
    let buf = [0x0C, 0x05, b'h', b'e', b'l', b'l', b'o'];
    let v = parse_single(&buf).unwrap();
    assert_eq!(v.as_string().unwrap(), "hello");
}

#[test]
fn context_specific_tag() {
    // [0] EXPLICIT INTEGER 42:  A0 03 02 01 2A
    let buf = [0xA0, 0x03, 0x02, 0x01, 0x2A];
    let v = parse_single(&buf).unwrap();
    assert!(v.is_context_specific());
    assert_eq!(v.context_tag_number(), 0);
    assert!(v.is_constructed());
    let mut inner = v.into_reader().unwrap();
    let n = inner.read_tag(TAG_INTEGER).unwrap();
    assert_eq!(n.as_i64().unwrap(), 42);
}

#[test]
fn trailing_data_rejected() {
    // Single INTEGER followed by an extra byte.
    let buf = [0x02, 0x01, 0x01, 0xFF];
    let r = parse_single(&buf);
    assert!(matches!(r, Err(DerError::TrailingData)));
}

#[test]
fn truncated_value_rejected() {
    // Tag says length 5 but only 3 content bytes present.
    let buf = [0x04, 0x05, 0x01, 0x02, 0x03];
    let r = parse_single(&buf);
    assert!(matches!(r, Err(DerError::UnexpectedEnd)));
}
