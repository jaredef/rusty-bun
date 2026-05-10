# node-fs pilot — coverage audit

**Fourteenth pilot. Tier-C #6 from the trajectory queue. First Tier-C pilot.** Node's `fs` module sync subset. Largest reference target the apparatus has compared against (21,540 LOC).

## Constraint inputs

From `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/fs.constraints.md`:
- 20 candidate properties / 255 cross-corroborated clauses
- 12 construction-style + 8 behavioral
- Top: `fs.existsSync` (76 clauses), `fs.lstatSync`, `fs.mkdirSync`, `fs.glob*`, etc.

## Pilot scope

**Sync subset only.** Async/Promise variants deferred (`fs.promises.*`).

In scope:
- `fs::exists_sync(path)` → bool
- `fs::read_file_sync(path) -> Result<Vec<u8>>`
- `fs::read_file_string_sync(path) -> Result<String>` (utf-8)
- `fs::write_file_sync(path, data)`
- `fs::append_file_sync(path, data)`
- `fs::mkdir_sync(path, recursive?)`
- `fs::rmdir_sync(path)`
- `fs::unlink_sync(path)`
- `fs::rename_sync(old, new)`
- `fs::copy_file_sync(src, dest)`
- `fs::stat_sync(path) -> Stats`
- `fs::lstat_sync(path) -> Stats` (no follow-symlink)
- `fs::readdir_sync(path) -> Vec<String>`
- `fs::access_sync(path) -> Result<()>` — exists + readable
- `Stats { size, mtime_ms, is_file, is_directory, is_symlink }`

Out of pilot scope:
- Async/Promise variants
- `fs.glob*` (pattern matching; would need glob substrate)
- `fs.watch`/`fs.watchFile` (event-loop integration)
- ReadStream/WriteStream (stream substrate composition deferred)
- `fs.cp` recursive copy (large surface; deferred)
- File descriptors (`fs.openSync`/`fs.closeSync`/`fs.readSync`/`fs.writeSync`)
- `chmodSync`/`chownSync` (permissions; deferred)
- `realpathSync` (symlink resolution; subset later)

## LOC budget

Bun reference: `runtime/node/node_fs.rs` (8,118 LOC) + `node_fs.zig` (7,344 LOC) + binding files = 21,540 LOC. Pilot scope is the sync subset, equivalent to maybe ~1,000-1,500 LOC of the reference. Pilot target: 200-300 code-only LOC.

## Approach

Wrap `std::fs`. Most pilot code is signature translation. Stats struct exposes Bun's API shape (mtime in ms, is_file/is_directory predicates) by extracting from `std::fs::Metadata`.

## Ahead-of-time hypotheses

1. **Pilot is small in LOC.** std::fs does the heavy lifting. Estimated 200-250 LOC.

2. **First-run clean closure expected.** sync semantics are concrete; Bun follows Node API.

3. **The Stats struct is the most likely place for divergence.** Node's `Stats` exposes ~20 fields (mtime/atime/ctime/birthtime in ms AND ns AND Date forms; size; nlink; ino; mode; uid; gid; etc.). Pilot will cover the most-cited subset (size, mtime_ms, is_file, is_directory) per the constraint corpus.

4. **First-run will surface at least one cross-platform path semantics bug.** `readdir` ordering varies across platforms; mtime resolution differs. AOT prediction: at least one platform-related test needs adjustment.

## Verifier strategy

~25-30 verifier tests using temp files/dirs with std::env::temp_dir() + std::process::id() for isolation. Tests touch the OS but stay fast.

Consumer regression: ~6-8 tests citing real Node-ecosystem fs usage.
