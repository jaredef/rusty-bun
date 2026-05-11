// TLS 1.3 client handshake driver.
//
// Π1.4.f.a round (the closure of substrate work; Π1.4.f.b will hook
// this into the host's fetch path with a Tier-J fixture against a
// localhost openssl s_server).
//
// This module composes Π1.4.a-e into a callable handshake state
// machine. The caller supplies blocking read/write callbacks against
// an established TCP connection; the driver returns a TlsSession with
// app-traffic AEAD keys plus methods to send/receive application
// data.

use crate::record::{TlsError, TlsRecord, ContentType, ProtocolVersion,
                    encode_record, decode_record, MAX_CIPHERTEXT_LEN};
use crate::handshake::*;
use crate::client::*;
use crate::store::*;

use rusty_x509::{Certificate as X509Cert, PublicKey, SubjectPublicKeyInfo};

/// Verify a TLS 1.3 CertificateVerify signature per RFC 8446 §4.4.3.
/// `scheme` is the SignatureScheme from the CertificateVerify wire format.
/// `spki` is the leaf cert's SubjectPublicKeyInfo.
/// `tbs` is the constructed to-be-signed bytes (64 spaces + context + 0 + transcript hash).
/// `signature` is the raw signature bytes from the CertificateVerify.
fn verify_certificate_verify_signature(
    scheme: u16,
    spki: &SubjectPublicKeyInfo,
    tbs: &[u8],
    signature: &[u8],
) -> Result<(), TlsError> {
    match scheme {
        // RSA-PKCS1-v1.5 (legacy; permitted only for cert signatures in 1.3 but used).
        SIG_RSA_PKCS1_SHA256 | SIG_RSA_PKCS1_SHA384 | SIG_RSA_PKCS1_SHA512 => {
            let (n, e) = match &spki.key {
                PublicKey::Rsa { n, e } => (n, e),
                _ => return Err(TlsError::SignatureFail("RSA scheme but leaf is not RSA".into())),
            };
            let (hash, name) = match scheme {
                SIG_RSA_PKCS1_SHA256 => (rusty_web_crypto::digest_sha256(tbs).to_vec(), "SHA-256"),
                SIG_RSA_PKCS1_SHA384 => (rusty_web_crypto::digest_sha384(tbs).to_vec(), "SHA-384"),
                SIG_RSA_PKCS1_SHA512 => (rusty_web_crypto::digest_sha512(tbs).to_vec(), "SHA-512"),
                _ => unreachable!(),
            };
            rusty_web_crypto::rsa_pkcs1_v15_verify(n, e, &hash, signature, name)
                .map_err(TlsError::SignatureFail)
        }
        // RSA-PSS-RSAE-* (the preferred RSA scheme for TLS 1.3 CertificateVerify).
        SIG_RSA_PSS_RSAE_SHA256 | SIG_RSA_PSS_RSAE_SHA384 => {
            let (n, e) = match &spki.key {
                PublicKey::Rsa { n, e } => (n, e),
                _ => return Err(TlsError::SignatureFail("RSA-PSS scheme but leaf is not RSA".into())),
            };
            let (hlen, hash_fn): (usize, fn(&[u8]) -> Vec<u8>) = match scheme {
                SIG_RSA_PSS_RSAE_SHA256 => (32, |d| rusty_web_crypto::digest_sha256(d).to_vec()),
                SIG_RSA_PSS_RSAE_SHA384 => (48, |d| rusty_web_crypto::digest_sha384(d).to_vec()),
                _ => unreachable!(),
            };
            // Per RFC 8446 §4.2.3: salt length equals the hash output length.
            rusty_web_crypto::rsa_pss_verify(n, e, tbs, signature, hlen, hash_fn, hlen)
                .map_err(TlsError::SignatureFail)
        }
        // ECDSA over a NIST curve.
        SIG_ECDSA_SECP256R1_SHA256 | SIG_ECDSA_SECP384R1_SHA384 => {
            let (curve_oid, point) = match &spki.key {
                PublicKey::Ec { curve_oid, point } => (curve_oid, point),
                _ => return Err(TlsError::SignatureFail("ECDSA scheme but leaf is not EC".into())),
            };
            let curve = match curve_oid.as_str() {
                rusty_x509::OID_P256_CURVE => rusty_web_crypto::curve_p256(),
                rusty_x509::OID_P384_CURVE => rusty_web_crypto::curve_p384(),
                _ => return Err(TlsError::SignatureFail(
                    format!("unsupported EC curve {}", curve_oid))),
            };
            if point.is_empty() || point[0] != 0x04 || point.len() != 1 + 2 * curve.coord_bytes {
                return Err(TlsError::SignatureFail("malformed EC pubkey".into()));
            }
            let coord = curve.coord_bytes;
            let qx = &point[1..1 + coord];
            let qy = &point[1 + coord..];
            let hash = match scheme {
                SIG_ECDSA_SECP256R1_SHA256 => rusty_web_crypto::digest_sha256(tbs).to_vec(),
                SIG_ECDSA_SECP384R1_SHA384 => rusty_web_crypto::digest_sha384(tbs).to_vec(),
                _ => unreachable!(),
            };
            // ECDSA TLS signature is DER SEQUENCE { r INTEGER, s INTEGER }.
            let sig_seq = rusty_asn1_der::parse_single(signature)
                .map_err(|e| TlsError::SignatureFail(format!("ECDSA sig DER: {}", e)))?;
            if sig_seq.tag != rusty_asn1_der::TAG_SEQUENCE {
                return Err(TlsError::SignatureFail("ECDSA sig not SEQUENCE".into()));
            }
            let mut reader = rusty_asn1_der::DerReader::new(sig_seq.content);
            let r_val = reader.read_tag(rusty_asn1_der::TAG_INTEGER)
                .map_err(|e| TlsError::SignatureFail(format!("ECDSA r: {}", e)))?;
            let s_val = reader.read_tag(rusty_asn1_der::TAG_INTEGER)
                .map_err(|e| TlsError::SignatureFail(format!("ECDSA s: {}", e)))?;
            let r = r_val.as_unsigned_integer()
                .map_err(|e| TlsError::SignatureFail(format!("ECDSA r unsigned: {}", e)))?;
            let s = s_val.as_unsigned_integer()
                .map_err(|e| TlsError::SignatureFail(format!("ECDSA s unsigned: {}", e)))?;
            let mut sig_raw = vec![0u8; 2 * coord];
            sig_raw[coord - r.len()..coord].copy_from_slice(r);
            sig_raw[2 * coord - s.len()..].copy_from_slice(s);
            rusty_web_crypto::ecdsa_verify(&curve, qx, qy, &hash, &sig_raw)
                .map_err(TlsError::SignatureFail)
        }
        _ => Err(TlsError::SignatureFail(
            format!("unsupported SignatureScheme 0x{:04x}", scheme))),
    }
}

