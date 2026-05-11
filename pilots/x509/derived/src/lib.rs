// rusty-x509 pilot — X.509 v3 certificate parsing + signature verification.
//
// Π1.4.b round of the TLS substrate-amortization sequence. Builds on
// rusty-asn1-der (Π1.4.a) for parsing and rusty-web-crypto for
// signature verification primitives (RSA-PKCS1-v1.5, ECDSA-P-256/384,
// SHA-1/256/384/512). Per RFC 5280.
//
// Scope: parse a Certificate (DER-encoded), extract structural fields
// (version, serial, signature algorithm, issuer/subject Names, validity,
// SubjectPublicKeyInfo, extensions), and verify the certificate's
// signature against a provided issuer public key. Chain walk + system
// root store loading are deferred to Π1.4.c.
//
// Per Pin-Art Doc 707 bidirectional reading: the implementation
// surfaces several X.509 invariants that real-world certificates rely
// on — version 3 is universal in production; sha256WithRSAEncryption
// and ecdsa-with-SHA256 are the dominant signature algorithms; the
// signed tbsCertificate is contiguous in the DER encoding and can be
// re-located by tracking the SEQUENCE boundary during parsing.

use rusty_asn1_der::*;

#[derive(Debug, Clone)]
pub enum X509Error {
    DerParse(DerError),
    UnsupportedVersion(i64),
    UnsupportedSigAlg(String),
    UnsupportedPubKeyAlg(String),
    InvalidSpki,
    InvalidValidity,
    InvalidSignature,
    CryptoFail(String),
    PemBadHeader,
    PemBadBase64,
}

impl From<DerError> for X509Error {
    fn from(e: DerError) -> Self { X509Error::DerParse(e) }
}

impl std::fmt::Display for X509Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            X509Error::DerParse(e) => write!(f, "DER parse: {}", e),
            X509Error::UnsupportedVersion(v) => write!(f, "unsupported X.509 version {}", v),
            X509Error::UnsupportedSigAlg(o) => write!(f, "unsupported signature algorithm {}", o),
            X509Error::UnsupportedPubKeyAlg(o) => write!(f, "unsupported public key algorithm {}", o),
            X509Error::InvalidSpki => write!(f, "invalid SubjectPublicKeyInfo"),
            X509Error::InvalidValidity => write!(f, "invalid validity period"),
            X509Error::InvalidSignature => write!(f, "signature verification failed"),
            X509Error::CryptoFail(s) => write!(f, "crypto: {}", s),
            X509Error::PemBadHeader => write!(f, "PEM bad header (expected BEGIN/END CERTIFICATE)"),
            X509Error::PemBadBase64 => write!(f, "PEM base64 decode failed"),
        }
    }
}

impl std::error::Error for X509Error {}

// ─────────────────────────────────────────────────────────────────────
// Known OIDs (RFC 5280 + RFC 8017)
// ─────────────────────────────────────────────────────────────────────

pub const OID_RSA_ENCRYPTION: &str = "1.2.840.113549.1.1.1";
pub const OID_SHA1_WITH_RSA: &str = "1.2.840.113549.1.1.5";
pub const OID_SHA256_WITH_RSA: &str = "1.2.840.113549.1.1.11";
pub const OID_SHA384_WITH_RSA: &str = "1.2.840.113549.1.1.12";
pub const OID_SHA512_WITH_RSA: &str = "1.2.840.113549.1.1.13";
pub const OID_EC_PUBLIC_KEY: &str = "1.2.840.10045.2.1";
pub const OID_ECDSA_WITH_SHA256: &str = "1.2.840.10045.4.3.2";
pub const OID_ECDSA_WITH_SHA384: &str = "1.2.840.10045.4.3.3";
pub const OID_ECDSA_WITH_SHA512: &str = "1.2.840.10045.4.3.4";
pub const OID_P256_CURVE: &str = "1.2.840.10045.3.1.7";
pub const OID_P384_CURVE: &str = "1.3.132.0.34";
pub const OID_P521_CURVE: &str = "1.3.132.0.35";

// RDN attribute OIDs
pub const OID_RDN_CN: &str = "2.5.4.3";
pub const OID_RDN_C: &str = "2.5.4.6";
pub const OID_RDN_O: &str = "2.5.4.10";
pub const OID_RDN_OU: &str = "2.5.4.11";

