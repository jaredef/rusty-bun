// Simulated-derivation v0 of URLSearchParams.
//
// Inputs declared at the head of major sections:
//   AUDIT — pilots/urlsearchparams/AUDIT.md (constraint coverage map)
//   SPEC  — https://url.spec.whatwg.org/#interface-urlsearchparams
//           https://url.spec.whatwg.org/#urlencoded-parsing
//           https://url.spec.whatwg.org/#urlencoded-serializing
//   CD    — auto-emitted constraint doc at runs/2026-05-10-deno-v0.13b-spec-batch/
//           constraints/urlsearchparams.constraints.md (17 properties / 35 clauses)
//
// Out-of-scope per AUDIT.md "Pilot scope":
//   - HTMLFormElement constructor input (browser-only)
//   - URL backreference (.searchParams association)
//   - IDL coercion of non-USVString inputs (Rust's type system handles)

use std::fmt;

/// Internal entry list. Order is significant — spec mandates an ordered list.
type Entries = Vec<(String, String)>;

#[derive(Debug, Default, Clone)]
pub struct URLSearchParams {
    list: Entries,
}

#[derive(Debug, Clone)]
pub enum Init<'a> {
    /// Form-urlencoded query string, with optional leading "?".
    QueryString(&'a str),
    /// Sequence of (name, value) pairs, in order.
    Pairs(&'a [(&'a str, &'a str)]),
    /// Record-shaped: same as Pairs but emphasized as "object map" semantics.
    /// Insertion order preserved (Rust does not have JS-engine record-iteration
    /// quirks).
    Record(&'a [(&'a str, &'a str)]),
}

impl URLSearchParams {
    // SPEC §5.2.constructor + §5.2.5 application/x-www-form-urlencoded parser.
    // CD URLSearchParams[card=5/spec] enumerates the three init forms.
    pub fn new() -> Self { Self { list: Entries::new() } }

    pub fn from_query(s: &str) -> Self {
        let mut list = Entries::new();
        // SPEC: optional leading "?" is stripped before parsing.
        let body = s.strip_prefix('?').unwrap_or(s);
        if body.is_empty() {
            return Self { list };
        }
        for chunk in body.split('&') {
            if chunk.is_empty() { continue; }
            let (name, value) = match chunk.find('=') {
                Some(i) => (&chunk[..i], &chunk[i + 1..]),
                None => (chunk, ""),
            };
            list.push((form_urldecode(name), form_urldecode(value)));
        }
        Self { list }
    }

    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        let list = pairs
            .iter()
            .map(|(n, v)| ((*n).to_string(), (*v).to_string()))
            .collect();
        Self { list }
    }

    pub fn from_init(init: Init<'_>) -> Self {
        match init {
            Init::QueryString(s) => Self::from_query(s),
            Init::Pairs(p) | Init::Record(p) => Self::from_pairs(p),
        }
    }

    // ─────────── Mutation methods ────────────
    // CD URLSearchParams.prototype.append[card=2/spec]: never replaces.
    pub fn append(&mut self, name: &str, value: &str) {
        self.list.push((name.to_string(), value.to_string()));
    }

    // CD URLSearchParams.prototype.delete[card=2/spec]: by name removes all;
    // by (name, value) removes only matching pairs.
    pub fn delete(&mut self, name: &str) {
        self.list.retain(|(n, _)| n != name);
    }

    pub fn delete_with_value(&mut self, name: &str, value: &str) {
        self.list.retain(|(n, v)| !(n == name && v == value));
    }

    // CD URLSearchParams.prototype.get[card=2/spec]: first match or None.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.list.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
    }

    // CD URLSearchParams.prototype.getAll[card=2/spec].
    pub fn get_all(&self, name: &str) -> Vec<&str> {
        self.list
            .iter()
            .filter(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
            .collect()
    }

    // CD URLSearchParams.prototype.has[card=2/spec].
    pub fn has(&self, name: &str) -> bool {
        self.list.iter().any(|(n, _)| n == name)
    }

    pub fn has_with_value(&self, name: &str, value: &str) -> bool {
        self.list.iter().any(|(n, v)| n == name && v == value)
    }

    // CD URLSearchParams.prototype.set[card=2/spec]:
    //   "replaces all existing entries with one"
    //   "preserves the position of the first existing entry"
    pub fn set(&mut self, name: &str, value: &str) {
        let mut first_index: Option<usize> = None;
        let mut new_list: Entries = Vec::with_capacity(self.list.len());
        for (i, (n, v)) in self.list.iter().enumerate() {
            if n == name {
                if first_index.is_none() {
                    first_index = Some(new_list.len());
                    new_list.push((name.to_string(), value.to_string()));
                }
                // skip subsequent matches
                let _ = (i, v);
            } else {
                new_list.push((n.clone(), v.clone()));
            }
        }
        if first_index.is_none() {
            new_list.push((name.to_string(), value.to_string()));
        }
        self.list = new_list;
    }

    // CD URLSearchParams.prototype.sort[card=2/spec]:
    //   "orders entries by name using a stable sort over UTF-16 code units"
    //   "preserves the relative order of entries with equal names"
    pub fn sort(&mut self) {
        self.list.sort_by(|a, b| utf16_compare(&a.0, &b.0));
    }

    // CD URLSearchParams.prototype.size[card=1/spec]: count of entries.
    pub fn size(&self) -> usize { self.list.len() }

    // CD URLSearchParams.prototype.entries / keys / values / forEach.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.list.iter().map(|(n, v)| (n.as_str(), v.as_str()))
    }
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.list.iter().map(|(n, _)| n.as_str())
    }
    pub fn values(&self) -> impl Iterator<Item = &str> {
        self.list.iter().map(|(_, v)| v.as_str())
    }
    pub fn for_each<F: FnMut(&str, &str)>(&self, mut f: F) {
        for (n, v) in &self.list {
            f(n, v);
        }
    }
}

