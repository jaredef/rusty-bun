# Cluster Phase Design — Property-Induction Filtering, Not Axis Classification

*Refines the planning at [`derivation-inversion-on-bun-tests.md §4.2`](./derivation-inversion-on-bun-tests.md#42-cluster). The keeper's conjecture: derived constraint sets can induce all the properties necessary for the runtime contract without over-fitting. The htmx→htxlang precedent demonstrates the ratio (9.4%); PRESTO's construction-style constraints supply the apparatus that makes it operational.*

## 1. The htmx → htxlang Precedent, Numerically

| Artifact                  | LoC       | Source                          |
|---------------------------|----------:|---------------------------------|
| htmx                      | 14,000    | [Doc 288](https://jaredfoy.com/resolve/doc/288-the-pin-art-derivation) line 39 |
| htxlang derivation (full) |  1,318    | [Doc 288](https://jaredfoy.com/resolve/doc/288-the-pin-art-derivation) line 5 |
| Constraint set (prose)    | 3,937 words / 19 constraints | [Doc 656](https://jaredfoy.com/resolve/doc/656) line 43 |

**Ratio: 9.4%.** The htxlang derivation produces 100% of htmx's behavior in 9.4% of htmx's code, derived from 19 constraints stated in ~4K words of prose. The size difference is exactly what [Doc 288 line 175](https://jaredfoy.com/resolve/doc/288-the-pin-art-derivation) names: "Both produce the same behavior. One carries 11 years of accretions. The other carries 19 sentences."

The 90.6% reduction is not compression. It is the difference between *implementation as accumulated history* and *implementation as derived consequence of the constraint set*. The corpus's standing position per [Doc 247 (Derivation Inversion)](https://jaredfoy.com/resolve/doc/247-the-derivation-inversion): the correct order of work is from *constraint (form)* to *implementation (instance)*, not from implementation to constraint by abstraction. When the inversion is performed, the derived implementation carries no incidental complexity from the original's implementation history; only what the constraints require.

## 2. The Conjecture Made Operational

> The 43,094 constraint clauses [`derive-constraints scan` extracted](../runs/2026-05-10-bun-derive-constraints/RUN-NOTES.md) from the Bun test corpus include heavy duplication and substantial incidental over-specification. Most of those clauses do *not* induce architectural properties Bun's runtime must have; they specify particular test fixtures, particular numerical expectations, particular implementation-defined behaviors that the test corpus happens to lock down without architectural reason.
>
> The *necessary* constraint set — the minimal antichain whose composition induces all properties the runtime must possess — is much smaller. Probably 2–3 orders of magnitude smaller (10² to 10³ rather than 10⁴).
>
> Applying [Doc 445 pulverization](https://jaredfoy.com/resolve/doc/445-pulverization-formalism)'s minimal-antichain operation, [Doc 541 SIPE-T](https://jaredfoy.com/resolve/doc/541-systems-induced-property-emergence)'s property-induction discipline, and the [PRESTO construction-style frame](https://jaredfoy.com/resolve/doc/420) is the apparatus that distinguishes necessary constraints from incidental ones.

The cluster phase, as originally designed in the planning doc, was framed as axis-classification clustering — group constraints by the lattice axis they touch. That is one part of the work but not the load-bearing part. The load-bearing part is **property-induction filtering**: drop constraints that don't contribute to property threshold-crossing.

## 3. PRESTO's Construction-Style Constraints

PRESTO ([Doc 185](https://jaredfoy.com/resolve/doc/185), [Doc 420](https://jaredfoy.com/resolve/doc/420), [Doc 123](https://jaredfoy.com/resolve/doc/123-letter-to-carson-gross)) defines five construction-style constraints that govern how a server-side resolution engine is built:

- **C1 — Bilateral Boundary.** Server and client namespaces are formally distinct; content in one does not interfere with content in the other.
- **C2 — Namespace Separation.** Each interpreter identifies its instructions by a consistent convention (`htx:` for server directives; standard HTML for client content).
- **C3 — Server-Consumed Directives.** All engine directives are consumed during resolution; the HTTP response contains no engine-specific syntax.
- **C4 — Progressive Code-on-Demand.** Client behavior is added progressively; the base document is complete HTML; JavaScript enhances a document that already works without it.
- **C5 — Server-Embedded Authorization.** Authentication and authorization are resolved during processing, before the response is sent.

These five constraints induce the property *ambivalent execution with agnostic determinism* — the engine is provably correct by construction and produces deterministic results regardless of which client renders them. The 22-stage pipeline ([Doc 185 line 130](https://jaredfoy.com/resolve/doc/185)) is **contingent**, not constraint-prescribed: "The constraint is deterministic ordering, not a specific stage count." Any number of stages works as long as the dependency relationships honor the construction constraints.

This is the load-bearing distinction for the present design. **Construction-style constraints prescribe what must hold across the engine's structure** (form / necessity); **behavioral constraints specify what the engine produces under particular inputs** (instance / contingency). PRESTO's five constraints are construction-style. The 22 pipeline stages are behavioral consequences. The htxlang derivation's 19 constraints are construction-style for htmx's behavior surface; the 14,000 lines of htmx are behavioral instances.

For Bun: the runtime's construction-style constraints are properties like *bun:test API surface contract*, *Node.js compatibility surface*, *FFI safety contract*, *event-loop ordering invariants*, *FS atomicity guarantees*, *HTTP semantic conformance*. These are the properties Bun must induce. The 43K extracted clauses are behavioral instances — particular tests of particular inputs against particular outputs that *together* span enough of the construction-style property space to certify the runtime, but most of which are individually over-specifying.

## 4. The Property-Induction Filter

The cluster phase becomes a four-step filter rather than a single classification step.

### Step 1 — Property identification

For each extracted constraint, identify the *property it tests* — the architectural invariant whose violation that constraint would expose. A constraint like `expect(Bun.serve({ port: 3000 })).toBeInstanceOf(Server)` tests the property *Bun.serve accepts a port number and returns a Server instance*. A constraint like `expect(result).toBe(42)` tests… nothing architectural; it tests a particular numerical expectation specific to its test fixture.

Properties are aggregations across many constraints. The same property can be tested by hundreds of constraint clauses with different specific values. The cluster phase groups constraints by induced property.

Practical extraction heuristic: a property is identifiable from the constraint's *subject* (the first argument of `expect(...)`) plus the *verb* (the matcher: `.toBe`, `.toBeInstanceOf`, `.toThrow`, `.toEqual`, etc.). The subject names what the property is about; the verb names the kind of property (existence, type, equivalence, error, etc.). The cluster phase's first move is property-canonicalization across constraints with congruent subject-verb shapes.

### Step 2 — Threshold diagnosis

Per [Doc 541 SIPE-T](https://jaredfoy.com/resolve/doc/541-systems-induced-property-emergence): for each property, identify the constraint-coverage threshold ρ\* at which the property emerges. Below the threshold, partial coverage does not induce the property reliably; above it, the property is operationally accessible.

For most Bun runtime properties, the threshold is empirical — it's the minimum constraint coverage at which the runtime's implementation can be derived without ambiguity. Properties differ in how many constraints are needed to induce them: *Bun.serve accepts a port* needs ~1 constraint; *Bun's HTTP/1.1 implementation handles chunked encoding correctly* needs many more, because the chunked-encoding semantic surface is wide.

The threshold diagnosis is the constraint-cardinality analog of [Doc 700 L2M](https://jaredfoy.com/resolve/doc/700-l2m-resolved-against-the-corpus-bipartite-mutual-information-scaling-as-empirical-grounding-for-the-pin-art-channel-ensemble)'s capacity bound applied at the property level: how many constraint clauses are needed to specify the property tightly enough that derivation is unambiguous?

### Step 3 — Minimal antichain selection

Per [Doc 445 pulverization](https://jaredfoy.com/resolve/doc/445-pulverization-formalism) and [Doc 701 ILL](https://jaredfoy.com/resolve/doc/701-ill-resolved-against-the-corpus-information-lattice-learning-as-the-mature-prior-art-framework-for-the-pin-art-bilateral-and-the-joint-mi-lattice): for each property, select a minimal antichain of constraints whose composition is sufficient to induce the property. The antichain is irreducible: removing any constraint drops coverage below the threshold; adding any constraint is dispensable (because the existing antichain already induces the property).

This is the operation that drops incidental constraints. A test asserting `expect(server.port).toBe(3000)` and a test asserting `expect(server.port).toBe(8080)` are both behavioral instances of the same construction-style property (*the port is exposed as a numeric attribute*). The minimal antichain keeps one or zero of these (depending on whether the type-existence is already covered by another constraint); the rest are incidental.

The htmx→htxlang ratio (9.4%) is the empirical signal of how much reduction is plausible at the *implementation* level when the constraint set is property-minimal. The cluster phase's reduction at the *constraint-set* level should be substantially larger — closer to the inverse of the constraint-set's redundancy, which on the Bun corpus is in the 100×–1000× range based on test-corpus duplication patterns.

### Step 4 — Construction-style classification

Once the minimal antichain is selected, classify each surviving constraint as construction-style or behavioral.

A construction-style constraint:
- States a property at the engine's structural level (what the engine *is*, how it's *built*).
- Is portable across implementations (true of any conformant runtime, not just one).
- Composes by induced-property semantics ([Doc 541 §3 SIPE-T](https://jaredfoy.com/resolve/doc/541-systems-induced-property-emergence)): downstream consumers depend on the property, not the implementation.

A behavioral constraint:
- States a property at the engine's input-output behavior layer (what the engine *does* under specific input).
- May not be portable across implementations (different conformant runtimes might handle the case differently).
- Is the *consequence* of the construction-style constraints, not part of them.

The classification is not a binary; properties exist at varying degrees of structurality. The PRESTO precedent: 5 explicitly construction-style constraints; 19 in the htmx case (which mixed construction and behavior).

For Bun, expected counts:
- Construction-style: ~50–200 (the JS runtime contract, the FFI contract, the threading model, the event-loop semantics, the FS atomicity guarantees, the HTTP semantic surface, the bun:test API contract).
- Behavioral: the rest of the antichain, likely ~10²–10³ more.

Together, the construction-style + behavioral antichain is the candidate replacement for `PORTING.md` — a hierarchical constraint document that is structurally complete, derivation-inverted from the test corpus, and bounded in size by the property-induction discipline rather than by the corpus's accumulated specification volume.

## 5. Pin-Art LOC Re-prediction

The earlier pre-prediction in `runs/2026-05-10-bun-derive-constraints/RUN-NOTES.md` extrapolated from htxlang's ~70 LOC/constraint ratio against 43K extracted constraints, yielding ~3M LOC predicted. That prediction was wrong because the input was wrong: the 43K is pre-filter constraint count, not the construction-style minimal antichain.

Re-prediction with the property-induction filter applied:

- htxlang case: **19 constraints → 1,318 LOC** = 69 LOC/constraint.
- Bun case (estimated minimal antichain): **~10²–10³ construction-style + behavioral constraints** at 50–100 LOC/constraint = **5,000–100,000 LOC** of derived implementation for the JS runtime contract surface (excluding FFI shims to JavaScriptCore, BoringSSL, libuv, zlib, zstd, etc., which are bound by the C library's interface, not by constraint-derivation).
- Observed phase-a-port: 933,000 LOC including all FFI shims; subtracting the FFI bindings yields perhaps 300,000–500,000 LOC of pure-Rust runtime logic. The constraint-derived prediction is below this by a factor of 3–10×, which is in the htmx→htxlang ratio range.

The 3-orders-of-magnitude gap between extracted-clause count (43K) and predicted minimal antichain (10²–10³) is the central operational question. The cluster phase produces the empirical answer.

## 6. Concrete Cluster-Phase MVP

A new subcommand on `derive-constraints`:

```
derive-constraints cluster <scan.json> -o cluster.json
```

Pipeline:

1. **Load scan.json.** Read the per-file extraction output from `derive-constraints scan`.
2. **Property canonicalization.** For each constraint, compute a *property key* = (canonicalized subject, verb-class). E.g., `expect(server.port).toBe(3000)` and `expect(server.port).toBe(8080)` both produce key `("server.port", "toBe-numeric")`. Group constraints by property key.
3. **Threshold heuristic (initial: per-property minimum-of-1).** For MVP, treat each property as needing at least 1 constraint (presence) and at most N constraints (where N is a tunable richness parameter, default 3 — keep enough to cover edge cases without over-specifying).
4. **Minimal-antichain selection per property.** Select N representative constraints per property: one *type-existence* clause, one *boundary-value* clause, one *error-shape* clause if available; drop the rest.
5. **Construction-style classification heuristic.** A constraint is provisionally construction-style if its property key references a public API surface (Bun.*, fs.*, http.*, web globals like fetch / Request / Response) and its verb-class is structural (toBeInstanceOf, toBeFunction, toHaveProperty, toThrow). Constraints touching internal implementation details get classified behavioral.
6. **Emit cluster.json** with the property catalog, the antichain per property, and the construction-style flags.

The MVP is heuristic and tunable. Subsequent iterations can refine property-key canonicalization, replace the heuristic threshold with empirical SIPE-T diagnosis, and improve the construction-vs-behavior classifier. The MVP's purpose is to make the *order-of-magnitude reduction* the keeper's conjecture predicts measurable: how many distinct properties are induced, how many constraints survive the antichain selection, what construction-style fraction emerges.

If the MVP yields a property catalog of size ~10²–10³ from the 43K input clauses, the keeper's conjecture is supported and the next phase (`invert`) operates on a tractable input.

## 7. Apparatus Anchors

- [Doc 247 — The Derivation Inversion](https://jaredfoy.com/resolve/doc/247-the-derivation-inversion) — constraint → implementation; not abstraction.
- [Doc 270 — Pin-Art Models](https://jaredfoy.com/resolve/doc/270-pin-art-models) — LOC predictability under tight constraints.
- [Doc 288 — The Pin-Art Derivation](https://jaredfoy.com/resolve/doc/288-the-pin-art-derivation) — htmx → htxlang at 14,000 → 1,318 (9.4%); 19 constraints / 3,937 words.
- [Doc 290 — The Pin-Art Formalization](https://jaredfoy.com/resolve/doc/290-the-pin-art-formalization) — formal constraint-to-implementation discipline.
- [Doc 185 / 420 — PRESTO](https://jaredfoy.com/resolve/doc/420) — construction-style constraints; the five C1–C5 of htxlang's parent style.
- [Doc 123 — Letter to Carson Gross](https://jaredfoy.com/resolve/doc/123-letter-to-carson-gross) — the htxlang articulation operationalizing PRESTO.
- [Doc 445 — Pulverization Formalism](https://jaredfoy.com/resolve/doc/445-pulverization-formalism) — minimal antichain of irreducible coherent sub-claims; π/μ/θ warrant tiers.
- [Doc 538 — The Architectural School: A Formalization](https://jaredfoy.com/resolve/doc/538-the-architectural-school-a-formalization) — induced properties P1–P4 at construction layer.
- [Doc 541 — SIPE-T](https://jaredfoy.com/resolve/doc/541-systems-induced-property-emergence) — threshold-conditional property emergence; necessary-vs-incidental constraint discrimination.
- [Doc 656 — Treat Agent Output Like Compiler Output](https://jaredfoy.com/resolve/doc/656) — the htxlang derivation case as a worked Pin-Art prediction.
- [Doc 700 — L2M Resolved](https://jaredfoy.com/resolve/doc/700-l2m-resolved-against-the-corpus-bipartite-mutual-information-scaling-as-empirical-grounding-for-the-pin-art-channel-ensemble) — capacity bound for property specification at scale.
- [Doc 701 — ILL Resolved](https://jaredfoy.com/resolve/doc/701-ill-resolved-against-the-corpus-information-lattice-learning-as-the-mature-prior-art-framework-for-the-pin-art-bilateral-and-the-joint-mi-lattice) — partition lattice + minimal-antichain optimization in formal form.
