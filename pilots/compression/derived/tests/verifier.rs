// Verifier suite for rusty-compression decoder.
//
// Strategy: encode inputs via the host gzip / zlib (real-world reference),
// decode under our pilot, and assert round-trip equality. Plus a few
// hand-crafted RFC test vectors for fixed-Huffman + stored blocks.

use rusty_compression::{gunzip, inflate, zlib_inflate, http_deflate_inflate};
use std::io::Write;
use std::process::{Command, Stdio};

fn gzip_via_system(input: &[u8]) -> Vec<u8> {
    // Use the system `gzip` binary as an external reference (POSIX).
    let mut child = Command::new("gzip")
        .arg("-c")
        .arg("-n")  // suppress filename/mtime in header for determinism
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("gzip binary missing");
    child.stdin.as_mut().unwrap().write_all(input).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    out.stdout
}

#[test]
fn inflate_stored_block_smoke() {
    // Hand-crafted: BFINAL=1, BTYPE=00 (stored), len=5, nlen=~5, "hello".
    // Header byte: 0b00000001 = 0x01.
    // Then 4 bytes: LEN=5,0,NLEN=0xFA,0xFF; then 5 bytes "hello".
    let data = [0x01, 0x05, 0x00, 0xFA, 0xFF, b'h', b'e', b'l', b'l', b'o'];
    let out = inflate(&data).unwrap();
    assert_eq!(out, b"hello");
}

#[test]
fn gunzip_roundtrip_short_text() {
    let input = b"Hello, gzip!";
    let g = gzip_via_system(input);
    let decoded = gunzip(&g).unwrap();
    assert_eq!(decoded, input);
}

#[test]
fn gunzip_roundtrip_repetitive_text() {
    // Repetitive input exercises LZ77 backreferences heavily.
    let input: Vec<u8> = "abc".repeat(1000).into_bytes();
    let g = gzip_via_system(&input);
    let decoded = gunzip(&g).unwrap();
    assert_eq!(decoded, input);
}

#[test]
fn gunzip_roundtrip_lorem() {
    // Realistic English text — exercises dynamic Huffman.
    let input = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, \
                  sed do eiusmod tempor incididunt ut labore et dolore magna \
                  aliqua. Ut enim ad minim veniam, quis nostrud exercitation \
                  ullamco laboris nisi ut aliquip ex ea commodo consequat.";
    let g = gzip_via_system(input);
    let decoded = gunzip(&g).unwrap();
    assert_eq!(decoded.as_slice(), input as &[u8]);
}

#[test]
fn gunzip_empty_input() {
    let input = b"";
    let g = gzip_via_system(input);
    let decoded = gunzip(&g).unwrap();
    assert_eq!(decoded, b"");
}

#[test]
fn gunzip_corrupt_crc_detected() {
    let input = b"detect this";
    let mut g = gzip_via_system(input);
    // Flip one byte in the CRC trailer (positions len-8..len-4).
    let n = g.len();
    g[n - 8] ^= 0x01;
    let r = gunzip(&g);
    assert!(matches!(r, Err(rusty_compression::DecodeError::GzipCrcMismatch)));
}

#[test]
fn gunzip_invalid_magic() {
    let r = gunzip(b"NOTGZIP\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00");
    assert!(matches!(r, Err(rusty_compression::DecodeError::InvalidGzipMagic)));
}

#[test]
fn zlib_roundtrip_via_python() {
    // Python's `zlib.compress` produces zlib-wrapped output; use it as
    // reference. If python3 isn't on PATH, skip.
    let input = b"Content-Encoding: deflate is zlib in practice.";
    let mut child = match Command::new("python3")
        .arg("-c")
        .arg("import sys, zlib; sys.stdout.buffer.write(zlib.compress(sys.stdin.buffer.read()))")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => { eprintln!("python3 missing — skip zlib test"); return; }
    };
    child.stdin.as_mut().unwrap().write_all(input).unwrap();
    let out = match child.wait_with_output() {
        Ok(o) if o.status.success() => o.stdout,
        _ => { eprintln!("python3 zlib invocation failed — skip"); return; }
    };
    let decoded = zlib_inflate(&out).unwrap();
    assert_eq!(decoded.as_slice(), input as &[u8]);
}

#[test]
fn http_deflate_accepts_both_wrappings() {
    // gzip's deflate output is raw DEFLATE if invoked with --no-name and
    // no header; but `gzip -c` always emits gzip-framed. To get raw
    // DEFLATE we use python3.
    let raw = Command::new("python3")
        .arg("-c")
        .arg("import sys, zlib; \
              d = zlib.compressobj(-1, zlib.DEFLATED, -15); \
              sys.stdout.buffer.write(d.compress(b'raw deflate') + d.flush())")
        .output();
    if let Ok(o) = raw {
        if o.status.success() {
            let decoded = http_deflate_inflate(&o.stdout).unwrap();
            assert_eq!(decoded, b"raw deflate");
        }
    }

    let wrapped = Command::new("python3")
        .arg("-c")
        .arg("import sys, zlib; sys.stdout.buffer.write(zlib.compress(b'zlib deflate'))")
        .output();
    if let Ok(o) = wrapped {
        if o.status.success() {
            let decoded = http_deflate_inflate(&o.stdout).unwrap();
            assert_eq!(decoded, b"zlib deflate");
        }
    }
}
