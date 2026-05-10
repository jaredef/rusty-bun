// Consumer-regression suite for node-path.

use rusty_node_path::*;

// ────────── webpack — module-resolution path.resolve ──────────
//
// Source: https://github.com/webpack/webpack/blob/main/lib/dependencies/Module.js
//   resolves request paths via `path.resolve(context, request)`. relies on
//   absolute-arg-overriding-cwd semantics.

#[test]
fn consumer_webpack_resolve_absolute_overrides_context() {
    assert_eq!(
        posix::resolve(&["/project/src", "/abs/elsewhere"], "/cwd"),
        "/abs/elsewhere"
    );
}

// ────────── npm cli — registry path.join ──────────
//
// Source: https://github.com/npm/cli/blob/latest/lib/utils/cache-file.js
//   builds cache paths via `path.join(npm.cache, 'registry', name, version)`.
//   relies on segment-trailing-slash collapsing.

#[test]
fn consumer_npm_join_collapses_separators() {
    assert_eq!(
        posix::join(&["/home/.npm/", "/registry/", "package", "1.0.0"]),
        "/home/.npm/registry/package/1.0.0"
    );
}

// ────────── express static — basename for security ──────────
//
// Source: https://github.com/expressjs/serve-static/blob/master/index.js
//   uses `path.basename` to extract requested file name; consumer relies on
//   trailing-slash stripping to prevent directory traversal exploits.

#[test]
fn consumer_express_basename_strips_trailing_slash() {
    assert_eq!(posix::basename("/var/www/files/", None), "files");
}

#[test]
fn consumer_express_dirname_handles_root() {
    assert_eq!(posix::dirname("/etc/passwd"), "/etc");
}

// ────────── jest — config path.relative ──────────
//
// Source: https://github.com/jestjs/jest/blob/main/packages/jest-config/src/utils.ts
//   computes `path.relative(rootDir, testFile)` for display in test reports.
//   consumer expectation: same-path produces "" (not "."), and ascending
//   produces "..".

#[test]
fn consumer_jest_relative_same_path_empty_string() {
    assert_eq!(posix::relative("/proj/src/foo.test.js", "/proj/src/foo.test.js"), "");
}

#[test]
fn consumer_jest_relative_to_parent_yields_dotdot() {
    assert_eq!(posix::relative("/proj/src/x", "/proj/src"), "..");
}

// ────────── eslint — extension test via extname ──────────
//
// Source: https://github.com/eslint/eslint/blob/main/lib/cli.js
//   filters files by extension via `path.extname(file) === ".js"`. relies on
//   dotfile-no-extension semantics.

#[test]
fn consumer_eslint_dotfile_has_no_extension() {
    assert_eq!(posix::extname(".eslintrc"), "");
}

#[test]
fn consumer_eslint_extname_dotjs() {
    assert_eq!(posix::extname("Component.test.js"), ".js");
}

// ────────── browserify — cross-platform path normalization ──────────
//
// Source: https://github.com/browserify/path-browserify
//   the npm `path-browserify` polyfill is what many bundlers ship for
//   browser builds. one of its consumer-test reps:
//   `path.normalize("/foo/bar//baz/asdf/quux/..") === "/foo/bar/baz/asdf"`

#[test]
fn consumer_browserify_normalize_canonical_test() {
    assert_eq!(
        posix::normalize("/foo/bar//baz/asdf/quux/.."),
        "/foo/bar/baz/asdf"
    );
}

// ────────── bun-specific — win32.isAbsolute on forward slash ──────────
//
// Source: Bun test `is-absolute.test.js`.
// Consumer expectation: Bun follows Node behavior where Win32 mode treats
// forward-slash root as absolute.

#[test]
fn consumer_bun_win32_isabsolute_forward_slash() {
    assert!(win32::is_absolute("/foo/bar"));
}

// ────────── parcel — bundler path.parse for asset extraction ──────────
//
// Source: https://github.com/parcel-bundler/parcel/blob/v2/packages/utils/
//   uses `path.parse(filePath)` to extract `name` for chunk-naming. consumer
//   expectation: `parse(...).name` excludes the extension cleanly.

#[test]
fn consumer_parcel_parse_name_excludes_ext() {
    let p = posix::parse("/src/components/Button.test.tsx");
    assert_eq!(p.name, "Button.test");
    assert_eq!(p.ext, ".tsx");
    assert_eq!(p.base, "Button.test.tsx");
}
