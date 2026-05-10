//! welch — Welch-bound packing diagnostic for AI-assisted cross-language code
//! translation. See README.md and the RESOLVE corpus's Doc 702 for the
//! apparatus this tool operationalizes.

mod compare;
mod metrics;
mod scan;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "welch", version, about = "Welch-bound diagnostic for Rust codebases.")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Walk a directory of Rust source and emit a per-file metrics JSON.
    Scan {
        /// Root directory to scan.
        path: PathBuf,
        /// Optional output file. When omitted, JSON is written to stdout.
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// Summarize a baseline scan into a per-metric distribution JSON.
    Baseline {
        /// Path to a scan JSON produced by `welch scan`.
        scan: PathBuf,
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// Compare a target scan against a baseline summary, emitting an anomaly
    /// report with per-file z-scores.
    Compare {
        /// Baseline summary JSON (produced by `welch baseline`).
        baseline: PathBuf,
        /// Target scan JSON (produced by `welch scan`).
        target: PathBuf,
        /// z-score threshold for flagging a file as anomalous.
        #[arg(short = 'z', long, default_value_t = 2.0)]
        threshold_z: f64,
        /// Output anomaly report; defaults to stdout.
        #[arg(short, long)]
        out: Option<PathBuf>,
        /// Print human-readable summary in addition to JSON output.
        #[arg(long)]
        summary: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Scan { path, out } => {
            let report = scan::scan_dir(&path)
                .with_context(|| format!("scanning {}", path.display()))?;
            write_json(&out, &report)?;
            eprintln!(
                "scanned {} files, {} parse failures",
                report.files.len(),
                report.parse_failures
            );
        }
        Cmd::Baseline { scan, out } => {
            let report: scan::ScanReport = read_json(&scan)?;
            let summary = compare::summarize_baseline(&report);
            write_json(&out, &summary)?;
            eprintln!(
                "baseline summary: {} files, {} LOC, {} metrics",
                summary.n_files,
                summary.total_loc,
                summary.distributions.len()
            );
        }
        Cmd::Compare {
            baseline,
            target,
            threshold_z,
            out,
            summary,
        } => {
            let baseline: compare::BaselineSummary = read_json(&baseline)?;
            let target: scan::ScanReport = read_json(&target)?;
            let report = compare::compare(&baseline, &target, threshold_z);
            write_json(&out, &report)?;
            if summary {
                print_summary(&report);
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

fn print_summary(report: &compare::AnomalyReport) {
    eprintln!("\n=== welch anomaly report ===");
    eprintln!("baseline:   {}", report.baseline_source);
    eprintln!("target:     {}", report.target_source);
    eprintln!("threshold:  z >= {:.2}", report.threshold_z);
    eprintln!("\naggregate (corpus-wide density per kLOC vs baseline distribution):");
    for m in &report.aggregate {
        eprintln!(
            "  {:<16} target={:>9.3}  baseline μ={:>9.3} σ={:>9.3}  z={:>+7.2}",
            m.metric, m.target_value, m.baseline_mean, m.baseline_std, m.z
        );
    }
    eprintln!(
        "\n{} anomalous files (z >= {:.2} on at least one metric):",
        report.anomalous_files.len(),
        report.threshold_z
    );
    for f in report.anomalous_files.iter().take(20) {
        let max_z = f
            .flagged_metrics
            .iter()
            .map(|m| m.z)
            .fold(f64::MIN, f64::max);
        let metric_summary: Vec<String> = f
            .flagged_metrics
            .iter()
            .map(|m| format!("{}(z={:+.1})", m.metric, m.z))
            .collect();
        eprintln!(
            "  z={:>+6.1}  loc={:>5}  {}  [{}]",
            max_z,
            f.loc,
            f.path,
            metric_summary.join(", ")
        );
    }
    if report.anomalous_files.len() > 20 {
        eprintln!("  ... ({} more)", report.anomalous_files.len() - 20);
    }
}
