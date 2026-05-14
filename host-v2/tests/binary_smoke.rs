//! Tier-Omega.4.c: end-to-end binary smoke. Compiles and runs the
//! host-v2 binary against fixtures/smoke.mjs, asserts stdout matches
//! the expected lattice: path/os/process intrinsics + Promise reaction
//! drained through the event loop.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_rusty-bun-host-v2")
}

#[test]
fn smoke_mjs_full_stack() {
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/smoke.mjs");
    let out = Command::new(bin()).arg(fixture).output().expect("run host-v2");
    assert!(out.status.success(), "binary failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("sep: /"), "stdout: {stdout}");
    assert!(stdout.contains("platform: linux") || stdout.contains("platform: darwin"), "stdout: {stdout}");
    assert!(stdout.contains("joined: a/b/c.txt"), "stdout: {stdout}");
    assert!(stdout.contains("ext: .txt"), "stdout: {stdout}");
    assert!(stdout.contains("promise then: 42"), "stdout: {stdout}");
}
