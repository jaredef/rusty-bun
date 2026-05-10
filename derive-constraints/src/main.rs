//! derive-constraints — extract behavioral constraints from a test corpus
//! by AST-walking each test file. Phase 1 (scan) of the pipeline articulated
//! in docs/derivation-inversion-on-bun-tests.md. The cluster / invert /
//! predict phases are downstream layers that consume this scan's JSON.

mod cluster;
mod extract;
mod invert;
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
    /// Cluster a scan's constraints into a property catalog: canonicalize
    /// each constraint by (subject, verb-class), select a minimal antichain
    /// per property, classify properties as construction-style or behavioral.
    Cluster {
        /// Path to a scan JSON produced by `derive-constraints scan`.
        scan: PathBuf,
        /// Optional output file; defaults to stdout.
        #[arg(short, long)]
        out: Option<PathBuf>,
        /// Print human-readable summary in addition to JSON output.
        #[arg(long)]
        summary: bool,
    },
    /// Emit `.constraints.md` documents in rederive grammar, one per
    /// architectural surface plus a top-level index. The output is
    /// consumable by rederive's parse stage; see docs/invert-phase-design.md.
    Invert {
        /// Path to a cluster JSON produced by `derive-constraints cluster`.
        cluster: PathBuf,
        /// Output directory for the .constraints.md documents.
        #[arg(short, long)]
        out: PathBuf,
        /// Print human-readable summary to stderr.
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
        Cmd::Cluster { scan, out, summary } => {
            let scan_report: scan::ScanReport = read_json(&scan)
                .with_context(|| format!("loading scan from {}", scan.display()))?;
            let mut report = cluster::cluster(&scan_report)?;
            report.source_path = Some(scan.to_string_lossy().into_owned());
            write_json(&out, &report)?;
            if summary {
                print_cluster_summary(&report);
            }
        }
        Cmd::Invert {
            cluster: cluster_path,
            out,
            summary,
        } => {
            let cluster_report: cluster::ClusterReport = read_json(&cluster_path)
                .with_context(|| format!("loading cluster from {}", cluster_path.display()))?;
            let report = invert::invert(&cluster_report, &out)?;
            if summary {
                print_invert_summary(&report);
            }
        }
    }
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Result<T> {
    let data = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let value = serde_json::from_slice(&data)
        .with_context(|| format!("parse json {}", path.display()))?;
    Ok(value)
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

fn print_invert_summary(report: &invert::InvertReport) {
    eprintln!("\n=== derive-constraints invert ===");
    eprintln!("output dir:           {}", report.output_dir.display());
    eprintln!("surfaces emitted:     {}", report.surfaces_emitted);
    eprintln!("constraints emitted:  {}", report.constraints_emitted);
    eprintln!(
        "properties skipped:   {} (sub-floor cardinality or noise subjects)",
        report.properties_skipped
    );
}

fn print_cluster_summary(report: &cluster::ClusterReport) {
    let s = &report.stats;
    eprintln!("\n=== derive-constraints cluster ===");
    if let Some(ref src) = report.source_path {
        eprintln!("scan source:        {}", src);
    }
    eprintln!("constraints in:     {}", s.constraints_in);
    eprintln!("properties out:     {}", s.properties_out);
    eprintln!("antichain size:     {}", s.antichain_size);
    eprintln!(
        "reduction ratio:    {:.4} (antichain / constraints_in)",
        s.reduction_ratio
    );
    eprintln!(
        "construction-style: {} ({:.1}% of properties)",
        s.construction_style_count,
        if s.properties_out == 0 {
            0.0
        } else {
            100.0 * s.construction_style_count as f64 / s.properties_out as f64
        }
    );
    eprintln!(
        "behavioral:         {} ({:.1}% of properties)",
        s.behavioral_count,
        if s.properties_out == 0 {
            0.0
        } else {
            100.0 * s.behavioral_count as f64 / s.properties_out as f64
        }
    );
    eprintln!("\nproperty cardinality buckets:");
    for (bucket, count) in &s.property_cardinality_buckets {
        eprintln!("  {:>8}  {} properties", bucket, count);
    }
    eprintln!("\nverb-class distribution:");
    for (verb, count) in &s.by_verb_class {
        eprintln!("  {:<18} {} properties", verb, count);
    }
    eprintln!("\ntop 20 construction-style properties (by cardinality):");
    let cs: Vec<&cluster::Property> = report
        .properties
        .iter()
        .filter(|p| p.construction_style)
        .take(20)
        .collect();
    for p in cs {
        eprintln!(
            "  n={:>5}  {:<16}  {}",
            p.constraints_in,
            format!("{:?}", p.verb_class).to_lowercase(),
            p.subject
        );
    }
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
