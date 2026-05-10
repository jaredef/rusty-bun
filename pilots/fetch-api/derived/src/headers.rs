// Headers — WHATWG Fetch §5.2.
//
// Inputs cited inline below.
//
// Pilot semantics:
// - Stores entries as Vec<(lowercase_name, value)> for ordered iteration.
// - Names compared case-insensitively (per HTTP RFC 7230).
// - Iterates with names already lowercased per spec.
// - HTTP whitespace stripped from values per SPEC §5.2.append.
// - TypeError on invalid name (must match HTTP token char set) or value
//   (must not contain CR/LF/NUL).

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderError {
    InvalidName(String),
    InvalidValue(String),
}

#[derive(Debug, Clone, Default)]
pub struct Headers {
    entries: Vec<(String, String)>,
}

impl Headers {
    pub fn new() -> Self { Self::default() }

    pub fn from_pairs(pairs: &[(&str, &str)]) -> Result<Self, HeaderError> {
        let mut h = Self::new();
        for (n, v) in pairs {
            h.append(n, v)?;
        }
        Ok(h)
    }

    /// SPEC §5.2.append: validates name + value, normalizes value (strips
    /// HTTP whitespace), appends entry. Repeated names ARE allowed; iteration
    /// will combine via "," for the .get() call.
    pub fn append(&mut self, name: &str, value: &str) -> Result<(), HeaderError> {
        let name = validate_name(name)?;
        let value = normalize_value(value)?;
        self.entries.push((name, value));
        Ok(())
    }

    /// SPEC §5.2.delete: case-insensitive removal.
    pub fn delete(&mut self, name: &str) {
        let lower = name.to_ascii_lowercase();
        self.entries.retain(|(n, _)| n != &lower);
    }

    /// SPEC §5.2.get: combined value (joined by ", ") or None.
    /// CD HEAD1 antichain ref: `headers.get("content-type") === "application/json"`.
    pub fn get(&self, name: &str) -> Option<String> {
        let lower = name.to_ascii_lowercase();
        let mut combined: Option<String> = None;
        for (n, v) in &self.entries {
            if n == &lower {
                match &mut combined {
                    None => combined = Some(v.clone()),
                    Some(c) => { c.push_str(", "); c.push_str(v); }
                }
            }
        }
        combined
    }

    /// SPEC §5.2.getSetCookie: returns ALL Set-Cookie values as separate
    /// strings (does NOT combine). Set-Cookie's special handling per RFC 6265.
    pub fn get_set_cookie(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter(|(n, _)| n == "set-cookie")
            .map(|(_, v)| v.clone())
            .collect()
    }

    /// SPEC §5.2.has.
    pub fn has(&self, name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        self.entries.iter().any(|(n, _)| n == &lower)
    }

    /// SPEC §5.2.set: replace all existing values for the name with one.
    pub fn set(&mut self, name: &str, value: &str) -> Result<(), HeaderError> {
        let n = validate_name(name)?;
        let v = normalize_value(value)?;
        let mut placed = false;
        let mut new_entries: Vec<(String, String)> = Vec::with_capacity(self.entries.len());
        for (existing_n, existing_v) in &self.entries {
            if existing_n == &n {
                if !placed {
                    new_entries.push((n.clone(), v.clone()));
                    placed = true;
                }
            } else {
                new_entries.push((existing_n.clone(), existing_v.clone()));
            }
        }
        if !placed {
            new_entries.push((n, v));
        }
        self.entries = new_entries;
        Ok(())
    }

    pub fn count(&self) -> usize { self.entries.len() }

    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> {
        // SPEC §5.2.entries: name + value pairs ordered by lexicographic
        // lowercased name (NOT insertion order). Pilot sorts on iteration.
        let mut sorted = self.entries.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        sorted.into_iter().map(|(n, v)| (n.as_str(), v.as_str()))
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        let mut sorted = self.entries.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        sorted.into_iter().map(|(n, _)| n.as_str())
    }

    pub fn for_each<F: FnMut(&str, &str)>(&self, mut f: F) {
        let mut sorted = self.entries.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        for (n, v) in sorted { f(n, v); }
    }
}

/// SPEC §5.2.append: name validation per RFC 7230 token-char set:
/// "!" / "#" / "$" / "%" / "&" / "'" / "*" / "+" / "-" / "." / "^" / "_"
/// / "`" / "|" / "~" / DIGIT / ALPHA. Empty name is invalid.
fn validate_name(name: &str) -> Result<String, HeaderError> {
    if name.is_empty() {
        return Err(HeaderError::InvalidName(name.to_string()));
    }
    for b in name.bytes() {
        let ok = matches!(b,
            b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' |
            b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~' |
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z');
        if !ok {
            return Err(HeaderError::InvalidName(name.to_string()));
        }
    }
    Ok(name.to_ascii_lowercase())
}

/// SPEC §5.2.append: value validation + normalization. CR/LF/NUL forbidden;
/// leading/trailing HTTP whitespace stripped (SP, HTAB, LF, CR).
fn normalize_value(value: &str) -> Result<String, HeaderError> {
    for b in value.bytes() {
        if b == 0 || b == b'\n' || b == b'\r' {
            return Err(HeaderError::InvalidValue(value.to_string()));
        }
    }
    Ok(value
        .trim_matches(|c: char| matches!(c, ' ' | '\t' | '\n' | '\r'))
        .to_string())
}
