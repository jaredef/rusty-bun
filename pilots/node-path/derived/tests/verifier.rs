// Verifier for the node-path pilot.
//
// Tests transcribe Bun's antichain representatives where applicable, plus
// Node-spec edge cases.

use rusty_node_path::*;

// ════════════════════ POSIX BASENAME ════════════════════

#[test]
fn cd_posix_basename_simple() {
    assert_eq!(posix::basename("/foo/bar/baz.html", Some(".html")), "baz");
}

#[test]
fn cd_posix_basename_empty_returns_empty() {
    assert_eq!(posix::basename("", None), "");
}

#[test]
fn cd_posix_basename_trailing_slash_stripped() {
    assert_eq!(posix::basename("/foo/bar/", None), "bar");
}

#[test]
fn cd_posix_basename_no_slash() {
    assert_eq!(posix::basename("baz.html", None), "baz.html");
    assert_eq!(posix::basename("baz.html", Some(".html")), "baz");
}

#[test]
fn cd_posix_basename_ext_not_matching_returns_full() {
    assert_eq!(posix::basename("file.txt", Some(".html")), "file.txt");
}

#[test]
fn cd_posix_basename_ext_equals_basename() {
    // Edge: don't strip if ext equals basename (would yield empty).
    assert_eq!(posix::basename(".bashrc", Some(".bashrc")), ".bashrc");
}

// ════════════════════ POSIX DIRNAME ════════════════════

#[test]
fn cd_posix_dirname_simple() {
    assert_eq!(posix::dirname("/foo/bar/baz"), "/foo/bar");
}

#[test]
fn cd_posix_dirname_empty_is_dot() {
    assert_eq!(posix::dirname(""), ".");
}

#[test]
fn cd_posix_dirname_no_slash_is_dot() {
    assert_eq!(posix::dirname("file.txt"), ".");
}

#[test]
fn cd_posix_dirname_root() {
    assert_eq!(posix::dirname("/"), "/");
    assert_eq!(posix::dirname("/foo"), "/");
}

// ════════════════════ POSIX EXTNAME ════════════════════

#[test]
fn cd_posix_extname_simple() {
    assert_eq!(posix::extname("file.txt"), ".txt");
}

#[test]
fn cd_posix_extname_dotfile_no_extension() {
    // Subtle: `.bashrc` has no extension per Node.
    assert_eq!(posix::extname(".bashrc"), "");
}

#[test]
fn cd_posix_extname_double_dot() {
    assert_eq!(posix::extname("file.tar.gz"), ".gz");
}

#[test]
fn cd_posix_extname_no_extension() {
    assert_eq!(posix::extname("README"), "");
}

#[test]
fn cd_posix_extname_empty() {
    assert_eq!(posix::extname(""), "");
}

// ════════════════════ POSIX PARSE / FORMAT ════════════════════

#[test]
fn cd_posix_parse_full_path() {
    let p = posix::parse("/home/user/dir/file.txt");
    assert_eq!(p.root, "/");
    assert_eq!(p.dir, "/home/user/dir");
    assert_eq!(p.base, "file.txt");
    assert_eq!(p.ext, ".txt");
    assert_eq!(p.name, "file");
}

#[test]
fn cd_posix_parse_relative() {
    let p = posix::parse("foo/bar.js");
    assert_eq!(p.root, "");
    assert_eq!(p.dir, "foo");
    assert_eq!(p.base, "bar.js");
    assert_eq!(p.ext, ".js");
    assert_eq!(p.name, "bar");
}

#[test]
fn spec_posix_format_round_trip() {
    let p = PathParsed {
        root: "/".into(),
        dir: "/home/user".into(),
        base: "file.txt".into(),
        ext: ".txt".into(),
        name: "file".into(),
    };
    assert_eq!(posix::format(&p), "/home/user/file.txt");
}

#[test]
fn spec_posix_format_uses_name_and_ext_when_base_empty() {
    let p = PathParsed {
        root: "/".into(),
        dir: "/home".into(),
        base: String::new(),
        ext: ".txt".into(),
        name: "file".into(),
    };
    assert_eq!(posix::format(&p), "/home/file.txt");
}

// ════════════════════ POSIX ISABSOLUTE ════════════════════

#[test]
fn cd_posix_isabsolute_root() {
    assert!(posix::is_absolute("/"));
    assert!(posix::is_absolute("/foo/bar"));
}

#[test]
fn cd_posix_isabsolute_relative() {
    assert!(!posix::is_absolute("foo/bar"));
    assert!(!posix::is_absolute("./foo"));
    assert!(!posix::is_absolute(""));
}

// ════════════════════ POSIX JOIN ════════════════════

#[test]
fn cd_posix_join_simple() {
    assert_eq!(posix::join(&["/foo", "bar", "baz/asdf"]), "/foo/bar/baz/asdf");
}

#[test]
fn cd_posix_join_with_dotdot() {
    assert_eq!(posix::join(&["/foo", "bar", "baz/asdf", "quux", ".."]), "/foo/bar/baz/asdf");
}

#[test]
fn cd_posix_join_empty_args_yields_dot() {
    assert_eq!(posix::join(&[]), ".");
    assert_eq!(posix::join(&["", ""]), ".");
}