// ─────────────────────── Display / toString ───────────────────────────────
//
// CD URLSearchParams[card=7 / cross-corroborated]:
//   Deno: assertEquals(searchParams, "str=this+string+has+spaces+in+it")
//   Deno: assertEquals(searchParams, "str=hello%2C+world%21")
// SPEC: form-urlencoded serializer joins entries with "&", pairs with "=",
//       and percent-encodes per the form-urlencoded character set with
//       SPACE → "+".

impl fmt::Display for URLSearchParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for (n, v) in &self.list {
            if !first { f.write_str("&")?; }
            first = false;
            form_urlencode_write(f, n)?;
            f.write_str("=")?;
            form_urlencode_write(f, v)?;
        }
        Ok(())
    }
}

// ─────────────────────────── form-urlencoded ──────────────────────────────
//
// SPEC §5.2.5 application/x-www-form-urlencoded percent-encode set:
// percent-encode every byte EXCEPT:
//   ASCII alphanumerics ([A-Za-z0-9])
//   *, -, ., _
// Additionally:
//   SPACE (0x20) → "+"   (NOT "%20")
//   Any other byte → "%XX"
// Decoding: "+" → SPACE; "%HH" → byte HH (case-insensitive hex); any
// undecodable %-sequence → preserve literally per SPEC's tolerant decoder.

fn is_form_urlencoded_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'*' | b'-' | b'.' | b'_')
}

fn form_urlencode_write(f: &mut fmt::Formatter<'_>, s: &str) -> fmt::Result {
    for &b in s.as_bytes() {
        if b == b' ' {
            f.write_str("+")?;
        } else if is_form_urlencoded_unreserved(b) {
            f.write_str(unsafe { std::str::from_utf8_unchecked(&[b]) })?;
        } else {
            write!(f, "%{:02X}", b)?;
        }
    }
    Ok(())
}

fn form_urldecode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'+' {
            out.push(b' ');
            i += 1;
        } else if b == b'%' && i + 2 < bytes.len() {
            let h1 = hex_digit(bytes[i + 1]);
            let h2 = hex_digit(bytes[i + 2]);
            match (h1, h2) {
                (Some(a), Some(c)) => {
                    out.push((a << 4) | c);
                    i += 3;
                }
                _ => {
                    out.push(b);
                    i += 1;
                }
            }
        } else {
            out.push(b);
            i += 1;
        }
    }
    // SPEC: result is decoded as UTF-8 (lossy on invalid sequences per the
    // form-urlencoded byte-sequence-to-USVString step).
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

// ─────────────────────────── UTF-16 sort key ──────────────────────────────
//
// SPEC: sort orders entries by name as a sequence of UTF-16 code units. For
// pure-ASCII this matches byte order. For non-ASCII the comparison must
// happen on UTF-16 code units, not Rust's char (USV) order — surrogate
// pairs sort *between* BMP code points, not after them.

fn utf16_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let mut ai = a.encode_utf16();
    let mut bi = b.encode_utf16();
    loop {
        match (ai.next(), bi.next()) {
            (Some(x), Some(y)) => match x.cmp(&y) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            },
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (None, None) => return std::cmp::Ordering::Equal,
        }
    }
}
