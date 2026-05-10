//! Compare a target scan against a baseline scan. Computes per-metric
//! distributions on the baseline (mean, std, quantiles) and z-scores the
//! target's per-file metrics against those distributions. Flags anomalous
//! regions whose z-score exceeds a configurable threshold.

use crate::metrics::FileMetrics;
use crate::scan::ScanReport;
use serde::{Deserialize, Serialize};

/// Per-metric distribution computed from a baseline scan, expressed as the
/// distribution of *per-LOC density* values across baseline files. Density
/// rather than raw count is the comparison object because file size varies
/// heavily and densities are scale-invariant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDistribution {
    pub name: String,
    pub n: u64,
    pub mean: f64,
    pub std: f64,
    pub p50: f64,
    pub p90: f64,
    pub p99: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineSummary {
    pub source: String,
    pub n_files: u64,
    pub total_loc: u64,
    pub distributions: Vec<MetricDistribution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    pub baseline_source: String,
    pub target_source: String,
    pub threshold_z: f64,
    /// Aggregate-level z-scores per metric, comparing the target's
    /// corpus-wide density against the baseline distribution.
    pub aggregate: Vec<MetricZScore>,
    /// Per-file outliers in the target whose z-score on at least one metric
    /// exceeds `threshold_z`.
    pub anomalous_files: Vec<AnomalousFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricZScore {
    pub metric: String,
    pub target_value: f64,
    pub baseline_mean: f64,
    pub baseline_std: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalousFile {
    pub path: String,
    pub loc: u64,
    pub flagged_metrics: Vec<MetricZScore>,
}

const METRICS: &[(&str, fn(&FileMetrics) -> u64)] = &[
    ("unsafe_blocks", |m| m.unsafe_blocks),
    ("unsafe_loc", |m| m.unsafe_loc),
    ("unsafe_fns", |m| m.unsafe_fns),
    ("raw_pointers", |m| m.raw_pointers),
    ("transmutes", |m| m.transmutes),
    ("extern_blocks", |m| m.extern_blocks),
];

/// Compute density (count per kLOC) for a metric on a single file. Files with
/// 0 LOC contribute nothing to the distribution.
fn density(numerator: u64, loc: u64) -> Option<f64> {
    if loc == 0 {
        None
    } else {
        Some((numerator as f64) * 1000.0 / (loc as f64))
    }
}

pub fn summarize_baseline(report: &ScanReport) -> BaselineSummary {
    let total_loc: u64 = report.files.iter().map(|f| f.loc).sum();
    let mut distributions = Vec::new();
    for (name, accessor) in METRICS {
        let mut samples: Vec<f64> = report
            .files
            .iter()
            .filter_map(|f| density(accessor(f), f.loc))
            .collect();
        if samples.is_empty() {
            continue;
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = samples.len() as u64;
        let mean = samples.iter().sum::<f64>() / (n as f64);
        let var = samples
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / (n as f64).max(1.0);
        let std = var.sqrt();
        distributions.push(MetricDistribution {
            name: name.to_string(),
            n,
            mean,
            std,
            p50: quantile(&samples, 0.50),
            p90: quantile(&samples, 0.90),
            p99: quantile(&samples, 0.99),
        });
    }
    BaselineSummary {
        source: report.root.clone(),
        n_files: report.files.len() as u64,
        total_loc,
        distributions,
    }
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * q).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

pub fn compare(
    baseline: &BaselineSummary,
    target: &ScanReport,
    threshold_z: f64,
) -> AnomalyReport {
    let agg = target.aggregate();
    let target_total_loc = agg.loc;

    // Aggregate-level z-scores: target's corpus-wide density vs baseline
    // distribution of per-file densities.
    let aggregate: Vec<MetricZScore> = METRICS
        .iter()
        .filter_map(|(name, accessor)| {
            let dist = baseline.distributions.iter().find(|d| d.name == *name)?;
            let target_count: u64 = target.files.iter().map(|f| accessor(f)).sum();
            let target_density = density(target_count, target_total_loc).unwrap_or(0.0);
            Some(MetricZScore {
                metric: name.to_string(),
                target_value: target_density,
                baseline_mean: dist.mean,
                baseline_std: dist.std,
                z: z_score(target_density, dist.mean, dist.std),
            })
        })
        .collect();

    // Per-file flagging: any file whose z-score on at least one metric exceeds
    // threshold_z.
    let mut anomalous_files: Vec<AnomalousFile> = target
        .files
        .iter()
        .filter_map(|f| {
            let mut flagged = Vec::new();
            for (name, accessor) in METRICS {
                let dist = match baseline.distributions.iter().find(|d| d.name == *name) {
                    Some(d) => d,
                    None => continue,
                };
                let d = match density(accessor(f), f.loc) {
                    Some(v) => v,
                    None => continue,
                };
                let z = z_score(d, dist.mean, dist.std);
                if z >= threshold_z {
                    flagged.push(MetricZScore {
                        metric: name.to_string(),
                        target_value: d,
                        baseline_mean: dist.mean,
                        baseline_std: dist.std,
                        z,
                    });
                }
            }
            if flagged.is_empty() {
                None
            } else {
                flagged.sort_by(|a, b| b.z.partial_cmp(&a.z).unwrap_or(std::cmp::Ordering::Equal));
                Some(AnomalousFile {
                    path: f.path.clone(),
                    loc: f.loc,
                    flagged_metrics: flagged,
                })
            }
        })
        .collect();

    // Sort anomalous files by max z-score descending so the worst offenders
    // surface first in the report.
    anomalous_files.sort_by(|a, b| {
        let max_a = a.flagged_metrics.iter().map(|m| m.z).fold(f64::MIN, f64::max);
        let max_b = b.flagged_metrics.iter().map(|m| m.z).fold(f64::MIN, f64::max);
        max_b.partial_cmp(&max_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    AnomalyReport {
        baseline_source: baseline.source.clone(),
        target_source: target.root.clone(),
        threshold_z,
        aggregate,
        anomalous_files,
    }
}

fn z_score(value: f64, mean: f64, std: f64) -> f64 {
    // Guard against zero-variance baseline distributions. When std is 0, any
    // deviation is "infinitely anomalous"; we clamp to a large finite value
    // so the report remains meaningful.
    if std == 0.0 {
        if value == mean {
            0.0
        } else if value > mean {
            f64::INFINITY
        } else {
            f64::NEG_INFINITY
        }
    } else {
        (value - mean) / std
    }
}
