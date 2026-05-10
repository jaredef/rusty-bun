// POSIX path semantics. Separator '/'. Path delimiter ':'.

pub const SEP: &str = "/";
pub const DELIMITER: &str = ":";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PathParsed {
    pub root: String,
    pub dir: String,
    pub base: String,
    pub ext: String,
    pub name: String,
}

/// CD: `path.posix.basename("/foo/bar/baz.html", ".html") === "baz"`.
/// Edge case (Bun test): `path.posix.basename("") === ""`.
pub fn basename(path: &str, ext: Option<&str>) -> String {
    if path.is_empty() {
        return String::new();
    }
    // Strip trailing separators.
    let trimmed = strip_trailing_seps(path);
    if trimmed.is_empty() {
        // Path was all '/'.
        return String::new();
    }
    let last_seg = match trimmed.rfind('/') {
        Some(i) => &trimmed[i + 1..],
        None => trimmed,
    };
    if let Some(suffix) = ext {
        if !suffix.is_empty() && last_seg.ends_with(suffix) && last_seg.len() > suffix.len() {
            return last_seg[..last_seg.len() - suffix.len()].to_string();
        }
    }
    last_seg.to_string()
}

/// CD: `path.posix.dirname("/foo/bar/baz") === "/foo/bar"`.
pub fn dirname(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let trimmed = strip_trailing_seps(path);
    if trimmed.is_empty() {
        return "/".to_string();
    }
    match trimmed.rfind('/') {
        Some(0) => "/".to_string(),
        Some(i) => trimmed[..i].to_string(),
        None => ".".to_string(),
    }
}

/// CD: `path.posix.extname("file.txt") === ".txt"`.
/// Subtle: `path.posix.extname(".bashrc") === ""` (leading dot is not extension).
pub fn extname(path: &str) -> String {
    let base = basename(path, None);
    // Skip leading dots; `.bashrc` has no extension per Node semantics.
    let mut start = 0;
    for c in base.chars() {
        if c == '.' { start += 1; } else { break; }
    }
    let scan = &base[start..];
    match scan.rfind('.') {
        Some(i) => scan[i..].to_string(),
        None => String::new(),
    }
}

/// CD: `path.posix.parse("/home/user/dir/file.txt") === {root: "/",
///   dir: "/home/user/dir", base: "file.txt", ext: ".txt", name: "file"}`.
pub fn parse(path: &str) -> PathParsed {
    let mut p = PathParsed::default();
    if path.is_empty() {
        return p;
    }
    p.root = if path.starts_with('/') { "/".to_string() } else { String::new() };
    p.dir = dirname(path);
    p.base = basename(path, None);
    p.ext = extname(path);
    p.name = if !p.ext.is_empty() && p.base.ends_with(&p.ext) {
        p.base[..p.base.len() - p.ext.len()].to_string()
    } else {
        p.base.clone()
    };
    // SPEC: when root is empty and dir is '.', path was relative without dir.
    p
}

/// CD format: builds a path string from a parsed object. dir + sep + base if
/// dir is set; otherwise root + base; otherwise just name + ext.
pub fn format(p: &PathParsed) -> String {
    let dir = if !p.dir.is_empty() { p.dir.as_str() } else { p.root.as_str() };
    let base = if !p.base.is_empty() {
        p.base.clone()
    } else {
        format!("{}{}", p.name, p.ext)
    };
    if dir.is_empty() {
        return base;
    }
    if dir == p.root {
        // Avoid double-separator when dir == root.
        return format!("{}{}", dir, base);
    }
    format!("{}/{}", dir, base)
}

/// SPEC: POSIX absolute = starts with '/'.
pub fn is_absolute(path: &str) -> bool {
    path.starts_with('/')
}

/// CD: `path.posix.join("/foo", "bar", "baz/asdf") === "/foo/bar/asdf"`.
/// Joins parts with `/`, normalizes the result.
pub fn join(parts: &[&str]) -> String {
    let mut joined = String::new();
    for part in parts {
        if part.is_empty() { continue; }
        if !joined.is_empty() && !joined.ends_with('/') {
            joined.push('/');
        }
        joined.push_str(part);
    }
    if joined.is_empty() {
        return ".".to_string();
    }
    normalize(&joined)
}

/// CD: `path.posix.normalize("/foo/bar//baz/asdf/quux/..") === "/foo/bar/baz/asdf"`.
/// Collapses `..`, `.`, redundant `/`.
pub fn normalize(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let is_abs = path.starts_with('/');
    let trailing_sep = path.len() > 1 && path.ends_with('/');
    let mut segs: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => continue,
            ".." => {
                let pop = match segs.last() {
                    Some(&".." ) => false,
                    Some(_) => true,
                    None => false,
                };
                if pop { segs.pop(); }
                else if !is_abs { segs.push(".."); }
            }
            other => segs.push(other),
        }
    }
    let mut out = String::new();
    if is_abs { out.push('/'); }
    out.push_str(&segs.join("/"));
    if out.is_empty() {
        return if is_abs { "/".to_string() } else { ".".to_string() };
    }
    if trailing_sep && !out.ends_with('/') { out.push('/'); }
    out
}

/// CD: `path.posix.relative("/foo/bar", "/foo/bar/baz") === "baz"`.
/// Both paths must be absolute for POSIX; if not, resolve against pilot's
/// fixed CWD ("/" by default).
pub fn relative(from: &str, to: &str) -> String {
    if from == to { return String::new(); }
    let from_abs = if is_absolute(from) {
        normalize(from)
    } else {
        normalize(&format!("/{}", from))
    };
    let to_abs = if is_absolute(to) {
        normalize(to)
    } else {
        normalize(&format!("/{}", to))
    };
    if from_abs == to_abs { return String::new(); }
    let from_segs: Vec<&str> = from_abs.trim_start_matches('/').split('/').filter(|s| !s.is_empty()).collect();
    let to_segs: Vec<&str> = to_abs.trim_start_matches('/').split('/').filter(|s| !s.is_empty()).collect();
    let common = from_segs.iter().zip(to_segs.iter()).take_while(|(a, b)| a == b).count();
    let up_count = from_segs.len() - common;
    let mut parts: Vec<&str> = vec![".."; up_count];
    parts.extend_from_slice(&to_segs[common..]);
    parts.join("/")
}

/// CD: `path.posix.resolve("/foo/bar", "./baz") === "/foo/bar/baz"`.
/// Right-to-left: process arguments from the end, accumulating until we hit
/// an absolute path. If we never hit one, prepend cwd.
pub fn resolve(parts: &[&str], cwd: &str) -> String {
    let mut resolved = String::new();
    let mut resolved_absolute = false;
    for part in parts.iter().rev() {
        if part.is_empty() { continue; }
        if resolved.is_empty() {
            resolved = part.to_string();
        } else {
            resolved = format!("{}/{}", part, resolved);
        }
        if part.starts_with('/') {
            resolved_absolute = true;
            break;
        }
    }
    if !resolved_absolute {
        let cwd_str = if cwd.is_empty() { "/" } else { cwd };
        resolved = if resolved.is_empty() {
            cwd_str.to_string()
        } else {
            format!("{}/{}", cwd_str.trim_end_matches('/'), resolved)
        };
    }
    let normalized = normalize(&resolved);
    if normalized == "." || normalized.is_empty() { "/".to_string() } else { normalized }
}

fn strip_trailing_seps(path: &str) -> &str {
    let mut end = path.len();
    while end > 1 && path.as_bytes()[end - 1] == b'/' {
        end -= 1;
    }
    &path[..end]
}
