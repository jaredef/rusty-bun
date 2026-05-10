//! Phase 2 — cluster. Reads the per-file extraction from `derive-constraints
//! scan`, canonicalizes each constraint into a property key (subject +
//! verb-class), groups constraints by property, selects a minimal-antichain
//! of representative constraints per property (default N=3), and classifies
//! each property as construction-style or behavioral via the public-API-
//! surface heuristic at `docs/cluster-phase-design.md §6 step 5`.

use crate::extract::{ConstraintKind, TestCase, TestFile};
use crate::scan::ScanReport;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum VerbClass {
    /// `toBe`, `toEqual`, `toStrictEqual`, `assert_eq!`, `assert.equal`.
    Equivalence,
    /// `toBeInstanceOf`, `toBeFunction`, `toBeObject`, `toBeArray`,
    /// `toBeString`, `toBeNumber`, `toBeBoolean`.
    TypeInstance,
    /// `toThrow`, `rejects.toThrow`, `panic!`, `unreachable!`,
    /// `expectError`.
    Error,
    /// `toBeDefined`, `toBeUndefined`, `toBeNull`, `toBeTruthy`,
    /// `toBeFalsy`, `toBeNullish`.
    Existence,
    /// `toContain`, `toMatch`, `toHaveProperty`, `expectEqualSlices`.
    Containment,
    /// `toBeGreaterThan`, `toBeLessThan`, `toBeCloseTo`.
    Ordering,
    /// `assert!`, `assert(...)`, `ok(...)`, `try testing.expect(cond)`.
    GenericAssertion,
    /// Anything that didn't pattern-match.
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterReport {
    pub source_path: Option<String>,
    pub stats: ClusterStats,
    pub properties: Vec<Property>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClusterStats {
    pub constraints_in: u64,
    pub properties_out: u64,
    pub antichain_size: u64,
    pub construction_style_count: u64,
    pub behavioral_count: u64,
    pub reduction_ratio: f64,
    pub by_verb_class: BTreeMap<String, u64>,
    /// Distribution: how many properties have N input constraints.
    /// Bucket keys: "1", "2-5", "6-20", "21-100", "101-500", "501+".
    pub property_cardinality_buckets: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub subject: String,
    pub verb_class: VerbClass,
    /// Number of input constraint clauses that canonicalized to this property.
    pub constraints_in: u64,
    /// Representative antichain — the constraints kept after the
    /// minimal-antichain selection. Default N=3.
    pub antichain: Vec<RepresentativeConstraint>,
    /// True if classified construction-style per the public-API-surface
    /// heuristic. Behavioral otherwise.
    pub construction_style: bool,
    /// Up to 5 source files containing constraints for this property.
    pub source_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepresentativeConstraint {
    pub test_name: String,
    pub file: String,
    pub line: u32,
    pub raw: String,
}

const ANTICHAIN_SIZE: usize = 3;

pub fn cluster(scan: &ScanReport) -> Result<ClusterReport> {
    // Group constraints by (subject, verb_class).
    let mut groups: BTreeMap<(String, VerbClass), Vec<Entry>> = BTreeMap::new();
    let mut total_in: u64 = 0;

    for file in &scan.files {
        for test in &file.tests {
            if test.todo {
                // Skip todo-marked tests: they are explicit acknowledgments
                // that the constraint is unimplemented, and including them
                // would inflate the property catalog with placeholders.
                continue;
            }
            for clause in &test.constraints {
                total_in += 1;
                let verb = classify_verb(&clause.raw, clause.kind);
                let subject = canonicalize_subject(clause.subject.as_deref(), &clause.raw);
                let key = (subject, verb);
                groups.entry(key).or_default().push(Entry::from_clause(file, test, clause));
            }
        }
    }

    // Build properties from the groups.
    let mut properties: Vec<Property> = groups
        .into_iter()
        .map(|((subject, verb_class), entries)| {
            let constraints_in = entries.len() as u64;
            let antichain = select_antichain(&entries, ANTICHAIN_SIZE);
            let source_files = collect_source_files(&entries, 5);
            let construction_style = classify_construction_style(&subject, verb_class, &entries);
            Property {
                subject,
                verb_class,
                constraints_in,
                antichain,
                construction_style,
                source_files,
            }
        })
        .collect();

    // Sort properties: construction-style first, then by cardinality
    // descending. The most cited construction-style surface lands first
    // in the report, which is what a human reader most wants.
    properties.sort_by(|a, b| {
        b.construction_style
            .cmp(&a.construction_style)
            .then_with(|| b.constraints_in.cmp(&a.constraints_in))
            .then_with(|| a.subject.cmp(&b.subject))
    });

    let stats = compute_stats(total_in, &properties);

    Ok(ClusterReport {
        source_path: None,
        stats,
        properties,
    })
}

struct Entry {
    test_name: String,
    file: String,
    line: u32,
    raw: String,
}

impl Entry {
    fn from_clause(
        file: &TestFile,
        test: &TestCase,
        clause: &crate::extract::ConstraintClause,
    ) -> Self {
        Entry {
            test_name: test.name.clone(),
            file: file.path.clone(),
            line: clause.line,
            raw: clause.raw.clone(),
        }
    }
}

fn select_antichain(entries: &[Entry], n: usize) -> Vec<RepresentativeConstraint> {
    if entries.len() <= n {
        return entries.iter().map(rep).collect();
    }
    // Strategy: pick representatives from distinct files where possible.
    // Falling back to first-by-source-order when fewer files are available
    // than the antichain size.
    let mut chosen: Vec<&Entry> = Vec::with_capacity(n);
    let mut seen_files: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for e in entries {
        if chosen.len() == n {
            break;
        }
        if seen_files.insert(e.file.as_str()) {
            chosen.push(e);
        }
    }
    if chosen.len() < n {
        // Fall back to filling from any remaining entries.
        for e in entries {
            if chosen.len() == n {
                break;
            }
            if !chosen.iter().any(|c| std::ptr::eq(*c, e)) {
                chosen.push(e);
            }
        }
    }
    chosen.iter().map(|e| rep(*e)).collect()
}

fn rep(e: &Entry) -> RepresentativeConstraint {
    RepresentativeConstraint {
        test_name: e.test_name.clone(),
        file: e.file.clone(),
        line: e.line,
        raw: e.raw.clone(),
    }
}

fn collect_source_files(entries: &[Entry], n: usize) -> Vec<String> {
    let mut files: Vec<&str> = entries.iter().map(|e| e.file.as_str()).collect();
    files.sort();
    files.dedup();
    files.into_iter().take(n).map(String::from).collect()
}

fn compute_stats(total_in: u64, properties: &[Property]) -> ClusterStats {
    let mut stats = ClusterStats::default();
    stats.constraints_in = total_in;
    stats.properties_out = properties.len() as u64;
    let mut antichain_size: u64 = 0;
    for p in properties {
        antichain_size += p.antichain.len() as u64;
        if p.construction_style {
            stats.construction_style_count += 1;
        } else {
            stats.behavioral_count += 1;
        }
        let bucket = bucket_for(p.constraints_in);
        *stats
            .property_cardinality_buckets
            .entry(bucket.into())
            .or_insert(0) += 1;
        let verb_key = format!("{:?}", p.verb_class).to_lowercase();
        *stats.by_verb_class.entry(verb_key).or_insert(0) += 1;
    }
    stats.antichain_size = antichain_size;
    stats.reduction_ratio = if total_in == 0 {
        0.0
    } else {
        antichain_size as f64 / total_in as f64
    };
    stats
}

fn bucket_for(n: u64) -> &'static str {
    match n {
        1 => "1",
        2..=5 => "2-5",
        6..=20 => "6-20",
        21..=100 => "21-100",
        101..=500 => "101-500",
        _ => "501+",
    }
}

// ─────────────────────────── Verb classification ────────────────────────────

/// Map a raw constraint clause to its verb class. The clause's `kind` from
/// the extractor narrows the search; the raw text disambiguates within the
/// kind.
pub fn classify_verb(raw: &str, kind: ConstraintKind) -> VerbClass {
    match kind {
        ConstraintKind::ExpectChain => classify_expect_chain(raw),
        ConstraintKind::AssertCall => classify_assert_call(raw),
        ConstraintKind::AssertMacro => classify_assert_macro(raw),
        ConstraintKind::ZigTestingExpect => classify_zig_testing(raw),
        ConstraintKind::GuardThrow => VerbClass::Error,
    }
}

fn classify_expect_chain(raw: &str) -> VerbClass {
    // Look for `.toX(` in the raw clause. Cheap text-based match suffices
    // because tree-sitter already captured the call structure correctly;
    // we only need the matcher name.
    for pat in EQUIVALENCE_MATCHERS {
        if raw.contains(&format!(".{}(", pat)) || raw.contains(&format!(".{}<", pat)) {
            return VerbClass::Equivalence;
        }
    }
    for pat in TYPE_MATCHERS {
        if raw.contains(&format!(".{}(", pat)) || raw.contains(&format!(".{}<", pat)) {
            return VerbClass::TypeInstance;
        }
    }
    for pat in ERROR_MATCHERS {
        if raw.contains(&format!(".{}(", pat))
            || raw.contains(&format!(".{}<", pat))
            || raw.contains(&format!("{}(", pat))
        {
            return VerbClass::Error;
        }
    }
    for pat in EXISTENCE_MATCHERS {
        if raw.contains(&format!(".{}(", pat))
            || raw.contains(&format!(".{}<", pat))
            || raw.contains(&format!(".{} ", pat))
            || raw.ends_with(&format!(".{}", pat))
            || raw.contains(&format!(".{}()", pat))
        {
            return VerbClass::Existence;
        }
    }
    for pat in CONTAINMENT_MATCHERS {
        if raw.contains(&format!(".{}(", pat)) || raw.contains(&format!(".{}<", pat)) {
            return VerbClass::Containment;
        }
    }
    for pat in ORDERING_MATCHERS {
        if raw.contains(&format!(".{}(", pat)) || raw.contains(&format!(".{}<", pat)) {
            return VerbClass::Ordering;
        }
    }
    VerbClass::Other
}

fn classify_assert_call(raw: &str) -> VerbClass {
    let head = raw.trim_start();
    if head.starts_with("assert.equal")
        || head.starts_with("assertEqual")
        || head.starts_with("assertEquals")
        || head.starts_with("assert.strictEqual")
        || head.starts_with("assert.deepEqual")
    {
        return VerbClass::Equivalence;
    }
    if head.starts_with("assert.notEqual") || head.starts_with("assertNotEqual") {
        return VerbClass::Equivalence;
    }
    if head.starts_with("assert.throws") || head.starts_with("assertThrows") {
        return VerbClass::Error;
    }
    if head.starts_with("assert(") || head.starts_with("ok(") {
        return VerbClass::GenericAssertion;
    }
    VerbClass::GenericAssertion
}

fn classify_assert_macro(raw: &str) -> VerbClass {
    if raw.starts_with("assert_eq!") || raw.starts_with("debug_assert_eq!") {
        return VerbClass::Equivalence;
    }
    if raw.starts_with("assert_ne!") || raw.starts_with("debug_assert_ne!") {
        return VerbClass::Equivalence;
    }
    if raw.starts_with("panic!") || raw.starts_with("unreachable!") {
        return VerbClass::Error;
    }
    VerbClass::GenericAssertion
}

fn classify_zig_testing(raw: &str) -> VerbClass {
    if raw.contains("expectEqual") || raw.contains("expectEqualStrings") {
        return VerbClass::Equivalence;
    }
    if raw.contains("expectEqualSlices") {
        return VerbClass::Containment;
    }
    if raw.contains("expectError") {
        return VerbClass::Error;
    }
    if raw.contains("expectStringStartsWith") || raw.contains("expectStringEndsWith") {
        return VerbClass::Containment;
    }
    if raw.contains("expectApproxEqAbs") || raw.contains("expectApproxEqRel") {
        return VerbClass::Ordering;
    }
    VerbClass::GenericAssertion
}

const EQUIVALENCE_MATCHERS: &[&str] = &[
    "toBe",
    "toEqual",
    "toStrictEqual",
    "toEqualBytes",
    "toEqualText",
];
const TYPE_MATCHERS: &[&str] = &[
    "toBeInstanceOf",
    "toBeFunction",
    "toBeObject",
    "toBeArray",
    "toBeString",
    "toBeNumber",
    "toBeBoolean",
    "toBeSymbol",
    "toBeBigInt",
    "toBeDate",
    "toBeRegExp",
    "toBeArrayOfSize",
    "toBeIterable",
    "toBeAsyncIterable",
    "toBeTypeOf",
];
const ERROR_MATCHERS: &[&str] = &["toThrow", "toThrowError", "rejects"];
const EXISTENCE_MATCHERS: &[&str] = &[
    "toBeDefined",
    "toBeUndefined",
    "toBeNull",
    "toBeNullish",
    "toBeTruthy",
    "toBeFalsy",
    "toBeEmpty",
];
const CONTAINMENT_MATCHERS: &[&str] = &[
    "toContain",
    "toContainEqual",
    "toMatch",
    "toMatchObject",
    "toMatchSnapshot",
    "toMatchInlineSnapshot",
    "toHaveProperty",
    "toHaveLength",
    "toIncludePackageJson",
];
const ORDERING_MATCHERS: &[&str] = &[
    "toBeGreaterThan",
    "toBeGreaterThanOrEqual",
    "toBeLessThan",
    "toBeLessThanOrEqual",
    "toBeCloseTo",
    "toBeWithin",
];

// ─────────────────────────── Subject canonicalization ────────────────────────

/// Reduce a constraint's raw subject to a canonical form for grouping.
///
/// Strategy: drop function-call argument lists, parenthesized expressions,
/// trailing whitespace, common prefixes (`await `, `new `). Keep the
/// identifier path so `Bun.serve({...})`, `Bun.serve(opts)`, and `Bun.serve`
/// all canonicalize to `Bun.serve`. For TS/JS this works against the
/// extracted `subject` field. For Rust assert macros and Zig
/// testing.expect calls, the extractor's subject is already cleaned;
/// we apply the same shaping for consistency.
pub fn canonicalize_subject(subject: Option<&str>, raw: &str) -> String {
    let candidate = subject.unwrap_or(raw).trim();
    let stripped = strip_prefixes(candidate);
    let head = take_identifier_path(stripped);
    if head.is_empty() {
        "<anonymous>".to_string()
    } else {
        head.to_string()
    }
}

fn strip_prefixes(s: &str) -> &str {
    let mut s = s;
    loop {
        let trimmed = s.trim_start();
        if let Some(rest) = trimmed.strip_prefix("await ") {
            s = rest;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("new ") {
            s = rest;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("typeof ") {
            // `expect(typeof X).toBe('function')` — strip the operator so
            // the architectural subject (`X`) is what canonicalizes; the
            // structural value-side classifier still sees the matcher arg.
            s = rest;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("(") {
            if rest.trim_end().ends_with(')') {
                s = &rest[..rest.trim_end().len() - 1];
                continue;
            }
        }
        return trimmed;
    }
}

/// Take the longest leading sequence of `[A-Za-z_$][A-Za-z0-9_$]*` segments
/// joined by `.` — i.e. the identifier-path prefix. Stops at the first
/// non-identifier-non-dot character (e.g. `(`, `[`, `<`, whitespace).
fn take_identifier_path(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut last_dot_or_id = 0;
    let mut state = State::Start;
    while i < bytes.len() {
        let c = bytes[i];
        match state {
            State::Start | State::AfterDot => {
                if is_ident_start(c) {
                    state = State::InIdent;
                    last_dot_or_id = i + 1;
                    i += 1;
                } else {
                    break;
                }
            }
            State::InIdent => {
                if is_ident_cont(c) {
                    last_dot_or_id = i + 1;
                    i += 1;
                } else if c == b'.' {
                    state = State::AfterDot;
                    i += 1;
                } else {
                    break;
                }
            }
        }
    }
    &s[..last_dot_or_id]
}

#[derive(Copy, Clone)]
enum State {
    Start,
    InIdent,
    AfterDot,
}

fn is_ident_start(c: u8) -> bool {
    matches!(c, b'_' | b'$' | b'A'..=b'Z' | b'a'..=b'z')
}
fn is_ident_cont(c: u8) -> bool {
    matches!(c, b'_' | b'$' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z')
}

// ────────────────────── Construction-style classification ────────────────────

/// Classify a property as construction-style if its subject names a public
/// API surface and its verb-class is structural — either intrinsically
/// (TypeInstance, Existence, Error) or via a structural value side
/// (Equivalence verb where the value compared against is a type-name
/// string, primitive constant, or class reference). Otherwise behavioral.
fn classify_construction_style(subject: &str, verb: VerbClass, entries: &[Entry]) -> bool {
    if !is_public_surface(subject) {
        return false;
    }
    if matches!(
        verb,
        VerbClass::TypeInstance | VerbClass::Existence | VerbClass::Error
    ) {
        return true;
    }
    // Refinement: Equivalence with structural value side.
    // Examples:  expect(typeof Bun.serve).toBe("function")
    //            expect(Bun.foo).toBe(undefined)
    //            expect(server).toBeInstanceOf(Server)  // already TypeInstance
    //            expect(URL.canParse).toEqual(URL.canParse)
    // We sample the entries' raw text — if any representative reads as
    // structural-equivalence, the property is promoted.
    if matches!(verb, VerbClass::Equivalence) {
        let sample_count = entries.len().min(8);
        for e in entries.iter().take(sample_count) {
            if has_structural_equivalence_value(&e.raw) {
                return true;
            }
        }
    }
    false
}

/// Returns true when the equivalence-clause `raw` compares its subject
/// against a structural value: a type-name string literal (`"function"`,
/// `"object"`, `"number"`, …), a primitive constant (`null`, `undefined`,
/// `NaN`, `true`, `false`), or a capitalized identifier that likely names
/// a class/constructor.
fn has_structural_equivalence_value(raw: &str) -> bool {
    // Find the first `.toBe(`, `.toEqual(`, `.toStrictEqual(` and read its
    // first argument. The opening paren is just past the matcher name; we
    // collect text until a balancing `)` or top-level `,`.
    let matchers = ["toBe", "toEqual", "toStrictEqual"];
    for m in matchers {
        let needle = format!(".{}(", m);
        if let Some(idx) = raw.find(&needle) {
            let after = &raw[idx + needle.len()..];
            let arg = first_call_arg(after);
            if is_structural_value(arg.trim()) {
                return true;
            }
        }
    }
    // Deno-style and Node-assert-style: the structural value lives in the
    // second positional argument: `assertEquals(subject, "function")`.
    let assert_matchers = [
        "assertEquals(",
        "assertStrictEquals(",
        "assertNotEquals(",
        "assert.equal(",
        "assert.strictEqual(",
        "assert.deepEqual(",
        "assert.deepStrictEqual(",
    ];
    for m in assert_matchers {
        if let Some(idx) = raw.find(m) {
            let after = &raw[idx + m.len()..];
            let first = first_call_arg(after);
            let rest_start = idx + m.len() + first.len();
            if rest_start < raw.len() && raw.as_bytes()[rest_start] == b',' {
                let after_comma = &raw[rest_start + 1..];
                let second = first_call_arg(after_comma);
                if is_structural_value(second.trim()) {
                    return true;
                }
            }
        }
    }
    false
}

/// Take the first comma-separated argument of a call, accounting for
/// nested `()`, `[]`, `{}`. `s` is the text immediately after the opening
/// `(`. Returns the substring up to (but not including) the terminating
/// `,` or balancing `)`.
fn first_call_arg(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut depth: i32 = 0;
    let mut in_str: Option<u8> = None;
    let mut prev_backslash = false;
    let mut end = 0;
    for (i, &c) in bytes.iter().enumerate() {
        if let Some(q) = in_str {
            if c == b'\\' && !prev_backslash {
                prev_backslash = true;
                continue;
            }
            if c == q && !prev_backslash {
                in_str = None;
            }
            prev_backslash = false;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => {
                in_str = Some(c);
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                if depth == 0 {
                    end = i;
                    break;
                }
                depth -= 1;
            }
            b',' if depth == 0 => {
                end = i;
                break;
            }
            _ => {}
        }
    }
    if end == 0 && !s.is_empty() {
        // Reached end of string with no terminator (truncated raw); return all.
        s
    } else {
        &s[..end]
    }
}

fn is_structural_value(arg: &str) -> bool {
    if arg.is_empty() {
        return false;
    }
    // Type-name string literal.
    let unquoted = strip_quotes(arg);
    if matches!(
        unquoted,
        "function"
            | "object"
            | "string"
            | "number"
            | "boolean"
            | "undefined"
            | "symbol"
            | "bigint"
    ) {
        return true;
    }
    // Primitive constants.
    if matches!(arg, "null" | "undefined" | "NaN" | "true" | "false") {
        return true;
    }
    // Capitalized identifier (likely class/constructor reference). Use a
    // conservative test: starts with ASCII uppercase, followed only by
    // identifier characters and dots. Excludes calls like `Foo()`.
    let bytes = arg.as_bytes();
    if !bytes.is_empty() && (bytes[0] as char).is_ascii_uppercase() {
        if arg
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '$' || c == '.')
        {
            return true;
        }
    }
    false
}

fn strip_quotes(s: &str) -> &str {
    let s = s.trim();
    if s.len() < 2 {
        return s;
    }
    let bytes = s.as_bytes();
    let first = bytes[0];
    let last = bytes[s.len() - 1];
    if (first == b'"' || first == b'\'' || first == b'`') && first == last {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

fn is_public_surface(subject: &str) -> bool {
    let head = subject.split('.').next().unwrap_or("");
    if head.is_empty() {
        return false;
    }
    PUBLIC_API_HEADS.iter().any(|s| *s == head)
        || PUBLIC_API_NAMESPACES
            .iter()
            .any(|prefix| subject.starts_with(prefix))
}

const PUBLIC_API_HEADS: &[&str] = &[
    // Bun runtime
    "Bun",
    // Deno runtime
    "Deno",
    // Web-platform globals
    "URL",
    "URLSearchParams",
    "Request",
    "Response",
    "Headers",
    "Blob",
    "File",
    "FormData",
    "ReadableStream",
    "WritableStream",
    "TransformStream",
    "ByteLengthQueuingStrategy",
    "CountQueuingStrategy",
    "TextEncoder",
    "TextDecoder",
    "AbortController",
    "AbortSignal",
    "WebSocket",
    "Worker",
    "MessagePort",
    "MessageChannel",
    "BroadcastChannel",
    "EventTarget",
    "Event",
    "CustomEvent",
    "fetch",
    "structuredClone",
    "queueMicrotask",
    "atob",
    "btoa",
    "performance",
    "console",
    // Node compat globals + namespaces
    "Buffer",
    "process",
    "global",
    "globalThis",
    "setTimeout",
    "setInterval",
    "setImmediate",
    "clearTimeout",
    "clearInterval",
    "clearImmediate",
    // Common Node-compat module names (when imported and called bare)
    "fs",
    "path",
    "http",
    "https",
    "http2",
    "net",
    "tls",
    "dgram",
    "dns",
    "url",
    "querystring",
    "crypto",
    "zlib",
    "stream",
    "events",
    "util",
    "os",
    "child_process",
    "cluster",
    "worker_threads",
    "vm",
    "v8",
    "perf_hooks",
    "async_hooks",
    "buffer",
    "inspector",
    "module",
    "readline",
    "repl",
    "tty",
    "string_decoder",
    "punycode",
    "assert",
    "timers",
    // Test API
    "expect",
    "test",
    "it",
    "describe",
    "beforeEach",
    "afterEach",
    "beforeAll",
    "afterAll",
    "mock",
    "spyOn",
];

const PUBLIC_API_NAMESPACES: &[&str] = &[
    "Bun.",
    "fs.",
    "path.",
    "http.",
    "https.",
    "crypto.",
    "process.",
    "Buffer.",
    "stream.",
    "URL.",
    "Response.",
    "Request.",
    "Headers.",
    "Worker.",
    "WebSocket.",
];
