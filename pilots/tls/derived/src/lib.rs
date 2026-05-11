// rusty-tls pilot — TLS 1.3 record layer + system trust store + chain walk.
//
// Π1.4.c round of the TLS substrate-amortization sequence. Composes on
// Π1.4.b (X.509 parsing + signature verification) and Π1.4.a (DER).
// Per RFC 8446 (TLS 1.3) primary, with RFC 5246 (TLS 1.2) compat for
// the version-negotiation prefix.
//
// Scope: wire-format encode/decode for TLSPlaintext records (record
// layer), TLS alert + content-type enumerations, system trust store
// loader (reads /etc/ssl/certs/ca-certificates.crt or platform
// equivalents), and chain-walk verification (cert → issuer → ... →
// trust anchor in the loaded root set). The actual TLS handshake state
// machine (ClientHello → ServerHello → key derivation → Finished) is
// deferred to Π1.4.d.
//
// Per Pin-Art Doc 707 bidirectional reading: this round surfaces
// invariants about real-world trust stores — the /etc/ssl/certs/
// path varies by distribution (Debian uses ca-certificates.crt; RHEL
// uses ca-bundle.crt; macOS uses cert.pem); chain walks use
// canonical DER byte equality on Distinguished Names per RFC 5280
// §7.1; trust anchors are by definition self-signed and verify against
// their own SPKI.

use rusty_x509::*;

pub mod record;
pub mod store;
pub mod handshake;
pub mod client;
pub mod driver;

pub use record::*;
pub use store::*;
pub use handshake::*;
pub use client::*;
pub use driver::*;