/// Synchronous I/O callbacks. The driver writes records via `write`
/// and reads records via `read` (the read should block until at least
/// one full record's worth of bytes is available, then return them;
/// short reads are handled by the driver's accumulator).
pub trait TlsTransport {
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), TlsError>;
    fn read_some(&mut self, buf: &mut Vec<u8>) -> Result<usize, TlsError>;
}

/// Ephemeral ECDH-P-256 keypair for the key_share extension.
pub struct EphemeralEcdh {
    pub private_scalar: Vec<u8>,    // 32 bytes
    pub public_point: Vec<u8>,      // 65 bytes: 0x04 || X || Y
}

impl EphemeralEcdh {
    /// Generate a new ephemeral P-256 keypair.
    pub fn generate() -> Result<Self, TlsError> {
        let mut sk = [0u8; 32];
        rusty_web_crypto::get_random_values(&mut sk)
            .map_err(|e| TlsError::SignatureFail(format!("RNG: {}", e)))?;
        // Clamp into [1, n-1]: the chance of an invalid sample for
        // P-256 is negligible (~2^-128); for hygiene, reject 0 and
        // any value ≥ n. We approximate by ensuring high bit clear
        // and re-sample if all zero.
        sk[0] &= 0x7F;  // ensure scalar < n with high probability
        if sk == [0u8; 32] { sk[31] = 1; }
        let curve = rusty_web_crypto::curve_p256();
        use rusty_web_crypto::{BigUInt, p256_scalar_mul};
        let scalar = BigUInt::from_be_bytes(&sk);
        let g = curve.g.clone();
        let pubpt = p256_scalar_mul(&scalar, &g);
        let (px, py) = match pubpt {
            rusty_web_crypto::P256Point::Affine { x, y } => (x.to_be_bytes(32), y.to_be_bytes(32)),
            rusty_web_crypto::P256Point::Identity => return Err(TlsError::SignatureFail(
                "ECDH ephemeral produced identity point".into())),
        };
        let mut public_point = Vec::with_capacity(65);
        public_point.push(0x04);
        public_point.extend_from_slice(&px);
        public_point.extend_from_slice(&py);
        Ok(EphemeralEcdh { private_scalar: sk.to_vec(), public_point })
    }

