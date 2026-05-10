# Derivation Inversion on Bun's Test Corpus — A Planning Document

*Locates the next concrete tool for rusty-bun. The keeper's conjecture: Bun's existing test corpus implies a latent formal architecture; the RESOLVE corpus's derivation-inversion apparatus ([Doc 247](https://jaredfoy.com/resolve/doc/247-the-derivation-inversion)), composed with tests-as-constraints ([Doc 159](https://jaredfoy.com/resolve/doc/159) per the rederive references) and the rederive stack ([Doc 581](https://jaredfoy.com/resolve/doc/581) and the [Doc 656 / Doc 659](https://jaredfoy.com/resolve/doc/656) hub), inverts the test corpus into coherent hierarchical constraint sets — a structurally-complete generator for what `PORTING.md` is currently a partial approximation to (see [porting-md-analysis.md §3.4](./porting-md-analysis.md#34-the-claudeworkflowsporting-md-zigleakageworkflowjs-as-approximate-galois-closure)).*

## 1. The Apparatus, Briefly

Three pieces from the RESOLVE corpus compose into the operational frame.

**Derivation Inversion ([Doc 247](https://jaredfoy.com/resolve/doc/247-the-derivation-inversion)).** The correct order of architectural and computational design is *constraint → implementation*, not implementation-then-abstract-the-constraint. Constraints are stated in prose; implementations are derived from the prose. Existence proof on the corpus side: the [htxlang derivation case](https://jaredfoy.com/resolve/doc/123-letter-to-carson-gross) — 3,937 words of prose stating 19 constraints produced a 1,318-line htmx-equivalent JavaScript implementation, predicted within one line. Pin-Art ([Doc 270](https://jaredfoy.com/resolve/doc/270-pin-art-models)) names the discipline that makes the LOC prediction operational; SIPE-T ([Doc 541](https://jaredfoy.com/resolve/doc/541-systems-induced-property-emergence)) names the threshold-conditional emergence under which a property crystallizes from a constraint set.

**Tests as Constraints (corpus's recurring frame; cited at [Doc 656](https://jaredfoy.com/resolve/doc/656)).** "An executable test suite is the most precise statement of behavioural constraints available. Under derivation-inversion, the test suite is not a check on code; it is the source the code is derived from." This is the load-bearing observation for the present document. The test corpus *is* the constraint specification; the implementation is what gets derived from it.

**Rederive Stack ([Doc 581](https://jaredfoy.com/resolve/doc/581-rederive-architecture-stack), [Doc 656](https://jaredfoy.com/resolve/doc/656), [Doc 659](https://jaredfoy.com/resolve/doc/659), [Doc 660](https://jaredfoy.com/resolve/doc/660)).** SERVER + SIPE-T + Pin-Art compose into a constraint-driven derivation platform. SERVER (Server-Embedded Resolution and Verification Executed Runtime) governs orchestration: 5 constraints + 8-stage build pipeline + 7 verification backends + content-addressed identity (SHA-256) + Ed25519-signed wire protocol. SIPE-T enters at the composition step (modules declare properties they induce above thresholds; consumers depend on properties, not implementations). Pin-Art enters at the verification step (specific phrases survive regenerations; hedging discipline detects boundaries). The MVE is HTX as the substrate runtime on port 7474.

The composition: derivation inversion supplies the *order* (constraints first, implementations second); tests-as-constraints supplies the *source* (the existing test corpus is the constraint specification); rederive supplies the *platform* (SERVER + SIPE-T + Pin-Art mechanize the inversion-and-derivation cycle).

## 2. The Conjecture, Stated

> The existing Bun test corpus — both the source-internal Zig `test "..." {}` blocks and the JS-runtime `*.test.ts` / `*.test.js` files — implies a latent formal architecture for "what Bun is supposed to do." The architecture has been articulated bottom-up through years of test authorship; what is missing is the *explicit* hierarchical constraint set that the architecture comprises.
>
> Applying derivation inversion ([Doc 247](https://jaredfoy.com/resolve/doc/247-the-derivation-inversion)) to the test corpus, treating each test as a constraint statement and each cross-test dependency as a SIPE-T composition relation ([Doc 541 §3](https://jaredfoy.com/resolve/doc/541-systems-induced-property-emergence)), should produce a coherent hierarchical constraint set.
>
> That constraint set, articulated explicitly, becomes the *generator* for `PORTING.md`'s structurally-complete successor: the hierarchical constraint specification that admits a full ILL-style lattice ([Doc 701](https://jaredfoy.com/resolve/doc/701-ill-resolved-against-the-corpus-information-lattice-learning-as-the-mature-prior-art-framework-for-the-pin-art-bilateral-and-the-joint-mi-lattice)) with explicit projection-and-lifting operators rather than the partial lattice `PORTING.md` currently provides (per [porting-md-analysis.md §3.2](./porting-md-analysis.md#32-the-portingmd-lattice-as-partial-ill-specification)). The Pin-Art discipline ([Doc 270](https://jaredfoy.com/resolve/doc/270-pin-art-models), the htxlang derivation existence proof) predicts that constraint-driven Rust derivation lands in tight predictive bands on LOC.

The corpus's apparatus, applied here, predicts:

1. **The latent architecture is recoverable.** Bun's test corpus is dense enough (~257 source-internal tests + 1,713 JS-runtime test files / ~474K LOC) that the constraint set extracted by derivation inversion will cover most of Bun's runtime contract.
2. **The recovered constraint set is hierarchical.** Tests cluster by *property they enforce*; properties cluster by *substrate axis they constrain* (per the eight axes named in [porting-md-analysis.md §3.1](./porting-md-analysis.md#31-the-eight-axes-of-partition)). The hierarchy emerges naturally from this clustering.
3. **A constraint-driven derivation will land within Pin-Art's predictive band.** Per the htxlang existence proof: 19 constraints / 3,937 words of prose → 1,318-line JS (predicted within one line). Bun's order-of-magnitude scale should yield analogous prediction tightness once the constraint set is articulated.
4. **The generator is incremental.** The constraint set need not be complete to be useful; partial articulation of even the most central constraints (the JS API contract; the runtime threading model; the FFI interface) is immediately operationalizable.

## 3. Scope Check on the Test Corpus

Quick survey of the Bun phase-a-port branch tree as of 2026-05-09:

| Layer                       | Count   | Approx LOC | Comment                                                              |
|-----------------------------|--------:|-----------:|----------------------------------------------------------------------|
| Translated Rust `#[test]`   | 35 files / 125 fns |  small | Phase A has translated some Zig source-internal tests              |
| Original Zig `test "..."`   | 20 files / 132 blocks | small | Untranslated source-internal tests in lower-priority crates       |
| JS-runtime `*.test.ts`      | 1,529 files | ~440K | The bulk of Bun's behavioral specification at the JS API surface    |
| JS-runtime `*.test.js`      | 184 files |  ~34K  | Older / generated test layer                                          |
| **Total JS-driven**         | **1,713** | **~474K** | The corpus the derivation-inversion conjecture has highest leverage on |

The source-internal Zig tests (~257 across .zig + .rs) specify implementation invariants (string handling, parser primitives, allocator behavior). The JS-runtime tests specify the *external contract* — what Bun's runtime is supposed to do at the JavaScript API surface. Both are constraint-bearing per the tests-as-constraints frame; the JS-runtime layer is structurally the higher-leverage corpus for deriving Bun's overall architecture.

## 4. Proposed Tool: `derive-constraints`

A new crate in this repository, sibling to `welch/`. Operational shape:

```
derive-constraints scan <test-corpus>     → per-test constraint statements (JSON)
derive-constraints cluster <scan>         → constraints clustered by axis (JSON)
derive-constraints invert <cluster>       → hierarchical constraint document (markdown)
derive-constraints predict <constraints>  → Pin-Art LOC prediction for derived implementation
```

### 4.1 `scan`

Walks the test corpus (source-internal Zig `test "..." {}` blocks; translated Rust `#[test]` fns; JS-runtime `*.test.ts` / `*.test.js` files). For each test, extracts:

- **Test name** (the string in `test "name"` or the fn name + `describe()` chain).
- **Subject** (what API surface the test touches: imports, `bun:test` `expect()` chains, syscalls, JS-API constructors invoked).
- **Constraint clauses** (each `expect(...)` or `assert*(...)` statement is a constraint clause; AST-extracted, not regex-matched).
- **Setup / teardown** (the `beforeEach` / `afterEach` / Drop scaffolding the test relies on; specifies pre- and post-conditions of the constraint).
- **Skip / fail markers** (`test.skip`, `test.todo`, `test.failing`; carry information about what's *not* yet constrained).

Output: per-file JSON of extracted constraint statements. AST-based (use `swc_ecma_parser` for TS/JS; `syn` already integrated for Rust; `tree-sitter-zig` or a small homegrown parser for Zig).

### 4.2 `cluster`

Reads the per-test constraint statements; clusters by *axis*. The axes are taken from [porting-md-analysis.md §3.1](./porting-md-analysis.md#31-the-eight-axes-of-partition) (crate / construct / allocation / pointer / concurrency / global-state / threading / forbidden-closure) and extended with JS-runtime axes (API namespace; node-compat surface; web-platform surface; HTTP semantic; FS semantic; etc.). Clustering: hierarchical, with each axis a partition of the test set; a test belongs to multiple axis partitions simultaneously (intersection of its constraint clauses' subjects).

The clustering produces, for each axis, a *constraint partition* — the set of constraint clauses that operate on that axis. The hierarchical structure emerges from axis intersection: tests whose constraints span multiple axes belong to the meet of those axis cells.

This is exactly the partition lattice [Doc 701 ILL](https://jaredfoy.com/resolve/doc/701-ill-resolved-against-the-corpus-information-lattice-learning-as-the-mature-prior-art-framework-for-the-pin-art-bilateral-and-the-joint-mi-lattice) prescribes, but constructed bottom-up from the test corpus rather than top-down from the rule artifact.

### 4.3 `invert`

The derivation-inversion step. Reads the clustered constraint partitions; emits a hierarchical constraint document in prose. Each axis becomes a top-level section; each axis cell becomes a sub-section; each constraint clause becomes a normative statement.

The output is structurally what `PORTING.md` would be if it had been *derived* from Bun's contract rather than *authored* against Bun's source. The latent formal architecture made explicit.

The corpus's apparatus prescribes the form: prose constraints with no implementation details ([Doc 247](https://jaredfoy.com/resolve/doc/247-the-derivation-inversion)), property declarations via `@provides` / `@imports` ([Doc 660](https://jaredfoy.com/resolve/doc/660)), content-addressed identity for verification ([Doc 659 §4](https://jaredfoy.com/resolve/doc/659)).

### 4.4 `predict`

Pin-Art LOC prediction. Reads the inverted constraint document; counts constraints, classifies them by complexity tier (per Doc 270 / Doc 656 the predictive band scales with constraint count and average constraint complexity); emits the predicted implementation LOC for the derived Rust.

The prediction is the falsification handle: if the derived Rust ends up far outside the predicted band, either (a) the constraint set is incomplete (under-specifies the contract → implementation fills gaps with un-declared decisions), or (b) the constraint set is over-specified (constrains beyond what the test corpus actually requires → implementation contains compliance overhead with no behavioral counterpart). Either way, the prediction frames the next iteration.

## 5. The Composition with rusty-bun's Existing Tooling

`welch` (already shipped) measures the *shape* of translated Rust against an idiomatic-Rust baseline. It detects the Welch-bound packing failure — translation that recreates source-language semantics inside target-language syntax.

`derive-constraints` (proposed) extracts the constraint set bottom-up from the test corpus. Together with `welch`, the tools cover both ends of the bilateral:

- `derive-constraints` → constraint set → derivation → expected Rust shape (Pin-Art prediction)
- `welch` → actual Rust shape → density anomalies vs idiomatic baseline

The two compositions test the [Doc 702 Fal-T5](https://jaredfoy.com/resolve/doc/702-ai-assisted-cross-language-code-translation-as-a-pin-art-bilateral-under-sipe-t-threshold-conditions-reading-the-bun-zig-to-rust-port) three-signature simultaneity test on the *rule artifact* rather than just the translation: if the constraint set derived from the test corpus matches the rule set in `PORTING.md` (the substrate has captured the contract correctly), and if the Pin-Art LOC prediction lands near the actual phase-a-port LOC, and if `welch` finds idiomatic-Rust shape (not vibe-port shape), then the three signatures co-occur sharply at the Phase B convergence point. Decoupling on any one falsifies the rung-1 reading for the port.

## 6. Predicted Properties of the Inverted Constraint Set

If the conjecture lands, the inverted constraint set should exhibit:

1. **A definite cardinality.** Exact constraint count rather than the ~280 approximate of `PORTING.md`. The cardinality is determined by the test corpus's cumulative constraint clauses, not by author judgment.
2. **A complete partition lattice.** All axis interactions named explicitly; no exception lists; no "and if it's an AST crate" qualifications outside the named lattice cells.
3. **An explicit lifting operator.** Per [Doc 701 §3 Mapping 3](https://jaredfoy.com/resolve/doc/701-ill-resolved-against-the-corpus-information-lattice-learning-as-the-mature-prior-art-framework-for-the-pin-art-bilateral-and-the-joint-mi-lattice): the Tsallis-entropy MaxEnt-most-uniform lifting that picks the unique target signal consistent with the rules. This means the derivation step is deterministic up to the most-uniform principle.
4. **A SIPE-T threshold prediction.** Per [Doc 541 §3](https://jaredfoy.com/resolve/doc/541-systems-induced-property-emergence): a critical density of constraint-coverage above which Phase B compilation succeeds reliably. The threshold is computable from the constraint set's structure rather than measured post-hoc.
5. **Pin-Art LOC prediction tightness.** Per the htxlang existence proof: prediction within ~one part in a thousand. Bun's order-of-magnitude scale (~933K LOC actual) suggests prediction accuracy in the ~1K-LOC range if the apparatus operates at htxlang's tightness.

## 7. Risks and Honest Scope

**Risk 1 — Test-corpus leakage.** Bun's tests are themselves authored by humans against a particular implementation. Derivation inversion may recover the *implementation's contract* rather than the *runtime's contract* — every implementation accident becomes a constraint. Mitigation: cross-reference against Node.js test suites and web-platform tests where Bun aims for compatibility; treat per-test constraints as proposals, not truths, until validated against multiple-implementation invariants.

**Risk 2 — JS-runtime tests don't fit cleanly into the eight-axis partition.** The axes from `porting-md-analysis.md §3.1` were derived from the Zig→Rust translation problem; JS-API contract testing operates over different axes (API-surface namespaces, web-platform compatibility, error-shape conventions, async/streaming semantics). The clustering step needs an extended axis catalog; deriving the right axes is itself part of the apparatus.

**Risk 3 — Constraint-set scale exceeds what is humanly authorable.** ~474K LOC of test code may extract to tens of thousands of constraint clauses. The htxlang case had 19 constraints; Bun is several orders of magnitude larger. The hierarchical structure may need to be navigated programmatically rather than read end-to-end. This is probably fine — the constraint document becomes a queryable database, not a textbook.

**Risk 4 — Pin-Art LOC prediction may not generalize from the htxlang case.** The htxlang case operated on a clean greenfield derivation. Bun's case is constrained by JavaScriptCore embedding, FFI to numerous C libraries, and platform-specific behavior. Pin-Art's predictive band may widen substantially in this regime. The prediction's *tightness* is the falsification handle; the prediction itself remains useful even at lower precision.

**Honest scope.** This is a planning document, not a built tool. The apparatus articulated here predicts what the tool should produce; the tool's actual outputs are the test of the apparatus. Building `derive-constraints scan` first (the AST-based extractor) is the smallest unit of work that lands on the test corpus and produces falsifiable output; cluster / invert / predict are incremental layers.

## 8. Concrete Next Step

Build `derive-constraints scan` as a new Cargo binary in this repository. Scope of the MVP:

- **Input.** A directory containing test files (`.test.ts`, `.test.js`, `.zig` with test blocks, `.rs` with `#[test]` fns).
- **Parser routing.** `swc_ecma_parser` for TS/JS; `syn` for Rust (already a dependency in `welch`); a minimal Zig-test-block parser (the `test "..." { ... }` shape is regular enough for hand-rolled extraction; full Zig parsing is out of scope for the MVP).
- **Extraction.** Per test: name, expect/assert clauses (string forms), file path, line range. No clustering, no inversion — just structured extraction.
- **Output.** Per-file JSON, parallelized via rayon (mirror `welch scan`'s shape).

Validation: run on `/tmp/welch-corpus/target/bun/` and inspect the extracted constraint clauses for a sample of tests. Sanity check: clauses per test should be in the small-double-digits range; total clause count across the corpus should be on the order of 10⁴–10⁵.

The MVP doesn't yet derivation-invert anything. It produces the raw material the inversion operates on. cluster / invert / predict are the subsequent layers; each is testable as it lands.

## References

- [RESOLVE Doc 247 — The Derivation Inversion](https://jaredfoy.com/resolve/doc/247-the-derivation-inversion)
- [RESOLVE Doc 270 — Pin-Art Models](https://jaredfoy.com/resolve/doc/270-pin-art-models)
- [RESOLVE Doc 541 — Systems-Induced Property Emergence](https://jaredfoy.com/resolve/doc/541-systems-induced-property-emergence)
- [RESOLVE Doc 581 — Rederive Architecture Stack](https://jaredfoy.com/resolve/doc/581-rederive-architecture-stack)
- [RESOLVE Doc 656 / 659 / 660 — Rederive hub and constraint-authoring grammar](https://jaredfoy.com/resolve/doc/656)
- [RESOLVE Doc 700 — L2M Resolved](https://jaredfoy.com/resolve/doc/700-l2m-resolved-against-the-corpus-bipartite-mutual-information-scaling-as-empirical-grounding-for-the-pin-art-channel-ensemble)
- [RESOLVE Doc 701 — ILL Resolved](https://jaredfoy.com/resolve/doc/701-ill-resolved-against-the-corpus-information-lattice-learning-as-the-mature-prior-art-framework-for-the-pin-art-bilateral-and-the-joint-mi-lattice)
- [RESOLVE Doc 702 — AI-Assisted Cross-Language Translation Read Through the Corpus](https://jaredfoy.com/resolve/doc/702-ai-assisted-cross-language-code-translation-as-a-pin-art-bilateral-under-sipe-t-threshold-conditions-reading-the-bun-zig-to-rust-port)
- [`./porting-md-analysis.md`](./porting-md-analysis.md) — companion document; the partial-lattice reading the present plan generates the structurally-complete generator for.