// ─────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AlgorithmIdentifier {
    pub oid: String,
    /// Raw DER bytes of the parameters field (may be NULL → empty here).
    pub params: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct DistinguishedName {
    /// Flat list of (OID, value) pairs across all RDNs. Order preserved.
    /// Multi-valued RDNs (RelativeDistinguishedName with > 1 attribute) are
    /// flattened; consumers comparing DNs byte-by-byte should use the
    /// raw_der byte range below for canonical comparison.
    pub attributes: Vec<(String, String)>,
    /// Raw DER bytes of the Name (the SEQUENCE OF RDN). Used for issuer-
    /// subject matching during chain walks (byte equality is the
    /// canonical RFC 5280 test).
    pub raw_der: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Validity {
    /// Raw bytes of the notBefore time (UTCTime or GeneralizedTime).
    pub not_before: Vec<u8>,
    pub not_before_tag: u8,
    pub not_after: Vec<u8>,
    pub not_after_tag: u8,
}

#[derive(Debug, Clone)]
pub enum PublicKey {
    /// RSA: (modulus_n_bytes, exponent_e_bytes), both big-endian unsigned.
    Rsa { n: Vec<u8>, e: Vec<u8> },
    /// EC: curve OID + uncompressed point bytes (0x04 || X || Y).
    Ec { curve_oid: String, point: Vec<u8> },
}

#[derive(Debug, Clone)]
pub struct SubjectPublicKeyInfo {
    pub algorithm: AlgorithmIdentifier,
    pub key: PublicKey,
    /// Raw DER bytes of the entire SPKI SEQUENCE. Used downstream by
    /// TLS handshake key extraction and by SAN-pinning consumers.
    pub raw_der: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Extension {
    pub oid: String,
    pub critical: bool,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Certificate {
    pub version: u8,
    pub serial_number: Vec<u8>,
    pub signature_algorithm: AlgorithmIdentifier,
    pub issuer: DistinguishedName,
    pub validity: Validity,
    pub subject: DistinguishedName,
    pub subject_public_key_info: SubjectPublicKeyInfo,
    pub extensions: Vec<Extension>,
    /// Raw DER bytes of tbsCertificate. The signature covers this exact
    /// byte sequence — the signature verification step hashes these
    /// bytes and verifies against signature_value.
    pub tbs_certificate: Vec<u8>,
    /// Signature value bit-string content (no unused-bits prefix).
    pub signature_value: Vec<u8>,
}

// ─────────────────────────────────────────────────────────────────────
// Parser
// ─────────────────────────────────────────────────────────────────────

/// Parse a DER-encoded X.509 certificate.
pub fn parse_certificate(der: &[u8]) -> Result<Certificate, X509Error> {
    let outer = parse_single(der)?;
    if outer.tag != TAG_SEQUENCE { return Err(X509Error::DerParse(
        DerError::WrongTag { expected: TAG_SEQUENCE, actual: outer.tag })); }

    // The outer SEQUENCE wraps three things: tbsCertificate (SEQUENCE),
    // signatureAlgorithm (SEQUENCE), signatureValue (BIT STRING).
    // We need to capture the byte range of tbsCertificate as it sits in
    // the original DER buffer for signature verification.

    let mut outer_reader = DerReader::new(outer.content);

    // Locate tbsCertificate start within the original buffer.
    // outer.content is &der[offset..offset+len], so we walk through
    // outer.content but compute byte offsets relative to der.
    let outer_content_start = outer.content.as_ptr() as usize - der.as_ptr() as usize;
    let tbs_start_in_outer = 0;
    let tbs_start_in_der = outer_content_start + tbs_start_in_outer;

    // Read tbsCertificate as a TLV; capture the full TLV bytes including
    // the header. Re-derive from der using the position.
    let tbs_value = outer_reader.read_tlv()?;
    if tbs_value.tag != TAG_SEQUENCE { return Err(X509Error::DerParse(
        DerError::WrongTag { expected: TAG_SEQUENCE, actual: tbs_value.tag })); }
    // tbsCertificate's full bytes are tag+length+content. Length-prefix
    // size is variable; rebuild by walking the DER form.
    let tbs_end_in_der = {
        let content_start = tbs_value.content.as_ptr() as usize - der.as_ptr() as usize;
        content_start + tbs_value.content.len()
    };
    let tbs_certificate = der[tbs_start_in_der..tbs_end_in_der].to_vec();

    let tbs = parse_tbs(tbs_value.content)?;

    // signatureAlgorithm (must equal the one inside tbs.signature per spec;
    // RFC 5280 §4.1.1.2 — we read but don't enforce equality here).
    let sig_alg_value = outer_reader.read_tag(TAG_SEQUENCE)?;
    let sig_alg = parse_algorithm_identifier(&sig_alg_value)?;

    // signatureValue: BIT STRING.
    let sig_bs = outer_reader.read_tag(TAG_BIT_STRING)?;
    let (_unused, sig_bytes) = sig_bs.as_bit_string()?;

    Ok(Certificate {
        version: tbs.version,
        serial_number: tbs.serial_number,
        signature_algorithm: sig_alg,
        issuer: tbs.issuer,
        validity: tbs.validity,
        subject: tbs.subject,
        subject_public_key_info: tbs.spki,
        extensions: tbs.extensions,
        tbs_certificate,
        signature_value: sig_bytes.to_vec(),
    })
}

struct TbsFields {
    version: u8,
    serial_number: Vec<u8>,
    issuer: DistinguishedName,
    validity: Validity,
    subject: DistinguishedName,
    spki: SubjectPublicKeyInfo,
    extensions: Vec<Extension>,
}

fn parse_tbs(tbs_content: &[u8]) -> Result<TbsFields, X509Error> {
    let mut r = DerReader::new(tbs_content);

    // [0] EXPLICIT Version DEFAULT v1.
    let mut version: u8 = 1;
    if let Some(t) = r.peek_tag() {
        if t == 0xA0 {
            let ver_wrap = r.read_tlv()?;
            let inner = ver_wrap.into_reader()?;
            let mut inner = inner;
            let v_val = inner.read_tag(TAG_INTEGER)?;
            let v = v_val.as_i64()?;
            // Encoded value: 0=v1, 1=v2, 2=v3. Map to 1/2/3.
            match v {
                0 => version = 1,
                1 => version = 2,
                2 => version = 3,
                _ => return Err(X509Error::UnsupportedVersion(v)),
            }
        }
    }

    // serialNumber INTEGER.
    let serial = r.read_tag(TAG_INTEGER)?;
    let serial_number = serial.content.to_vec();

    // signature AlgorithmIdentifier (read but discard; outer copy is the
    // canonical one).
    let _sig_inner = r.read_tag(TAG_SEQUENCE)?;

    // issuer Name.
    let issuer_value = r.read_tag(TAG_SEQUENCE)?;
    let issuer = parse_name(&issuer_value)?;

    // validity SEQUENCE { notBefore, notAfter }.
    let validity_value = r.read_tag(TAG_SEQUENCE)?;
    let validity = parse_validity(&validity_value)?;

    // subject Name.
    let subject_value = r.read_tag(TAG_SEQUENCE)?;
    let subject = parse_name(&subject_value)?;

    // subjectPublicKeyInfo SEQUENCE.
    let spki_value = r.read_tag(TAG_SEQUENCE)?;
    let spki = parse_spki(&spki_value)?;

    // Optional issuerUniqueID [1] IMPLICIT BIT STRING (skip).
    // Optional subjectUniqueID [2] IMPLICIT BIT STRING (skip).
    // Optional extensions [3] EXPLICIT SEQUENCE OF Extension.
    let mut extensions = Vec::new();
    while let Some(t) = r.peek_tag() {
        let v = r.read_tlv()?;
        match t {
            0x81 => { /* issuerUniqueID, ignore */ }
            0x82 => { /* subjectUniqueID, ignore */ }
            0xA3 => {
                // [3] EXPLICIT Extensions ::= SEQUENCE OF Extension.
                let mut inner = v.into_reader()?;
                let ext_seq = inner.read_tag(TAG_SEQUENCE)?;
                let mut ext_reader = ext_seq.into_reader()?;
                while !ext_reader.is_empty() {
                    let ext_v = ext_reader.read_tag(TAG_SEQUENCE)?;
                    extensions.push(parse_extension(&ext_v)?);
                }
            }
            _ => { /* unknown; skip */ }
        }
    }

    Ok(TbsFields {
        version, serial_number, issuer, validity, subject, spki, extensions,
    })
}

fn parse_algorithm_identifier(v: &DerValue) -> Result<AlgorithmIdentifier, X509Error> {
    let mut r = DerReader::new(v.content);
    let oid_val = r.read_tag(TAG_OID)?;
    let oid = oid_to_string(&oid_val.as_oid()?);
    let params = if r.is_empty() { Vec::new() } else {
        r.remaining().to_vec()
    };
    Ok(AlgorithmIdentifier { oid, params })
}

fn parse_name(v: &DerValue) -> Result<DistinguishedName, X509Error> {
    let raw_der = {
        // Reconstruct the SEQUENCE bytes (tag + length + content) so the
        // raw_der byte equality holds at chain-walk time.
        let mut bytes = vec![v.tag];
        append_length(v.content.len(), &mut bytes);
        bytes.extend_from_slice(v.content);
        bytes
    };
    let mut attrs = Vec::new();
    let mut rdn_reader = DerReader::new(v.content);
    while !rdn_reader.is_empty() {
        let rdn = rdn_reader.read_tag(TAG_SET)?;
        let mut atv_reader = DerReader::new(rdn.content);
        while !atv_reader.is_empty() {
            let atv = atv_reader.read_tag(TAG_SEQUENCE)?;
            let mut atv_inner = DerReader::new(atv.content);
            let oid_val = atv_inner.read_tag(TAG_OID)?;
            let oid = oid_to_string(&oid_val.as_oid()?);
            let val_v = atv_inner.read_tlv()?;
            let val_s = match val_v.tag {
                TAG_UTF8_STRING | TAG_PRINTABLE_STRING | TAG_IA5_STRING | TAG_TELETEX_STRING => {
                    val_v.as_string().unwrap_or("").to_string()
                }
                _ => String::new(),
            };
            attrs.push((oid, val_s));
        }
    }
    Ok(DistinguishedName { attributes: attrs, raw_der })
}

fn parse_validity(v: &DerValue) -> Result<Validity, X509Error> {
    let mut r = DerReader::new(v.content);
    let nb = r.read_tlv()?;
    let na = r.read_tlv()?;
    if !matches!(nb.tag, TAG_UTC_TIME | TAG_GENERALIZED_TIME) {
        return Err(X509Error::InvalidValidity);
    }
    if !matches!(na.tag, TAG_UTC_TIME | TAG_GENERALIZED_TIME) {
        return Err(X509Error::InvalidValidity);
    }
    Ok(Validity {
        not_before: nb.content.to_vec(),
        not_before_tag: nb.tag,
        not_after: na.content.to_vec(),
        not_after_tag: na.tag,
    })
}

fn parse_spki(v: &DerValue) -> Result<SubjectPublicKeyInfo, X509Error> {
    let raw_der = {
        let mut bytes = vec![v.tag];
        append_length(v.content.len(), &mut bytes);
        bytes.extend_from_slice(v.content);
        bytes
    };
    let mut r = DerReader::new(v.content);
    let alg_v = r.read_tag(TAG_SEQUENCE)?;
    let alg = parse_algorithm_identifier(&alg_v)?;
    let bs = r.read_tag(TAG_BIT_STRING)?;
    let (unused, key_bytes) = bs.as_bit_string()?;
    if unused != 0 { return Err(X509Error::InvalidSpki); }
    let key = match alg.oid.as_str() {
        OID_RSA_ENCRYPTION => {
            // SubjectPublicKey for RSA is RSAPublicKey ::= SEQUENCE { n INTEGER, e INTEGER }.
            let rsa_seq = parse_single(key_bytes)?;
            if rsa_seq.tag != TAG_SEQUENCE { return Err(X509Error::InvalidSpki); }
            let mut rsa_reader = DerReader::new(rsa_seq.content);
            let n_val = rsa_reader.read_tag(TAG_INTEGER)?;
            let n = n_val.as_unsigned_integer()?.to_vec();
            let e_val = rsa_reader.read_tag(TAG_INTEGER)?;
            let e = e_val.as_unsigned_integer()?.to_vec();
            PublicKey::Rsa { n, e }
        }
        OID_EC_PUBLIC_KEY => {
            // params is the curve OID.
            let params_value = parse_single(&alg.params)?;
            if params_value.tag != TAG_OID { return Err(X509Error::InvalidSpki); }
            let curve_arcs = params_value.as_oid()?;
            let curve_oid = oid_to_string(&curve_arcs);
            // subjectPublicKey BIT STRING is the uncompressed point
            // 0x04 || X || Y.
            PublicKey::Ec { curve_oid, point: key_bytes.to_vec() }
        }
        _ => return Err(X509Error::UnsupportedPubKeyAlg(alg.oid.clone())),
    };
    Ok(SubjectPublicKeyInfo { algorithm: alg, key, raw_der })
}

fn parse_extension(v: &DerValue) -> Result<Extension, X509Error> {
    let mut r = DerReader::new(v.content);
    let oid_val = r.read_tag(TAG_OID)?;
    let oid = oid_to_string(&oid_val.as_oid()?);
    let mut critical = false;
    let next = r.peek_tag();
    if next == Some(TAG_BOOLEAN) {
        let cv = r.read_tag(TAG_BOOLEAN)?;
        critical = cv.as_bool()?;
    }
    let val_v = r.read_tag(TAG_OCTET_STRING)?;
    Ok(Extension { oid, critical, value: val_v.content.to_vec() })
}

fn append_length(n: usize, out: &mut Vec<u8>) {
    if n < 0x80 {
        out.push(n as u8);
    } else {
        let mut len_bytes = Vec::new();
        let mut tmp = n;
        while tmp > 0 { len_bytes.push((tmp & 0xFF) as u8); tmp >>= 8; }
        len_bytes.reverse();
        out.push(0x80 | (len_bytes.len() as u8));
        out.extend_from_slice(&len_bytes);
    }
}

// ─────────────────────────────────────────────────────────────────────
// Signature verification
// ─────────────────────────────────────────────────────────────────────

/// Verify `cert`'s signature using `issuer_spki`'s public key. The
/// chain walk to a trust anchor is the consumer's responsibility this
/// round; we verify one link.
pub fn verify_signature(
    cert: &Certificate,
    issuer_spki: &SubjectPublicKeyInfo,
) -> Result<(), X509Error> {
    let sig_oid = cert.signature_algorithm.oid.as_str();
    match sig_oid {
        OID_SHA256_WITH_RSA | OID_SHA384_WITH_RSA | OID_SHA512_WITH_RSA |
        OID_SHA1_WITH_RSA => {
            let (n, e) = match &issuer_spki.key {
                PublicKey::Rsa { n, e } => (n, e),
                _ => return Err(X509Error::UnsupportedSigAlg(sig_oid.into())),
            };
            let (hash, hash_name) = compute_hash_for_rsa(sig_oid, &cert.tbs_certificate)?;
            rusty_web_crypto::rsa_pkcs1_v15_verify(n, e, &hash, &cert.signature_value, hash_name)
                .map_err(X509Error::CryptoFail)
        }
        OID_ECDSA_WITH_SHA256 | OID_ECDSA_WITH_SHA384 | OID_ECDSA_WITH_SHA512 => {
            let (curve_oid, point) = match &issuer_spki.key {
                PublicKey::Ec { curve_oid, point } => (curve_oid, point),
                _ => return Err(X509Error::UnsupportedSigAlg(sig_oid.into())),
            };
            let curve = match curve_oid.as_str() {
                OID_P256_CURVE => rusty_web_crypto::curve_p256(),
                OID_P384_CURVE => rusty_web_crypto::curve_p384(),
                _ => return Err(X509Error::UnsupportedPubKeyAlg(curve_oid.clone())),
            };
            // Uncompressed point: 0x04 || X || Y.
            if point.is_empty() || point[0] != 0x04 {
                return Err(X509Error::InvalidSpki);
            }
            let coord = curve.coord_bytes;
            if point.len() != 1 + 2 * coord { return Err(X509Error::InvalidSpki); }
            let qx = &point[1..1 + coord];
            let qy = &point[1 + coord..];
            // ECDSA signature is encoded as SEQUENCE { INTEGER r, INTEGER s }.
            let sig_seq = parse_single(&cert.signature_value)?;
            if sig_seq.tag != TAG_SEQUENCE { return Err(X509Error::InvalidSignature); }
            let mut sig_reader = DerReader::new(sig_seq.content);
            let r_val = sig_reader.read_tag(TAG_INTEGER)?;
            let s_val = sig_reader.read_tag(TAG_INTEGER)?;
            let r = r_val.as_unsigned_integer()?;
            let s = s_val.as_unsigned_integer()?;
            // Pad r and s to coord_bytes (left-pad with zeros).
            let mut sig_raw = vec![0u8; 2 * coord];
            sig_raw[coord - r.len()..coord].copy_from_slice(r);
            sig_raw[2 * coord - s.len()..].copy_from_slice(s);
            let hash = match sig_oid {
                OID_ECDSA_WITH_SHA256 => rusty_web_crypto::digest_sha256(&cert.tbs_certificate).to_vec(),
                OID_ECDSA_WITH_SHA384 => rusty_web_crypto::digest_sha384(&cert.tbs_certificate).to_vec(),
                OID_ECDSA_WITH_SHA512 => rusty_web_crypto::digest_sha512(&cert.tbs_certificate).to_vec(),
                _ => unreachable!(),
            };
            rusty_web_crypto::ecdsa_verify(&curve, qx, qy, &hash, &sig_raw)
                .map_err(X509Error::CryptoFail)
        }
        _ => Err(X509Error::UnsupportedSigAlg(sig_oid.into())),
    }
}

fn compute_hash_for_rsa(sig_oid: &str, tbs: &[u8]) -> Result<(Vec<u8>, &'static str), X509Error> {
    match sig_oid {
        OID_SHA1_WITH_RSA => Ok((rusty_web_crypto::digest_sha1(tbs).to_vec(), "SHA-1")),
        OID_SHA256_WITH_RSA => Ok((rusty_web_crypto::digest_sha256(tbs).to_vec(), "SHA-256")),
        OID_SHA384_WITH_RSA => Ok((rusty_web_crypto::digest_sha384(tbs).to_vec(), "SHA-384")),
        OID_SHA512_WITH_RSA => Ok((rusty_web_crypto::digest_sha512(tbs).to_vec(), "SHA-512")),
        _ => Err(X509Error::UnsupportedSigAlg(sig_oid.into())),
    }
}

// ─────────────────────────────────────────────────────────────────────
// PEM decoding (RFC 7468)
// ─────────────────────────────────────────────────────────────────────

const BEGIN: &str = "-----BEGIN CERTIFICATE-----";
const END: &str = "-----END CERTIFICATE-----";

/// Decode a PEM-encoded certificate to DER bytes. Returns the bytes of
/// the first BEGIN/END CERTIFICATE block.
pub fn pem_to_der(pem: &str) -> Result<Vec<u8>, X509Error> {
    let begin_pos = pem.find(BEGIN).ok_or(X509Error::PemBadHeader)?;
    let after_begin = begin_pos + BEGIN.len();
    let end_pos = pem[after_begin..].find(END).ok_or(X509Error::PemBadHeader)?;
    let b64_block: String = pem[after_begin..after_begin + end_pos]
        .chars().filter(|c| !c.is_whitespace()).collect();
    base64_decode(&b64_block)
}

/// Decode all PEM CERTIFICATE blocks in `pem` to DER.
pub fn pem_all_to_der(pem: &str) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut cursor = pem;
    while let Some(b) = cursor.find(BEGIN) {
        let after_begin = b + BEGIN.len();
        let rest = &cursor[after_begin..];
        if let Some(e) = rest.find(END) {
            let b64: String = rest[..e].chars().filter(|c| !c.is_whitespace()).collect();
            if let Ok(der) = base64_decode(&b64) {
                out.push(der);
            }
            cursor = &rest[e + END.len()..];
        } else {
            break;
        }
    }
    out
}

fn base64_decode(s: &str) -> Result<Vec<u8>, X509Error> {
    const BAD: u8 = 255;
    let mut table = [BAD; 256];
    for (i, c) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
        table[*c as usize] = i as u8;
    }
    let mut out = Vec::with_capacity((s.len() * 3) / 4);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 3 < bytes.len() {
        let a = table[bytes[i] as usize];
        let b = table[bytes[i + 1] as usize];
        let c = table[bytes[i + 2] as usize];
        let d = table[bytes[i + 3] as usize];
        i += 4;
        if a == BAD || b == BAD { return Err(X509Error::PemBadBase64); }
        if bytes[i - 4 + 2] == b'=' {
            // Two-byte block with two padding chars (only `a b = =`).
            out.push((a << 2) | (b >> 4));
            break;
        }
        if c == BAD { return Err(X509Error::PemBadBase64); }
        if bytes[i - 4 + 3] == b'=' {
            out.push((a << 2) | (b >> 4));
            out.push((b << 4) | (c >> 2));
            break;
        }
        if d == BAD { return Err(X509Error::PemBadBase64); }
        out.push((a << 2) | (b >> 4));
        out.push((b << 4) | (c >> 2));
        out.push((c << 6) | d);
    }
    Ok(out)
}
