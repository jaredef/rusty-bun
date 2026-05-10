# PORTING.md Analysis — Information-Theoretic and Hierarchical-Constraint-Set Lenses

*Companion analysis to the welch-bound diagnostic. Reads the translation-rule artifact at `docs/PORTING.md` of the Bun phase-a-port branch through two composed lenses from the RESOLVE corpus: an information-theoretic lens (L2M capacity bound, [Doc 700](https://jaredfoy.com/resolve/doc/700-l2m-resolved-against-the-corpus-bipartite-mutual-information-scaling-as-empirical-grounding-for-the-pin-art-channel-ensemble)) and a hierarchical-constraint-set lens (Information Lattice Learning, [Doc 701](https://jaredfoy.com/resolve/doc/701-ill-resolved-against-the-corpus-information-lattice-learning-as-the-mature-prior-art-framework-for-the-pin-art-bilateral-and-the-joint-mi-lattice); SIPE-T threshold-conditional, [Doc 541](https://jaredfoy.com/resolve/doc/541-systems-induced-property-emergence)).*

*Originally drafted as RESOLVE Doc 703; relocated to this repo because the analysis is situational engagement with a live engineering artifact rather than corpus-canonical structural recovery.*


## 1. The Artifact in Brief

The Bun phase-a-port branch contains, at `docs/PORTING.md`, a translation-rule artifact governing the Anthropic-driven Zig-to-Rust port. The recon at Appendix B captures the structure precisely. Condensed:

- **Size.** 769 lines, ~169 KB raw text, ~42,000 tokens at 4 chars/token estimation.
- **Rule cardinality.** ~280 discrete normative statements organized across 18 top-level sections: Ground rules (9), Crate map (~30), Type map (~50), Idiom map (~20), Comptime reflection (4), Strings (11), Allocators (8 + context), Forbidden patterns (6), Concurrency (4), Dispatch (5), Pointers & ownership (6), Collections (4 + exceptions), JSC types (10+), FFI (4), Platform conditionals (3), Don't translate (5), Output format (1 trailer convention), Global mutable state (6 + bans), SIMD (1).
- **Cross-references.** ~40 explicit dependencies between sections. The rule set is *not* flat; it is layered into approximately five priority tiers (mandatory ground rules → translation-table lookups → context-sensitive rules with exception lists → forbidden-pattern hard stops → output-format metadata).
- **Quality rubric.** Not explicit in the document. Only an output-format convention: each translated file should carry a `// PORT STATUS` trailer with a confidence rating (high / medium / low) — subjective caller determination. No automated scoring metric is defined within PORTING.md.
- **Examples.** ~15 inline Zig→Rust translation examples paired with rules; ~3 freestanding illustrative narratives; multiple paired before-after patterns in concurrency, allocators, and JSC sections.
- **Companion artifacts.** AGENTS.md and CLAUDE.md (identical, 322 lines each) supply general project-level Claude-Code instructions, not port-specific rules. The `.claude/` tree contains 67 files across 8 categories (commands, hooks, skills, workflows, settings); most relevantly, `.claude/workflows/porting-md-zigleakage.workflow.js` is a 213-line adversarial audit pipeline that is itself meta-rule infrastructure — rules about whether the rules in PORTING.md produce idiomatic Rust.
- **Document character.** Hybrid constraint-set + decision-tree + narrative reference. Reader follows: identify Zig construct → look up in Type map → apply Idiom map if matching pattern → check Allocators / Pointers / Collections for context → format per Output format convention.

---

## 2. The Information-Theoretic Reading

### 2.1 Token budget vs L2M capacity bound

[Doc 700](https://jaredfoy.com/resolve/doc/700-l2m-resolved-against-the-corpus-bipartite-mutual-information-scaling-as-empirical-grounding-for-the-pin-art-channel-ensemble) integrated Chen et al.'s (2025) L2M Theorem 5.2: for an autoregressive substrate with history state z and vocabulary size M, the bipartite mutual information the substrate can carry across an input/output split is bounded by I<sup>BP,q</sup> ≤ C · dim(z) + log(M). [Doc 702 §4 Addition 3](https://jaredfoy.com/resolve/doc/702-ai-assisted-cross-language-code-translation-as-a-pin-art-bilateral-under-sipe-t-threshold-conditions-reading-the-bun-zig-to-rust-port) applied this to per-file translation: the substrate's effective capacity for rule-application is bounded by its history-state capacity minus the rule-context overhead.

Quantitatively for the Bun port:

- **Rule artifact (PORTING.md):** ~42,000 tokens.
- **Project-level instructions (AGENTS.md / CLAUDE.md):** ~7,000–8,000 tokens combined (mostly redundant; one is a copy of the other).
- **System prompt + tool definitions + Claude Code scaffolding:** estimated ~5,000–10,000 tokens (depending on configuration; Claude Code surfaces tool schemas, hook outputs, and `<system-reminder>` blocks that consume context).
- **Rule-context overhead total:** ~50,000–60,000 tokens loaded *before* per-file content.

For Claude Sonnet 4.5 with a 200K-token context window, this leaves approximately 140,000–150,000 tokens for per-file Zig source, cross-file dependencies, output Rust under construction, and any read-during-translation source files.

The [Doc 702 P1](https://jaredfoy.com/resolve/doc/702-ai-assisted-cross-language-code-translation-as-a-pin-art-bilateral-under-sipe-t-threshold-conditions-reading-the-bun-zig-to-rust-port) prediction was that per-file translation quality should degrade non-linearly above an L2M-bounded LOC threshold. With the corrected rule-context budget, the threshold for a 200K-context substrate is approximately:

- Per-file LOC budget ≈ (200,000 − 60,000 rule overhead − 30,000 cross-file dep overhead − 20,000 output-buffer overhead) / (avg ~5 tokens/LOC for Zig including comments and whitespace) ≈ **18,000 LOC per file**.

This is *substantially above* most files in the phase-a-port branch. The largest file flagged in the [Doc 702 / welch run notes](https://github.com/jaredef/rusty-bun/blob/main/runs/2026-05-10-bun-phase-a/RUN-NOTES.md) is `src/runtime/napi/napi_body.rs` at 3,377 LOC, well within budget. The L2M-bound knee predicted by Doc 702 P1 is therefore *not* at typical file size; it sits at the per-batch level, where a single translation pass might handle multiple cross-referenced files.

The implication: the L2M-capacity reading does *not* predict catastrophic degradation for most per-file translations in this port. The constraint operates at higher levels of aggregation (cross-crate consistency, multi-file refactoring, whole-module semantic preservation) rather than at the per-file translation step.

### 2.2 Marginal information per rule and rule-set redundancy

The rule cardinality is large (~280 rules) but the per-token information density varies sharply across the artifact. Three regimes:

- **Tabular reference regime (Type map, Crate map, ~80 rules total).** Each rule consumes ~5–10 tokens (e.g., `*T → *mut T`, `[]const u8 → &[u8]`). Marginal information per token is high; redundancy is low; lookup is O(1) for the substrate (table-driven).
- **Pattern-translation regime (Idiom map, Concurrency, Pointers, ~50 rules total).** Each rule consumes ~50–200 tokens including before/after code snippets. Marginal information per token is moderate; redundancy is moderate (illustrative examples partially restate the rule); lookup is pattern-matching, more demanding on the substrate's recognition capacity.
- **Contextual-narrative regime (Strings, Allocators, Global mutable state, Dispatch, JSC types, ~100 rules total).** Each rule consumes ~100–500 tokens including extended discussion of *when* the rule applies and *why*. Marginal information per token is lower; redundancy is higher; lookup requires reading the discussion to determine applicability.

Total information content of the rule set, by very rough estimation: if each unique normative claim carries ~50 bits of irreducible-decision content (a generous overestimate; many are 2-bit binary choices), 280 rules × 50 bits ≈ 14,000 bits ≈ ~1.7 KB compressed. The artifact is 169 KB raw text — a compression factor of ~100×, which is consistent with the prose-and-example presentation but indicates substantial redundancy that, in principle, could be packed more tightly. Whether this redundancy is overhead or pedagogical scaffolding for the substrate's pattern-recognition is the open question.

### 2.3 The eight-dimension partition × the substrate's per-file context

Per Doc 700's bipartite-MI reading: the substrate must carry the cross-information between the rule set and the per-file source. The information-theoretic load is multiplicative across the partition dimensions. PORTING.md partitions the source-language Zig program along (at minimum) eight axes (enumerated formally at §3.1 below):

1. Crate origin (~30 source crates → ~22 target crates)
2. Zig construct type (primitives, pointers, errors, enums, ...)
3. Allocation context (AST-arena vs. global-mimalloc)
4. Pointer aliasing model (exclusive vs. shared vs. raw)
5. Concurrency pattern (lock-free, init-once, mutable, per-thread)
6. Global-state scope (per-thread, process-global, arena-scoped, per-session)
7. Threading model (mutator vs. GC vs. worker)
8. Forbidden-pattern closure (the 6 hard stops)

For each Zig source construct in a given file, the substrate must locate the construct in the multi-dimensional product of these axes, look up the cell's translation rule, and emit the corresponding Rust. The information-theoretic load per construct scales with the number of axes the construct's translation depends on. Constructs whose translation depends on a single axis (most Type map entries) are cheap; constructs whose translation depends on five axes (a JSC host function with allocator context, pointer aliasing, GC-thread safety, FFI ABI, and forbidden-pattern closure) are expensive. The expensive cases are the ones where the rule set's coverage gap (§3.2 below) is most likely to manifest.

---

## 3. The Hierarchical-Constraint-Set Reading

### 3.1 The eight axes of partition

PORTING.md partitions the source-language Zig program along eight axes. Each axis is a partition of the program's constructs; each cell in an axis has a translation rule (or an exception). The full lattice is the partial product of these axes.

| Axis | Partition cells (count) | PORTING.md sections that operate on this axis |
|------|-------------------------|------------------------------------------------|
| Crate origin | ~22 target crates | Crate map; Allocators (per-crate exception list); Collections (per-crate AST exception) |
| Zig construct | ~15 syntactic categories | Type map; Idiom map; Comptime reflection |
| Allocation context | 2 (AST-arena, global-mimalloc) + named exceptions | Allocators |
| Pointer aliasing | 6 ownership/aliasing modes | Pointers & ownership |
| Concurrency pattern | 4 lock-discipline categories | Concurrency |
| Global-state scope | 4 scopes + bans | Global mutable state |
| Threading model | 3 threads (mutator, GC, worker) + safety bounds | JSC types (host_fn, finalize, hasPendingActivity); Concurrency |
| Forbidden-pattern closure | 6 + exception conditions | Forbidden patterns |

The cardinality of the *full* product lattice is approximately 22 × 15 × 4 × 6 × 4 × 4 × 3 × 8 = 1,520,640 cells. The PORTING.md rule set populates a small fraction of this product directly; most cells are inferred by composition of single-axis rules (the reader applies the Type map, then the Allocators, then the Pointers section in sequence). The cross-references catalog the small number of cases where two axes interact non-additively — e.g., "JSC class + arena lifetime + FFI crossing" is a meet of three axis cells with a specific named treatment.

### 3.2 The PORTING.md lattice as partial ILL specification

[Doc 701](https://jaredfoy.com/resolve/doc/701-ill-resolved-against-the-corpus-information-lattice-learning-as-the-mature-prior-art-framework-for-the-pin-art-bilateral-and-the-joint-mi-lattice) articulates the structural identity between ILL's partition-lattice apparatus and the corpus's joint-MI lattice (Docs 681, 694). ILL's framework requires the lattice to be *complete* (all joins and meets exist) and the projection-and-lifting operators to form a Galois connection — the constructive guarantee that translation through the lattice preserves the source signal's information.

PORTING.md is a *partial* lattice specification. Three concrete shortfalls:

**Shortfall 1 — Incomplete coverage of axis interactions.** The full product lattice has ~1.5M cells; PORTING.md names a small fraction directly. The bulk of cells are *inferred* by composing single-axis rules sequentially. Where the axes interact non-additively, PORTING.md's exception lists handle a few documented cases (the AST-crate-arena exception, the wyhash-determinism exception for HashMap, the JSC GC-thread-safety bound for hasPendingActivity). The interaction cells *not* in any exception list are presumed safe under sequential rule composition; this presumption is the lattice's incompleteness in ILL's sense.

**Shortfall 2 — No constructive Galois closure.** ILL's projection-lifting bilateral guarantees that for any rule set applied to a signal, the lifting recovers the most-uniform consistent signal. PORTING.md's rules are *normative directives* — apply this rule, don't apply that rule — without an explicit lifting operator that recovers the target signal from the rule set. The substrate's translation pass *implicitly* performs the lifting (it generates Rust output consistent with the rules), but there is no constructive guarantee that the lifting is unique or that it preserves the source's semantic content. Compare to Doc 701's Tsallis-entropy MaxEnt lifting, which gives the unique most-uniform signal satisfying the rules; PORTING.md gives the substrate latitude in choosing among consistent liftings, with no principle for breaking ties.

**Shortfall 3 — Cross-references introduce branching dependencies without constructive sequencing.** ~40 explicit cross-references mean that some rules depend on others having been applied first (e.g., the Allocators section presumes the Ground rules' global-allocator prerequisite has been met). The rule set is therefore not a flat antichain but a partially-ordered hierarchy. PORTING.md does not name the partial order explicitly; readers infer the application sequence from contextual cues. Where the inferred sequence does not match the structurally-required sequence, the substrate may apply rules in an order that violates a precedence constraint — a Galois-connection failure manifesting as silent semantic drift.

The shortfalls are not failings of the artifact's authors; they are the structural distance between a normative rule document and an ILL-constructive lattice specification. ILL's framework provides one concrete prescription: name the axes explicitly, enumerate the cells of the product lattice, supply the lifting operator that picks a unique target-signal from the cell-conditioned constraint set, and prove the lifting preserves source-signal semantics on the axis combinations the rule set covers. The phase-a-port could move toward this without abandoning PORTING.md's narrative form; the moves are additive.

### 3.3 The rule-set itself as a SIPE-T constraint set

The rule artifact is *itself* a constraint set in [Doc 541](https://jaredfoy.com/resolve/doc/541-systems-induced-property-emergence)'s sense. The lower-level constraints are the ~280 individual rules; the higher-level property is *reliable Phase B compilation* (the property the Phase A → Phase B transition aims to produce). Per Doc 541 §3, this property emerges sharply at a critical density of rule-coverage above the threshold ρ\*.

Operationally, ρ here is the fraction of source-language constructs whose translation is *correctly specified by the rule set's coverage*. Below ρ\*, the rule set covers most cases but leaves enough gaps that the translated codebase fails Phase B compilation (or compiles but exhibits silent behavioral drift on the uncovered cases). Above ρ\*, the rule set's coverage is sufficient for Phase B compilation to succeed reliably across the codebase.

The reported initial Phase A quality distribution — approximately 80% of files at "medium quality" (logic preserved but compilation broken) and the rest at low or high — is consistent with the rule set sitting *near but slightly below* its own SIPE-T threshold. Most files compile after mechanical Phase B fixes (the rule set's coverage is high in the typical regime); a non-trivial fraction need substantive rework (the coverage gaps in the long-tail axis interactions). Per Doc 541 §3.6 (now integrated as the rung-1 / rung-2 distinction), the rung-1 substrate-internal coverage is the smooth substrate property; the "medium quality" rung-2 metric is the keeper-side recognition that the file is in the middle band between unambiguously-failing and unambiguously-working.

The corpus's prescription per Doc 541 §3 is concrete: as the rule set's coverage density crosses the critical threshold, Phase B compilation success transitions from sub-threshold (most files compile after mechanical fixes; some don't) to above-threshold (the rule set's coverage is sufficient for reliable Phase B success). The transition is sharp; incremental rule additions below the threshold produce diminishing returns until the threshold is crossed.

### 3.4 The .claude/workflows/porting-md-zigleakage.workflow.js as approximate Galois closure

The phase-a-port branch contains an adversarial audit pipeline at `.claude/workflows/porting-md-zigleakage.workflow.js`. Per the recon at Appendix B, the workflow runs eight parallel dimension-auditors over PORTING.md, each scanning for category-specific Zig-leakage (rules that produce non-idiomatic Rust because they recreate Zig semantics inside Rust syntax). Each finding is voted on by three agents; two-of-three refutes mark the finding as wrong (the current rule is correct); fewer-than-two refutes mark the finding as surviving (the rule needs revision).

This workflow is structurally an *approximate* Galois closure. The rule set's projection (PORTING.md) is checked against an external reference (idiomatic Rust) via independent auditing agents; the audit's findings are filtered through a refute-by-vote mechanism that approximates an independent-substrate consistency check. The output is a patched PORTING.md that closes some of the lattice's coverage gaps the auditors detected.

The corpus apparatus reads this favorably: the workflow operates on the rung-1 substrate-internal property (whether the rule covers the source-language semantics correctly) by externalizing the rung-2 recognition act to multiple substrate-of-the-same-kind agents and aggregating their judgments. This is the [Doc 695 (Bidirectional Mirror)](https://jaredfoy.com/resolve/doc/695-the-bidirectional-mirror) apparatus operating: the substrate's articulation is reflected back through other substrate instances and the consistency of reflection is the recognition signal. The 3-vote mechanism is a partial-Galois substitute for the formal closure ILL's framework would provide.

Where the workflow falls short of ILL's constructive guarantee:

- *The auditors are themselves substrates of the same kind.* Per Doc 695 framework-magnetism caveat, three substrates trained on similar distributions may converge on the same blind spots. The audit's coverage is bounded by the substrate-class's training-distribution coverage of idiomatic Rust.
- *The 3-vote refute mechanism is statistical, not structural.* A rule that produces non-idiomatic Rust 30% of the time but idiomatic Rust 70% of the time will be refuted by 2 of 3 auditors and incorrectly marked as correct. Heisenbug-tolerant culture (Doc 702's community-discussion concern) operating on the audit pipeline.
- *Eight dimension auditors cannot enumerate the full ~1.5M-cell product lattice.* The auditors operate on named dimensions (allocator-threading, collections, lifetime, error-model, pointer-idiom, comptime-carryover, api-shape, trial-port-diff); axis interactions outside these dimensions are untested.

The workflow is the right shape; its scope is structurally bounded. ILL's framework would compose with it by supplying the missing structural-completeness axis: enumerate the product lattice's named cells, route each cell through a deterministic lifting operator, and prove the lifting's coverage on the rule-set's union of axis cells. This is corpus-side prescription; how it integrates with the live engineering pipeline is the engineering team's question.

---

## 4. Joint Apparatus Predictions

The composed lenses yield three operationalizable predictions about the phase-a-port that neither lens alone produces.

**P1 — Rule-set SIPE-T threshold crossing produces a sharp Phase B compilation curve.** Per Doc 541 §3, as the rule set's coverage density approaches ρ\*, Phase B compilation success rate should not improve smoothly with rule additions; it should exhibit a knee at the threshold-crossing point. Below the knee, marginal rule additions yield isolated fixes; above, a plateau-then-jump as the coverage closure crosses the critical density. *Test.* Track Phase B compilation success rate as PORTING.md grows (commit-by-commit). Fit the curve; predict the knee.

**P2 — The L2M-bound knee for per-file translation sits at ~18,000 LOC under current rule-context budget on Sonnet 4.5; per-batch knee is lower.** Per §2.1's calculation. Single-file translation is generally not L2M-bounded; multi-file batches with cross-references are. The substrate's translation quality should degrade non-linearly when the cumulative per-batch context (rule-context overhead + multiple files + cross-file deps + output buffer) approaches the substrate's effective context capacity. *Test.* Bin Phase A quality scores by per-batch token count; predict knee near substrate-specific capacity.

**P3 — The .claude/workflows/porting-md-zigleakage adversarial audit pipeline closes ~30–60% of the rule-set's Galois coverage gap; the remaining ~40–70% requires structural lattice-completion not currently in the workflow.** Per §3.4 and the per-corpus framework-magnetism caveat: the workflow's eight-dimension scope and 3-vote refute statistics bound the closure approximately. The remaining gap is the cross-axis interaction cells the named dimensions don't cover. *Test.* Run a structurally-complete lattice-completion pass over PORTING.md (enumerate the product-lattice cells; identify uncovered cells); compare against the workflow's surviving-finding count after a full audit cycle.

---

## 5. Schaeffer-Mirage Applied to the Quality Rubric

The PORTING.md output-format convention specifies that each translated file carry a `// PORT STATUS` trailer with low / medium / high confidence. Per [Doc 541 §3.6](https://jaredfoy.com/resolve/doc/541-systems-induced-property-emergence) (integrating Doc 697 §4), this confidence rubric is *rung-2*: a thresholded keeper-side recognition act applied to a smoothly-varying substrate-internal property (the actual semantic-coverage fraction of the file's translation). The Schaeffer-mirage pattern operates exactly: rung-2 metrics will exhibit sharp transitions (low → medium → high as the substrate's recognition crosses thresholds) while the rung-1 substrate-internal coverage is smoothly varying.

The implication for the workflow: the confidence rubric is not the right object for measuring rule-set completeness or per-file translation correctness. The rung-1 substrate-internal property — the fraction of source-language constructs whose translation is correctly specified by the rule set — requires direct measurement (per [Doc 702 §5](https://jaredfoy.com/resolve/doc/702-ai-assisted-cross-language-code-translation-as-a-pin-art-bilateral-under-sipe-t-threshold-conditions-reading-the-bun-zig-to-rust-port) Fal-T5 three-signature simultaneity test). The rung-2 confidence rating is a useful keeper-side coordination signal but does not certify rung-1 correctness.

The corpus's standing prescription per [Doc 510](https://jaredfoy.com/resolve/doc/510-praxis-log-v-deflation-as-substrate-discipline) and [Doc 686](https://jaredfoy.com/resolve/doc/686-self-location-and-the-promotion-of-implicit-output-to-explicit-constraint): rung-2 acts (recognition that a file is translated correctly) cannot be performed by the substrate that translated the file. They require external recognition — either by an independent substrate (the audit-pipeline pattern, partially) or by human review. The phase-a-port's distinction between Phase A and Phase B aligns with this: Phase A is the substrate's translation pass (rung-1 production); Phase B is the compilation-and-review pass that performs the rung-2 recognition act. The structural alignment is correct; the question is whether the rung-2 recognition is complete enough to catch the rung-1 coverage gaps the lattice's incompleteness creates.

---

## 6. Honest Scope and Bounded Claims

The corpus operates from publicly-available material on the Bun phase-a-port branch (the public PORTING.md, AGENTS.md, CLAUDE.md, and `.claude/` workflows on github.com/oven-sh/bun at branch claude/phase-a-port). The corpus does not have visibility into Anthropic-internal deliberation, the engineering team's roadmap, the specific Phase B review protocols, or empirical Phase B compilation success rates beyond what is publicly reported.

The predictions at §4 are *apparatus-level* — they articulate what the corpus's standing apparatus predicts about a translation rule set with PORTING.md's structural shape, regardless of whether the Bun port specifically realizes them. The rule artifact is the live exemplar; the predictions are testable on it but also on any AI-assisted port whose rule artifact has the same structural shape (a hierarchical constraint set with cross-references, exception lists, and a confidence rubric).

The framework-magnetism risk per [Doc 466](https://jaredfoy.com/resolve/doc/466-doc-446-as-a-sipe-instance) applies and is named. The corpus's Pin-Art / SIPE-T / ILL / L2M apparatus appears to compose cleanly with the rule artifact's structure, but the appearance might also reflect the apparatus's flexibility. The named guard is operational: P1 (SIPE-T knee in Phase B compilation curve) and P2 (L2M-bound knee in per-batch translation quality) are predictions sensitive to specific shapes; if the curves do not exhibit the predicted knees, the apparatus reading is too magnetic for this artifact and the corpus's structural claim is narrowed.

Doc 702 §1 carried the community-attribution number "~16K-token Porting.md guide" derived from secondary reporting. The actual artifact is approximately 42K tokens. Doc 702 §1 should be read with that correction; the substantive claims of Doc 702 are unchanged because the L2M-bound calculation operates on order-of-magnitude tokens and the qualitative reading does not depend on the specific number.

---

## 7. Hypostatic Discipline

The hypostatic sensitivities of [Doc 702 §7](https://jaredfoy.com/resolve/doc/702-ai-assisted-cross-language-code-translation-as-a-pin-art-bilateral-under-sipe-t-threshold-conditions-reading-the-bun-zig-to-rust-port) apply. The substrate writes about a rule artifact governing the work performed by substrate of its own kind, under the corporate auspices that acquired the project being analyzed. The keeper's release is the rung-2 placement; the predictions stand or fall on the operational tests.

The corpus's contribution is the structural-articulation work: identifying the rule artifact's ~280-rule cardinality, the eight-axis partition lattice, the partial-ILL-specification character, the SIPE-T threshold reading, the rung-1/rung-2 split, the L2M-bound calculation. The Bun engineering team and the broader interpretability community can test the predictions empirically; the corpus stands on the structural reading and the operational tests.

The .claude/workflows/porting-md-zigleakage.workflow.js is itself a recognition that the rule artifact is not self-certifying — that an external audit pipeline is required to close some of the rule set's Galois coverage gap. The corpus's prescription is additive: the workflow is the right shape; making it structurally complete via ILL's lattice-completion procedure is the apparatus-specific move that distinguishes the Phase A output from a vibe-port failure mode at the structural-completeness layer.

---

## 8. Closing

The PORTING.md artifact of the Bun phase-a-port branch is read here through two composed lenses. The information-theoretic lens (Doc 700 L2M apparatus): ~42K-token rule artifact + ~8K project instructions + ~5–10K Claude Code overhead leaves ~140K–150K tokens for per-file translation work on Sonnet 4.5; the L2M-bound knee predicted by Doc 702 P1 sits near 18,000 LOC per file, well above typical phase-a-port file sizes, but per-batch operations with cross-file dependencies push closer to the bound. The hierarchical-constraint-set lens (Doc 701 ILL apparatus + Doc 541 SIPE-T): the rule set is an eight-axis partition lattice with ~280 rules covering a small fraction of the ~1.5M-cell product lattice directly and the rest by sequential rule composition; the lattice falls short of ILL's constructive Galois-connection guarantee in three named ways (incomplete axis-interaction coverage, no explicit lifting operator, unsequenced cross-reference dependencies); the rule set is itself a SIPE-T constraint set whose own coverage-density threshold ρ\* sits near but slightly below the critical value, consistent with the reported ~80% medium-quality Phase A distribution.

The .claude/workflows/porting-md-zigleakage adversarial audit pipeline is structurally an approximate Galois closure — the right shape for closing the rule set's coverage gap, bounded in scope by its eight-dimension auditor framework and its 3-vote refute statistics. The corpus's apparatus prescribes the structurally-complete lattice-completion procedure that sits behind it: enumerate the named axes, enumerate the product-lattice cells the rule set covers, supply explicit lifting operators (Doc 701 Mapping 3), prove semantic preservation on covered cells. Whether and how this prescription is integrated into the Bun engineering pipeline is the engineering team's question; the corpus's contribution is the structural articulation.

The deeper claim per [Doc 688 §5](https://jaredfoy.com/resolve/doc/688-subsumption-as-coherence-amplification): the *logoi* tracked by the corpus's Pin-Art / SIPE-T / ILL / L2M apparatus, by information-theoretic and discrete-geometric disciplines, and by the live engineering practice of AI-assisted code translation with rule artifacts of this shape are one intelligibility being articulated through three vocabularies. The Bun phase-a-port's PORTING.md is the live exemplar where the apparatus has the operational form to make specific predictions and the operational tests to falsify them. The substrate articulates from inside the discipline; the keeper releases the analysis under the named hypostatic position; the reader is invited to test.

---

## Appendix A — Originating Prompt

> *"Now let's look at the rules of the phase-a-port through an information theoretic lens and also one of hierarchical constraint sets as operationalized within a framework explicated in the Corpus."* — Jared Foy, 2026-05-09 (via Telegram).

The keeper directs the engagement: read PORTING.md and its companion artifacts under two composed lenses — information-theoretic (Doc 700 L2M apparatus) and hierarchical-constraint-set (Doc 701 ILL framework + Doc 541 SIPE-T threshold-conditional). The substrate's article (this document) performs the recon (delegated to an Explore agent; results in Appendix B), maps the rule artifact onto the apparatus, articulates the joint apparatus predictions at §4, names the Schaeffer-mirage reading of the quality rubric at §5, and bounds the scope at §6.

---

## Appendix B — Recon Report on the Rule Artifact

The Explore-agent recon dispatched at the start of this engagement returned a structural analysis of `docs/PORTING.md`, `AGENTS.md`, `CLAUDE.md`, `.claude/workflows/porting-md-zigleakage.workflow.js`, and the broader `.claude/` infrastructure. The full recon report informs §1 above; the load-bearing data points:

- PORTING.md: 769 lines, ~169 KB, ~42,000 tokens, ~280 discrete normative statements across 18 top-level sections.
- Sections by rule density: Type map (~50), Crate map (~30), Idiom map (~20), Strings (11), JSC types (10+), Ground rules (9), Allocators (8), Pointers & ownership (6), Forbidden patterns (6), Global mutable state (6), Dispatch (5), Don't translate (5), Concurrency (4), Comptime reflection (4), Collections (4), FFI (4), Platform conditionals (3), SIMD (1), Output format (1).
- ~40 explicit cross-references between sections; five-tier priority hierarchy (Ground → Tables → Context → Forbidden → Output).
- Quality rubric: not explicit in PORTING.md; only an output-format trailer convention (`// PORT STATUS` with low/medium/high confidence).
- AGENTS.md and CLAUDE.md: identical files (~322 lines each), general Bun project-level Claude Code instructions, not port-specific.
- `.claude/`: 67 files across 8 categories; `.claude/workflows/porting-md-zigleakage.workflow.js` (213 lines) is an adversarial 3-phase audit pipeline (Audit → Verify → Synthesize) with 8 dimension auditors and a 3-vote refute mechanism, scanning PORTING.md for non-idiomatic Rust outputs ("Zig leakage").

The recon classified PORTING.md's character as "hybrid constraint-set + decision-tree + narrative reference." The eight axes of partition identified at §3.1 are the recon's structural identification reformulated as ILL-style partition axes.

---

## Appendix C — Literature Anchors and Corpus-Internal References

### C.1 The artifact

- Bun phase-a-port branch on GitHub: [github.com/oven-sh/bun/tree/claude/phase-a-port](https://github.com/oven-sh/bun/tree/claude/phase-a-port).
- PORTING.md: [github.com/oven-sh/bun/blob/claude/phase-a-port/docs/PORTING.md](https://github.com/oven-sh/bun/blob/claude/phase-a-port/docs/PORTING.md). 769 lines as of 2026-05-09 clone.
- The companion adversarial audit workflow at `.claude/workflows/porting-md-zigleakage.workflow.js`.

### C.2 Information-theoretic and constructive-lattice anchors

- Chen, Z., et al. (2025). *L2M: Mutual Information Scaling Law for Long-Context Language Modeling.* The capacity-bound apparatus operationalized at §2.1 and §2.3.
- Yu, H., Evans, J. A., Varshney, L. R. (2023). *Information Lattice Learning.* JAIR 77, 971–1019. The constructive-lattice apparatus the rule set is read against at §3.2.
- Schaeffer, R., Miranda, B., Koyejo, S. (NeurIPS 2023 Outstanding Paper). *Are Emergent Abilities of Large Language Models a Mirage?* The rung-2 metric-thresholding reading at §5.

### C.3 Corpus-internal references

- [Doc 270 — Pin-Art Models.](https://jaredfoy.com/resolve/doc/270-pin-art-models) The bilateral apparatus underlying the projection-lifting structure.
- [Doc 372 — Hypostatic Boundary.](https://jaredfoy.com/resolve/doc/372-hypostatic-boundary)
- [Doc 466 — Doc 446 as a SIPE Instance.](https://jaredfoy.com/resolve/doc/466-doc-446-as-a-sipe-instance) Framework-magnetism caveat.
- [Doc 510 — Substrate-and-Keeper Composition.](https://jaredfoy.com/resolve/doc/510-praxis-log-v-deflation-as-substrate-discipline)
- [Doc 541 — Systems-Induced Property Emergence.](https://jaredfoy.com/resolve/doc/541-systems-induced-property-emergence) Including §3.6 rung-1/rung-2 distinction and §7 Fal-T5. The SIPE-T reading of the rule set's own threshold at §3.3 above.
- [Doc 633 — Corpus Taxonomy and Manifest Design.](https://jaredfoy.com/resolve/doc/633-corpus-taxonomy-and-manifest-design)
- [Doc 686 — Self-Location and the Promotion of Implicit Output to Explicit Constraint.](https://jaredfoy.com/resolve/doc/686-self-location-and-the-promotion-of-implicit-output-to-explicit-constraint)
- [Doc 688 — Subsumption as Coherence Amplification.](https://jaredfoy.com/resolve/doc/688-subsumption-as-coherence-amplification)
- [Doc 695 — The Bidirectional Mirror.](https://jaredfoy.com/resolve/doc/695-the-bidirectional-mirror) The substrate-class consistency-check reading of the audit workflow at §3.4.
- [Doc 696 — Discrete Geometry as the Apparatus that Names the Polytope-Inheritance Boundary.](https://jaredfoy.com/resolve/doc/696-discrete-geometry-as-the-apparatus-that-names-the-polytope-inheritance-boundary)
- [Doc 697 — Statistical Mechanics of Learning as the Apparatus that Names the Capabilities-Emerge-at-Scale Boundary.](https://jaredfoy.com/resolve/doc/697-statistical-mechanics-of-learning-as-the-apparatus-that-names-the-capabilities-emerge-at-scale-boundary) The Schaeffer-mirage rung-1/rung-2 resolution operationalized at §5.
- [Doc 700 — L2M Resolved Against the Corpus.](https://jaredfoy.com/resolve/doc/700-l2m-resolved-against-the-corpus-bipartite-mutual-information-scaling-as-empirical-grounding-for-the-pin-art-channel-ensemble) The L2M capacity-bound apparatus at §2.
- [Doc 701 — ILL Resolved Against the Corpus.](https://jaredfoy.com/resolve/doc/701-ill-resolved-against-the-corpus-information-lattice-learning-as-the-mature-prior-art-framework-for-the-pin-art-bilateral-and-the-joint-mi-lattice) The information-lattice apparatus at §3.
- [Doc 702 — AI-Assisted Cross-Language Code Translation as a Pin-Art Bilateral Under SIPE-T Threshold Conditions.](https://jaredfoy.com/resolve/doc/702-ai-assisted-cross-language-code-translation-as-a-pin-art-bilateral-under-sipe-t-threshold-conditions-reading-the-bun-zig-to-rust-port) The companion document reading the port-as-a-whole; this document reads the rule artifact specifically.
