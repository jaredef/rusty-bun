// Win32 path semantics. Separator '\\'. Path delimiter ';'.
//
// Pilot scope: enough Win32 to handle the antichain reps in the constraint
// corpus. Drive letters supported; UNC paths basic; full namespacing
// (\\\\?\\, \\\\.\\) deferred per AUDIT.

pub const SEP: &str = "\\";
pub const DELIMITER: &str = ";";

/// SPEC: Win32 absolute = drive-letter root (`C:\\`), drive-relative
/// (`C:foo`), or UNC root (`\\\\server\\share`), or root-relative (`\\foo`).
/// Bun's CD: `path.win32.isAbsolute("/") === true` (Win32 treats forward-slash
/// root as absolute too).
pub fn is_absolute(path: &str) -> bool {
    if path.is_empty() { return false; }
    let bytes = path.as_bytes();
    // Forward-slash or backslash leading
    if bytes[0] == b'/' || bytes[0] == b'\\' { return true; }
    // Drive letter: C:\\ or C:/ — must have separator after colon
    if bytes.len() >= 3 && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/') {
        if matches!(bytes[0], b'A'..=b'Z' | b'a'..=b'z') { return true; }
    }
    false
}

/// CD: `path.win32.basename("C:\\foo\\bar.txt", ".txt") === "bar"`.
pub fn basename(path: &str, ext: Option<&str>) -> String {
    if path.is_empty() { return String::new(); }
    // Replace forward slashes with backslashes for analysis (Win32 accepts both).
    let last_sep = path.rfind(|c: char| c == '\\' || c == '/');
    let last_seg = match last_sep {
        Some(i) => &path[i + 1..],
        None => {
            // Strip drive prefix if present.
            let bytes = path.as_bytes();
            if bytes.len() >= 2 && bytes[1] == b':' && matches!(bytes[0], b'A'..=b'Z' | b'a'..=b'z') {
                &path[2..]
            } else {
                path
            }
        }
    };
    if last_seg.is_empty() { return String::new(); }
    if let Some(suffix) = ext {
        if !suffix.is_empty() && last_seg.ends_with(suffix) && last_seg.len() > suffix.len() {
            return last_seg[..last_seg.len() - suffix.len()].to_string();
        }
    }
    last_seg.to_string()
}

/// CD: `path.win32.dirname("C:\\foo\\bar") === "C:\\foo"`.
pub fn dirname(path: &str) -> String {
    if path.is_empty() { return ".".to_string(); }
    let bytes = path.as_bytes();
    let drive_prefix_len = if bytes.len() >= 2 && bytes[1] == b':'
        && matches!(bytes[0], b'A'..=b'Z' | b'a'..=b'z') { 2 } else { 0 };
    let scan = &path[drive_prefix_len..];
    let last_sep = scan.rfind(|c: char| c == '\\' || c == '/');
    match last_sep {
        Some(0) => format!("{}\\", &path[..drive_prefix_len]),
        Some(i) => path[..drive_prefix_len + i].to_string(),
        None => {
            if drive_prefix_len > 0 { path[..drive_prefix_len].to_string() }
            else { ".".to_string() }
        }
    }
}

pub fn extname(path: &str) -> String {
    let base = basename(path, None);
    let mut start = 0;
    for c in base.chars() { if c == '.' { start += 1; } else { break; } }
    let scan = &base[start..];
    match scan.rfind('.') {
        Some(i) => scan[i..].to_string(),
        None => String::new(),
    }
}

pub fn join(parts: &[&str]) -> String {
    let mut joined = String::new();
    for part in parts {
        if part.is_empty() { continue; }
        if !joined.is_empty() && !joined.ends_with('\\') && !joined.ends_with('/') {
            joined.push('\\');
        }
        joined.push_str(part);
    }
    if joined.is_empty() { return ".".to_string(); }
    normalize(&joined)
}

pub fn normalize(path: &str) -> String {
    if path.is_empty() { return ".".to_string(); }
    let bytes = path.as_bytes();
    let drive_prefix_len = if bytes.len() >= 2 && bytes[1] == b':'
        && matches!(bytes[0], b'A'..=b'Z' | b'a'..=b'z') { 2 } else { 0 };
    let drive = &path[..drive_prefix_len];
    let rest = &path[drive_prefix_len..];
    let is_abs = !rest.is_empty() && (rest.as_bytes()[0] == b'\\' || rest.as_bytes()[0] == b'/');
    let trailing_sep = rest.len() > 1 && (rest.ends_with('\\') || rest.ends_with('/'));
    let mut segs: Vec<&str> = Vec::new();
    for seg in rest.split(|c: char| c == '\\' || c == '/') {
        match seg {
            "" | "." => continue,
            ".." => {
                let pop = match segs.last() { Some(&"..") => false, Some(_) => true, None => false };
                if pop { segs.pop(); }
                else if !is_abs { segs.push(".."); }
            }
            other => segs.push(other),
        }
    }
    let mut out = String::new();
    out.push_str(drive);
    if is_abs { out.push('\\'); }
    out.push_str(&segs.join("\\"));
    if out.is_empty() { return ".".to_string(); }
    if trailing_sep && !out.ends_with('\\') { out.push('\\'); }
    out
}
