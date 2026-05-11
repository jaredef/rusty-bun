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

use rusty_x509::Certificate as X509Cert;

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