    /// Derive the ECDH shared secret given the server's public point
    /// (uncompressed: 0x04 || X || Y, 65 bytes).
    pub fn shared_secret(&self, server_pubkey: &[u8]) -> Result<Vec<u8>, TlsError> {
        if server_pubkey.len() != 65 || server_pubkey[0] != 0x04 {
            return Err(TlsError::SignatureFail("server key_share not uncompressed P-256".into()));
        }
        use rusty_web_crypto::{BigUInt, P256Point, p256_scalar_mul, curve_p256};
        let x = BigUInt::from_be_bytes(&server_pubkey[1..33]);
        let y = BigUInt::from_be_bytes(&server_pubkey[33..65]);
        let q = P256Point::Affine { x, y };
        let scalar = BigUInt::from_be_bytes(&self.private_scalar);
        let _curve = curve_p256();
        let shared_point = p256_scalar_mul(&scalar, &q);
        match shared_point {
            P256Point::Affine { x, .. } => Ok(x.to_be_bytes(32)),
            P256Point::Identity => Err(TlsError::SignatureFail("ECDH produced identity point".into())),
        }
    }
}

/// Result of a completed handshake: app-traffic AEAD keys plus the
/// TCP transport ownership.
pub struct TlsSession<T: TlsTransport> {
    pub transport: T,
    pub client_app_keys: TrafficKeys,
    pub server_app_keys: TrafficKeys,
    pub client_app_seq: u64,
    pub server_app_seq: u64,
    /// Hash algorithm negotiated (per the cipher suite).
    pub hash: HashAlgorithm,
}

impl<T: TlsTransport> TlsSession<T> {
    /// Send application data through the negotiated AEAD.
    pub fn send_application_data(&mut self, data: &[u8]) -> Result<(), TlsError> {
        let ct = aead_encrypt_record(&self.client_app_keys, self.client_app_seq,
                                     ContentType::ApplicationData as u8, data)?;
        self.client_app_seq += 1;
        let record = TlsRecord {
            content_type: ContentType::ApplicationData,
            version: ProtocolVersion::LEGACY,
            fragment: ct,
        };
        self.transport.write_all(&encode_record(&record)?)
    }

