// Verifier for the node-fs pilot. Tests use temp files for isolation.

use rusty_node_fs::*;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("rusty-node-fs-{}-{}", name, std::process::id()));
    p
}

fn make_fixture_file(name: &str, contents: &[u8]) -> PathBuf {
    let p = fixture_path(name);
    write_file_sync(&p, contents).unwrap();
    p
}

fn cleanup(p: &std::path::Path) {
    let _ = std::fs::remove_file(p);
    let _ = std::fs::remove_dir_all(p);
}

// ════════════════════ EXISTS ════════════════════

#[test]
fn cd_exists_sync_true_for_real_file() {
    let p = make_fixture_file("exists-true", b"x");
    assert!(exists_sync(&p));
    cleanup(&p);
}

#[test]
fn cd_exists_sync_false_for_missing() {
    assert!(!exists_sync("/nonexistent/path/asdfqwer"));
}

// ════════════════════ READ / WRITE ════════════════════

#[test]
fn cd_read_file_sync_returns_bytes() {
    let p = make_fixture_file("read-bytes", &[1u8, 2, 3, 0xFF]);
    assert_eq!(read_file_sync(&p).unwrap(), vec![1u8, 2, 3, 0xFF]);
    cleanup(&p);
}

#[test]
fn spec_read_file_string_sync_utf8() {
    let p = make_fixture_file("read-string", "héllo".as_bytes());
    assert_eq!(read_file_string_sync(&p).unwrap(), "héllo");
    cleanup(&p);
}

#[test]
fn spec_write_file_sync_creates_file() {
    let p = fixture_path("write-create");
    cleanup(&p);
    write_file_sync(&p, b"new content").unwrap();
    assert!(exists_sync(&p));
    assert_eq!(read_file_sync(&p).unwrap(), b"new content".to_vec());
    cleanup(&p);
}

#[test]
fn spec_write_file_sync_overwrites() {
    let p = make_fixture_file("write-overwrite", b"old");
    write_file_sync(&p, b"new").unwrap();
    assert_eq!(read_file_sync(&p).unwrap(), b"new".to_vec());
    cleanup(&p);
}

#[test]
fn spec_write_file_string_sync_utf8() {
    let p = fixture_path("write-string");
    cleanup(&p);
    write_file_string_sync(&p, "test").unwrap();
    assert_eq!(read_file_string_sync(&p).unwrap(), "test");
    cleanup(&p);
}

// ════════════════════ APPEND ════════════════════

#[test]
fn spec_append_file_sync_appends() {
    let p = make_fixture_file("append", b"line1\n");
    append_file_sync(&p, b"line2\n").unwrap();
    assert_eq!(read_file_string_sync(&p).unwrap(), "line1\nline2\n");
    cleanup(&p);
}

#[test]
fn spec_append_file_sync_creates_if_missing() {
    let p = fixture_path("append-create");
    cleanup(&p);
    append_file_sync(&p, b"first").unwrap();
    assert_eq!(read_file_string_sync(&p).unwrap(), "first");
    cleanup(&p);
}

// ════════════════════ MKDIR / RMDIR ════════════════════

#[test]
fn cd_mkdir_sync_creates_directory() {
    let p = fixture_path("mkdir");
    cleanup(&p);
    mkdir_sync(&p, false).unwrap();
    assert!(p.is_dir());
    cleanup(&p);
}

#[test]
fn spec_mkdir_sync_recursive() {
    let mut p = fixture_path("mkdir-recursive");
    cleanup(&p);
    p.push("a/b/c");
    mkdir_sync(&p, true).unwrap();
    assert!(p.is_dir());
    cleanup(&fixture_path("mkdir-recursive"));
}

#[test]
fn spec_mkdir_sync_non_recursive_fails_for_missing_parent() {
    let mut p = fixture_path("mkdir-fail");
    p.push("missing/parent/child");
    assert!(mkdir_sync(&p, false).is_err());
}

#[test]
fn spec_rmdir_sync_removes_empty_dir() {
    let p = fixture_path("rmdir");
    mkdir_sync(&p, false).unwrap();
    rmdir_sync(&p).unwrap();
    assert!(!exists_sync(&p));
}

#[test]
fn spec_rm_sync_recursive_removes_tree() {
    let mut p = fixture_path("rm-recursive");
    cleanup(&p);
    p.push("a/b/c");
    mkdir_sync(&p, true).unwrap();
    let f = fixture_path("rm-recursive").join("a/b/c/file");
    write_file_sync(&f, b"x").unwrap();
    rm_sync_recursive(&fixture_path("rm-recursive")).unwrap();
    assert!(!exists_sync(&fixture_path("rm-recursive")));
}

