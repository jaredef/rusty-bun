// System trust store loader + chain-walk validator.
//
// Loads the platform's CA bundle, parses all certificates via Π1.4.b,
// indexes by subject DN (canonical raw_der equality per RFC 5280 §7.1),
// and exposes a chain_walk function that verifies a leaf certificate's
// signature chain up to a trust anchor in the store.
//
// Platform paths tried in order (first existing wins):
//   /etc/ssl/certs/ca-certificates.crt    (Debian/Ubuntu)
//   /etc/pki/tls/certs/ca-bundle.crt      (RHEL/CentOS/Fedora)
//   /etc/ssl/cert.pem                     (macOS / Alpine / FreeBSD)
//   /etc/ssl/ca-bundle.pem                (SUSE)
//
// Chain walk semantics per RFC 5280 §6:
//   - Start at leaf.
//   - For each cert: look up issuer by matching cert.issuer.raw_der ==
//     candidate.subject.raw_der. If candidate is in trust store and
//     candidate is self-signed (issuer == subject) and signature
//     verifies against itself, accept as trust anchor.
//   - Verify cert.signature against issuer.subject_public_key_info.
//   - Loop with cert := issuer.
//   - Max-depth guard (default 8) to prevent unbounded recursion.

use rusty_x509::*;
use std::collections::HashMap;

use crate::record::TlsError;

#[derive(Debug)]
pub struct TrustStore {
    certs: Vec<Certificate>,
    /// Index: subject.raw_der → indices into `certs`.
    by_subject: HashMap<Vec<u8>, Vec<usize>>,
}

impl TrustStore {
    pub fn new() -> Self {
        TrustStore { certs: Vec::new(), by_subject: HashMap::new() }
    }

    pub fn len(&self) -> usize { self.certs.len() }
    pub fn is_empty(&self) -> bool { self.certs.is_empty() }

    /// Add a single parsed certificate to the store.
    pub fn add(&mut self, cert: Certificate) {
        let key = cert.subject.raw_der.clone();
        let idx = self.certs.len();
        self.certs.push(cert);
        self.by_subject.entry(key).or_default().push(idx);
    }

    /// Add all certificates from a PEM bundle string.
    pub fn add_pem_bundle(&mut self, pem: &str) -> Result<usize, TlsError> {
        let ders = pem_all_to_der(pem);
        let mut n = 0;
        for der in ders {
            // Skip certs that fail to parse — trust stores commonly have
            // malformed-but-legacy entries we don't need.
            if let Ok(cert) = parse_certificate(&der) {
                self.add(cert);
                n += 1;
            }
        }
        Ok(n)
    }

    /// Load the platform's default CA bundle.
    pub fn load_system_default() -> Result<Self, TlsError> {
        let mut store = TrustStore::new();
        let candidates = [
            "/etc/ssl/certs/ca-certificates.crt",
            "/etc/pki/tls/certs/ca-bundle.crt",
            "/etc/ssl/cert.pem",
            "/etc/ssl/ca-bundle.pem",
        ];
        for path in &candidates {
            if let Ok(contents) = std::fs::read_to_string(path) {
                store.add_pem_bundle(&contents)?;
                if !store.is_empty() { return Ok(store); }
            }
        }
        Err(TlsError::StoreLoad("no platform CA bundle found".into()))
    }

    /// Look up candidate issuer certificates by matching subject DN.
    /// Returns references to all matching certs in the store.
    pub fn find_issuers(&self, child: &Certificate) -> Vec<&Certificate> {
        if let Some(idxs) = self.by_subject.get(&child.issuer.raw_der) {
            idxs.iter().map(|i| &self.certs[*i]).collect()
        } else {
            Vec::new()
        }
    }

    /// Determine whether a certificate is a trust anchor: present in
    /// the store AND self-signed (issuer == subject).
    pub fn is_trust_anchor(&self, cert: &Certificate) -> bool {
        if cert.issuer.raw_der != cert.subject.raw_der { return false; }
        // Check the cert appears in the store with this subject.
        if let Some(idxs) = self.by_subject.get(&cert.subject.raw_der) {
            for i in idxs {
                if self.certs[*i].tbs_certificate == cert.tbs_certificate {
                    return true;
                }
            }
        }
        false
    }
}

/// Walk a certificate chain starting at `leaf`. Each step finds an
/// issuer candidate in the trust store (or in `intermediates`),
/// verifies the leaf's signature against it, and steps up. Terminates
/// when a self-signed trust anchor in the store is reached.
///
/// `intermediates` is the list of additional certificates supplied by
/// the server (e.g., from the TLS Certificate handshake message) that
/// were not in the trust store but are needed to complete the chain.
pub fn chain_walk(
    leaf: &Certificate,
    intermediates: &[Certificate],
    store: &TrustStore,
    max_depth: usize,
) -> Result<(), TlsError> {
    let mut current = leaf;
    for _depth in 0..max_depth {
        // If current is self-signed and present in the store, we have
        // reached a trust anchor.
        if current.issuer.raw_der == current.subject.raw_der {
            if store.is_trust_anchor(current) {
                // Self-signature verification (defense against a
                // tampered self-signed cert that happens to match a
                // subject DN by coincidence).
                verify_signature(current, &current.subject_public_key_info)?;
                return Ok(());
            }
            return Err(TlsError::SelfSignedNotInTrust);
        }
        // Find issuer candidates first in the trust store, then among
        // intermediates supplied by the server.
        let mut issuer_opt = None;
        for candidate in store.find_issuers(current) {
            if verify_signature(current, &candidate.subject_public_key_info).is_ok() {
                issuer_opt = Some(candidate);
                break;
            }
        }
        if issuer_opt.is_none() {
            for candidate in intermediates {
                if candidate.subject.raw_der == current.issuer.raw_der &&
                   verify_signature(current, &candidate.subject_public_key_info).is_ok()
                {
                    issuer_opt = Some(candidate);
                    break;
                }
            }
        }
        let issuer = issuer_opt.ok_or(TlsError::NoIssuerFound)?;
        current = issuer;
    }
    Err(TlsError::NoIssuerFound)
}