    /// Receive application data, decrypting one record at a time.
    /// Returns the decrypted application payload (may be empty on
    /// alert records, which the caller should handle).
    pub fn receive_application_data(&mut self, accumulator: &mut Vec<u8>) -> Result<Vec<u8>, TlsError> {
        loop {
            // Try to decode a complete record from the accumulator.
            if let Ok((rec, n)) = decode_record(accumulator) {
                accumulator.drain(..n);
                if rec.content_type != ContentType::ApplicationData {
                    // Plaintext records during the application phase
                    // are unexpected except for ChangeCipherSpec which
                    // we ignore per RFC 8446 §5.
                    if rec.content_type == ContentType::ChangeCipherSpec { continue; }
                    return Err(TlsError::SignatureFail("unexpected plaintext record post-handshake".into()));
                }
                let (inner_ct, plaintext) = aead_decrypt_record(
                    &self.server_app_keys, self.server_app_seq, &rec.fragment)?;
                self.server_app_seq += 1;
                match inner_ct {
                    23 /* ApplicationData */ => return Ok(plaintext),
                    21 /* Alert */ => return Err(TlsError::SignatureFail(
                        format!("TLS alert during app data: {:?}", plaintext))),
                    22 /* Handshake */ => {
                        // Post-handshake messages (NewSessionTicket,
                        // KeyUpdate). Ignore for this round.
                        continue;
                    }
                    _ => return Err(TlsError::SignatureFail(
                        format!("unknown inner content type {}", inner_ct))),
                }
            } else {
                // Need more bytes.
                let _n = self.transport.read_some(accumulator)?;
                if accumulator.len() > MAX_CIPHERTEXT_LEN + 5 + 16 && decode_record(accumulator).is_err() {
                    return Err(TlsError::SignatureFail("record buffer overflow without progress".into()));
                }
            }
        }
    }
}

/// Run the TLS 1.3 1-RTT client handshake.
///
/// Scope (Π1.4.f.a): ECDH key generation, build ClientHello, write to
/// transport. Reading the server's response, decrypting EncryptedExtensions /
/// Certificate / CertificateVerify / Finished, validating the chain
/// against `trust_store`, and sending the client Finished is the work
/// of Π1.4.f.b — this round establishes the call shape and the
/// keygen + ClientHello pieces.
pub fn initiate_handshake<T: TlsTransport>(
    transport: &mut T,
    hostname: &str,
    _trust_store: &TrustStore,
) -> Result<EphemeralEcdh, TlsError> {
    let ephemeral = EphemeralEcdh::generate()?;
    let mut client_random = [0u8; 32];
    rusty_web_crypto::get_random_values(&mut client_random)
        .map_err(|e| TlsError::SignatureFail(format!("RNG: {}", e)))?;
    let ch = ClientHelloParams {
        random: &client_random,
        legacy_session_id: &[],
        cipher_suites: &[CIPHER_AES_128_GCM_SHA256],
        server_name: Some(hostname),
        supported_groups: &[GROUP_SECP256R1],
        signature_algorithms: &[
            SIG_ECDSA_SECP256R1_SHA256,
            SIG_RSA_PKCS1_SHA256,
            SIG_RSA_PSS_RSAE_SHA256,
        ],
        key_shares: &[(GROUP_SECP256R1, ephemeral.public_point.clone())],
        alpn: None,
    };
    let ch_bytes = encode_client_hello(&ch)?;
    let record = TlsRecord {
        content_type: ContentType::Handshake,
        version: ProtocolVersion::LEGACY,
        fragment: ch_bytes,
    };
    transport.write_all(&encode_record(&record)?)?;
    Ok(ephemeral)
}

