//! Tier-Omega.4.d: end-to-end binary smoke for the fs surface.
//! Exercises sync round-trip + async readFile + async write→read chain,
//! proving the PollIo macrotask path drives Promise reactions through
//! the JobQueue under run_to_completion.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_rusty-bun-host-v2")
}

#[test]
fn fs_smoke_full_stack() {
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/fs_smoke.mjs");
    let out = Command::new(bin()).arg(fixture).output().expect("run host-v2");
    assert!(out.status.success(), "binary failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("sync: sync-roundtrip"), "stdout: {stdout}");
    assert!(stdout.contains("async read: sync-roundtrip"), "stdout: {stdout}");
    assert!(stdout.contains("async chain: chained-write"), "stdout: {stdout}");
    assert!(stdout.contains("done"), "stdout: {stdout}");
}
