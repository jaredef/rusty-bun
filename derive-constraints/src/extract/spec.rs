//! Spec-source extractor.
//!
//! Reads a manually-curated spec-extract markdown file and emits a `TestFile`
//! shaped like a test-derived extraction so downstream phases (cluster,
//! invert, seams, couple) can ingest it uniformly.
//!
//! Format:
//!
//! ```markdown
//! # <Surface name> — <spec authority>
//!
//! [surface] TextEncoder
//! [spec] https://encoding.spec.whatwg.org/#textencoder
//!
//! ## <invariant group name>
//! - <invariant clause 1>
//! - <invariant clause 2>
//!
//! ## <another invariant group>
//! - <clause>
//! ```
//!
//! Each `##` heading becomes a `TestCase`; each `- ` bullet becomes a
//! `ConstraintClause` with `kind: SpecInvariant`. The clause's subject is
//! the leading identifier path (e.g. `TextEncoder.prototype.encode`) before
//! the first space or non-identifier-non-dot character — matching the
//! cluster-phase canonicalization rules.
//!
//! Spec extracts live anywhere in the scan tree as `*.spec.md` files.

use crate::extract::{ConstraintClause, ConstraintKind, Language, TestCase, TestFile, TestKind};
use anyhow::Result;

pub fn extract(rel_path: &str, src: &str) -> Result<TestFile> {
    let mut tests: Vec<TestCase> = Vec::new();
    let mut current: Option<TestCase> = None;
    let mut surface_hint: Option<String> = None;

    for (lineno, line) in src.lines().enumerate() {
        let line_num = (lineno + 1) as u32;
        let trimmed = line.trim();

        // Capture `[surface] X` / `[spec] X` annotations as document-level
        // metadata. `[surface]` becomes the default subject for clauses
        // whose own leading identifier is non-public (lowercase, generic).
        if let Some(rest) = trimmed.strip_prefix("[surface]") {
            surface_hint = Some(rest.trim().to_string());
            continue;
        }
        if trimmed.starts_with("[spec]") || trimmed.starts_with("[spec_url]") {
            // Recorded for human readers; not used downstream yet.
            continue;
        }

        // `##` opens a new invariant-group test case.
        if let Some(name) = trimmed.strip_prefix("## ") {
            if let Some(prev) = current.take() {
                tests.push(prev);
            }
            current = Some(TestCase {
                name: name.trim().to_string(),
                kind: TestKind::Test,
                line_start: line_num,
                line_end: line_num,
                constraints: Vec::new(),
                skip: false,
                todo: false,
                failing: false,
            });
            continue;
        }

        // `# ` is the document title; ignore.
        if trimmed.starts_with("# ") {
            continue;
        }

        // `- ` clause bullet inside a current test case.
        if let Some(body) = trimmed.strip_prefix("- ") {
            let body = body.trim();
            if body.is_empty() {
                continue;
            }
            if let Some(test) = current.as_mut() {
                let subject = subject_for_clause(body, surface_hint.as_deref());
                test.constraints.push(ConstraintClause {
                    line: line_num,
                    raw: body.to_string(),
                    kind: ConstraintKind::SpecInvariant,
                    subject,
                });
                test.line_end = line_num;
            }
        }
    }

    if let Some(last) = current {
        tests.push(last);
    }

    Ok(TestFile {
        path: rel_path.to_string(),
        language: Language::Spec,
        loc: src.lines().count() as u32,
        tests,
        parse_failure: None,
    })
}

/// Heuristic subject extraction for a spec-clause bullet.
///
/// Strategy: take the longest leading identifier path before the first
/// non-identifier-non-dot character. If that path doesn't start with an
/// uppercase letter or known public-API head, fall back to the
/// document-level `surface_hint`. This mirrors the cluster-phase
/// canonicalization but lets prose like "the encoder produces..." still
/// attribute to the document's subject.
fn subject_for_clause(text: &str, surface_hint: Option<&str>) -> Option<String> {
    let leading = take_identifier_path(text);
    if leading.is_empty() {
        return surface_hint.map(String::from);
    }
    let head = leading.split('.').next().unwrap_or("");
    let starts_uppercase = head
        .chars()
        .next()
        .map(|c| c.is_ascii_uppercase())
        .unwrap_or(false);
    if starts_uppercase || head.eq_ignore_ascii_case(surface_hint.unwrap_or("")) {
        Some(leading.to_string())
    } else {
        surface_hint.map(String::from)
    }
}

fn take_identifier_path(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut last = 0;
    let mut state = 0u8;
    while i < bytes.len() {
        let c = bytes[i];
        match state {
            0 | 2 => {
                if matches!(c, b'_' | b'$' | b'A'..=b'Z' | b'a'..=b'z') {
                    state = 1;
                    last = i + 1;
                    i += 1;
                } else {
                    break;
                }
            }
            1 => {
                if matches!(c, b'_' | b'$' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z') {
                    last = i + 1;
                    i += 1;
                } else if c == b'.' {
                    state = 2;
                    i += 1;
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    &s[..last]
}