/// Complete a TLS 1.3 1-RTT handshake. Caller has already invoked
/// `initiate_handshake` (which wrote ClientHello and returned the
/// ephemeral keypair). This function reads from the transport,
/// processes ServerHello + encrypted handshake messages, validates
/// the server certificate chain, sends client Finished, and returns
/// the established TlsSession.
///
/// Parameters:
/// - `transport`: the I/O backend (taken by value; returned inside TlsSession on success).
/// - `ephemeral`: the keypair from initiate_handshake.
/// - `client_hello_bytes`: the exact handshake-message body bytes of the
///   ClientHello (NOT the record wrapper), needed for the transcript hash.
/// - `trust_store`: the system trust store; chain walks against it.
/// - `hostname`: server hostname for SNI matching (not validated here;
///   caller should also verify SAN/CN after cert parsing).
pub fn complete_handshake<T: TlsTransport>(
    mut transport: T,
    ephemeral: EphemeralEcdh,
    client_hello_handshake_msg: &[u8],
    trust_store: &TrustStore,
    _hostname: &str,
) -> Result<TlsSession<T>, TlsError> {
    let hash = HashAlgorithm::Sha256;
    let mut transcript = Vec::new();
    transcript.extend_from_slice(client_hello_handshake_msg);

    // ── Phase 1: read records until ServerHello ──
    let mut accumulator: Vec<u8> = Vec::new();
    let mut server_hello: Option<ServerHello> = None;
    let mut server_hello_handshake_msg: Vec<u8> = Vec::new();
    while server_hello.is_none() {
        let (rec, n) = match decode_record(&accumulator) {
            Ok(r) => r,
            Err(_) => { transport.read_some(&mut accumulator)?; continue; }
        };
        accumulator.drain(..n);
        match rec.content_type {
            ContentType::ChangeCipherSpec => continue,  // ignore per §5
            ContentType::Handshake => {
                let mut pos = 0;
                while pos < rec.fragment.len() {
                    let (msg, used) = decode_handshake(&rec.fragment[pos..])?;
                    let msg_bytes = &rec.fragment[pos..pos + used];
                    pos += used;
                    if msg.msg_type == HandshakeType::ServerHello {
                        server_hello = Some(decode_server_hello(&msg.body)?);
                        server_hello_handshake_msg = msg_bytes.to_vec();
                        transcript.extend_from_slice(msg_bytes);
                        break;
                    } else {
                        return Err(TlsError::SignatureFail(
                            format!("unexpected handshake type {:?} before ServerHello", msg.msg_type)));
                    }
                }
            }
            ContentType::Alert => {
                return Err(TlsError::SignatureFail(format!("server alert: {:?}", rec.fragment)));
            }
            _ => return Err(TlsError::SignatureFail(
                "unexpected content type before ServerHello".into())),
        }
    }
    let sh = server_hello.unwrap();
    let _ = server_hello_handshake_msg;

    // ── Phase 2: derive keys ──
    if sh.selected_version() != Some(0x0304) {
        return Err(TlsError::SignatureFail("server did not select TLS 1.3".into()));
    }
    if sh.cipher_suite != CIPHER_AES_128_GCM_SHA256 {
        return Err(TlsError::SignatureFail(
            format!("server selected unsupported cipher 0x{:04x}", sh.cipher_suite)));
    }
    let (group, server_pub) = sh.server_key_share()
        .ok_or(TlsError::SignatureFail("ServerHello missing key_share".into()))?;
    if group != GROUP_SECP256R1 {
        return Err(TlsError::SignatureFail("server selected non-P256 group".into()));
    }
    let dhe = ephemeral.shared_secret(server_pub)?;
    let schedule = KeySchedule::new(hash, &dhe, &hash.digest(&transcript))?;
    let transcript_hash_sh = hash.digest(&transcript);
    let server_hs_secret = schedule.server_handshake_traffic(&transcript_hash_sh)?;
    let client_hs_secret = schedule.client_handshake_traffic(&transcript_hash_sh)?;
    let server_hs_keys = derive_traffic_keys(hash, &server_hs_secret, 16, 12)?;
    let client_hs_keys = derive_traffic_keys(hash, &client_hs_secret, 16, 12)?;

    // ── Phase 3: read encrypted handshake messages ──
    // Expect EncryptedExtensions → Certificate → CertificateVerify → Finished.
    let mut server_seq: u64 = 0;
    let mut handshake_buffer: Vec<u8> = Vec::new();
    let mut server_certs: Vec<rusty_x509::Certificate> = Vec::new();
    let mut got_finished = false;
    let mut transcript_through_cv: Option<Vec<u8>> = None;
    let mut transcript_through_finished: Option<Vec<u8>> = None;

    'outer: while !got_finished {
        // Pull more records if buffer is empty.
        while !decode_record(&accumulator).is_ok() {
            transport.read_some(&mut accumulator)?;
        }
        let (rec, n) = decode_record(&accumulator)?;
        accumulator.drain(..n);
        if rec.content_type == ContentType::ChangeCipherSpec { continue; }
        if rec.content_type != ContentType::ApplicationData {
            return Err(TlsError::SignatureFail("unexpected plaintext in handshake phase".into()));
        }
        let (inner_ct, plaintext) = aead_decrypt_record(&server_hs_keys, server_seq, &rec.fragment)?;
        server_seq += 1;
        if inner_ct != 22 /* Handshake */ {
            return Err(TlsError::SignatureFail(
                format!("expected Handshake inner type, got {}", inner_ct)));
        }
        handshake_buffer.extend_from_slice(&plaintext);
        // Drain complete handshake messages.
        loop {
            let (msg, used) = match decode_handshake(&handshake_buffer) {
                Ok(p) => p,
                Err(_) => continue 'outer,  // need more bytes
            };
            let msg_bytes = handshake_buffer[..used].to_vec();
            handshake_buffer.drain(..used);
            transcript.extend_from_slice(&msg_bytes);
            match msg.msg_type {
                HandshakeType::EncryptedExtensions => {
                    // No-op for this round; we ignore the contents.
                }
                HandshakeType::Certificate => {
                    server_certs = parse_certificate_message(&msg.body)?;
                    if server_certs.is_empty() {
                        return Err(TlsError::SignatureFail("server sent zero certs".into()));
                    }
                }
                HandshakeType::CertificateVerify => {
                    // RFC 8446 §4.4.3.
                    if msg.body.len() < 4 {
                        return Err(TlsError::SignatureFail("CertificateVerify body too short".into()));
                    }
                    let scheme = ((msg.body[0] as u16) << 8) | (msg.body[1] as u16);
                    let sig_len = ((msg.body[2] as usize) << 8) | (msg.body[3] as usize);
                    if msg.body.len() < 4 + sig_len {
                        return Err(TlsError::SignatureFail("CertificateVerify truncated".into()));
                    }
                    let signature = &msg.body[4..4 + sig_len];
                    // Construct the to-be-signed content per §4.4.3.
                    let mut tbs = Vec::new();
                    tbs.extend_from_slice(&[0x20u8; 64]);
                    tbs.extend_from_slice(b"TLS 1.3, server CertificateVerify");
                    tbs.push(0x00);
                    // Transcript hash through Certificate (excluding this CV msg).
                    let cv_len = msg_bytes.len();
                    let mut transcript_through_cert = transcript.clone();
                    transcript_through_cert.truncate(transcript.len() - cv_len);
                    tbs.extend_from_slice(&hash.digest(&transcript_through_cert));
                    // Verify against leaf cert's pubkey per the SignatureScheme.
                    let leaf = server_certs.first()
                        .ok_or(TlsError::SignatureFail("CertificateVerify before Certificate".into()))?;
                    verify_certificate_verify_signature(scheme, &leaf.subject_public_key_info,
                                                        &tbs, signature)?;
                    transcript_through_cv = Some(transcript.clone());
                }
                HandshakeType::Finished => {
                    // Verify server Finished MAC.
                    let th_through_cv = transcript_through_cv
                        .clone()
                        .ok_or(TlsError::SignatureFail("Finished before CertificateVerify".into()))?;
                    // transcript currently includes Finished. Server's
                    // Finished is over transcript_through_cv.
                    let th = hash.digest(&th_through_cv);
                    let expected = finished_mac(hash, &server_hs_secret, &th)?;
                    if msg.body != expected {
                        return Err(TlsError::SignatureFail("server Finished MAC mismatch".into()));
                    }
                    transcript_through_finished = Some(transcript.clone());
                    got_finished = true;
                    break;
                }
                _ => {
                    return Err(TlsError::SignatureFail(
                        format!("unexpected handshake type {:?} in encrypted phase", msg.msg_type)));
                }
            }
        }
    }

    // ── Phase 4: validate server certificate chain ──
    let leaf = server_certs.first()
        .ok_or(TlsError::SignatureFail("no leaf cert".into()))?;
    let intermediates: Vec<_> = server_certs.iter().skip(1).cloned().collect();
    chain_walk(leaf, &intermediates, trust_store, 8)?;

    // ── Phase 5: derive application-traffic keys ──
    let transcript_sf = transcript_through_finished.unwrap();
    let th_sf = hash.digest(&transcript_sf);
    let client_app_secret = schedule.client_application_traffic(&th_sf)?;
    let server_app_secret = schedule.server_application_traffic(&th_sf)?;
    let client_app_keys = derive_traffic_keys(hash, &client_app_secret, 16, 12)?;
    let server_app_keys = derive_traffic_keys(hash, &server_app_secret, 16, 12)?;

    // ── Phase 6: send client Finished ──
    let client_finished_mac = finished_mac(hash, &client_hs_secret, &th_sf)?;
    let cf_msg = HandshakeMessage {
        msg_type: HandshakeType::Finished,
        body: client_finished_mac,
    };
    let cf_bytes = encode_handshake(&cf_msg);
    let cf_ct = aead_encrypt_record(&client_hs_keys, 0,
                                    ContentType::Handshake as u8, &cf_bytes)?;
    let cf_record = TlsRecord {
        content_type: ContentType::ApplicationData,
        version: ProtocolVersion::LEGACY,
        fragment: cf_ct,
    };
    transport.write_all(&encode_record(&cf_record)?)?;

    // Hand back the session.
    Ok(TlsSession {
        transport,
        client_app_keys,
        server_app_keys,
        client_app_seq: 0,
        server_app_seq: 0,
        hash,
    })
}

