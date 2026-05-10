// Consumer-regression suite for node-fs.

use rusty_node_fs::*;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("rusty-node-fs-cons-{}-{}", name, std::process::id()));
    p
}

fn cleanup(p: &std::path::Path) {
    let _ = std::fs::remove_file(p);
    let _ = std::fs::remove_dir_all(p);
}

// ────────── npm cli — tarball extraction relies on existsSync ──────────
//
// Source: https://github.com/npm/cli/blob/latest/lib/utils/...
//   pre-install hooks check existence via fs.existsSync before reading.

#[test]
fn consumer_npm_existssync_for_preinstall_check() {
    let p = fixture_path("npm-preinstall");
    cleanup(&p);
    assert!(!exists_sync(&p));
    write_file_sync(&p, b"package.json").unwrap();
    assert!(exists_sync(&p));
    cleanup(&p);
}

// ────────── webpack — readFileSync for module resolution ──────────
//
// Source: https://github.com/webpack/webpack/blob/main/lib/...
//   reads source files synchronously during dependency analysis. relies on
//   utf-8 decoding + io::Error on missing.

#[test]
fn consumer_webpack_readfile_returns_io_error_on_missing() {
    let r = read_file_string_sync("/nonexistent/source.ts");
    assert!(r.is_err());
}

// ────────── eslint — readdirSync with sorted output ──────────
//
// Source: https://github.com/eslint/eslint/blob/main/lib/...
//   walks directories for .js/.ts files. consumers expect deterministic
//   ordering for reproducible test output.

#[test]
fn consumer_eslint_readdir_deterministic_sort() {
    let dir = fixture_path("eslint-readdir");
    cleanup(&dir);
    mkdir_sync(&dir, false).unwrap();
    for name in ["zebra.ts", "apple.ts", "mango.ts"] {
        write_file_sync(dir.join(name), b"").unwrap();
    }
    let entries = readdir_sync(&dir).unwrap();
    // Pilot sorts; consumer can rely on alphabetical order.
    assert_eq!(entries, vec!["apple.ts", "mango.ts", "zebra.ts"]);
    rm_sync_recursive(&dir).unwrap();
}

// ────────── prettier — writeFileSync for in-place formatting ──────────
//
// Source: https://github.com/prettier/prettier
//   writes formatted output back via fs.writeFileSync; consumer expects
//   atomic overwrite (truncates existing file).

#[test]
fn consumer_prettier_writefile_truncates() {
    let p = fixture_path("prettier-truncate");
    write_file_sync(&p, b"long original content goes here").unwrap();
    write_file_sync(&p, b"short").unwrap();
    assert_eq!(read_file_sync(&p).unwrap(), b"short".to_vec());
    cleanup(&p);
}

// ────────── postgres / sqlite — file size for backup verification ───────
//
// Source: many DB-backup tools compare expected vs actual file size.
//   Stats.size must equal the byte count.

#[test]
fn consumer_db_backup_size_byte_exact() {
    let payload = vec![0xABu8; 1024];
    let p = fixture_path("backup-size");
    write_file_sync(&p, &payload).unwrap();
    let s = stat_sync(&p).unwrap();
    assert_eq!(s.size as usize, payload.len());
    cleanup(&p);
}

// ────────── git — rename across directories ──────────
//
// Source: git's rename detection relies on fs.renameSync working across
// directories on the same filesystem.

#[test]
fn consumer_git_rename_across_dirs() {
    let dir1 = fixture_path("git-dir1");
    let dir2 = fixture_path("git-dir2");
    cleanup(&dir1);
    cleanup(&dir2);
    mkdir_sync(&dir1, false).unwrap();
    mkdir_sync(&dir2, false).unwrap();
    let src = dir1.join("file");
    let dst = dir2.join("file");
    write_file_sync(&src, b"content").unwrap();
    rename_sync(&src, &dst).unwrap();
    assert!(!exists_sync(&src));
    assert_eq!(read_file_sync(&dst).unwrap(), b"content".to_vec());
    rm_sync_recursive(&dir1).unwrap();
    rm_sync_recursive(&dir2).unwrap();
}

// ────────── docker — recursive mkdir for volume mounts ──────────
//
// Source: container tooling uses fs.mkdirSync(path, { recursive: true })
//   to ensure intermediate directories exist for bind mounts.

#[test]
fn consumer_docker_recursive_mkdir() {
    let mut p = fixture_path("docker-recursive");
    cleanup(&p);
    p.push("a/b/c/d");
    mkdir_sync(&p, true).unwrap();
    assert!(p.is_dir());
    rm_sync_recursive(&fixture_path("docker-recursive")).unwrap();
}

// ────────── jest — copy fixtures for test isolation ──────────
//
// Source: jest test setup copies fixture files into per-test temp dirs.
//   consumer expects copy_file to leave src intact.

#[test]
fn consumer_jest_copy_preserves_src() {
    let src = fixture_path("jest-fixture-src");
    let dst = fixture_path("jest-fixture-dst");
    cleanup(&src);
    cleanup(&dst);
    write_file_sync(&src, b"fixture data").unwrap();
    copy_file_sync(&src, &dst).unwrap();
    assert!(exists_sync(&src));
    assert!(exists_sync(&dst));
    cleanup(&src);
    cleanup(&dst);
}
