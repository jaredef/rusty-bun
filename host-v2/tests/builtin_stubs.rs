//! Tier-Ω.5.r: smoke tests for the node:http + node:crypto stubs.
//! Drives the host-v2 binary against fixtures/builtin_stubs.mjs (and
//! a sibling fixture for the stub-error path) and asserts the expected
//! stdout / stderr lattice.
//!
//! Acceptance items (from Tier-Ω.5.r locked design):
//!   (1) `import http from "node:http"` succeeds; non-empty key set.
//!   (2) `import crypto from "node:crypto"` succeeds; `createHash` is fn.
//!   (3) `crypto.randomUUID()` returns a string.
//!   (4) `http.request(...)` raises the documented stub error.
//!
//! v1 note: the engine's host-error surface uses RuntimeError::TypeError
//! which propagates as a binary-level failure (stderr + non-zero exit),
//! not as a JS-catchable throw. Acceptance item (4) is verified through
//! the binary's stderr in a separate fixture invocation.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_rusty-bun-host-v2")
}

#[test]
fn t01_http_namespace_has_keys() {
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/builtin_stubs.mjs");
    let out = Command::new(bin()).arg(fixture).output().expect("run host-v2");
    assert!(out.status.success(), "binary failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("http keys non-empty: true"),
        "expected populated http namespace; stdout: {stdout}"
    );
    // STATUS_CODES coverage rolled into the same fixture run.
    assert!(stdout.contains("status 200: OK"), "stdout: {stdout}");
    assert!(stdout.contains("status 404: Not Found"), "stdout: {stdout}");
}

#[test]
fn t02_crypto_create_hash_is_function() {
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/builtin_stubs.mjs");
    let out = Command::new(bin()).arg(fixture).output().expect("run host-v2");
    assert!(out.status.success(), "binary failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("crypto.createHash typeof: function"),
        "expected createHash to be a function; stdout: {stdout}"
    );
}

#[test]
fn t03_crypto_random_uuid_returns_string() {
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/builtin_stubs.mjs");
    let out = Command::new(bin()).arg(fixture).output().expect("run host-v2");
    assert!(out.status.success(), "binary failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("uuid typeof: string"),
        "expected randomUUID to return a string; stdout: {stdout}"
    );
    assert!(
        stdout.contains("uuid sample: 00000000-0000-0000-0000-000000000000"),
        "expected placeholder uuid sample; stdout: {stdout}"
    );
}

#[test]
fn t04_http_request_throws_documented_stub_message() {
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/builtin_stubs_throw.mjs");
    let out = Command::new(bin()).arg(fixture).output().expect("run host-v2");
    // Expected: non-success exit; stderr names the stub message.
    assert!(
        !out.status.success(),
        "binary should have failed when invoking http.request stub; out: {:?}",
        out
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("node:http http.request: not yet implemented"),
        "stderr should carry the documented stub message; stderr: {stderr}"
    );
    assert!(
        stderr.contains("Tier-Ω.5.r"),
        "stderr should tag the stub with its tier marker; stderr: {stderr}"
    );
}