// ════════════════════ UNLINK / RENAME / COPY ════════════════════

#[test]
fn spec_unlink_sync_removes_file() {
    let p = make_fixture_file("unlink", b"x");
    unlink_sync(&p).unwrap();
    assert!(!exists_sync(&p));
}

#[test]
fn spec_rename_sync_moves_file() {
    let src = make_fixture_file("rename-src", b"contents");
    let dst = fixture_path("rename-dst");
    cleanup(&dst);
    rename_sync(&src, &dst).unwrap();
    assert!(!exists_sync(&src));
    assert_eq!(read_file_sync(&dst).unwrap(), b"contents".to_vec());
    cleanup(&dst);
}

#[test]
fn spec_copy_file_sync_copies_bytes() {
    let src = make_fixture_file("copy-src", b"original");
    let dst = fixture_path("copy-dst");
    cleanup(&dst);
    let n = copy_file_sync(&src, &dst).unwrap();
    assert_eq!(n, 8);
    assert_eq!(read_file_sync(&dst).unwrap(), b"original".to_vec());
    assert!(exists_sync(&src), "src must still exist after copy");
    cleanup(&src);
    cleanup(&dst);
}

// ════════════════════ STAT / LSTAT ════════════════════

// CD: `fs.lstatSync(path)` — primary lstat surface in constraint corpus
#[test]
fn cd_lstat_sync_returns_stats() {
    let p = make_fixture_file("lstat", b"hello");
    let stats = lstat_sync(&p).unwrap();
    assert_eq!(stats.size, 5);
    assert!(stats.is_file);
    assert!(!stats.is_directory);
    cleanup(&p);
}

#[test]
fn spec_stat_sync_size_byte_count() {
    let p = make_fixture_file("stat-size", b"abcdefghij");
    let stats = stat_sync(&p).unwrap();
    assert_eq!(stats.size, 10);
    cleanup(&p);
}

#[test]
fn spec_stat_sync_is_directory() {
    let p = fixture_path("stat-dir");
    mkdir_sync(&p, false).unwrap();
    let stats = stat_sync(&p).unwrap();
    assert!(stats.is_directory);
    assert!(!stats.is_file);
    cleanup(&p);
}

#[test]
fn spec_stat_sync_mtime_modern_timestamp() {
    let p = make_fixture_file("stat-mtime", b"x");
    let stats = stat_sync(&p).unwrap();
    assert!(stats.mtime_ms > 1_700_000_000_000,
        "expected modern mtime in ms, got {}", stats.mtime_ms);
    cleanup(&p);
}

// ════════════════════ READDIR ════════════════════

#[test]
fn cd_readdir_sync_returns_entries() {
    let dir = fixture_path("readdir");
    cleanup(&dir);
    mkdir_sync(&dir, false).unwrap();
    write_file_sync(dir.join("a.txt"), b"").unwrap();
    write_file_sync(dir.join("b.txt"), b"").unwrap();
    write_file_sync(dir.join("c.txt"), b"").unwrap();
    let entries = readdir_sync(&dir).unwrap();
    assert_eq!(entries, vec!["a.txt", "b.txt", "c.txt"]);
    rm_sync_recursive(&dir).unwrap();
}

#[test]
fn spec_readdir_sync_empty_dir() {
    let dir = fixture_path("readdir-empty");
    cleanup(&dir);
    mkdir_sync(&dir, false).unwrap();
    let entries = readdir_sync(&dir).unwrap();
    assert!(entries.is_empty());
    rmdir_sync(&dir).unwrap();
}

// ════════════════════ ACCESS / REALPATH ════════════════════

#[test]
fn spec_access_sync_succeeds_for_existing() {
    let p = make_fixture_file("access", b"x");
    assert!(access_sync(&p).is_ok());
    cleanup(&p);
}

#[test]
fn spec_access_sync_fails_for_missing() {
    assert!(access_sync("/nonexistent/asdfqwer").is_err());
}

#[test]
fn spec_realpath_sync_canonicalizes() {
    let p = make_fixture_file("realpath", b"x");
    let real = realpath_sync(&p).unwrap();
    // Real path should be absolute
    assert!(real.is_absolute());
    cleanup(&p);
}

// ════════════════════ EDGE CASES ════════════════════

#[test]
fn spec_read_file_missing_returns_error() {
    assert!(read_file_sync("/nonexistent/path").is_err());
}

#[test]
fn spec_unlink_missing_returns_error() {
    assert!(unlink_sync("/nonexistent/path").is_err());
}
