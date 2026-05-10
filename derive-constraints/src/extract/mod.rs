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