/// Server-side cert chain extraction from the Certificate handshake
/// message body (after AEAD-decrypt). RFC 8446 §4.4.2:
/// Certificate ::= struct {
///   opaque certificate_request_context<0..2^8-1>;
///   CertificateEntry certificate_list<0..2^24-1>;
/// }
/// CertificateEntry ::= struct {
///   opaque cert_data<1..2^24-1>;  (DER-encoded X.509)
///   Extension extensions<0..2^16-1>;
/// }
pub fn parse_certificate_message(body: &[u8]) -> Result<Vec<X509Cert>, TlsError> {
    if body.is_empty() { return Err(TlsError::UnexpectedEnd); }
    let ctx_len = body[0] as usize;
    if body.len() < 1 + ctx_len + 3 { return Err(TlsError::UnexpectedEnd); }
    let list_start = 1 + ctx_len;
    let list_len = ((body[list_start] as usize) << 16) |
                   ((body[list_start + 1] as usize) << 8) |
                   (body[list_start + 2] as usize);
    let mut pos = list_start + 3;
    let list_end = pos + list_len;
    if body.len() < list_end { return Err(TlsError::UnexpectedEnd); }
    let mut certs = Vec::new();
    while pos < list_end {
        if body.len() < pos + 3 { return Err(TlsError::UnexpectedEnd); }
        let cert_len = ((body[pos] as usize) << 16) |
                       ((body[pos + 1] as usize) << 8) |
                       (body[pos + 2] as usize);
        pos += 3;
        if body.len() < pos + cert_len { return Err(TlsError::UnexpectedEnd); }
        let cert = rusty_x509::parse_certificate(&body[pos..pos + cert_len])
            .map_err(TlsError::X509)?;
        certs.push(cert);
        pos += cert_len;
        // Skip extensions vector (u16 length + entries).
        if body.len() < pos + 2 { return Err(TlsError::UnexpectedEnd); }
        let ext_len = ((body[pos] as usize) << 8) | (body[pos + 1] as usize);
        pos += 2 + ext_len;
    }
    Ok(certs)
}
