//! Walk a directory tree, identify test files by extension and content,
//! dispatch each to the appropriate language extractor, and collect the
//! per-file `TestFile` reports.

use crate::extract::{rust, ts_js, zig, Language, TestFile};
use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanReport {
    pub root: String,
    pub files: Vec<TestFile>,
    pub stats: ScanStats,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ScanStats {
    pub files_scanned: u64,
    pub parse_failures: u64,
    pub tests_total: u64,
    pub constraints_total: u64,
    pub by_language: ByLanguage,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ByLanguage {
    pub rust: LangStats,
    pub typescript: LangStats,
    pub javascript: LangStats,
    pub zig: LangStats,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LangStats {
    pub files: u64,
    pub tests: u64,
    pub constraints: u64,
}

pub fn scan_dir(root: &Path) -> Result<ScanReport> {
    let root = root.canonicalize()?;

    // Walk the tree, skipping common non-source directories.
    let candidates: Vec<PathBuf> = WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e
                .path()
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            !matches!(name, ".git" | "node_modules")
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| classify_file(e.path()).is_some())
        .map(|e| e.path().to_path_buf())
        .collect();

    let files: Vec<TestFile> = candidates
        .par_iter()
        .map(|p| extract_one(p, &root))
        .collect();

    let mut stats = ScanStats::default();
    for f in &files {
        stats.files_scanned += 1;
        if f.parse_failure.is_some() {
            stats.parse_failures += 1;
        }
        stats.tests_total += f.tests.len() as u64;
        let cs: u64 = f.tests.iter().map(|t| t.constraints.len() as u64).sum();
        stats.constraints_total += cs;
        let bucket = match f.language {
            Language::Rust => &mut stats.by_language.rust,
            Language::TypeScript => &mut stats.by_language.typescript,
            Language::JavaScript => &mut stats.by_language.javascript,
            Language::Zig => &mut stats.by_language.zig,
        };
        bucket.files += 1;
        bucket.tests += f.tests.len() as u64;
        bucket.constraints += cs;
    }

    Ok(ScanReport {
        root: root.to_string_lossy().into_owned(),
        files,
        stats,
    })
}

/// Classifies a file path as a test source by extension. For ambiguous
/// extensions (.rs, .zig), the extractor itself decides whether the file
/// contains tests; we include all such files and let the extractor return
/// an empty test list when there are none.
fn classify_file(path: &Path) -> Option<Language> {
    let name = path.file_name()?.to_str()?;
    // Bun / Jest / Vitest convention (`.test.ts`, `.spec.ts`).
    if name.ends_with(".test.ts") || name.ends_with(".test.tsx") || name.ends_with(".spec.ts") {
        return Some(Language::TypeScript);
    }
    if name.ends_with(".test.js") || name.ends_with(".test.jsx") || name.ends_with(".spec.js")
        || name.ends_with(".test.mjs") || name.ends_with(".test.cjs")
    {
        return Some(Language::JavaScript);
    }
    // Deno / Go-style `_test` convention (`*_test.ts`, `*_test.js`).
    if name.ends_with("_test.ts") || name.ends_with("_test.tsx") || name.ends_with("_test.mjs") {
        return Some(Language::TypeScript);
    }
    if name.ends_with("_test.js") || name.ends_with("_test.jsx") || name.ends_with("_test.cjs") {
        return Some(Language::JavaScript);
    }
    let ext = path.extension()?.to_str()?;
    match ext {
        "zig" => Some(Language::Zig),
        "rs" => Some(Language::Rust),
        _ => None,
    }
}

fn extract_one(path: &Path, root: &Path) -> TestFile {
    let rel = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned();
    let language = classify_file(path).unwrap_or(Language::Rust);
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            return TestFile {
                path: rel,
                language,
                loc: 0,
                tests: Vec::new(),
                parse_failure: Some(format!("read error: {}", e)),
            };
        }
    };
    let result = match language {
        Language::Rust => rust::extract(&rel, &src),
        Language::Zig => zig::extract(&rel, &src),
        Language::TypeScript | Language::JavaScript => ts_js::extract(&rel, &src, language),
    };
    match result {
        Ok(f) => f,
        Err(e) => TestFile {
            path: rel,
            language,
            loc: src.lines().count() as u32,
            tests: Vec::new(),
            parse_failure: Some(format!("extractor error: {}", e)),
        },
    }
}
