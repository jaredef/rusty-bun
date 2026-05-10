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
use std::path::{Path, PathBuf};

/// How many lines above and below the cited line to scan for setup-context
/// patterns. The Bun test pattern places `process.platform === "darwin"`
/// guards in `beforeEach` / `describe` scope ~5-30 lines above the
/// expect; ±40 captures the typical case without significant I/O cost.
const CONTEXT_LINES: u32 = 40;

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

/// The architectural-hedging signals from Doc 705 §4 (six) plus four
/// extended signals queued at SEAMS-NOTES.md v0.2 (S7–S10), reduced to
/// a hashable per-property vector. Equality of vectors is the agreement
/// criterion for the simple-cluster MVP at Doc 705 §5 step 2.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignalVector {
    pub cfg: bool,                       // S1: conditional compilation present
    pub path_top: Option<String>,        // S2: dominant test-path segment (node/web/bun/regression/…)
    pub sync_async: SyncAsync,           // S3
    pub throw_return: ThrowReturn,       // S4
    pub native: bool,                    // S5: native / FFI / sys
    pub construct_handle: bool,          // S6: prototype-method or constructor-then-method
    // ── v0.2 extensions ────────────────────────────────────────────────
    pub weak_ref: bool,                  // S7: ownership/reference-cycle
    pub error_shape: ErrorShape,         // S8: refines S4 — distinguish Result-shape from {ok,errors}
    pub allocator_aware: bool,           // S9: arena/bumpalo/slab references
    pub threaded: bool,                  // S10: Worker/MessageChannel/Atomics/SharedArrayBuffer
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ErrorShape {
    /// `Result<T, E>` / `result.ok` / `if (result.ok)` — Rust-style discriminated union.
    Result,
    /// `{ ok: bool, errors: [...] }` — Bun's compound shape.
    OkErrorsArray,
    /// `{ success, errors }` shape with array-of-errors.
    SuccessErrors,
    /// Plain Error object thrown.
    PlainThrow,
    /// Mixed signals.
    Mixed,
    /// No error-shape signal observed.
    None,
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

pub fn detect_seams(report: &ClusterReport, corpus_root: Option<&Path>) -> Result<SeamsReport> {
    let mut signal_groups: BTreeMap<SignalVector, Vec<&Property>> = BTreeMap::new();
    let context_cache = ContextCache::new(corpus_root.map(|p| p.to_path_buf()));

    for prop in &report.properties {
        let v = signal_vector_of(prop, &context_cache);
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
            weak_ref: false,
            error_shape: ErrorShape::None,
            allocator_aware: false,
            threaded: false,
        }
    }
}

fn name_for_signal(v: &SignalVector) -> String {
    let mut parts: Vec<String> = Vec::new();
    if v.cfg {
        parts.push("platform-cfg".into());
    }
    match v.sync_async {
        SyncAsync::Sync => parts.push("sync".into()),
        SyncAsync::Async => parts.push("async".into()),
        SyncAsync::Mixed => parts.push("sync+async".into()),
        SyncAsync::Neither => {}
    }
    match v.throw_return {
        ThrowReturn::Throw => parts.push("throws".into()),
        ThrowReturn::ReturnError => parts.push("returns-error".into()),
        ThrowReturn::Mixed => parts.push("throws+returns-error".into()),
        ThrowReturn::Neither => {}
    }
    if v.native {
        parts.push("native-ffi".into());
    }
    if v.construct_handle {
        parts.push("constructor+handle".into());
    }
    if v.weak_ref {
        parts.push("weak-ref".into());
    }
    match v.error_shape {
        ErrorShape::Result => parts.push("result-shape".into()),
        ErrorShape::OkErrorsArray => parts.push("ok-errors-array".into()),
        ErrorShape::SuccessErrors => parts.push("success-errors".into()),
        ErrorShape::PlainThrow => parts.push("plain-throw".into()),
        ErrorShape::Mixed => parts.push("mixed-error-shape".into()),
        ErrorShape::None => {}
    }
    if v.allocator_aware {
        parts.push("allocator-aware".into());
    }
    if v.threaded {
        parts.push("threaded".into());
    }
    if let Some(ref p) = v.path_top {
        parts.push(format!("@{}", p));
    }
    if parts.is_empty() {
        "slack".into()
    } else {
        parts.join("/")
    }
}

// ───────────────────────── Test-fn-body context cache ──────────────────────

