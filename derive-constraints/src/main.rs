//! derive-constraints — extract behavioral constraints from a test corpus
//! by AST-walking each test file. Phase 1 (scan) of the pipeline articulated
//! in docs/derivation-inversion-on-bun-tests.md. The cluster / invert /
//! predict phases are downstream layers that consume this scan's JSON.

mod cluster;
mod extract;
mod invert;
mod scan;
mod seams;

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
    /// Detect architectural seams over a cluster catalog by extracting
    /// per-property architectural-hedging signal vectors and grouping
    /// by signal-vector agreement. Operationalizes RESOLVE Doc 705
    /// (Pin-Art for intra-architectural seam detection) per the design
    /// at docs/seam-detection-design.md.
    Seams {
        /// Path to a cluster JSON produced by `derive-constraints cluster`.
        cluster: PathBuf,
        /// Optional output file; defaults to stdout.
        #[arg(short, long)]
        out: Option<PathBuf>,
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
        Cmd::Seams {
            cluster: cluster_path,
            out,
            summary,
        } => {
            let cluster_report: cluster::ClusterReport = read_json(&cluster_path)
                .with_context(|| format!("loading cluster from {}", cluster_path.display()))?;
            let mut report = seams::detect_seams(&cluster_report)?;
            report.cluster_source = Some(cluster_path.to_string_lossy().into_owned());
            write_json(&out, &report)?;
            if summary {
                print_seams_summary(&report);
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

fn print_seams_summary(report: &seams::SeamsReport) {
    let s = &report.stats;
    eprintln!("\n=== derive-constraints seams ===");
    if let Some(ref src) = report.cluster_source {
        eprintln!("cluster source:        {}", src);
    }
    eprintln!("properties in:         {}", s.properties_in);
    eprintln!("distinct signal vecs:  {}", s.distinct_signal_vectors);
    eprintln!("clusters emitted:      {}", s.clusters_emitted);
    eprintln!("cross-namespace seams: {}", s.cross_namespace_seam_count);
    eprintln!("\ntop 20 clusters by total cardinality:");
    for c in report.signal_clusters.iter().take(20) {
        let surfaces = if c.surfaces_touched.len() <= 5 {
            c.surfaces_touched.join(", ")
        } else {
            format!(
                "{} (and {} more)",
                c.surfaces_touched[..5].join(", "),
                c.surfaces_touched.len() - 5
            )
        };
        let name = name_for_signal_terse(&c.signal_vector);
        eprintln!(
            "  {}  card={:>5} props={:>4} cs={:>3}  surfaces=[{}]  signal={}",
            c.id, c.cardinality_total, c.property_count, c.construction_style_count, surfaces, name
        );
    }
    eprintln!("\ntop 20 cross-namespace seams (by cardinality):");
    let mut cs = report.cross_namespace_seams.clone();
    cs.sort_by(|a, b| b.cardinality_total.cmp(&a.cardinality_total));
    for s in cs.iter().take(20) {
        let surfaces = if s.surfaces.len() <= 6 {
            s.surfaces.join(", ")
        } else {
            format!("{} (+{})", s.surfaces[..6].join(", "), s.surfaces.len() - 6)
        };
        eprintln!(
            "  {}  card={:>5}  action={}  surfaces=[{}]  seam={}",
            s.cluster_id, s.cardinality_total, s.action, surfaces, s.seam_name
        );
    }
}

fn name_for_signal_terse(v: &seams::SignalVector) -> String {
    let mut parts = Vec::new();
    if v.cfg {
        parts.push("cfg".to_string());
    }
    match v.sync_async {
        seams::SyncAsync::Sync => parts.push("sync".into()),
        seams::SyncAsync::Async => parts.push("async".into()),
        seams::SyncAsync::Mixed => parts.push("sync+async".into()),
        seams::SyncAsync::Neither => {}
    }
    match v.throw_return {
        seams::ThrowReturn::Throw => parts.push("throw".into()),
        seams::ThrowReturn::ReturnError => parts.push("ret-err".into()),
        seams::ThrowReturn::Mixed => parts.push("throw+ret".into()),
        seams::ThrowReturn::Neither => {}
    }
    if v.native {
        parts.push("ffi".into());
    }
    if v.construct_handle {
        parts.push("ctor".into());
    }
    if v.weak_ref {
        parts.push("weak".into());
    }
    match v.error_shape {
        seams::ErrorShape::Result => parts.push("res".into()),
        seams::ErrorShape::OkErrorsArray => parts.push("ok-err".into()),
        seams::ErrorShape::SuccessErrors => parts.push("succ-err".into()),
        seams::ErrorShape::PlainThrow => parts.push("p-throw".into()),
        seams::ErrorShape::Mixed => parts.push("mix-err".into()),
        seams::ErrorShape::None => {}
    }
    if v.allocator_aware {
        parts.push("alloc".into());
    }
    if v.threaded {
        parts.push("thr".into());
    }
    if let Some(ref p) = v.path_top {
        parts.push(format!("@{}", p));
    }
    if parts.is_empty() {
        "slack".into()
    } else {
        parts.join("|")
    }
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
