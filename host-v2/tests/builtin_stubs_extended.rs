//! Tier-Ω.5.s: smoke tests for the assert / https / stream / url / util
//! built-in stubs. Drives the host-v2 binary against
//! fixtures/builtin_stubs_extended.mjs and asserts the expected stdout
//! lattice.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_rusty-bun-host-v2")
}

fn run() -> std::process::Output {
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/builtin_stubs_extended.mjs");
    Command::new(bin()).arg(fixture).output().expect("run host-v2")
}

#[test]
fn t01_assert_ok_true_succeeds() {
    let out = run();
    assert!(out.status.success(), "binary failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("assert.ok(true) ok"), "stdout: {stdout}");
}

#[test]
fn t02_https_request_typeof_function() {
    let out = run();
    assert!(out.status.success(), "binary failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("https.request typeof: function"), "stdout: {stdout}");
}

#[test]
fn t03_stream_readable_typeof_function() {
    let out = run();
    assert!(out.status.success(), "binary failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("stream.Readable typeof: function"), "stdout: {stdout}");
}

#[test]
fn t04_url_file_url_to_path() {
    let out = run();
    assert!(out.status.success(), "binary failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("fileURLToPath: /tmp/x"), "stdout: {stdout}");
}

#[test]
fn t05_util_format_substitution() {
    let out = run();
    assert!(out.status.success(), "binary failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("util.format: hi world"), "stdout: {stdout}");
}
