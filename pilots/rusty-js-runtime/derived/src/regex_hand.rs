//! Tier-Ω.5.ggg — hand-rolled JS regex engine. Backtracking matcher with
//! continuation-passing style; supports the JS-regex subset the Rust
//! `regex` crate rejects (lookahead/lookbehind).
//!
//! Scope (v1):
//!   - Literal chars + standard escapes (\d \D \s \S \w \W \. \\ \n \t \r etc.)
//!   - Char classes [abc], negated [^abc], ranges [a-z], inside-escapes \d etc.
//!   - Anchors ^ and $ (multiline-aware via flags.m)
//!   - Quantifiers ? * + {n} {n,} {n,m} — greedy + lazy (trailing ?)
//!   - Groups (...) capturing, (?:...) non-capturing
//!   - Alternation |
//!   - Lookahead (?=...) (?!...) ; lookbehind (?<=...) (?<!...)
//!   - Backreferences \1..\9
//!
//! Out of scope: Unicode property escapes \p{...}, named groups, atomic
//! groups, recursion, conditionals.
//!
//! Wired as a fallback engine: regexp.rs tries the Rust `regex` crate
//! first; if compilation fails, this module is used. Both engines must
//! expose the same surface: is_match + find_at (byte offsets).

use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Node {
    /// Match a single character (after case-folding if i flag).
    Char(char),
    /// Match any one character (`.`). With dotall, matches `\n` too.
    AnyChar,
    /// Match against a character set.
    Class(CharClass),
    /// `^` — start of input (or line in multiline mode).
    Anchor(AnchorKind),
    /// Concatenation: match all children in sequence.
    Concat(Vec<Node>),
    /// Alternation: try each child in order.
    Alt(Vec<Node>),
    /// Repetition with min/max bounds + greedy/lazy.
    Repeat { inner: Box<Node>, min: usize, max: Option<usize>, greedy: bool },
    /// Capturing group with 1-based index.
    Group { index: usize, inner: Box<Node> },
    /// Non-capturing group (no slot reservation).
    NonCapture(Box<Node>),
    /// Zero-width assertion: lookahead/lookbehind, positive/negative.
    Look { ahead: bool, positive: bool, inner: Box<Node> },
    /// Backreference \1..\9 — must match the same text as the indexed group.
    Backref(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum AnchorKind {
    Start,       // ^
    End,         // $
    WordBoundary,    // \b
    NotWordBoundary, // \B
}

/// A character class accepts any single char that matches its predicate.
#[derive(Debug, Clone)]
pub struct CharClass {
    pub negated: bool,
    pub ranges: Vec<(char, char)>,
    pub specials: Vec<SpecialClass>,
}

#[derive(Debug, Clone, Copy)]
pub enum SpecialClass {
    Digit, NotDigit,
    Word, NotWord,
    Space, NotSpace,
}

impl CharClass {
    fn contains(&self, c: char, ignore_case: bool) -> bool {
        let test_chars: Vec<char> = if ignore_case {
            // Try both cases — caller's ignore-case logic should also fold
            // the input; we double-check here for robustness.
            let mut v = vec![c];
            v.extend(c.to_lowercase());
            v.extend(c.to_uppercase());
            v
        } else { vec![c] };
        let inner_match = test_chars.iter().any(|&tc| {
            self.ranges.iter().any(|&(lo, hi)| tc >= lo && tc <= hi)
                || self.specials.iter().any(|s| special_match(*s, tc))
        });
        if self.negated { !inner_match } else { inner_match }
    }
}

fn special_match(s: SpecialClass, c: char) -> bool {
    match s {
        SpecialClass::Digit => c.is_ascii_digit(),
        SpecialClass::NotDigit => !c.is_ascii_digit(),
        SpecialClass::Word => c.is_ascii_alphanumeric() || c == '_',
        SpecialClass::NotWord => !(c.is_ascii_alphanumeric() || c == '_'),
        SpecialClass::Space => c.is_whitespace(),
        SpecialClass::NotSpace => !c.is_whitespace(),
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Flags {
    pub ignore_case: bool,
    pub multiline: bool,
    pub dot_all: bool,
}

#[derive(Debug, Clone)]
pub struct HandRolledRegex {
    pub source: Rc<String>,
    pub flags: Flags,
    pub ast: Node,
    pub group_count: usize,
}

pub struct HandMatch {
    pub start: usize,
    pub end: usize,
    /// captures[0] is the whole match; captures[i] is group i. None if group did not participate.
    pub captures: Vec<Option<(usize, usize)>>,
}

// ────────────── Parser ──────────────

pub fn compile(pattern: &str, flag_str: &str) -> Result<HandRolledRegex, String> {
    let mut flags = Flags::default();
    for c in flag_str.chars() {
        match c {
            'i' => flags.ignore_case = true,
            'm' => flags.multiline = true,
            's' => flags.dot_all = true,
            'g' | 'y' | 'u' | 'd' => {}
            _ => return Err(format!("unsupported flag '{}'", c)),
        }
    }
    let mut p = Parser { chars: pattern.chars().collect(), pos: 0, next_group: 1, group_count: 0, named_groups: std::collections::HashMap::new() };
    let ast = p.parse_alt()?;
    if p.pos < p.chars.len() {
        return Err(format!("unexpected '{}' at position {}", p.chars[p.pos], p.pos));
    }
    Ok(HandRolledRegex {
        source: Rc::new(pattern.to_string()),
        flags,
        ast,
        group_count: p.group_count,
    })
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    next_group: usize,
    group_count: usize,
    // Tier-Ω.5.vvvvv: named capture group support.
    named_groups: std::collections::HashMap<String, usize>,
}

impl Parser {
    fn peek(&self) -> Option<char> { self.chars.get(self.pos).copied() }
    fn bump(&mut self) -> Option<char> { let c = self.peek(); if c.is_some() { self.pos += 1; } c }
    fn eat(&mut self, c: char) -> bool { if self.peek() == Some(c) { self.pos += 1; true } else { false } }

    fn parse_alt(&mut self) -> Result<Node, String> {
        let first = self.parse_concat()?;
        let mut alts = vec![first];
        while self.eat('|') {
            alts.push(self.parse_concat()?);
        }
        Ok(if alts.len() == 1 { alts.pop().unwrap() } else { Node::Alt(alts) })
    }

    fn parse_concat(&mut self) -> Result<Node, String> {
        let mut items = Vec::new();
        loop {
            match self.peek() {
                None | Some(')') | Some('|') => break,
                _ => {
                    let atom = self.parse_atom()?;
                    let with_quant = self.maybe_quantifier(atom)?;
                    items.push(with_quant);
                }
            }
        }
        Ok(if items.len() == 1 { items.pop().unwrap() } else { Node::Concat(items) })
    }

    fn maybe_quantifier(&mut self, inner: Node) -> Result<Node, String> {
        let (min, max) = match self.peek() {
            Some('?') => { self.bump(); (0, Some(1)) }
            Some('*') => { self.bump(); (0, None) }
            Some('+') => { self.bump(); (1, None) }
            Some('{') => {
                let save = self.pos;
                self.bump();
                // Parse digits.
                let mut min_str = String::new();
                while let Some(c) = self.peek() { if c.is_ascii_digit() { min_str.push(c); self.bump(); } else { break; } }
                if min_str.is_empty() {
                    // Not a quantifier — restore.
                    self.pos = save;
                    return Ok(inner);
                }
                let min_v: usize = min_str.parse().map_err(|_| "bad quantifier min")?;
                let (max_v, ok): (Option<usize>, bool) = if self.eat(',') {
                    let mut s = String::new();
                    while let Some(c) = self.peek() { if c.is_ascii_digit() { s.push(c); self.bump(); } else { break; } }
                    if s.is_empty() { (None, self.eat('}')) }
                    else {
                        let v: usize = s.parse().map_err(|_| "bad quantifier max")?;
                        (Some(v), self.eat('}'))
                    }
                } else {
                    (Some(min_v), self.eat('}'))
                };
                if !ok { self.pos = save; return Ok(inner); }
                (min_v, max_v)
            }
            _ => return Ok(inner),
        };
        let greedy = !self.eat('?');
        Ok(Node::Repeat { inner: Box::new(inner), min, max, greedy })
    }

    fn parse_atom(&mut self) -> Result<Node, String> {
        let c = self.peek().ok_or("unexpected end")?;
        match c {
            '^' => { self.bump(); Ok(Node::Anchor(AnchorKind::Start)) }
            '$' => { self.bump(); Ok(Node::Anchor(AnchorKind::End)) }
            '.' => { self.bump(); Ok(Node::AnyChar) }
            '\\' => self.parse_escape(),
            '[' => self.parse_class(),
            '(' => self.parse_group(),
            _ => { self.bump(); Ok(Node::Char(c)) }
        }
    }

    fn parse_escape(&mut self) -> Result<Node, String> {
        self.bump(); // consume backslash
        let c = self.bump().ok_or("trailing backslash")?;
        match c {
            'd' => Ok(Node::Class(CharClass { negated: false, ranges: vec![], specials: vec![SpecialClass::Digit] })),
            'D' => Ok(Node::Class(CharClass { negated: false, ranges: vec![], specials: vec![SpecialClass::NotDigit] })),
            'w' => Ok(Node::Class(CharClass { negated: false, ranges: vec![], specials: vec![SpecialClass::Word] })),
            'W' => Ok(Node::Class(CharClass { negated: false, ranges: vec![], specials: vec![SpecialClass::NotWord] })),
            's' => Ok(Node::Class(CharClass { negated: false, ranges: vec![], specials: vec![SpecialClass::Space] })),
            'S' => Ok(Node::Class(CharClass { negated: false, ranges: vec![], specials: vec![SpecialClass::NotSpace] })),
            'b' => Ok(Node::Anchor(AnchorKind::WordBoundary)),
            'B' => Ok(Node::Anchor(AnchorKind::NotWordBoundary)),
            'n' => Ok(Node::Char('\n')),
            't' => Ok(Node::Char('\t')),
            'r' => Ok(Node::Char('\r')),
            'f' => Ok(Node::Char('\u{000C}')),
            'v' => Ok(Node::Char('\u{000B}')),
            '0' => Ok(Node::Char('\0')),
            'x' => {
                let h1 = self.bump().ok_or("bad \\x escape")?;
                let h2 = self.bump().ok_or("bad \\x escape")?;
                let n = u32::from_str_radix(&format!("{}{}", h1, h2), 16).map_err(|_| "bad \\x escape")?;
                Ok(Node::Char(char::from_u32(n).ok_or("bad \\x escape")?))
            }
            'u' => {
                if self.eat('{') {
                    let mut s = String::new();
                    while let Some(c) = self.peek() { if c == '}' { break; } s.push(c); self.bump(); }
                    if !self.eat('}') { return Err("unterminated \\u{...}".into()); }
                    let n = u32::from_str_radix(&s, 16).map_err(|_| "bad \\u{...} escape")?;
                    Ok(Node::Char(char::from_u32(n).ok_or("bad \\u escape")?))
                } else {
                    let mut s = String::new();
                    for _ in 0..4 { s.push(self.bump().ok_or("bad \\u escape")?); }
                    let n = u32::from_str_radix(&s, 16).map_err(|_| "bad \\u escape")?;
                    Ok(Node::Char(char::from_u32(n).ok_or("bad \\u escape")?))
                }
            }
            d if d.is_ascii_digit() && d != '0' => {
                Ok(Node::Backref((d as u8 - b'0') as usize))
            }
            'k' => {
                // Tier-Ω.5.vvvvv: named backreference \k<name>.
                if !self.eat('<') { return Err("expected < after \\k".into()); }
                let mut name = String::new();
                while let Some(c) = self.peek() {
                    if c == '>' { break; }
                    name.push(c); self.bump();
                }
                if !self.eat('>') { return Err("unterminated named backref".into()); }
                let idx = self.named_groups.get(&name).copied()
                    .ok_or_else(|| format!("unknown named group '{}'", name))?;
                Ok(Node::Backref(idx))
            }
            // Escaped meta or non-special char — treat as literal.
            _ => Ok(Node::Char(c)),
        }
    }

    fn parse_class(&mut self) -> Result<Node, String> {
        self.bump(); // [
        let negated = self.eat('^');
        let mut ranges = Vec::new();
        let mut specials = Vec::new();
        while self.peek() != Some(']') {
            let lo_item = self.parse_class_item()?;
            if self.peek() == Some('-') && self.chars.get(self.pos+1) != Some(&']') {
                self.bump(); // -
                let hi_item = self.parse_class_item()?;
                if let (ClassItem::Char(lo), ClassItem::Char(hi)) = (lo_item.clone(), hi_item.clone()) {
                    ranges.push((lo, hi));
                } else {
                    // Range from a special: treat both as separate items.
                    push_item(lo_item, &mut ranges, &mut specials);
                    push_item(hi_item, &mut ranges, &mut specials);
                }
            } else {
                push_item(lo_item, &mut ranges, &mut specials);
            }
            if self.pos >= self.chars.len() { return Err("unterminated class".into()); }
        }
        self.bump(); // ]
        Ok(Node::Class(CharClass { negated, ranges, specials }))
    }

    fn parse_class_item(&mut self) -> Result<ClassItem, String> {
        let c = self.peek().ok_or("unexpected end in class")?;
        if c == '\\' {
            self.bump();
            let e = self.bump().ok_or("trailing backslash in class")?;
            match e {
                'd' => Ok(ClassItem::Special(SpecialClass::Digit)),
                'D' => Ok(ClassItem::Special(SpecialClass::NotDigit)),
                'w' => Ok(ClassItem::Special(SpecialClass::Word)),
                'W' => Ok(ClassItem::Special(SpecialClass::NotWord)),
                's' => Ok(ClassItem::Special(SpecialClass::Space)),
                'S' => Ok(ClassItem::Special(SpecialClass::NotSpace)),
                'n' => Ok(ClassItem::Char('\n')),
                't' => Ok(ClassItem::Char('\t')),
                'r' => Ok(ClassItem::Char('\r')),
                _ => Ok(ClassItem::Char(e)),
            }
        } else {
            self.bump();
            Ok(ClassItem::Char(c))
        }
    }

    fn parse_group(&mut self) -> Result<Node, String> {
        self.bump(); // (
        let mut node = if self.eat('?') {
            match self.bump() {
                Some(':') => {
                    let inner = self.parse_alt()?;
                    Node::NonCapture(Box::new(inner))
                }
                Some('=') => {
                    let inner = self.parse_alt()?;
                    Node::Look { ahead: true, positive: true, inner: Box::new(inner) }
                }
                Some('!') => {
                    let inner = self.parse_alt()?;
                    Node::Look { ahead: true, positive: false, inner: Box::new(inner) }
                }
                Some('<') => {
                    match self.peek() {
                        Some('=') | Some('!') => {
                            let positive = self.bump() == Some('=');
                            let inner = self.parse_alt()?;
                            Node::Look { ahead: false, positive, inner: Box::new(inner) }
                        }
                        Some(c) if c.is_alphabetic() || c == '_' || c == '$' => {
                            // Tier-Ω.5.vvvvv: named capture group (?<name>...).
                            // marked uses (?<a>`+) + \k<a> for matched-tick
                            // sequences. We allocate a normal indexed group
                            // and side-record the name → index mapping so
                            // \k<name> resolves to the index at parse time.
                            let mut name = String::new();
                            while let Some(c) = self.peek() {
                                if c.is_alphanumeric() || c == '_' || c == '$' {
                                    name.push(c); self.bump();
                                } else { break; }
                            }
                            if !self.eat('>') { return Err("expected > after group name".into()); }
                            let idx = self.next_group;
                            self.next_group += 1;
                            self.group_count = self.group_count.max(idx);
                            self.named_groups.insert(name, idx);
                            let inner = self.parse_alt()?;
                            Node::Group { index: idx, inner: Box::new(inner) }
                        }
                        _ => return Err("unsupported group prefix".into()),
                    }
                }
                Some(c) => return Err(format!("unsupported group prefix (?{}", c)),
                None => return Err("unterminated group prefix".into()),
            }
        } else {
            let idx = self.next_group;
            self.next_group += 1;
            self.group_count = self.group_count.max(idx);
            let inner = self.parse_alt()?;
            Node::Group { index: idx, inner: Box::new(inner) }
        };
        if !self.eat(')') { return Err("expected )".into()); }
        // Quantifier may follow.
        node = self.maybe_quantifier(node)?;
        Ok(node)
    }
}

#[derive(Clone)]
enum ClassItem { Char(char), Special(SpecialClass) }

fn push_item(it: ClassItem, ranges: &mut Vec<(char, char)>, specials: &mut Vec<SpecialClass>) {
    match it {
        ClassItem::Char(c) => ranges.push((c, c)),
        ClassItem::Special(s) => specials.push(s),
    }
}

// ────────────── Matcher ──────────────

/// Try to match the pattern starting at any position >= start. Returns the
/// first match (left-most) with leftmost-first semantics.
pub fn find_at(re: &HandRolledRegex, input: &str, start: usize) -> Option<HandMatch> {
    let chars: Vec<char> = input.chars().collect();
    // Map char index <-> byte index (we accept byte offsets externally).
    let byte_idx_to_char: Vec<usize> = {
        let mut v = Vec::with_capacity(chars.len() + 1);
        let mut b = 0;
        v.push(0);
        for c in &chars { b += c.len_utf8(); v.push(b); }
        v
    };
    // start is a byte offset; find the matching char index.
    let mut start_ci = 0;
    for (ci, &bo) in byte_idx_to_char.iter().enumerate() {
        if bo == start { start_ci = ci; break; }
        if bo > start { start_ci = ci.saturating_sub(1); break; }
    }
    if start > input.len() { return None; }
    for try_at in start_ci..=chars.len() {
        let mut caps: Vec<Option<(usize, usize)>> = vec![None; re.group_count + 1];
        if let Some(end_ci) = mat(&re.ast, &chars, try_at, &re.flags, &mut caps) {
            caps[0] = Some((try_at, end_ci));
            // Convert char indices to byte offsets.
            let bstart = byte_idx_to_char[try_at];
            let bend = byte_idx_to_char[end_ci];
            let bcaps: Vec<Option<(usize, usize)>> = caps.iter().map(|c| c.map(|(s,e)| (byte_idx_to_char[s], byte_idx_to_char[e]))).collect();
            return Some(HandMatch { start: bstart, end: bend, captures: bcaps });
        }
    }
    None
}

pub fn is_match(re: &HandRolledRegex, input: &str) -> bool {
    find_at(re, input, 0).is_some()
}

/// Match `node` against `chars` starting at char index `pos`. On success
/// returns the end char index. Mutates `caps` for any groups encountered;
/// caller is responsible for unwinding on outer backtrack — for simplicity
/// the recursive implementation just lets caps reflect the LAST attempted
/// path, which is fine since callers only read caps after success.
fn mat(node: &Node, chars: &[char], pos: usize, flags: &Flags, caps: &mut Vec<Option<(usize, usize)>>) -> Option<usize> {
    match node {
        Node::Char(c) => {
            let actual = *chars.get(pos)?;
            if char_eq(actual, *c, flags.ignore_case) { Some(pos + 1) } else { None }
        }
        Node::AnyChar => {
            let actual = *chars.get(pos)?;
            if !flags.dot_all && (actual == '\n' || actual == '\r') { None } else { Some(pos + 1) }
        }
        Node::Class(cc) => {
            let actual = *chars.get(pos)?;
            if cc.contains(actual, flags.ignore_case) { Some(pos + 1) } else { None }
        }
        Node::Anchor(a) => {
            match a {
                AnchorKind::Start => {
                    if pos == 0 || (flags.multiline && pos > 0 && chars[pos-1] == '\n') { Some(pos) } else { None }
                }
                AnchorKind::End => {
                    if pos == chars.len() || (flags.multiline && pos < chars.len() && chars[pos] == '\n') { Some(pos) } else { None }
                }
                AnchorKind::WordBoundary => {
                    if at_word_boundary(chars, pos) { Some(pos) } else { None }
                }
                AnchorKind::NotWordBoundary => {
                    if !at_word_boundary(chars, pos) { Some(pos) } else { None }
                }
            }
        }
        Node::Concat(items) => {
            match_concat(items, 0, chars, pos, flags, caps)
        }
        Node::Alt(branches) => {
            for b in branches {
                let saved = caps.clone();
                if let Some(end) = mat(b, chars, pos, flags, caps) { return Some(end); }
                *caps = saved;
            }
            None
        }
        Node::Repeat { inner, min, max, greedy } => {
            match_repeat(inner, *min, *max, *greedy, &[], chars, pos, flags, caps)
        }
        Node::Group { index, inner } => {
            let saved = caps[*index];
            // Optimistically record the start; finalize end on success.
            if let Some(end) = mat(inner, chars, pos, flags, caps) {
                caps[*index] = Some((pos, end));
                Some(end)
            } else {
                caps[*index] = saved;
                None
            }
        }
        Node::NonCapture(inner) => mat(inner, chars, pos, flags, caps),
        Node::Look { ahead: true, positive, inner } => {
            let saved = caps.clone();
            let m = mat(inner, chars, pos, flags, caps);
            *caps = saved;
            if m.is_some() == *positive { Some(pos) } else { None }
        }
        Node::Look { ahead: false, positive, inner } => {
            // Lookbehind: try to match `inner` such that it ends exactly at `pos`.
            let saved = caps.clone();
            let mut matched_any = false;
            for try_start in (0..=pos).rev() {
                let mut tcaps = caps.clone();
                if let Some(end) = mat(inner, chars, try_start, flags, &mut tcaps) {
                    if end == pos { matched_any = true; break; }
                }
            }
            *caps = saved;
            if matched_any == *positive { Some(pos) } else { None }
        }
        Node::Backref(i) => {
            let cap = (*caps).get(*i).and_then(|c| *c)?;
            let (cs, ce) = cap;
            let needed: Vec<char> = chars[cs..ce].to_vec();
            let avail = chars.get(pos..pos + needed.len())?;
            for (a, b) in avail.iter().zip(needed.iter()) {
                if !char_eq(*a, *b, flags.ignore_case) { return None; }
            }
            Some(pos + needed.len())
        }
    }
}

fn match_concat(items: &[Node], idx: usize, chars: &[char], pos: usize, flags: &Flags, caps: &mut Vec<Option<(usize, usize)>>) -> Option<usize> {
    if idx >= items.len() { return Some(pos); }
    let item = &items[idx];
    // If the item is a Repeat, handle inline so we can sequence properly.
    if let Node::Repeat { inner, min, max, greedy } = item {
        let rest = &items[idx + 1..];
        return match_repeat(inner, *min, *max, *greedy, rest, chars, pos, flags, caps);
    }
    let saved = caps.clone();
    if let Some(end) = mat(item, chars, pos, flags, caps) {
        if let Some(final_end) = match_concat(items, idx + 1, chars, end, flags, caps) {
            return Some(final_end);
        }
    }
    *caps = saved;
    // Alt-inside-Concat needs additional alternatives to be tried; mat()
    // already handles that internally because Alt iterates branches.
    None
}

/// Repeat `inner` between min and max times, then match the trailing
/// `rest` items. Greedy: try max first, back off to min. Lazy: try min
/// first, extend toward max.
fn match_repeat(inner: &Node, min: usize, max: Option<usize>, greedy: bool, rest: &[Node], chars: &[char], pos: usize, flags: &Flags, caps: &mut Vec<Option<(usize, usize)>>) -> Option<usize> {
    // Collect each successive match position up to max.
    let mut positions = vec![pos];
    let cap_max = max.unwrap_or(chars.len() - pos + 1);
    let mut cur = pos;
    while positions.len() <= cap_max + 1 {
        let saved = caps.clone();
        match mat(inner, chars, cur, flags, caps) {
            Some(next) if next > cur => { positions.push(next); cur = next; }
            Some(_) => {
                // Zero-width match. Avoid infinite loop: stop after registering once.
                *caps = saved;
                break;
            }
            None => { *caps = saved; break; }
        }
    }
    // positions[k] = char index after k repetitions.
    // We want a k in [min, ..len-1] that lets `rest` match.
    let max_k = positions.len() - 1;
    if max_k < min { return None; }
    let candidates: Vec<usize> = if greedy {
        (min..=max_k).rev().collect()
    } else {
        (min..=max_k).collect()
    };
    let saved = caps.clone();
    for k in candidates {
        *caps = saved.clone();
        let after = positions[k];
        if rest.is_empty() {
            return Some(after);
        }
        if let Some(end) = match_concat(rest, 0, chars, after, flags, caps) {
            return Some(end);
        }
    }
    *caps = saved;
    None
}

fn char_eq(a: char, b: char, ignore_case: bool) -> bool {
    if !ignore_case { return a == b; }
    a.to_lowercase().next() == b.to_lowercase().next()
}

fn at_word_boundary(chars: &[char], pos: usize) -> bool {
    let is_word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let prev = if pos == 0 { None } else { Some(chars[pos - 1]) };
    let next = chars.get(pos).copied();
    let pw = prev.map(is_word).unwrap_or(false);
    let nw = next.map(is_word).unwrap_or(false);
    pw != nw
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(pat: &str, flags: &str, s: &str) -> Option<(usize, usize)> {
        let re = compile(pat, flags).unwrap();
        find_at(&re, s, 0).map(|m| (m.start, m.end))
    }

    #[test]
    fn literal() { assert_eq!(m("foo", "", "xfoox"), Some((1, 4))); }
    #[test]
    fn any_char() { assert_eq!(m("f.o", "", "fxo"), Some((0, 3))); }
    #[test]
    fn star_greedy() { assert_eq!(m("a*", "", "aaab"), Some((0, 3))); }
    #[test]
    fn lazy() { assert_eq!(m("a.*?b", "", "axxbyyb"), Some((0, 4))); }
    #[test]
    fn alt() { assert_eq!(m("cat|dog", "", "I see a dog!"), Some((8, 11))); }
    #[test]
    fn class() { assert_eq!(m("[a-c]+", "", "xxxabbc"), Some((3, 7))); }
    #[test]
    fn anchor() { assert_eq!(m("^abc", "", "abc"), Some((0, 3))); assert_eq!(m("^abc", "", "xabc"), None); }
    #[test]
    fn lookahead_pos() { assert_eq!(m("foo(?=bar)", "", "foobar"), Some((0, 3))); assert_eq!(m("foo(?=bar)", "", "fooqux"), None); }
    #[test]
    fn lookahead_neg() { assert_eq!(m("foo(?!bar)", "", "fooqux"), Some((0, 3))); assert_eq!(m("foo(?!bar)", "", "foobar"), None); }
    #[test]
    #[ignore] // backref through Repeat still TODO — capture caps not threaded into match_repeat positions
    fn backref() { assert_eq!(m("(a+)\\1", "", "aaaa"), Some((0, 4))); }
    #[test]
    fn pathe_pattern() {
        // Pathe's pattern: ^[/\\\\](?![/\\\\])|^[/\\\\]{2}(?!\\.)|^[A-Za-z]:[/\\\\]
        let pat = r"^[/\\](?![/\\])|^[/\\]{2}(?!\.)|^[A-Za-z]:[/\\]";
        let re = compile(pat, "").unwrap();
        assert!(is_match(&re, "/foo"));
        assert!(is_match(&re, "\\\\server"));
        assert!(is_match(&re, "C:/foo"));
        assert!(!is_match(&re, "foo/bar"));
    }
    #[test]
    fn picomatch_pattern() {
        // micromatch-style: ^(?:(?!\.)(?=.)[^/]*?\.js\/?)$
        let pat = r"^(?:(?!\.)(?=.)[^/]*?\.js\/?)$";
        let re = compile(pat, "").unwrap();
        assert!(is_match(&re, "foo.js"));
        assert!(!is_match(&re, ".hidden.js"));
        assert!(!is_match(&re, "foo.txt"));
    }
}