#[test]
fn cd_posix_join_preserves_leading_slash() {
    assert_eq!(posix::join(&["/foo"]), "/foo");
}

// ════════════════════ POSIX NORMALIZE ════════════════════

#[test]
fn cd_posix_normalize_collapse_dotdot() {
    assert_eq!(posix::normalize("/foo/bar//baz/asdf/quux/.."), "/foo/bar/baz/asdf");
}

#[test]
fn cd_posix_normalize_redundant_slashes() {
    assert_eq!(posix::normalize("//foo//bar"), "/foo/bar");
}

#[test]
fn cd_posix_normalize_dot_segments() {
    assert_eq!(posix::normalize("/foo/./bar/./baz"), "/foo/bar/baz");
}

#[test]
fn cd_posix_normalize_relative_dotdot_preserved() {
    assert_eq!(posix::normalize("../../foo"), "../../foo");
}

#[test]
fn cd_posix_normalize_root_only() {
    assert_eq!(posix::normalize("/"), "/");
}

#[test]
fn cd_posix_normalize_empty_is_dot() {
    assert_eq!(posix::normalize(""), ".");
}

// ════════════════════ POSIX RELATIVE ════════════════════

#[test]
fn cd_posix_relative_descend() {
    assert_eq!(posix::relative("/foo/bar", "/foo/bar/baz"), "baz");
}

#[test]
fn cd_posix_relative_ascend() {
    assert_eq!(posix::relative("/foo/bar/baz", "/foo/bar"), "..");
}

#[test]
fn cd_posix_relative_diverge() {
    assert_eq!(posix::relative("/foo/bar", "/foo/baz"), "../baz");
}

#[test]
fn cd_posix_relative_same_path_empty() {
    assert_eq!(posix::relative("/foo/bar", "/foo/bar"), "");
}

// ════════════════════ POSIX RESOLVE ════════════════════

#[test]
fn cd_posix_resolve_simple() {
    assert_eq!(posix::resolve(&["/foo/bar", "./baz"], "/cwd"), "/foo/bar/baz");
}

#[test]
fn cd_posix_resolve_absolute_overrides() {
    assert_eq!(posix::resolve(&["/foo/bar", "/tmp/file"], "/cwd"), "/tmp/file");
}

#[test]
fn cd_posix_resolve_with_cwd_when_relative() {
    assert_eq!(posix::resolve(&["foo", "bar"], "/cwd"), "/cwd/foo/bar");
}

#[test]
fn cd_posix_resolve_empty_returns_cwd() {
    assert_eq!(posix::resolve(&[], "/cwd"), "/cwd");
}

#[test]
fn cd_posix_resolve_collapses_dotdot() {
    assert_eq!(posix::resolve(&["/foo/bar", "../baz"], "/"), "/foo/baz");
}

// ════════════════════ POSIX CONSTANTS ════════════════════

#[test]
fn spec_posix_sep_is_slash() { assert_eq!(posix::SEP, "/"); }
#[test]
fn spec_posix_delimiter_is_colon() { assert_eq!(posix::DELIMITER, ":"); }

// ════════════════════ WIN32 ISABSOLUTE ════════════════════

// CD-PATH1: `path.win32.isAbsolute("/foo/bar") === true` (Win32 treats /-rooted as absolute)
#[test]
fn cd_win32_isabsolute_forward_slash_rooted() {
    assert!(win32::is_absolute("/foo/bar"));
}

#[test]
fn cd_win32_isabsolute_drive_letter() {
    assert!(win32::is_absolute("C:\\foo"));
    assert!(win32::is_absolute("C:/foo"));
    assert!(win32::is_absolute("c:\\foo"));
}

#[test]
fn cd_win32_isabsolute_relative() {
    assert!(!win32::is_absolute(""));
    assert!(!win32::is_absolute("foo\\bar"));
    assert!(!win32::is_absolute("C:foo"));  // drive-relative without separator
}

// ════════════════════ WIN32 BASENAME / DIRNAME ════════════════════

#[test]
fn cd_win32_basename_with_drive() {
    assert_eq!(win32::basename("C:\\foo\\bar.txt", Some(".txt")), "bar");
}

#[test]
fn cd_win32_basename_forward_slash_supported() {
    assert_eq!(win32::basename("C:/foo/bar.txt", None), "bar.txt");
}

#[test]
fn cd_win32_dirname_with_drive() {
    assert_eq!(win32::dirname("C:\\foo\\bar"), "C:\\foo");
}

// ════════════════════ WIN32 NORMALIZE ════════════════════

#[test]
fn cd_win32_normalize_dotdot() {
    assert_eq!(win32::normalize("C:\\foo\\bar\\..\\baz"), "C:\\foo\\baz");
}

#[test]
fn cd_win32_normalize_mixed_separators() {
    assert_eq!(win32::normalize("C:\\foo/bar\\baz"), "C:\\foo\\bar\\baz");
}

// ════════════════════ TOP-LEVEL CONVENIENCE ════════════════════

#[test]
fn spec_toplevel_delegates_to_posix() {
    // Top-level path::* mirrors path.posix on POSIX-like platforms (Bun
    // follows this convention).
    assert_eq!(basename("/foo/bar.html", Some(".html")), "bar");
    assert_eq!(dirname("/foo/bar/baz"), "/foo/bar");
    assert_eq!(SEP, "/");
}
