//! Phase 4 — seams. Reads a `ClusterReport`; extracts per-property
//! architectural-hedging signal vectors per [Doc 705 §4](https://jaredfoy.com/resolve/doc/705-pin-art-operationalized-for-intra-architectural-seam-detection);
//! groups by signal-vector agreement (Doc 705 §5 step 2); reads
//! cross-namespace seams (Doc 705 §5 step 3); emits `seams.json`.
//!
//! Step 4 (resistance-as-boundary verification via rederive) and Step 5
//! (revised surface decomposition) are queued for the rederive pilot;
//! v0.1 produces the candidate-seam catalog the pilot will validate.

use crate::cluster::{ClusterReport, Property, RepresentativeConstraint, VerbClass};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SyncAsync {
    Sync,
    Async,
    Mixed,
    Neither,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ThrowReturn {
    Throw,
    ReturnError,
    Mixed,
    Neither,
}

/// The six architectural-hedging signals from Doc 705 §4 reduced to a
/// hashable per-property vector. Equality of vectors is the agreement
/// criterion for the simple-cluster MVP at Doc 705 §5 step 2.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignalVector {
    pub cfg: bool,                       // S1: conditional compilation present
    pub path_top: Option<String>,        // S2: dominant test-path segment (node/web/bun/regression/…)
    pub sync_async: SyncAsync,           // S3
    pub throw_return: ThrowReturn,       // S4
    pub native: bool,                    // S5: native / FFI / sys
    pub construct_handle: bool,          // S6: prototype-method or constructor-then-method
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeamsReport {
    pub cluster_source: Option<String>,
    pub stats: SeamsStats,
    pub signal_clusters: Vec<SignalCluster>,
    pub cross_namespace_seams: Vec<CrossNamespaceSeam>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SeamsStats {
    pub properties_in: u64,
    pub distinct_signal_vectors: u64,
    pub clusters_emitted: u64,
    pub cross_namespace_seam_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalCluster {
    pub id: String,
    pub signal_vector: SignalVector,
    pub property_count: u64,
    pub cardinality_total: u64,
    pub construction_style_count: u32,
    /// Distinct first-identifier-segment surfaces these properties span.
    pub surfaces_touched: Vec<String>,
    /// Up to 16 representative property subjects, ordered by cardinality.
    pub example_subjects: Vec<ExampleSubject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleSubject {
    pub subject: String,
    pub surface: String,
    pub cardinality: u64,
    pub construction_style: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossNamespaceSeam {
    pub seam_name: String,
    pub cluster_id: String,
    pub action: String, // "split <surface>" / "merge <surfaces>" / "annotate <surface>"
    pub surfaces: Vec<String>,
    pub property_count: u64,
    pub cardinality_total: u64,
}

pub fn detect_seams(report: &ClusterReport) -> Result<SeamsReport> {
    let mut signal_groups: BTreeMap<SignalVector, Vec<&Property>> = BTreeMap::new();

    for prop in &report.properties {
        let v = signal_vector_of(prop);
        signal_groups.entry(v).or_default().push(prop);
    }

    let mut clusters: Vec<SignalCluster> = signal_groups
        .into_iter()
        .map(|(vector, props)| build_cluster(&vector, &props))
        .collect();

    // Sort clusters by cardinality_total descending — biggest seams first
    // in the report.
    clusters.sort_by(|a, b| b.cardinality_total.cmp(&a.cardinality_total));

    // Assign stable IDs after sort.
    for (i, c) in clusters.iter_mut().enumerate() {
        c.id = format!("SC{:04}", i + 1);
    }

    let cross_namespace_seams = read_cross_namespace_seams(&clusters);

    let stats = SeamsStats {
        properties_in: report.properties.len() as u64,
        distinct_signal_vectors: clusters.len() as u64,
        clusters_emitted: clusters.len() as u64,
        cross_namespace_seam_count: cross_namespace_seams.len() as u64,
    };

    Ok(SeamsReport {
        cluster_source: None,
        stats,
        signal_clusters: clusters,
        cross_namespace_seams,
    })
}

fn build_cluster(vector: &SignalVector, props: &[&Property]) -> SignalCluster {
    let cardinality_total: u64 = props.iter().map(|p| p.constraints_in).sum();
    let cs_count = props.iter().filter(|p| p.construction_style).count() as u32;
    let mut surfaces: Vec<String> = props
        .iter()
        .map(|p| surface_of(&p.subject))
        .collect();
    surfaces.sort();
    surfaces.dedup();

    let mut sorted_props: Vec<&&Property> = props.iter().collect();
    sorted_props.sort_by(|a, b| b.constraints_in.cmp(&a.constraints_in));
    let example_subjects: Vec<ExampleSubject> = sorted_props
        .iter()
        .take(16)
        .map(|p| ExampleSubject {
            subject: p.subject.clone(),
            surface: surface_of(&p.subject),
            cardinality: p.constraints_in,
            construction_style: p.construction_style,
        })
        .collect();

    SignalCluster {
        id: String::new(), // assigned post-sort
        signal_vector: vector.clone(),
        property_count: props.len() as u64,
        cardinality_total,
        construction_style_count: cs_count,
        surfaces_touched: surfaces,
        example_subjects,
    }
}

fn surface_of(subject: &str) -> String {
    subject.split('.').next().unwrap_or(subject).to_string()
}

/// Read each cluster's surfaces_touched against the existing first-segment
/// partition and propose a decomposition action: split (cluster confined
/// to one surface but signal vector is non-trivial), merge (cluster spans
/// > 1 surfaces and signal vector identifies a real seam crosscutting
/// them), annotate (cluster aligns with the existing partition; no
/// reorganization needed).
fn read_cross_namespace_seams(clusters: &[SignalCluster]) -> Vec<CrossNamespaceSeam> {
    let mut out = Vec::new();
    for c in clusters {
        if c.cardinality_total < 20 {
            // Small clusters are noise at the seam-decomposition layer; skip.
            continue;
        }
        if c.signal_vector == SignalVector::default() {
            // The "no architectural-hedging signal" cluster — slack
            // hedging analogue, not a seam. Skip.
            continue;
        }
        let action = if c.surfaces_touched.len() == 1 {
            format!("split {}", c.surfaces_touched[0])
        } else if c.surfaces_touched.len() <= 8 {
            format!("merge across [{}]", c.surfaces_touched.join(", "))
        } else {
            "meta-seam (crosscuts most surfaces)".to_string()
        };
        out.push(CrossNamespaceSeam {
            seam_name: name_for_signal(&c.signal_vector),
            cluster_id: c.id.clone(),
            action,
            surfaces: c.surfaces_touched.clone(),
            property_count: c.property_count,
            cardinality_total: c.cardinality_total,
        });
    }
    out
}

impl Default for SignalVector {
    fn default() -> Self {
        SignalVector {
            cfg: false,
            path_top: None,
            sync_async: SyncAsync::Neither,
            throw_return: ThrowReturn::Neither,
            native: false,
            construct_handle: false,
        }
    }
}

fn name_for_signal(v: &SignalVector) -> String {
    let mut parts = Vec::new();
    if v.cfg {
        parts.push("platform-cfg");
    }
    match v.sync_async {
        SyncAsync::Sync => parts.push("sync"),
        SyncAsync::Async => parts.push("async"),
        SyncAsync::Mixed => parts.push("sync+async"),
        SyncAsync::Neither => {}
    }
    match v.throw_return {
        ThrowReturn::Throw => parts.push("throws"),
        ThrowReturn::ReturnError => parts.push("returns-error"),
        ThrowReturn::Mixed => parts.push("throws+returns-error"),
        ThrowReturn::Neither => {}
    }
    if v.native {
        parts.push("native-ffi");
    }
    if v.construct_handle {
        parts.push("constructor+handle");
    }
    if let Some(ref p) = v.path_top {
        parts.push(p);
    }
    if parts.is_empty() {
        "slack".into()
    } else {
        parts.join("/")
    }
}

// ─────────────────────────── Signal extractors ──────────────────────────────

fn signal_vector_of(prop: &Property) -> SignalVector {
    let mut v = SignalVector::default();
    let antichain = &prop.antichain;
    let path_components = collect_path_components(antichain);

    v.cfg = signal_cfg(antichain, &path_components);
    v.path_top = signal_path_top(&path_components);
    v.sync_async = signal_sync_async(prop, antichain);
    v.throw_return = signal_throw_return(prop, antichain);
    v.native = signal_native(antichain, &path_components);
    v.construct_handle = signal_construct_handle(prop);

    v
}

fn collect_path_components(antichain: &[RepresentativeConstraint]) -> Vec<Vec<String>> {
    antichain
        .iter()
        .map(|r| {
            r.file
                .split('/')
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// S1 — conditional compilation. Look at antichain text and source-file
/// paths for platform-conditional patterns, including Zig idioms and
/// JS-side `process.platform` style guards.
fn signal_cfg(antichain: &[RepresentativeConstraint], paths: &[Vec<String>]) -> bool {
    const CFG_RAW_PATTERNS: &[&str] = &[
        "process.platform",
        "isWindows",
        "isPosix",
        "isMacOS",
        "isLinux",
        "Environment.isWindows",
        "#[cfg(target_os",
        "#[cfg(unix)",
        "#[cfg(windows)",
        "#[cfg(any(target_os",
        "if process.platform",
        "platform === \"darwin\"",
        "platform === \"linux\"",
        "platform === \"win32\"",
    ];
    const CFG_PATH_NAMES: &[&str] = &["darwin", "linux", "windows", "posix", "win32"];
    const CFG_TEST_NAME_HINTS: &[&str] = &["on Windows", "on macOS", "on Linux", "Windows-only", "POSIX"];

    for r in antichain {
        for pat in CFG_RAW_PATTERNS {
            if r.raw.contains(pat) {
                return true;
            }
        }
        for hint in CFG_TEST_NAME_HINTS {
            if r.test_name.contains(hint) {
                return true;
            }
        }
    }
    for components in paths {
        for c in components {
            for n in CFG_PATH_NAMES {
                if c.eq_ignore_ascii_case(n) {
                    return true;
                }
            }
        }
    }
    false
}

/// S2 — test-file path partitioning. The dominant top-level path segment
/// beyond `test/` for this property's antichain. Returns None if the
/// antichain spans multiple top-level segments, indicating the property
/// is not localized to one path-defined area.
fn signal_path_top(paths: &[Vec<String>]) -> Option<String> {
    let mut tops: Vec<String> = Vec::new();
    for components in paths {
        // Skip "test" itself, take the next segment.
        let mut iter = components.iter().peekable();
        while let Some(c) = iter.next() {
            if c == "test" {
                if let Some(next) = iter.next() {
                    tops.push(next.clone());
                }
                break;
            }
        }
    }
    if tops.is_empty() {
        return None;
    }
    // If all entries agree, return the agreed value. Otherwise None.
    let first = tops[0].clone();
    if tops.iter().all(|t| *t == first) {
        Some(first)
    } else {
        None
    }
}

/// S3 — sync/async partitioning.
fn signal_sync_async(prop: &Property, antichain: &[RepresentativeConstraint]) -> SyncAsync {
    let mut has_sync = false;
    let mut has_async = false;

    if prop.subject.ends_with("Sync") {
        has_sync = true;
    }
    if prop.subject.contains("Async") || prop.subject.starts_with("async") {
        has_async = true;
    }

    for r in antichain {
        if r.raw.contains("await ")
            || r.raw.contains(".rejects.")
            || r.raw.contains("toResolve")
            || r.raw.contains("Promise.")
            || r.raw.contains(".then(")
        {
            has_async = true;
        }
        if r.raw.contains("synchronously") || r.test_name.contains("sync") {
            has_sync = true;
        }
        if r.test_name.contains("async") {
            has_async = true;
        }
    }

    match (has_sync, has_async) {
        (true, true) => SyncAsync::Mixed,
        (true, false) => SyncAsync::Sync,
        (false, true) => SyncAsync::Async,
        (false, false) => SyncAsync::Neither,
    }
}

/// S4 — throw vs return-error partitioning.
fn signal_throw_return(prop: &Property, antichain: &[RepresentativeConstraint]) -> ThrowReturn {
    let mut throws = matches!(prop.verb_class, VerbClass::Error);
    let mut returns_error = false;

    for r in antichain {
        if r.raw.contains(".toThrow")
            || r.raw.contains(".rejects.toThrow")
            || r.raw.contains("toThrowError")
            || r.raw.contains("assert.throws")
        {
            throws = true;
        }
        if r.raw.contains("result.success")
            || r.raw.contains("result.errors")
            || r.raw.contains(".success).toBe(false)")
            || r.raw.contains(".success).toBe(true)")
            || r.raw.contains(".errors).")
            || r.raw.contains(".error).toEqual")
            || r.raw.contains("{ ok: false")
            || r.raw.contains("{ ok: true")
        {
            returns_error = true;
        }
    }

    match (throws, returns_error) {
        (true, true) => ThrowReturn::Mixed,
        (true, false) => ThrowReturn::Throw,
        (false, true) => ThrowReturn::ReturnError,
        (false, false) => ThrowReturn::Neither,
    }
}

/// S5 — native vs userland partitioning. Detects FFI shims, `_sys/`
/// directories, raw-pointer constructions, and explicit FFI APIs.
fn signal_native(antichain: &[RepresentativeConstraint], paths: &[Vec<String>]) -> bool {
    const NATIVE_RAW_PATTERNS: &[&str] = &[
        "Bun.dlopen",
        "Bun.FFI",
        "napi_",
        "extern \"C\"",
        "*const ",
        "*mut ",
        "ffi.dlopen",
        "@import(\"napi",
    ];
    const NATIVE_PATH_HINTS: &[&str] = &[
        "_sys",
        "bindings",
        "napi",
        "boringssl",
        "libuv",
        "windows_sys",
    ];

    for r in antichain {
        for pat in NATIVE_RAW_PATTERNS {
            if r.raw.contains(pat) {
                return true;
            }
        }
    }
    for components in paths {
        for c in components {
            for hint in NATIVE_PATH_HINTS {
                if c.contains(hint) {
                    return true;
                }
            }
        }
    }
    false
}

/// S6 — construct-then-method partitioning. Subjects of shape
/// `X.prototype.Y` indicate a method on a constructed handle; subjects
/// like `new X(...)` patterns in raw text indicate a constructor.
fn signal_construct_handle(prop: &Property) -> bool {
    if prop.subject.contains(".prototype.") {
        return true;
    }
    for r in &prop.antichain {
        if r.raw.contains("new ") && r.raw.contains(&prop.subject) {
            return true;
        }
    }
    false
}
