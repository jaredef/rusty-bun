//! Phase 5 — couple. Cross-references the seams output (architectural
//! signals over the test corpus) with welch's per-file anomaly report
//! (idiomaticity diagnostic over the implementation source) by matching
//! seams surface names against welch file-path substrings.
//!
//! The composition addresses the [Doc 705 §10.2 P2](https://jaredfoy.com/resolve/doc/705-pin-art-operationalized-for-intra-architectural-seam-detection)
//! "native byte-pool merge — NOT SURFACING" partial by surfacing
//! *implementation-internal* seams that don't appear at the test-corpus
//! probe layer but do appear at the implementation-source layer where
//! welch operates.
//!
//! Output: `CoupledReport` listing per-surface (seams_summary,
//! welch_summary, mismatch_indicator). A mismatch — high welch anomaly
//! but low seams architectural-signal density, or vice versa —
//! identifies a candidate implementation-internal seam.

use crate::seams::{SeamsReport, SignalCluster, SignalVector};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoupledReport {
    pub seams_source: Option<String>,
    pub welch_source: Option<String>,
    pub stats: CoupledStats,
    /// Per-surface couplings. A surface is included if it appears in the
    /// seams report at all; the welch-side may be None when no
    /// implementation-source files match the surface name.
    pub surfaces: Vec<SurfaceCoupling>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CoupledStats {
    pub surfaces_total: u64,
    pub surfaces_with_welch_match: u64,
    pub mismatch_candidates: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceCoupling {
    pub surface: String,
    pub seams_summary: SeamsSummary,
    pub welch_summary: Option<WelchSummary>,
    /// Heuristic flag: this surface looks like a candidate implementation-
    /// internal seam — high welch anomaly density on implementation
    /// source but low seams architectural-hedging signal density at the
    /// test surface, or vice versa.
    pub mismatch: Option<MismatchKind>,
    pub mismatch_rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeamsSummary {
    /// Number of seams clusters this surface appears in.
    pub clusters_count: u64,
    /// Total cardinality across all clusters (sum of cluster cardinality_total).
    pub cardinality_total: u64,
    /// Construction-style example-subject count across clusters.
    pub construction_style_count: u64,
    /// Densest single signal vector this surface participates in.
    pub dominant_signal_name: String,
    /// True if any cluster carrying this surface fired any architectural-
    /// hedging signal beyond path-partition. Slack-only surfaces are
    /// the test-corpus-invisible suspects when welch shows anomaly.
    pub any_architectural_signal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelchSummary {
    pub matched_files: u64,
    pub max_z: Option<f64>,
    pub max_z_kind: Option<String>,
    /// Whether any matched file carried an unbounded-upward anomaly
    /// (welch's `z_infinite == 1` shape — baseline std was zero).
    pub any_unbounded_upward: bool,
    pub example_files: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MismatchKind {
    /// welch flags implementation source as anomalous; seams has no
    /// architectural-hedging signal at the test-corpus surface. The
    /// classic "implementation-internal seam" candidate.
    WelchHotSeamsCold,
    /// seams shows architectural-hedging at the test-corpus surface but
    /// welch finds no implementation-source anomalies. Suggests the
    /// seam exists in the API contract but the implementation has been
    /// idiomatic-Rust-shaped already.
    SeamsHotWelchCold,
}

// Welch's serialized format — replicate the relevant subset rather than
// importing welch as a crate dependency. Keeps coupling loose between
// the tools.
#[derive(Debug, Clone, Deserialize)]
pub struct WelchAnomalyReport {
    pub anomalous_files: Vec<WelchAnomalousFile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WelchAnomalousFile {
    pub path: String,
    pub loc: u64,
    pub flagged_metrics: Vec<WelchMetricZScore>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WelchMetricZScore {
    pub metric: String,
    pub z: Option<f64>,
    pub z_infinite: Option<i8>,
}

pub fn couple(
    seams: &SeamsReport,
    welch: &WelchAnomalyReport,
    seams_path: Option<&Path>,
    welch_path: Option<&Path>,
) -> Result<CoupledReport> {
    // Index welch by lowercased path for substring matching.
    let welch_index: Vec<(String, &WelchAnomalousFile)> = welch
        .anomalous_files
        .iter()
        .map(|f| (f.path.to_lowercase(), f))
        .collect();

    // Aggregate seams by surface — a surface (first-identifier-segment of
    // a property's subject) may appear in many clusters; for coupling we
    // collapse across clusters to per-surface summaries.
    let mut by_surface: BTreeMap<String, Vec<&SignalCluster>> = BTreeMap::new();
    for cluster in &seams.signal_clusters {
        for surface in &cluster.surfaces_touched {
            by_surface
                .entry(surface.clone())
                .or_default()
                .push(cluster);
        }
    }

    let mut surfaces: Vec<SurfaceCoupling> = Vec::new();
    let mut surfaces_with_welch = 0u64;
    let mut mismatches = 0u64;

    for (surface, clusters) in by_surface {
        // Only operate on surfaces that look like real architectural
        // surfaces (uppercase first letter or known-namespace lowercase).
        // Skips local-variable noise that the seams pipeline emitted but
        // shouldn't drive coupling decisions.
        if !is_relevant_surface(&surface) {
            continue;
        }
        let seams_summary = seams_summary_of(&clusters);
        let welch_summary = welch_summary_for_surface(&surface, &welch_index);
        if welch_summary.is_some() {
            surfaces_with_welch += 1;
        }
        let (mismatch, rationale) =
            classify_mismatch(&surface, &seams_summary, welch_summary.as_ref());
        if mismatch.is_some() {
            mismatches += 1;
        }
        surfaces.push(SurfaceCoupling {
            surface,
            seams_summary,
            welch_summary,
            mismatch,
            mismatch_rationale: rationale,
        });
    }

    // Sort surfaces: mismatch candidates first (those carry the
    // information the coupling apparatus is built to surface), then by
    // welch max-z descending.
    surfaces.sort_by(|a, b| {
        let a_priority = a.mismatch.is_some() as u8;
        let b_priority = b.mismatch.is_some() as u8;
        b_priority
            .cmp(&a_priority)
            .then_with(|| {
                let a_z = a
                    .welch_summary
                    .as_ref()
                    .and_then(|w| w.max_z)
                    .unwrap_or(0.0);
                let b_z = b
                    .welch_summary
                    .as_ref()
                    .and_then(|w| w.max_z)
                    .unwrap_or(0.0);
                b_z.partial_cmp(&a_z).unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let stats = CoupledStats {
        surfaces_total: surfaces.len() as u64,
        surfaces_with_welch_match: surfaces_with_welch,
        mismatch_candidates: mismatches,
    };

    Ok(CoupledReport {
        seams_source: seams_path.map(|p| p.to_string_lossy().into_owned()),
        welch_source: welch_path.map(|p| p.to_string_lossy().into_owned()),
        stats,
        surfaces,
    })
}

fn is_relevant_surface(surface: &str) -> bool {
    if surface.is_empty() {
        return false;
    }
    let bytes = surface.as_bytes();
    if (bytes[0] as char).is_ascii_uppercase() {
        return true;
    }
    matches!(
        surface,
        "fetch"
            | "structuredClone"
            | "queueMicrotask"
            | "atob"
            | "btoa"
            | "performance"
            | "console"
            | "globalThis"
            | "process"
            | "fs"
            | "path"
            | "http"
            | "https"
            | "http2"
            | "net"
            | "tls"
            | "dgram"
            | "dns"
            | "url"
            | "querystring"
            | "crypto"
            | "zlib"
            | "stream"
            | "events"
            | "util"
            | "os"
            | "cluster"
            | "vm"
            | "v8"
            | "buffer"
            | "module"
            | "readline"
            | "tty"
            | "assert"
            | "timers"
    )
}

fn seams_summary_of(clusters: &[&SignalCluster]) -> SeamsSummary {
    let cardinality_total: u64 = clusters.iter().map(|c| c.cardinality_total).sum();
    let cs_count: u64 = clusters
        .iter()
        .map(|c| c.construction_style_count as u64)
        .sum();
    // Dominant signal: take the cluster with highest cardinality and use
    // its signal-vector name.
    let dominant = clusters
        .iter()
        .max_by_key(|c| c.cardinality_total)
        .map(|c| signal_name_terse(&c.signal_vector))
        .unwrap_or_else(|| "none".into());
    let any_architectural = clusters.iter().any(|c| has_architectural_signal(&c.signal_vector));

    SeamsSummary {
        clusters_count: clusters.len() as u64,
        cardinality_total,
        construction_style_count: cs_count,
        dominant_signal_name: dominant,
        any_architectural_signal: any_architectural,
    }
}

/// "Architectural" signal here means any of the six original Doc 705 §4
/// probes (S1-S6) or the v0.2 extensions (S7-S10) — *excluding* the
/// path-partition signal S2, which encodes the test team's directory
/// taxonomy rather than runtime architectural form.
fn has_architectural_signal(v: &SignalVector) -> bool {
    v.cfg
        || v.native
        || v.construct_handle
        || v.weak_ref
        || v.allocator_aware
        || v.threaded
        || !matches!(v.sync_async, crate::seams::SyncAsync::Neither)
        || !matches!(v.throw_return, crate::seams::ThrowReturn::Neither)
        || !matches!(v.error_shape, crate::seams::ErrorShape::None)
}

fn signal_name_terse(v: &SignalVector) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if v.cfg {
        parts.push("cfg");
    }
    match v.sync_async {
        crate::seams::SyncAsync::Sync => parts.push("sync"),
        crate::seams::SyncAsync::Async => parts.push("async"),
        crate::seams::SyncAsync::Mixed => parts.push("sync+async"),
        _ => {}
    }
    match v.throw_return {
        crate::seams::ThrowReturn::Throw => parts.push("throw"),
        crate::seams::ThrowReturn::ReturnError => parts.push("ret-err"),
        crate::seams::ThrowReturn::Mixed => parts.push("throw+ret"),
        _ => {}
    }
    if v.native {
        parts.push("ffi");
    }
    if v.construct_handle {
        parts.push("ctor");
    }
    if v.weak_ref {
        parts.push("weak");
    }
    if v.allocator_aware {
        parts.push("alloc");
    }
    if v.threaded {
        parts.push("thr");
    }
    if parts.is_empty() {
        "slack".into()
    } else {
        parts.join("|")
    }
}

fn welch_summary_for_surface(
    surface: &str,
    welch_index: &[(String, &WelchAnomalousFile)],
) -> Option<WelchSummary> {
    // Surfaces shorter than 4 chars are too noisy for substring matching
    // ("URL" matches "URL.rs", "URLSearchParams.rs", "JSURL.rs", etc. cleanly,
    // but "C" or "S3" would match "config", "Class", "S3Client.rs", and
    // arbitrary short matches across the codebase). At 4+ chars the
    // substring matcher's false-positive rate is operationally acceptable.
    let needle = surface.to_lowercase();
    let matched: Vec<&WelchAnomalousFile> = if needle.len() < 4 {
        welch_index
            .iter()
            .filter(|(p, _)| match_strict(p, &needle))
            .map(|(_, f)| *f)
            .collect()
    } else {
        welch_index
            .iter()
            .filter(|(p, _)| match_substring_in_components(p, &needle))
            .map(|(_, f)| *f)
            .collect()
    };

    if matched.is_empty() {
        return None;
    }

    let mut max_z: Option<f64> = None;
    let mut max_z_kind: Option<String> = None;
    let mut any_unbounded = false;
    for f in &matched {
        for m in &f.flagged_metrics {
            if m.z_infinite == Some(1) {
                any_unbounded = true;
            }
            if let Some(z) = m.z {
                if max_z.map(|cur| z > cur).unwrap_or(true) {
                    max_z = Some(z);
                    max_z_kind = Some(m.metric.clone());
                }
            }
        }
    }
    let example_files: Vec<String> = matched.iter().take(5).map(|f| f.path.clone()).collect();
    Some(WelchSummary {
        matched_files: matched.len() as u64,
        max_z,
        max_z_kind,
        any_unbounded_upward: any_unbounded,
        example_files,
    })
}

/// Strict matcher (used for short surface names). The needle must appear
/// as a complete path component, or as a prefix of a component followed
/// by `_` or `.`.
fn match_strict(path_lower: &str, needle: &str) -> bool {
    path_lower.split('/').any(|seg| {
        seg == needle
            || seg.starts_with(&format!("{}_", needle))
            || seg.starts_with(&format!("{}.", needle))
    })
}

/// Permissive matcher (used for surface names ≥ 4 chars). The needle may
/// appear *inside* a path component case-insensitively. This catches:
/// - `src/jsc/array_buffer.rs` for "Buffer"
/// - `src/jsc/JSUint8Array.rs` for "Uint8Array"
/// - `src/runtime/webcore/FileReader.rs` for "File"
/// - `src/jsc/FetchHeaders.rs` for "Headers"
/// while remaining bounded enough at 4+ chars to keep false-positives
/// operationally low.
fn match_substring_in_components(path_lower: &str, needle: &str) -> bool {
    path_lower.split('/').any(|seg| seg.contains(needle))
}

fn classify_mismatch(
    surface: &str,
    seams: &SeamsSummary,
    welch: Option<&WelchSummary>,
) -> (Option<MismatchKind>, Option<String>) {
    let welch = match welch {
        Some(w) => w,
        None => return (None, None),
    };
    let welch_hot = welch.any_unbounded_upward
        || welch.max_z.map(|z| z >= 5.0).unwrap_or(false);
    let seams_hot = seams.any_architectural_signal && seams.cardinality_total >= 50;

    if welch_hot && !seams_hot {
        let max_z = welch
            .max_z
            .map(|z| format!("{:+.1}", z))
            .unwrap_or_else(|| "+inf".into());
        return (
            Some(MismatchKind::WelchHotSeamsCold),
            Some(format!(
                "{}: welch flags impl-source anomaly (max z={} on {}; \
                 unbounded={}); seams shows {} clusters with cardinality \
                 {} but no architectural-hedging signal beyond path \
                 partition. Candidate implementation-internal seam.",
                surface,
                max_z,
                welch
                    .max_z_kind
                    .clone()
                    .unwrap_or_else(|| "?".into()),
                welch.any_unbounded_upward,
                seams.clusters_count,
                seams.cardinality_total
            )),
        );
    }
    if seams_hot && !welch_hot {
        return (
            Some(MismatchKind::SeamsHotWelchCold),
            Some(format!(
                "{}: seams shows architectural-hedging signal '{}' across \
                 {} clusters (cardinality {}); welch finds {} matched \
                 impl files with max-z={:.1}. The seam exists in the API \
                 contract but the implementation is idiomatic-Rust-shaped.",
                surface,
                seams.dominant_signal_name,
                seams.clusters_count,
                seams.cardinality_total,
                welch.matched_files,
                welch.max_z.unwrap_or(0.0)
            )),
        );
    }
    (None, None)
}
