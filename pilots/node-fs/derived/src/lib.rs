// node-fs pilot — Node's `fs` module sync subset.
//
// Inputs:
//   AUDIT — pilots/node-fs/AUDIT.md
//   CD    — runs/2026-05-10-bun-v0.13b-spec-batch/constraints/fs.constraints.md
//           (20 properties / 255 cross-corroborated clauses)
//   REF   — Node.js docs §fs (https://nodejs.org/api/fs.html)
//
// Tier-2 ecosystem-compat. std::fs wrapper. Sync subset only;
// fs.promises.* deferred. ReadStream/WriteStream deferred (stream substrate
// composition).

use std::fs;
use std::io;
use std::path::Path;

/// Stats — subset of Node's fs.Stats with the most-cited fields.
#[derive(Debug, Clone)]
pub struct Stats {
    pub size: u64,
    pub mtime_ms: i64,
    pub atime_ms: i64,
    pub ctime_ms: i64,
    pub is_file: bool,
    pub is_directory: bool,
    pub is_symlink: bool,
}

impl Stats {
    fn from_metadata(m: &fs::Metadata) -> Self {
        let size = m.len();
        let mtime_ms = system_time_to_ms(m.modified().ok());
        let atime_ms = system_time_to_ms(m.accessed().ok());
        let ctime_ms = system_time_to_ms(m.created().ok());
        let is_file = m.is_file();
        let is_directory = m.is_dir();
        let is_symlink = m.file_type().is_symlink();
        Self {
            size, mtime_ms, atime_ms, ctime_ms,
            is_file, is_directory, is_symlink,
        }
    }
}

fn system_time_to_ms(t: Option<std::time::SystemTime>) -> i64 {
    t.and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// CD: `expect(fs.existsSync(path)).toBe(true)` — the most-cited fs sync method.
pub fn exists_sync(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

/// `fs.readFileSync(path)` — bytes.
pub fn read_file_sync(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    fs::read(path)
}

/// `fs.readFileSync(path, "utf-8")` — string.
pub fn read_file_string_sync(path: impl AsRef<Path>) -> io::Result<String> {
    fs::read_to_string(path)
}

/// `fs.writeFileSync(path, data)`.
pub fn write_file_sync(path: impl AsRef<Path>, data: &[u8]) -> io::Result<()> {
    fs::write(path, data)
}

/// `fs.writeFileSync(path, str, "utf-8")`.
pub fn write_file_string_sync(path: impl AsRef<Path>, data: &str) -> io::Result<()> {
    fs::write(path, data.as_bytes())
}

/// `fs.appendFileSync(path, data)`.
pub fn append_file_sync(path: impl AsRef<Path>, data: &[u8]) -> io::Result<()> {
    use io::Write;
    let mut f = fs::OpenOptions::new().append(true).create(true).open(path)?;
    f.write_all(data)
}

/// `fs.mkdirSync(path, { recursive })`. Pilot defaults non-recursive per Node;
/// pass `recursive=true` for the `mkdir -p` analog.
pub fn mkdir_sync(path: impl AsRef<Path>, recursive: bool) -> io::Result<()> {
    if recursive {
        fs::create_dir_all(path)
    } else {
        fs::create_dir(path)
    }
}

/// `fs.rmdirSync(path)` — empty dir only.
pub fn rmdir_sync(path: impl AsRef<Path>) -> io::Result<()> {
    fs::remove_dir(path)
}

/// `fs.rmSync(path, { recursive: true })` — recursive remove.
pub fn rm_sync_recursive(path: impl AsRef<Path>) -> io::Result<()> {
    fs::remove_dir_all(path)
}

/// `fs.unlinkSync(path)` — remove file.
pub fn unlink_sync(path: impl AsRef<Path>) -> io::Result<()> {
    fs::remove_file(path)
}

/// `fs.renameSync(old, new)`.
pub fn rename_sync(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    fs::rename(from, to)
}

/// `fs.copyFileSync(src, dest)`.
pub fn copy_file_sync(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> io::Result<u64> {
    fs::copy(src, dest)
}

/// `fs.statSync(path)` — follows symlinks.
pub fn stat_sync(path: impl AsRef<Path>) -> io::Result<Stats> {
    Ok(Stats::from_metadata(&fs::metadata(path)?))
}

/// `fs.lstatSync(path)` — does NOT follow symlinks.
pub fn lstat_sync(path: impl AsRef<Path>) -> io::Result<Stats> {
    Ok(Stats::from_metadata(&fs::symlink_metadata(path)?))
}

/// `fs.readdirSync(path)`. Returns sorted to match Node's typical
/// alphabetical order (Node doesn't actually sort; pilot does for verifier
/// determinism).
pub fn readdir_sync(path: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let mut entries: Vec<String> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    entries.sort();
    Ok(entries)
}

/// `fs.accessSync(path)` — succeeds if path exists and is readable.
pub fn access_sync(path: impl AsRef<Path>) -> io::Result<()> {
    fs::metadata(path).map(|_| ())
}

/// `fs.realpathSync(path)` — canonicalize.
pub fn realpath_sync(path: impl AsRef<Path>) -> io::Result<std::path::PathBuf> {
    fs::canonicalize(path)
}