/// Lazy file cache keyed on relative path. The corpus_root + file gives an
/// absolute path; we read the file once and split into lines. Subsequent
/// queries return slices around a target line. With a moderate antichain-
/// representative count (a few thousand) the cache keeps file I/O bounded
/// even though many representatives may share files.
struct ContextCache {
    corpus_root: Option<PathBuf>,
    files: std::cell::RefCell<std::collections::HashMap<String, Option<Vec<String>>>>,
}

impl ContextCache {
    fn new(corpus_root: Option<PathBuf>) -> Self {
        ContextCache {
            corpus_root,
            files: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }

    /// Returns lines [line - CONTEXT_LINES, line + CONTEXT_LINES] joined
    /// for pattern scanning. None when no corpus_root configured or when
    /// the file can't be read.
    fn context_around(&self, file: &str, line: u32) -> Option<String> {
        let root = self.corpus_root.as_ref()?;
        let mut cache = self.files.borrow_mut();
        let entry = cache.entry(file.to_string()).or_insert_with(|| {
            let path = root.join(file);
            std::fs::read_to_string(&path)
                .ok()
                .map(|s| s.lines().map(String::from).collect::<Vec<_>>())
        });
        let lines = entry.as_ref()?;
        let start = (line as i64 - CONTEXT_LINES as i64).max(0) as usize;
        let end = ((line as usize + CONTEXT_LINES as usize)).min(lines.len());
        if start >= lines.len() {
            return None;
        }
        Some(lines[start..end].join("\n"))
    }
}

// ─────────────────────────── Signal extractors ──────────────────────────────

fn signal_vector_of(prop: &Property, ctx: &ContextCache) -> SignalVector {
    let mut v = SignalVector::default();
    let antichain = &prop.antichain;
    let path_components = collect_path_components(antichain);

    // Build a per-property concatenated context string. When corpus_root
    // isn't configured, context will be empty and signals fall back to
    // antichain-text-only detection (the v0.2 behaviour).
    let test_body_ctx = build_context(antichain, ctx);

    v.cfg = signal_cfg(antichain, &path_components, &test_body_ctx);
    v.path_top = signal_path_top(&path_components);
    v.sync_async = signal_sync_async(prop, antichain);
    v.throw_return = signal_throw_return(prop, antichain);
    v.native = signal_native(antichain, &path_components);
    v.construct_handle = signal_construct_handle(prop);
    v.weak_ref = signal_weak_ref(prop, antichain);
    v.error_shape = signal_error_shape(prop, antichain);
    v.allocator_aware = signal_allocator_aware(antichain);
    v.threaded = signal_threaded(prop, antichain);

    v
}

/// S7 — ownership / reference-cycle. Detects WeakRef, WeakMap, WeakSet,
/// FinalizationRegistry, and structuredClone (deep-copy semantics that
/// arise specifically when reference structure must be preserved or
/// broken). These signal a property whose contract crosses the
/// ownership / lifetime / cycle boundary.
fn signal_weak_ref(prop: &Property, antichain: &[RepresentativeConstraint]) -> bool {
    const WEAK_REF_PATTERNS: &[&str] = &[
        "WeakRef",
        "WeakMap",
        "WeakSet",
        "FinalizationRegistry",
        "structuredClone",
        ".deref()",
        ".register(",
    ];
    if WEAK_REF_PATTERNS.iter().any(|p| prop.subject.contains(p)) {
        return true;
    }
    for r in antichain {
        for pat in WEAK_REF_PATTERNS {
            if r.raw.contains(pat) {
                return true;
            }
        }
    }
    false
}

/// S8 — error-shape distinction. Refines S4's binary throw/return-error
/// into specific shapes the runtime exposes:
/// - Result: `{ ok: true | false }` Rust-style discriminated union pattern
/// - OkErrorsArray: Bun's specific `{ ok, errors: [...] }` compound
/// - SuccessErrors: `{ success: bool, errors: [...] }`
/// - PlainThrow: bare throw with no compound shape
fn signal_error_shape(prop: &Property, antichain: &[RepresentativeConstraint]) -> ErrorShape {
    let mut has_ok_errors = false;
    let mut has_success = false;
    let mut has_result_ok = false;
    let mut has_plain_throw = matches!(prop.verb_class, VerbClass::Error);

    for r in antichain {
        if r.raw.contains(".errors).") || r.raw.contains(".errors[") || r.raw.contains("errors: [") {
            if r.raw.contains(".ok)") || r.raw.contains("ok: ") {
                has_ok_errors = true;
            } else if r.raw.contains(".success)") || r.raw.contains("success: ") {
                has_success = true;
            } else {
                has_ok_errors = true; // ambiguous — bias to ok-errors-array shape
            }
        } else if r.raw.contains(".success).toBe(true)") || r.raw.contains(".success).toBe(false)") {
            has_success = true;
        } else if r.raw.contains(".ok).toBe(true)") || r.raw.contains(".ok).toBe(false)") {
            has_result_ok = true;
        }
        if r.raw.contains(".toThrow") || r.raw.contains("rejects.toThrow") {
            has_plain_throw = true;
        }
    }

    let count = (has_ok_errors as u8)
        + (has_success as u8)
        + (has_result_ok as u8)
        + (has_plain_throw as u8);
    if count == 0 {
        return ErrorShape::None;
    }
    if count > 1 {
        return ErrorShape::Mixed;
    }
    if has_ok_errors {
        ErrorShape::OkErrorsArray
    } else if has_success {
        ErrorShape::SuccessErrors
    } else if has_result_ok {
        ErrorShape::Result
    } else {
        ErrorShape::PlainThrow
    }
}

/// S9 — allocator-discipline awareness. Properties whose contract surfaces
/// allocator behavior — arena lifetime, slab allocation, bump-pool. The
/// seam is between heap-allocated values whose lifetime is tied to a
/// caller-controlled scope vs values whose lifetime is the global heap.
fn signal_allocator_aware(antichain: &[RepresentativeConstraint]) -> bool {
    const ALLOCATOR_PATTERNS: &[&str] = &[
        "arena",
        "Arena",
        "bumpalo",
        "Bump",
        "slab",
        "MimallocArena",
        "ArrayList(",
        "BabyList",
        "MultiArrayList",
    ];
    for r in antichain {
        for pat in ALLOCATOR_PATTERNS {
            if r.raw.contains(pat) {
                return true;
            }
        }
    }
    false
}

/// S10 — threading-model awareness. Properties whose contract crosses a
/// thread boundary (Worker, MessageChannel, BroadcastChannel, Atomics,
/// SharedArrayBuffer). Distinct from S3 sync/async which is the
/// execution-discipline boundary; threading is the address-space-sharing
/// boundary.
fn signal_threaded(prop: &Property, antichain: &[RepresentativeConstraint]) -> bool {
    const THREAD_HEADS: &[&str] = &[
        "Worker",
        "MessageChannel",
        "MessagePort",
        "BroadcastChannel",
        "Atomics",
        "SharedArrayBuffer",
        "AsyncLocalStorage",
    ];
    let head = prop.subject.split('.').next().unwrap_or("");
    if THREAD_HEADS.iter().any(|h| *h == head) {
        return true;
    }
    for r in antichain {
        for pat in THREAD_HEADS {
            if r.raw.contains(pat) {
                return true;
            }
        }
    }
    false
}

fn build_context(antichain: &[RepresentativeConstraint], ctx: &ContextCache) -> String {
    let mut buf = String::new();
    for r in antichain {
        if let Some(ctx_lines) = ctx.context_around(&r.file, r.line) {
            buf.push_str(&ctx_lines);
            buf.push('\n');
        }
    }
    buf
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

/// S1 — conditional compilation. Probes antichain raw text, file paths,
/// test names, AND (when corpus_root is configured) the surrounding
/// test-fn-body context. The body context is the dominant source for
/// platform-cfg patterns in JS test corpora — Bun's tests typically
/// place `process.platform === "darwin"` guards in `beforeEach` or
/// `describe`-scope, not in `expect` clauses.
fn signal_cfg(
    antichain: &[RepresentativeConstraint],
    paths: &[Vec<String>],
    test_body_ctx: &str,
) -> bool {
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
        // Bun-specific: tests often guard on bun-runtime feature flags
        // and target predicates set via `it.if` / `test.if` patterns.
        "test.if(",
        "it.if(",
        "describe.if(",
        ".skipIf(",
        ".runIf(",
        // Common Node/Bun cross-platform guard expressions:
        "platform !== \"win32\"",
        "platform === \"darwin\"",
        "os.platform()",
        "isBroken",
        "isWindows()",
        "isCI",
    ];
    const CFG_PATH_NAMES: &[&str] = &["darwin", "linux", "windows", "posix", "win32"];
    const CFG_TEST_NAME_HINTS: &[&str] = &[
        "on Windows", "on macOS", "on Linux", "Windows-only",
        "POSIX", "Linux-only", "macOS-only", "Darwin-only",
    ];

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
    // Test-fn-body context — the dominant location of cfg-style guards
    // in Bun's test corpus. Empty when corpus_root not configured.
    if !test_body_ctx.is_empty() {
        for pat in CFG_RAW_PATTERNS {
            if test_body_ctx.contains(pat) {
                return true;
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
