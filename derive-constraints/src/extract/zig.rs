//! Zig test-block extractor. The shape `test "name" { ... }` is regular
//! enough that hand-rolled extraction beats pulling in a full Zig parser
//! for the MVP. Brace tracking handles nested scopes; constraint-clause
//! detection looks for `try testing.expect*(...)` and `try expect*(...)`
//! call patterns.

use super::{ConstraintClause, ConstraintKind, Language, TestCase, TestFile, TestKind};
use anyhow::Result;
use regex::Regex;

pub fn extract(path: &str, src: &str) -> Result<TestFile> {
    let test_re = Regex::new(r#"^test\s+"([^"]*)"\s*\{"#).unwrap();
    let testing_re =
        Regex::new(r"\btry\s+(?:std\.)?testing\.(expect[A-Za-z]*)\s*\(").unwrap();
    // Bun's tests sometimes import testing aliases — `bun.testing.expect*` etc.
    let alias_re = Regex::new(r"\btry\s+\w+\.(expect[A-Za-z]*)\s*\(").unwrap();

    let mut tests = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some(caps) = test_re.captures(line) {
            let name = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            let line_start = (i + 1) as u32;
            let (body_end, body) = collect_braces(&lines, i);
            let mut constraints = Vec::new();
            for (off, body_line) in body.iter().enumerate() {
                if let Some(c) = capture_clause(&testing_re, body_line, line_start + off as u32) {
                    constraints.push(c);
                } else if let Some(c) = capture_clause(&alias_re, body_line, line_start + off as u32) {
                    constraints.push(c);
                }
            }
            tests.push(TestCase {
                name,
                kind: TestKind::ZigTest,
                line_start,
                line_end: (body_end + 1) as u32,
                constraints,
                skip: false,
                todo: false,
                failing: false,
            });
            i = body_end + 1;
        } else {
            i += 1;
        }
    }

    Ok(TestFile {
        path: path.to_string(),
        language: Language::Zig,
        loc: src.lines().count() as u32,
        tests,
        parse_failure: None,
    })
}

/// Returns (last_line_index_inclusive, lines_between_braces). Tracks brace
/// depth from the opening `{` on `lines[start]`; ignores braces inside
/// `"..."` and `// ...` and `\\\\` block comments.
fn collect_braces<'a>(lines: &[&'a str], start: usize) -> (usize, Vec<&'a str>) {
    let mut depth: i64 = 0;
    let mut out = Vec::new();
    let mut started = false;
    for (i, line) in lines.iter().enumerate().skip(start) {
        for ch in CharsIgnoringStrings::new(line) {
            if ch == '{' {
                depth += 1;
                started = true;
            } else if ch == '}' {
                depth -= 1;
            }
        }
        if i > start || started {
            out.push(*line);
        }
        if started && depth == 0 {
            return (i, out);
        }
    }
    (lines.len() - 1, out)
}

/// Iterator over a line's chars that skips characters inside string
/// literals (`"..."`, `'...'`) and line comments (`//...`). Handles
/// backslash escapes inside double-quoted strings.
struct CharsIgnoringStrings<'a> {
    chars: std::str::Chars<'a>,
    in_str: Option<char>,
    in_comment: bool,
    prev_was_backslash: bool,
}
impl<'a> CharsIgnoringStrings<'a> {
    fn new(s: &'a str) -> Self {
        CharsIgnoringStrings {
            chars: s.chars(),
            in_str: None,
            in_comment: false,
            prev_was_backslash: false,
        }
    }
}
impl<'a> Iterator for CharsIgnoringStrings<'a> {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        loop {
            let c = self.chars.next()?;
            if self.in_comment {
                continue;
            }
            if let Some(q) = self.in_str {
                if c == '\\' && !self.prev_was_backslash {
                    self.prev_was_backslash = true;
                    continue;
                }
                if c == q && !self.prev_was_backslash {
                    self.in_str = None;
                }
                self.prev_was_backslash = false;
                continue;
            }
            if c == '"' || c == '\'' {
                self.in_str = Some(c);
                continue;
            }
            if c == '/' {
                if let Some(&n) = self.chars.clone().next().as_ref() {
                    if n == '/' {
                        self.in_comment = true;
                        continue;
                    }
                }
            }
            return Some(c);
        }
    }
}

fn capture_clause(re: &Regex, line: &str, lineno: u32) -> Option<ConstraintClause> {
    let m = re.captures(line)?;
    let verb = m.get(1)?.as_str().to_string();
    let raw = collapse(line.trim());
    let subject = extract_call_arg(&raw);
    let subject = Some(format!("testing.{}/{}", verb, subject.unwrap_or_default()));
    let authority_tier = crate::extract::classify_authority_tier(
        subject.as_deref(),
        ConstraintKind::ZigTestingExpect,
    );
    Some(ConstraintClause {
        line: lineno,
        raw,
        kind: ConstraintKind::ZigTestingExpect,
        subject,
        authority_tier,
    })
}

fn extract_call_arg(raw: &str) -> Option<String> {
    // Pull the first comma-separated argument out of `try testing.foo(arg, ...)`.
    let lparen = raw.find('(')?;
    let after = &raw[lparen + 1..];
    let mut depth = 0i32;
    let mut end = 0;
    for (i, ch) in after.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    end = i;
                    break;
                }
                depth -= 1;
            }
            ',' if depth == 0 => {
                end = i;
                break;
            }
            _ => {}
        }
    }
    let arg = after.get(..end).unwrap_or("").trim();
    if arg.is_empty() {
        None
    } else {
        Some(arg.to_string())
    }
}

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
