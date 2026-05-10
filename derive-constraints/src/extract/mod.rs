//! Shared types for the extraction layer. Each language extractor produces
//! the same `TestFile` shape; downstream phases (cluster / invert / predict)
//! consume the unified structure regardless of source language.

use serde::{Deserialize, Serialize};

pub mod rust;
pub mod spec;
pub mod ts_js;
pub mod zig;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Zig,
    /// Spec-source markdown — manually-curated invariants extracted from
    /// external specifications (WHATWG, ECMA, RFC, Node API docs).
    /// Treated as test-equivalent input by the cluster phase.
    Spec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFile {
    /// Path relative to the scan root.
    pub path: String,
    pub language: Language,
    /// Source line count of the entire file (incl. non-test code).
    pub loc: u32,
    /// Discovered test cases. Empty for files with no tests.
    pub tests: Vec<TestCase>,
    /// Set when the parser failed; absent on success.
    pub parse_failure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Human-readable name. For TS/JS, reconstructs nested describe → it
    /// chains (e.g. "Bun.serve > 404 fallback > with custom handler").
    pub name: String,
    /// Test-API kind: `test`, `it`, `describe`, `#[test]`, `zig-test`.
    pub kind: TestKind,
    pub line_start: u32,
    pub line_end: u32,
    /// Constraint clauses found inside the test body — assertion calls,
    /// expect() chains, etc. Order is source order.
    pub constraints: Vec<ConstraintClause>,
    /// `test.skip` / `xtest` / `#[ignore]` markers — the test is registered
    /// but disabled. Carries information about un-validated constraints.
    pub skip: bool,
    /// `test.todo` markers — the constraint is acknowledged but not yet implemented.
    pub todo: bool,
    /// `test.failing` markers — the constraint is expected to currently fail.
    pub failing: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TestKind {
    /// `test(name, fn)` in TS/JS, or `#[test]` in Rust.
    Test,
    /// `it(name, fn)` in TS/JS — semantic alias for `test` under
    /// describe blocks.
    It,
    /// `describe(name, fn)` block — emits as a TestCase carrying its
    /// nested children's names but with no direct constraints; useful as
    /// scope context only.
    Describe,
    /// Zig `test "name" { ... }` source-internal test block.
    ZigTest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintClause {
    pub line: u32,
    /// Best-effort verbatim extraction of the assertion site. May span
    /// multiple lines of the original source compressed onto one line.
    pub raw: String,
    /// Discriminator between expect-chain, top-level assertion macro,
    /// Zig testing.expect call, and Rust assert! family.
    pub kind: ConstraintKind,
    /// First identifier referenced before the assertion verb, if any —
    /// e.g. for `expect(Bun.serve).toBeFunction()` the subject is
    /// `Bun.serve`. Heuristic; may be None.
    pub subject: Option<String>,
    /// Authority tier per Doc 707. Defaulted at extraction time by
    /// `classify_authority_tier(subject)`; cluster phase aggregates per
    /// property. See `cluster::aggregate_authority_tier`.
    #[serde(default = "default_authority_tier")]
    pub authority_tier: AuthorityTier,
}

fn default_authority_tier() -> AuthorityTier { AuthorityTier::Contingent }

/// Per [Doc 707](https://jaredfoy.com/resolve/doc/707-pin-art-at-the-behavioral-surface-bidirectional-probes)
/// §"Plug-and-play criterion as forward operational consequence":
///
/// **Spec** — WHATWG / W3C / RFC says it. Must conform.
/// **Ecosystem** — Bun-extension or Node-API. Bun's tests are the spec.
/// **Contingent** — Implementation choice. Optional divergence with reason.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityTier {
    /// Lowest tier in the ordering: implementation-contingent. Default
    /// when extraction can't classify with confidence.
    Contingent,
    /// Middle tier: ecosystem-compat (Node API + Bun-namespace extensions).
    Ecosystem,
    /// Highest tier: spec-mandated. Strongest binding constraint.
    Spec,
}

impl AuthorityTier {
    pub fn as_str(self) -> &'static str {
        match self {
            AuthorityTier::Spec => "spec",
            AuthorityTier::Ecosystem => "ecosystem",
            AuthorityTier::Contingent => "contingent",
        }
    }
}

/// Default tagging at extraction time. Conservative: when in doubt, return
/// Contingent. Spec-source clauses (from `*.spec.md`) are tagged at the
/// extractor level; this function classifies test-derived clauses by
/// inspecting their subject.
pub fn classify_authority_tier(subject: Option<&str>, kind: ConstraintKind) -> AuthorityTier {
    // Spec extracts always Spec.
    if matches!(kind, ConstraintKind::SpecInvariant) {
        return AuthorityTier::Spec;
    }
    let subj = match subject { Some(s) => s, None => return AuthorityTier::Contingent };
    let head = subj.split('.').next().unwrap_or("");

    // Web-platform surfaces (WHATWG / W3C spec'd) → Spec.
    if matches!(head,
        "URL" | "URLSearchParams" | "Request" | "Response" | "Headers"
        | "Blob" | "File" | "FormData" | "ReadableStream" | "WritableStream"
        | "TransformStream" | "TextEncoder" | "TextDecoder"
        | "AbortController" | "AbortSignal" | "WebSocket" | "Worker"
        | "MessagePort" | "MessageChannel" | "BroadcastChannel"
        | "EventTarget" | "Event" | "CustomEvent" | "fetch"
        | "structuredClone" | "queueMicrotask" | "atob" | "btoa"
        | "performance" | "crypto" | "Image" | "Notification"
    ) {
        return AuthorityTier::Spec;
    }

    // Bun-namespace + Node-API surfaces → Ecosystem.
    if matches!(head,
        "Bun" | "Buffer" | "process" | "global" | "globalThis"
        | "fs" | "path" | "http" | "https" | "http2"
        | "net" | "tls" | "dgram" | "dns" | "url"
        | "querystring" | "zlib" | "stream" | "events" | "util"
        | "os" | "child_process" | "cluster" | "worker_threads"
        | "vm" | "v8" | "perf_hooks" | "async_hooks" | "buffer"
        | "inspector" | "module" | "readline" | "repl" | "tty"
        | "string_decoder" | "punycode" | "assert" | "timers"
        | "setTimeout" | "setInterval" | "setImmediate"
        | "clearTimeout" | "clearInterval" | "clearImmediate"
    ) {
        return AuthorityTier::Ecosystem;
    }

    // Default: contingent. The subject doesn't match a known authoritative
    // surface; the constraint is implementation-internal.
    AuthorityTier::Contingent
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintKind {
    /// `expect(x).toY(z)` chain.
    ExpectChain,
    /// `assert(...)`, `assert.equal(...)`, `assertEquals(...)`.
    AssertCall,
    /// Rust `assert!`, `assert_eq!`, `assert_ne!`, `debug_assert!`.
    AssertMacro,
    /// Zig `try testing.expect(...)`, `try testing.expectEqual(...)`.
    ZigTestingExpect,
    /// Top-level `if (cond) throw new Error(...)` or `panic!(...)` —
    /// the substrate may emit these instead of explicit assertions.
    GuardThrow,
    /// Manually-curated invariant extracted from an external spec.
    /// Carries the same shape as test-derived clauses but with `source: spec`
    /// semantics: the assertion is normative, not test-witnessed.
    SpecInvariant,
}
