//! derive-constraints — extract behavioral constraints from a test corpus
//! by AST-walking each test file. Phase 1 (scan) of the pipeline articulated
//! in docs/derivation-inversion-on-bun-tests.md. The cluster / invert /
//! predict phases are downstream layers that consume this scan's JSON.

mod extract;
mod scan;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "derive-constraints", version, about = "Extract test-corpus constraints.")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Walk a directory of test sources and emit per-file extraction JSON.
    Scan {
        /// Root directory to scan.
        path: PathBuf,
        /// Optional output file. When omitted, JSON is written to stdout.
        #[arg(short, long)]
        out: Option<PathBuf>,
        /// Print human-readable stats to stderr in addition to JSON output.
        #[arg(long)]
        summary: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Scan { path, out, summary } => {
            let report = scan::scan_dir(&path)
                .with_context(|| format!("scanning {}", path.display()))?;
            write_json(&out, &report)?;
            if summary {
                print_summary(&report);
            }
        }
    }
    Ok(())
}

fn write_json<T: serde::Serialize>(out: &Option<PathBuf>, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    match out {
        Some(p) => std::fs::write(p, bytes).with_context(|| format!("write {}", p.display()))?,
        None => {
            use std::io::Write;
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(&bytes)?;
            stdout.write_all(b"\n")?;
        }
    }
    Ok(())
}

fn print_summary(report: &scan::ScanReport) {
    let s = &report.stats;
    eprintln!("\n=== derive-constraints scan ===");
    eprintln!("root:               {}", report.root);
    eprintln!("files scanned:      {}", s.files_scanned);
    eprintln!("parse failures:     {}", s.parse_failures);
    eprintln!("tests extracted:    {}", s.tests_total);
    eprintln!("constraints found:  {}", s.constraints_total);
    eprintln!("by language:");
    let langs = [
        ("rust", &s.by_language.rust),
        ("typescript", &s.by_language.typescript),
        ("javascript", &s.by_language.javascript),
        ("zig", &s.by_language.zig),
    ];
    for (name, l) in langs {
        if l.files > 0 {
            eprintln!(
                "  {:<10}  files={:>5}  tests={:>6}  constraints={:>7}",
                name, l.files, l.tests, l.constraints
            );
        }
    }
}
