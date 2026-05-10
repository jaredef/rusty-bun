//! Walk a directory tree, parse each Rust source file, and accumulate per-file
//! metrics. Parallelism via rayon over the discovered file set.

use crate::metrics::{FileMetrics, MetricsVisitor};
use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanReport {
    pub root: String,
    pub files: Vec<FileMetrics>,
    /// Files we found but failed to parse. Contributes LOC but no AST metrics.
    pub parse_failures: u64,
}

impl ScanReport {
    /// Whole-corpus aggregate counts (sum across files).
    pub fn aggregate(&self) -> AggregateMetrics {
        let mut agg = AggregateMetrics::default();
        for f in &self.files {
            agg.files += 1;
            agg.loc += f.loc;
            agg.fns += f.fns;
            agg.unsafe_blocks += f.unsafe_blocks;
            agg.unsafe_loc += f.unsafe_loc;
            agg.unsafe_fns += f.unsafe_fns;
            agg.raw_pointers += f.raw_pointers;
            agg.transmutes += f.transmutes;
            agg.extern_blocks += f.extern_blocks;
        }
        agg
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AggregateMetrics {
    pub files: u64,
    pub loc: u64,
    pub fns: u64,
    pub unsafe_blocks: u64,
    pub unsafe_loc: u64,
    pub unsafe_fns: u64,
    pub raw_pointers: u64,
    pub transmutes: u64,
    pub extern_blocks: u64,
}

pub fn scan_dir(root: &Path) -> Result<ScanReport> {
    let root = root
        .canonicalize()
        .with_context(|| format!("canonicalize {}", root.display()))?;

    let files: Vec<PathBuf> = WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Directory-level pruning. Always skip .git and node_modules.
            // For directories literally named `target` and `vendor`, only
            // skip when they look like Cargo build artifacts — i.e., a
            // Cargo.toml exists in their parent. Otherwise users with paths
            // containing those names (e.g. /tmp/welch-test/target) get
            // unexpectedly empty scans.
            let p = e.path();
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            match name {
                ".git" | "node_modules" => false,
                "target" | "vendor" => {
                    if !e.file_type().is_dir() {
                        return true;
                    }
                    p.parent()
                        .map(|parent| !parent.join("Cargo.toml").exists())
                        .unwrap_or(true)
                }
                _ => true,
            }
        })
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            p.is_file() && p.extension().and_then(|x| x.to_str()) == Some("rs")
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    let parse_failures = std::sync::atomic::AtomicU64::new(0);

    let metrics: Vec<FileMetrics> = files
        .par_iter()
        .map(|path| {
            let rel = path
                .strip_prefix(&root)
                .unwrap_or(path)
                .to_string_lossy()
                .into_owned();
            scan_one(path, &rel, &parse_failures)
        })
        .collect();

    Ok(ScanReport {
        root: root.to_string_lossy().into_owned(),
        files: metrics,
        parse_failures: parse_failures.load(std::sync::atomic::Ordering::Relaxed),
    })
}

fn scan_one(
    path: &Path,
    rel: &str,
    parse_failures: &std::sync::atomic::AtomicU64,
) -> FileMetrics {
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => {
            return FileMetrics::new(rel.to_string(), 0, 0);
        }
    };

    match syn::parse_file(&src) {
        Ok(file) => {
            let mut v = MetricsVisitor::new(rel.to_string(), Some(&src));
            v.visit_file(&file);
            v.metrics
        }
        Err(_) => {
            parse_failures.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            // Record LOC even on parse failure so the corpus size is honest.
            let mut m = FileMetrics::new(rel.to_string(), src.len() as u64, src.lines().count() as u64);
            m.parsed = false;
            m
        }
    }
}
