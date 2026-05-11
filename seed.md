# rusty-bun — Seed

The stable kernel of the rusty-bun engagement. Per [Doc 581 (the Resume Vector)](https://jaredfoy.com/resolve/doc/581-the-resume-vector), this document holds the constraints, architecture decisions, deferred-list discipline, and future-move discipline that do not change session to session. The companion [trajectory.md](trajectory.md) holds what does change. Together with the resume protocol at the trajectory's tail, the pair makes this engagement resumable.

## I. Frame (what this engagement is)

The engagement applies the RESOLVE corpus's Pin-Art apparatus ([Doc 270](https://jaredfoy.com/resolve/doc/270-pin-art), [Doc 619](https://jaredfoy.com/resolve/doc/619-pin-art-canonical-formalization)) to AI-assisted cross-language code translation, specifically to the Bun runtime's Zig→Rust port. The work is not translation per se: it is **formalization-then-derivation** ([Doc 704](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error)). Constraints are extracted from Bun's test corpus + curated spec extracts; derivations are produced from the constraints; verifiers close the loop on each derivation; consumer regression suites close the loop on real-world dependencies.

The apparatus is bidirectional ([Doc 707](https://jaredfoy.com/resolve/doc/707-pin-art-at-the-behavioral-surface-bidirectional-probes)): each constraint is a pin that constrains a future derivation AND surfaces an invariant of the original implementation that was otherwise implicit. Two outputs per pilot: a derivation, and a dependency-surface map.

## II. Binding constraints

**C1. Plug-and-play interoperability with no regressions, NOT 100% behavior parity.** The target is "any consumer that worked with Bun continues to work with the derivation." Bun has implementation accidents and contingent inefficiencies; deliberate divergence on Tier-3 (implementation-contingent) details is permitted with recorded reason. Spec conformance + consumer-corpus matching is the criterion, not byte-for-byte Bun matching. Per Doc 707's three-tier framing.

**C2. Cite-source discipline.** Every consumer-regression test must cite a real production codepath at file-and-function granularity. Without cites, a consumer test is indistinguishable from a spec test. With cites, the test is anchored to a real downstream dependency and the regression claim is falsifiable.

**C3. Simulated derivation, not wired rederive.** Derivation is performed by an LLM (the substrate) reading the constraint corpus + spec material, with input bundle declared in source-code comments at the head of each module. A wired rederive engine is the eventual goal; the simulation establishes that there is something there to wire. Per [Doc 706's framing](https://jaredfoy.com/resolve/doc/706-three-pilot-evidence-chain-derivation-from-constraints).

**C4. Two test categories per pilot.** Verifier (prescriptive: does it conform to constraints + spec?) AND consumer regression (descriptive: would it break real consumers?). Both required for pilot to count.

**C5. Honest LOC accounting.** Naive ratios reported alongside scope-honest adjusted ratios. Bun's reference targets often include transport / FFI / runtime-integration that pilot scope omits. Pilot-versus-target comparison must say which scope is being measured.

**C6. Em-dash restraint in writing.** Per keeper memory: target 0–1 per 1000 words. Use commas, parens, periods.

**C7. No commits without explicit keeper request.** Every commit is keeper-authorized. No co-author lines.

## III. Architecture decisions

### A1. Pipeline shape (`derive-constraints` binary)

Eight phases: scan → cluster → invert → seams → couple → (derivation by LLM) → verifier → consumer regression. Phases 1-5 are tooled; phase 6 is simulated; phases 7-8 are Rust test scaffolding.

### A2. Five pin classes for the constraint corpus

- **Spec invariant** (normative authority; from `specs/*.spec.md`)
- **Test rep** (observational; from Bun/Deno test corpora)
- **Consumer expectation** (dependency-survey; from cited consumer source)
- **WPT entry** (conformance-suite; subset of consumer corpus)
- **Implementation-source probe** (architectural-witness; from welch/seams analysis)
- **Runtime-integration probe** (host-binding-witness; from `host/` integration tests)

Each carries different bidirectional information per Doc 707. The runtime-integration pin is added as of 2026-05-10 — when a pilot's API is wired into the JS host, the binding shape becomes a probe in its own right. Forward direction: the binding constrains the pilot's API to be JS-host-friendly. Backward direction: the binding reveals which APIs require adaptation (e.g., the QuickJS-GC + stateful-types finding at A8 below).

### A3. Three-tier authority taxonomy

**Tier 1 — Spec-mandated.** WHATWG / W3C / RFC says it. Must conform.
**Tier 2 — Ecosystem-compat.** Bun-extension or Node-API. Bun's tests are the spec.
**Tier 3 — Implementation-contingent.** Performance / allocation / lazy I/O. Optional divergence with recorded reason.

### A4. Cross-corroboration as tier-1 signal

A property witnessed by BOTH a spec source AND a test source is the strongest constraint. The cluster phase tracks this via per-property `source_files`. Cross-corroborated cardinality is the apparatus' clearest pilot-readiness indicator.

### A5. Per-pilot crate convention

Each pilot lives at `pilots/<surface>/derived/`. Cargo crate per pilot. `src/lib.rs` for the derivation, `tests/verifier.rs` for prescriptive tests, `tests/consumer_regression.rs` for descriptive tests. Path-dependencies between pilots when one is a substrate for another (File depends on Blob; multi-surface system pilots compose locally).

### A6. Run-notes per artifact run

Every pipeline run lands at `runs/<date>-<corpus>-<version>/RUN-NOTES.md`. Every pilot lands at `pilots/<surface>/RUN-NOTES.md` (run notes for the pilot's verifier closure). Cross-cutting summaries land alongside (e.g., `pilots/CONSUMER-REGRESSION-SUMMARY.md`).

### A7. Spec corpus is part of the apparatus

`specs/*.spec.md` is curated content but is read by the same `derive-constraints scan` pass that reads test corpora. Spec material flows through cluster / invert / seams identically to test-derived clauses, distinguishable by their `language: spec` tag.

### A8. JS host integration pattern: stateless Rust + JS-side classes

For Sub-criterion 4 of the completion telos (JS host integration), pilots wire into JS through `host/` per the pattern documented at `host/HOST-INTEGRATION-PATTERN.md`:

1. **Pure-value pilot APIs wire directly.** Rust functions that take and return owned values (atob/btoa, path.basename, crypto.randomUUID, fs.readFileSyncUtf8) bind via `Function::new(ctx.clone(), |args| -> result {...})` with no closure-captured state. The JS-side calls them as plain functions or namespaced methods.

2bis. **Spec-formalization pilots may be JS-side reimplementations against the same constraint set.** When a pilot's Rust crate models an algorithm against a custom representation (e.g., the structured-clone pilot's Heap/Value), routing JS values through that representation requires a bridge that adds no value: the JS engine already has all the primitives the algorithm operates on (Date, RegExp, Map, Set, ArrayBuffer, TypedArrays, Blob, File). Such pilots wire as JS-side reimplementations against the **same constraint set the pilot was derived from**. The pilot's Rust crate stays the canonical algorithmic reference (verifier-tests, doc citations, ratio anchor); the host's JS implementation is a sibling instantiation. Use this pattern when (a) the pilot's Rust API takes/returns a custom representation, and (b) the algorithm is pure value-recursion plus memo. structuredClone uses this pattern.

2. **Stateful pilot APIs wire indirectly: stateless Rust helpers + JS-side class.** Rust closures that capture `Rc<RefCell<...>>` and are stored as methods on JS objects break QuickJS' GC (it does not track Rust references). Instead: expose a private `__namespace` of stateless Rust helper functions; install a JS-side class via `ctx.eval()` that holds its own state in pure-JS arrays/objects and calls into the Rust helpers for algorithm work. URLSearchParams + TextEncoder + TextDecoder use this pattern; future stateful types (Blob, File, Headers, Request, Response, AbortController, structuredClone-Heap) MUST follow it.

3. **Optional-arg semantics: JS omits, doesn't pass undefined.** rquickjs `Opt<T>` requires the JS-side to OMIT the argument, not pass `undefined`. JS-side classes that delegate to Rust helpers must branch: `if (val === undefined) call(without arg) else call(with arg)`.

4. **Testing surface:** every wired pilot has at least one JS-side integration test in `host/tests/integration.rs` plus appears in `host/examples/runtime-demo.js`. The workspace runner (`./bin/run-pilots.sh`) covers the host suite alongside per-pilot suites.

5. **Decode polymorphic JS shapes JS-side, not Rust-side.** When a JS API accepts a polymorphic argument shape — e.g., Bun.serve's `routes: { "/x": fn | { GET: fn, POST: fn } }`, fetch's `init: { headers: HeadersInit | Headers, body: BodyInit }` — the JS-side wrapper performs the discrimination and only hands canonical values to the Rust helpers. The Rust pilot stays a pure algorithm (e.g., `match_pattern(pattern, url)`); decoding the user's polymorphic input is JS work. This keeps pilot crates clean of host-encoding concerns and lets a single Rust helper serve many JS surface shapes.

6. **Cross-boundary type translation:** rquickjs does not bind tuples or structs as function args; use `Vec<Vec<String>>` as a pair-list across the FFI when the data is naturally `Vec<(String, String)>`. The JS-side wrapper assembles/disassembles into objects.

7. **Canonical-docs composition test.** Every wired flagship surface ships with at least one integration test that mirrors the upstream's documented usage example *verbatim* (see `js_compose_bun_serve_canonical_pattern`). This test is the smallest unit of "real consumer can swap rusty-bun for Bun" and is the verification of choice for sub-criterion 4. Per-method tests verify the surface; canonical-docs tests verify the **swap-in property**.

9. **Second SIPE-T threshold: rule-standing-in-production.** The first SIPE-T threshold (item 8 + §III.A8.8) was the substrate moving from primitive-discovery to rule-composition — rules becoming jointly legible. The second threshold, observed 2026-05-10 after M9's institution, is the substrate moving from rule-composition to **rule-standing-in-production**: the M-rule set (M7+M8+M9) becomes load-bearing enough that consecutive rounds produce predictable substrate work — one J.1.a fixture + one in-round M8 reconciliation each — without requiring keeper rung-2 input to identify what should happen next. The rules do the cognitive work that previously required keeper mediation per round.

Three markers of the crossing:
  - **Predictable per-round output.** Each "Continue" produces a fixture plus a divergence reconciliation, mechanically following the M9 protocol. No new rule needs to be named.
  - **Vacuous-with-reconciliation pattern.** Consecutive M7 fold-backs classified vacuous (apparatus-side) with one M8(a) reconciliation in-round (apparatus catches up to spec). Same shape repeatedly. Three or more consecutive rounds of this shape is the signature.
  - **Keeper-mediation shifts tiers.** The keeper no longer names primitives the substrate produced (rung-2 at the rule-discovery tier); instead the keeper names the *regime* the substrate is now operating in (rung-2 at the meta-rule tier). M7→M8→M9 were each named at the rule-discovery tier. This threshold is named at the regime tier.

Doc 705's standing-apparatus framing applies one tier inward: where Doc 705 named cross-engagement durability of an apparatus's methodology, this names **cross-round durability of an engagement's rule-set within that engagement**. Same structural shape; finer grain.

Operational consequence: when consecutive rounds produce vacuous-with-reconciliation fold-backs against orthogonal pilot/fixture axes, the apparatus is in standing-rule production mode. The next move is *not* "look for a new rule" — it is "advance the count under the existing rules." A new rule only enters when the existing rules fail to produce a clean fold-back on a round; at that point the regime returns to rule-discovery temporarily and then ascends again.

10. **Persistence-across-orthogonal-axes is the live observable of Phase-2 basin stability.** Per Doc 709's pendulum-vs-basin resolution, Phase 2 (rule-standing-in-production) is named by the second SIPE-T threshold but its *persistence* needs a continuously-updated diagnostic. The signature: count the number of consecutive Tier-J fixture rounds that land J.1.a with **zero apparatus reconciliation** AND cover an **axis the prior fixtures did not cover**. Call this `N_persist`.

- `N_persist = 0` after any round that required an M8(a) reconciliation (the basin had to be widened — back to Phase 1 transiently).
- `N_persist` increments by 1 only when both conditions hold simultaneously (zero reconciliation **and** orthogonal axis).
- A round that lands J.1.a with zero reconciliation but covers an axis already covered does not increment — it confirms basin stability over the known basin, not extension over its actual coverage.

Predictive value: `N_persist ≥ 2` corroborates Doc 709's §7 deep reading (rule-set generalizes beyond named divergences). Drop to 0 corroborates Doc 709 §6 P1 (basin boundary = M-rule coverage, new pilot class re-enters Phase 1). The metric is testable each round; it is the engagement's live falsifier-direction signal for the Phase-2 claim.

Live tracking lives in the trajectory header; the seed names the discipline.

11. **M7 outcome taxonomy.** Each round's fold-back is classified as one of: (a) **primitive** — a new rule is folded back; (b) **vacuous** — no new findings, individual surface tested in isolation; (c) **compositionally vacuous** — a multi-pilot consumer-shape test exercises rule-composition across many primitives and produces zero findings; (d) **compositional finding** — a structural relationship between two existing rules is recognized; (e) **author-side (Mode 5)** — a test-discipline issue, not apparatus state; (f) **scope-limit verified** — a previously-recorded apparatus scope-limit is hit by a real consumer in expected fashion, validating the limit as binding rather than overly cautious. The taxonomy is itself a SIPE-T artifact: compositional and scope-limit-verified outcomes only become legible once enough primitives exist that *not finding* a primitive is informative. Compositional vacuity is qualitatively stronger than (b): it certifies the rule-set's joint behavior, not just per-rule behavior. The first Tier-J consumer (todo-api fixture, ESM + bare-specifier resolution + Bun.serve route table + URL/URLSearchParams/Request/Response + structuredClone/Map/Set/Date + Buffer + JSON, 10 self-test cases) produced compositional vacuity on first run. When a Tier-J consumer hits this state, sub-criterion 5 ("real consumer can swap rusty-bun for Bun") is demonstrated for that consumer.

12. **Three substrate modes, not two.** Doc 709's binary Phase-1 / Phase-2 framing is refined by the empirical record: three observable modes, not two.

  - **Phase-1 (basin-construction).** Pre-M7. Substrate drifts; each rung-2 intervention names a new rule that extends the basin. Pendulum-control regime per Doc 709.
  - **Phase-2-traversal.** Post-second-SIPE-T-threshold, within current basin. Substrate produces J.1.a fixtures with zero apparatus reconciliation; N_persist increments. Constructive-interference regime per Doc 710.
  - **Phase-2-extension.** Post-second-SIPE-T-threshold but the round deliberately widens the basin (close a recorded boundary via M8(a) apparatus extension). N_persist resets to 0 per §III.A8.10 — *not* because the substrate drifted but because the apparatus state changed. The mode looks like Phase-1 (apparatus extension, M8(a) firing) but occurs inside the Phase-2 regime; the basin is otherwise stable, one specific axis is being deliberately widened.

  Phase-2-extension was first observed at commit `59c5691` (node:os wiring, 2026-05-10). The persistence-tracker's reset behavior in that round is correct, not a regression. Doc 709 §4's two-phase resolution is preserved at the *engagement* tier; within-Phase-2 the substrate cycles between traversal and extension sub-modes. Each sub-mode has its own optimal K per Doc 710:
  - Traversal: K opportunistically high (1-3 observed) as long as basin has probe surface.
  - Extension: K typically lower (the round's productive surface is the apparatus work itself, not basket-expansion volume); accompanying fixture confirms the extension landed.

  Doc 709 future amendment may fold this back at corpus tier; current evidence is one extension round, sufficient to name the sub-mode, insufficient to characterize its asymptotic behavior.

13. **Substrate-amortization staging principle.** When a closure family shares an underlying mathematical or structural substrate (bigint arithmetic for RSA-OAEP/PSS; elliptic-curve arithmetic for ECDSA/ECDH/multiple-curves; presumably finite-field-extension arithmetic for any future pairing-based crypto), the optimal staging is **one substrate-introduction round** followed by **N closure rounds reusing the substrate**.

  The substrate-introduction round looks like Phase-2-extension at large LOC: a new primitive class lands (~200-400 LOC of fundamental machinery: BigUInt, Curve struct, finite-field operations). M7 fold-back is **primitive at the substrate layer**. K is typically low (1, sometimes 2 if a single fixture exercises the new substrate immediately).

  Closure rounds are **compositionally vacuous** at the rule layer: ~30-150 LOC each, threading the existing substrate to new surfaces (RSA-OAEP via MGF1 + EME-OAEP padding; RSA-PSS via EMSA-PSS encoding; ECDSA via FIPS 186-4 §6.4; ECDH via x-coord-of-d·Q). Each closure ships a Tier-J fixture; K may climb to 2-3 per round once the substrate is fluent.

  This pattern was empirically observed twice in the 2026-05-11 engagement run:
  - **Bigint substrate → RSA family**: `fb71d2d` (bigint primitives, no host wiring, no fixture — math-layer-only Phase-2-extension) → `2b86462` (RSA-OAEP closure, 4 hashes, fixture) → `660f94d` (RSA-PSS closure, fixture).
  - **EC substrate → EC family**: `8cc2ac5` (P-256 substrate + ECDSA-P-256 closure together, ~250 LOC) → `aae8dc2` (ECDH-P-256 closure, ~30 LOC) → `5a6ab71` (curve-parameterization refactor → ECDSA + ECDH over P-384 and P-521, four surfaces in one round).

  Doc 710 P1 is fully corroborated by these two runs: K-feasibility curve becomes gentler once a shared substrate is in place because the marginal cost of an additional closure has dropped from "introduce primitives + apply them" to "thread existing primitives through one more padding/dispatch rule."

  Operational implication: when planning a basket-expansion against a family of related surfaces, *do not* attempt to close all surfaces in one round if doing so requires also landing a new substrate. Stage: substrate first (small or no fixture, primitive M7 fold-back), then close surfaces in subsequent rounds with high K. Avoids >800-LOC rounds and isolates substrate bugs from application-layer bugs.

14. **Hand-typed multi-byte constant discipline.** Standard cryptographic constants (NIST curve parameters, RFC test vectors, FIPS coefficients) hand-typed into source code MUST be sanity-checked against an independent implementation (Python `cryptography`, OpenSSL, Bun WebCrypto, another standards-aware library) before being trusted by any downstream operation.

  Mode-5 (author-side) typos in these constants are **silent**: every operation downstream produces wrong-but-plausible output because the math doesn't error on a value that just happens to not satisfy a curve equation or wire-vector. The 2026-05-11 engagement run surfaced three such bugs (bug-catcher F4, F5, F6):
  - **F4**: RFC 7914 PBKDF2-HMAC-SHA-256 expected hex had two transposed hex digits in the verifier test; my implementation was correct, the hardcoded *test expectation* was wrong.
  - **F5**: P-256 G_y in the pilot was 16 bytes of a different value (apparently copied from a typo'd reference); the curve equation failed silently and 2G computed to a non-canonical point that nevertheless lay on a *different* curve consistent with the typo'd G_y.
  - **F6**: P-521 prime hex was missing 2 'f' digits (130 chars instead of 132 → bit_length 513 vs canonical 521); modular arithmetic gave plausible-but-wrong outputs, the on-curve check failed for the generator.

  In all three, character-by-character visual review of my hand-typed value against the standard source did not catch the bug. The catching mechanism was always **sanity-check against an external independent implementation** — Python's `cryptography` library + Tonelli-Shanks-from-NIST-spec for F5, Python bigint + same curve formula for F6, Python PBKDF2-HMAC-SHA-256 vs my impl for F4.

  Operational rule: any commit landing >32 bytes of hand-typed standard constants must include or be preceded by a sanity-check run against an external reference. The check costs ~30 seconds; the alternative is hours of debugging silently-wrong cryptography that LOOKS like it works.

15. **Third SIPE-T threshold: author-side-bug-dominance.** The first two SIPE-T thresholds (§A8.8 + §A8.9) named substrate transitions: primitive-discovery → rule-composition (composition discipline emerges) → rule-standing-in-production (M-rules do the cognitive work). The third threshold, observed during the seven-round Phase-2-traversal sequence 2026-05-11 (`1e18c71` JWKS-verifier → `bcae7bc` mustache-mini → `18283d3` csv-mini → `d502c68` markdown-mini → `3d0bd81` async-pool → `4c9d1c0` signals-mini → `056484c` mini-router), is the substrate's bug-population inverting: apparatus-side bugs drop to zero per round while Mode-5 (author-side, per §A8.12 modes) bugs become the only failure mode the differential surfaces.

  Empirical record across the seven rounds:
  - Apparatus reconciliations (M8(a) firings): **zero**
  - Mode-5 author-side bugs surfaced and caught by the comparator (Bun) differential: **seven** — F7 (regex alternation ordering), three unrecorded in-fixture iterations (HTML pass-through in markdown-mini, abortAll-misses-in-flight in async-pool, diamond-fires-twice in signals-mini), and the equivalent silent-typo-class bugs in earlier rounds (F4/F5/F6 during Phase-2-extension).

  Operational signature: when the bug-population at fixture-author time is dominated by author-side (typos, semantic-ordering, lifecycle-coverage-gaps, composition-bugs in the test author's code) and the catch mechanism is universally the comparator-differential (not the apparatus's internal tests), the apparatus has crossed the third threshold. Practically: this signals that the basin is mature enough that the engagement's productive surface is no longer "extend the apparatus" but "extend the consumer-side evidence" — Tier-J fixtures that exercise novel real-world composition patterns. Doc 709 §7's deep reading (rule-set generalizes beyond named divergences) is at this point empirically validated, not just predictive.

  Predictive value: above this threshold, K-multiplicity per round is no longer the bottleneck (apparatus extensions are rare). The bottleneck becomes axis-novelty selection (does this Tier-J fixture exercise a basin-area the prior fixtures didn't?). Phase-2-traversal rounds against axes the apparatus already serves predictably add ~1 to N_persist with no other apparatus change; this is the steady-state regime, and the engagement may pivot to corpus-tier doc writing (per M3) or scope-extension (e.g., Tier-G transport) for further high-leverage moves.

8. **Composition discipline (SIPE-T tier):** canonical-docs tests + M7 fold-back compose. Canonical tests use idioms that exercise language-level affordances (iteration protocols, async iteration, polymorphic argument shapes, prototype-chain checks) which per-method tests do not exercise. When a canonical test breaks, the failure mode is often indistinguishable at first glance from a higher-level bug (e.g., the CJS-loader round mistook URLSearchParams' missing `[Symbol.iterator]` for a module-resolution bug). M7 reflection on such breaks recovers the actual primitive gap — a *language-affordance gap* in the JS-side wrapper — and folds it back. Neither discipline alone catches these: per-method tests miss them because no individual method is broken; M7 alone would not surface them because nothing visible misbehaves until idiomatic composition is attempted. The two together name the gap. Treat this as a structural relationship between the two rules, not a third rule — the apparatus has reached the tier where the productive surface is **rule-composition**, not new-rule-discovery.

## IV. Future-move discipline

**M1. Pilot prioritization.** The next pilot is chosen from the trajectory's queue. Selection criteria, in order:
1. **Dependency unblocking** — the pilot is a substrate other queued pilots need.
2. **Class diversity** — the pilot anchors a class the apparatus has not yet validated.
3. **LOC leverage** — the pilot's reference target is large enough to anchor the value claim materially.
4. **Cross-corroboration density** — the pilot's constraint corpus is rich enough to drive a clean derivation.

**M2. Apparatus-vs-pilot triage.** When a pilot surfaces an apparatus refinement (e.g., the cluster-phase leakage fix from the TextEncoder pilot), prioritize the refinement over the next pilot. The hardening floor compounds — see [Doc 706](https://jaredfoy.com/resolve/doc/706-three-pilot-evidence-chain-derivation-from-constraints) on the v0.12-v0.13 chain.

**M3. Doc-tier writing trigger.** A corpus-tier doc is written when (a) a structural insight crystallizes that wasn't articulable in the prior doc, OR (b) the keeper directs explicitly. Don't auto-write docs after every pilot; let understanding mature.

**M4. Bigger pilots are run only after the hardening floor is firm.** A pilot like Bun.serve or Buffer (Tier-2, large) is attempted only when smaller pilots have validated the apparatus class structure for that pattern. Don't scale to 6,000-LOC reference targets until 300-LOC reference targets are clean.

**M5. Deferred items have explicit re-open conditions.** Per Doc 581 D4. "Reopen when X obtains" — not "someday."

**M7. Resolution-increase pass is a recurring mode, not a keeper-triggered event.** Between any two implementation rounds (e.g., consecutive Tier-H wirings), the apparatus must self-check: *did the just-completed round expose patterns, type-translation idioms, JS-side decoding shapes, or verification disciplines that are not yet captured in seed §III/§IV, bug-catcher, or HOST-INTEGRATION-PATTERN.md?* If yes, fold them back BEFORE picking the next implementation item.

This mode exists because the level-2 cybernetic loop (apparatus self-iteration, per Doc 708) was empirically observed to be keeper-mediated: in the 2026-05-10 session, three rounds of host wirings landed without their patterns being formalized; only a keeper rung-2 prompt ("have we increased resolution against context?") triggered the fold-back. The loop is not self-closing without this rule. M7 closes it: the fold-back trigger fires automatically between rounds, not on keeper prompt.

Concrete trigger conditions (any one fires the pass):
  - A new cross-boundary type translation was used (e.g., `Vec<Vec<String>>` for tuple-list).
  - A new JS-side decoding shape was discovered (e.g., method-keyed dispatch, polymorphic shape discrimination).
  - A new verification discipline emerged (e.g., canonical-docs composition test).
  - An author-side bug recurred (Mode 5 of the operational modes) suggesting a bug-catcher entry.
  - rquickjs / QuickJS interaction surprised the integration in a way not yet in HOST-INTEGRATION-PATTERN.md.

The pass updates seed §III/§IV, bug-catcher, or HOST-INTEGRATION-PATTERN.md, then is committed as `Sharpen resume vector: integrate <round-name> patterns`. Only after that commit lands may the next implementation round begin.

**M9. Spec-first fixture authoring.** Tier-J fixtures are authored against the comparator runtime's *specified* API from inception, not against rusty-bun's current surface. The authoring loop: (1) write the fixture using Bun-spec idioms; (2) run under Bun first to capture the comparator's output; (3) run under rusty-bun-host; (4) for each divergence surfaced during step 3, apply M8 in-round (align the apparatus, or scope-limit + remove); (5) commit fixture and reconciliations together. Consequence: fixtures ship J.1.a directly without ever transiting J.1.b.

**M9.bis — No dual-path emission.** A fixture must use the canonical Bun-spec surface *directly* without `if-typeof-defined-then-X-else-Y` graceful-degradation paths. Reason: graceful degradation silently bypasses absent surfaces. The engagement learned this concretely at commit `c0567e3` — 17 fixtures had `if (typeof process !== "undefined" && process.stdout) { process.stdout.write(...) } else { globalThis.__esmResult = ...; }` patterns where the first branch was silently bypassed under rusty-bun-host (process was absent), the fallback carried the result, and the differential was passing while rusty-bun-host had a major boundary the apparatus had not surfaced. The persistence metric `N_persist` cannot detect this — it tracks fixture-level outcomes, not surface-level coverage; only the probe-then-extend discipline catches it. M9.bis forbids the silent-bypass class of fixture by mandating single-path emission. Real consumer code does not have dual-paths; Tier-J fixtures must not either. A fixture that fails on rusty-bun-host because a Bun-spec surface is absent is M8's job to handle; it is NOT the fixture's job to mask the absence.

This is the inverse of the natural flow (write against what you have, then maybe align later). It works because a fixture authored against the comparator's spec is *already in J.1.a's shape by construction* — the only question is whether the apparatus has caught up. M8 catches the apparatus up; M9 ensures the question gets asked at fixture-author time, not after a separate "porting" round that would itself be drift.

Operational consequence: J.1.b becomes a transient never-occupied state in the current-cycle basket. A fixture occupies J.1.b only when a divergence cannot be reconciled in the current round and the fixture must be temporarily parked with explicit re-open conditions. Under M9, this is rare; under the prior implicit practice of "build against rusty-bun then port," J.1.b was the default landing state.

M9 was operationalized after consumer-request-signer (2026-05-10) shipped J.1.a from inception with one in-round M8 reconciliation (digest API), demonstrating that the fixture-build → divergence-surfacing → reconciliation → commit cycle works as a single coherent unit rather than as separate phases.

**M8. Divergence reconciliation is non-deferrable.** When a Tier-J differential surfaces a divergence between rusty-bun and the comparator runtime (Bun), the divergence must be reconciled in the round it is discovered, before the next round begins. There are exactly two acceptable reconciliations: (a) bring the apparatus into alignment with the comparator (preferred, if feasible within the current round); (b) explicitly record the divergence as an intentional scope-limit with a re-open condition per Doc 581 D4 (the deferred-list discipline) AND remove from the Tier-J fixture set every fixture that depends on the divergent shape, so subsequent fixtures cannot be built on the misaligned plank.

What is forbidden: "noted, will deal with later." That phrasing is the drift mechanism. Each Tier-J fixture built atop an unreconciled divergence inherits the misalignment; the cumulative error grows monotonically with rounds. M7 closes the level-2 loop for primitive-discovery; M8 closes it for divergence-reconciliation. Both are needed because both are mechanisms by which substrate work can drift out of plumb.

This rule was instituted after the first body-async asymmetry was nearly deferred under a "vacuous-with-asymmetry-noted" classification (the rusty-bun host's sync .text()/.json() body methods diverge from Bun's spec-async API). The classification was wrong — it normalized deferral. The keeper named the drift risk explicitly: *"each plank must be plumb or else it will drift out of plumb over subsequent planks."* M8 is the cybernetic compensation.

**M6. Host-wirability is a pilot design constraint.** New pilots' Rust APIs are designed to wire cleanly through the JS host pattern (A8). Concretely: prefer pure-value APIs; avoid `Rc<RefCell<...>>` in public interfaces; stateful types should provide stateless algorithm helpers alongside their owned-state types so the host can wire the helpers without adapting the type's storage. A pilot is "host-wirable" when its public API can be exposed via `host/` with no apparatus refinements — verifying this is a pilot-completion check.

**M10. Substrate-amortization staging.** When a queued surface family shares an underlying mathematical or structural substrate not yet in the apparatus, do NOT attempt to land both the substrate and all dependent surfaces in one round. Stage:
  1. Substrate-introduction round (Phase-2-extension; primitive M7 fold-back; small or no Tier-J fixture; pilot tests only).
  2. N closure rounds reusing the substrate, each landing one or more Tier-J fixtures (compositionally vacuous M7 fold-backs; K may climb).

  The rule operationalizes §III.A8.13. Trigger: if the next planned round's pilot diff exceeds ~400 LOC AND a >50-LOC subset of the diff is shared-substrate machinery (bigint, EC, finite-field ext, etc.), split the round into substrate-first + surfaces-second. See §III.A8.13 for the empirical record (bigint→RSA family; EC→ECDSA/ECDH family) corroborating Doc 710 P1's substrate-amortization prediction.

**M11. External-reference sanity-check for hand-typed multi-byte constants.** Any commit landing >32 bytes of hand-typed standard cryptographic constants (NIST curve parameters, RFC test vectors, FIPS coefficients, ASN.1 OIDs) must include or be preceded by a sanity-check run against an independent implementation (Python `cryptography`, OpenSSL, Bun WebCrypto, or another standards-aware library).

  The rule operationalizes §III.A8.14. Visual character-by-character review is insufficient — Mode-5 typos in cryptographic constants are silent (the math doesn't error; downstream operations produce wrong-but-plausible output). Three such bugs surfaced in the 2026-05-11 engagement (bug-catcher F4/F5/F6); each was caught only by external sanity-check, never by visual review.

  Operational cost: ~30 seconds of Python/Bun invocation per constant. The alternative is hours of debugging silently-wrong cryptography that LOOKS like it works.

## V. Deferred-list discipline

The trajectory's Deferred section lists items considered and explicitly *deferred*, with re-open conditions. The seed names the discipline; the trajectory holds the items.

**Examples of permanent deferrals (re-open conditions are negation-of-pilot-goal):**
- Bun's transpiler / bundler (Bun.build internals): different problem class — compiler, not runtime surface.
- HTTP/2 / HTTP/3 transport-layer details: scope is the runtime API surface, not the wire protocol.
- Inspector / debugger / DevTools protocol: tooling, not runtime API.

**Examples of conditional deferrals:**
- Bun.serve full (with sockets): re-open when streams pilot lands AND the apparatus is ready to model HTTP transport at data-layer fidelity.
- Wired rederive integration: re-open when the LLM-simulated derivation has saturated the apparatus' useful pilot space.
- WPT suite execution against pilots: re-open when a JS-host shim (Boa or QuickJS) is on the table.

## VI. Operational interfaces

**Apparatus binary.** `derive-constraints` at `derive-constraints/`. Version state recorded in commit messages and `runs/<run>/RUN-NOTES.md`.

**Pilot crates.** Single Cargo workspace at the repo root `Cargo.toml` registers all 16 pilots + `derive-constraints` + `welch`. `cargo test --workspace --release` runs every test in one shot. Wrapper script at `bin/run-pilots.sh` emits a structured summary.

**Constraint corpus.** Re-extracted at `runs/2026-05-10-bun-v0.13b-spec-batch/cluster.json`. Older runs preserved for diff. Per the pipeline driver, re-running takes ~10s on Bun's full test corpus.

**Spec corpus.** `specs/*.spec.md` — 15 surfaces curated as of v0.13b. Format documented at `specs/README.md`.

**Test runner.** `bin/run-pilots.sh` from the repo root runs the entire workspace and emits a structured summary. Equivalent: `cargo test --workspace --release`.

## VII. What completion looks like

**Telos: the rusty-bun derivation is complete against Bun.** A real consumer can swap rusty-bun for Bun and run their JS-using application without regression, against the cited consumer corpus and against Web Platform Tests for spec'd surfaces. Per C1 (plug-and-play interoperability with no regressions), but at the **runtime level**, not just the surface-API level.

This is a much larger commitment than the prior framing. The engagement's prior milestone — apparatus saturation at 16 pilots / 8 architectural classes (Doc 708) — is a **necessary** precondition for completion but not sufficient. Saturation establishes that the apparatus' methodology works; completion requires applying the methodology across Bun's full runtime API surface, integrating with a JS engine, and demonstrating differential equivalence against actual Bun-using applications.

Three cybernetic compensation rules govern progress toward completion and prevent drift:
  - **§IV.M7** closes the level-2 loop for primitive-discovery: every round must self-check for new patterns and fold them back before the next round begins.
  - **§IV.M8** closes the level-2 loop for divergence-reconciliation: every divergence between rusty-bun and the comparator runtime must be reconciled in the round it is discovered, not deferred.
  - **§IV.M9** prevents the divergence in the first place: Tier-J fixtures are authored spec-first against Bun, not against rusty-bun's current surface, so divergences surface during authoring and reconciliations land in the same commit.
Without M7 the substrate accumulates work without consolidating its primitives; without M8 fixtures inherit misalignment from prior fixtures and the differential count never converges; without M9 every fixture round transits J.1.b before reaching J.1.a, doubling the work and creating windows where drift can compound. M7 and M8 were instituted under keeper rung-2 intervention; M9 was operationalized after a fixture (consumer-request-signer) shipped J.1.a from inception under spec-first authoring, demonstrating the workflow as a single coherent unit. All three are self-triggering.

The completion telos has five sub-criteria, in dependency order:

**Sub-criterion 1 — Apparatus saturation.** ✓ MET (Doc 708, 2026-05-10).
Sixteen pilots × eight architectural classes × five cybernetic modes × ~3% aggregate LOC ratio. The methodology is empirically anchored and ready for application.

**Sub-criterion 2 — Surface-API completeness.** Every Bun runtime API has a pilot anchor with verifier + consumer-regression closure. Estimated ~50-80 additional pilots beyond the current 16 to cover:
- Web Crypto full (subtle.generateKey/deriveKey/importKey/exportKey/sign/verify, HMAC, AES, RSA, ECDSA, Ed25519, HKDF, PBKDF2, SHA-384/512)
- Streams full (BYOB reads, async iterator protocol, transferable streams, pipeTo/pipeThrough automation)
- Node-compat: net, tls, dgram, dns, zlib, stream (Node), events, os, cluster, worker_threads, vm, perf_hooks, async_hooks, readline, repl, tty, assert, timers, inspector, module
- Bun-namespace: Bun.password, Bun.SQLite, bun:redis, bun:s3, Bun.Cookie, Bun.JSONL, Bun.Image, Bun.Archive, Bun.Terminal, Bun.cron, Bun.Glob, Bun.YAML, Bun.CryptoHasher, Bun.deepEquals, Bun.inspect, Bun.write, Bun.connect, Bun.listen, Bun.dns, Bun.fileURLToPath, Bun.pathToFileURL, etc.

**Sub-criterion 3 — Transport-layer pilots.** The data-layer-only pilots (fetch-api, Bun.serve, Bun.spawn, node-http) lift to wire-format pilots. Includes HTTP/1.1 + HTTP/2 wire parsing, socket binding, TLS handshake, WebSocket upgrade, IPC channels, streaming stdio. Required for any of these surfaces to function as runtime API.

**Sub-criterion 4 — JS host integration.** Embed a JS engine (QuickJS or Boa) and expose all pilots to JS code via FFI. Includes module loader / resolver, console + global setup, the `globalThis` shape Bun provides. Without this, no JS code can execute against the derived runtime; with it, rusty-bun becomes a runtime in the operational sense. SUBSTANTIALLY MET 2026-05-10: rquickjs embedded; 19 pilot families wired; CommonJS + ESM module loaders both honor relative + bare-specifier resolution with node_modules walk-up + `node:*` builtin scheme; timers + queueMicrotask + performance + URL globals wired; Buffer wrapped as Bun-portable Uint8Array subclass.

**Sub-criterion 5 — Differential testing against Bun-using applications.** The operational form of plug-and-play. For a representative basket of Bun-using applications (frameworks like Hono / Elysia, real-world apps): run `npm test` under Bun → record P_bun. Run under integrated rusty-bun → record P_drv. Diff. **Zero regressions across the basket** = real plug-and-play.

Closure of this sub-criterion is **per-fixture differential**: a fixture counts toward sub-criterion 5 only when it produces byte-identical output under Bun and rusty-bun-host (the J.1.a category in the trajectory's Tier-J basket). Fixtures that exercise the apparatus but depend on rusty-bun-only shapes (J.1.b) are host-internal regression tests, not sub-criterion-5 evidence — they must either be reconciled per §IV.M8 to enter J.1.a, or remain explicitly out of the differential count with a re-open condition.

This is what makes the criterion non-deferrable. Each fixture not yet in J.1.a is a permanent ratchet against the eventual count; new fixtures cannot be built atop unreconciled divergences without inheriting the misalignment (the plank metaphor in §IV.M8). M8 enforces the rule at the round level; sub-criterion 5 enforces it at the telos level.

A complementary signal: run Web Platform Tests against the integrated runtime via `wpt run` adapter. WPT pass-rate per surface is a published number for browser engines and Bun itself; rusty-bun's WPT pass-rate becomes an operational comparison.

**The trajectory holds the per-surface pilot list, the transport-layer queue, the JS-host integration plan, and the differential-test basket.** The seed names the criterion; the trajectory holds the work.

A list of which surfaces are "load-bearing" lives in the trajectory and is updated as new ones surface. The four criteria from the prior framing of completion (coverage of architectural classes / aggregate-ratio holding / consumer-corpus closure / doc-tier production) collapse into Sub-criterion 1 (apparatus saturation) under the new telos. Doc 708 records that sub-criterion as met.

## VIII. Hypostatic boundary

This seed describes the structural shape of an apparatus and an engagement. It does not assert that "Bun is a constraint corpus" or "the apparatus is the truth of Bun." Bun is what Bun is; the apparatus produces a *reading* of Bun useful for derivation and for dependency-surface mapping. Per Doc 372's discipline — and Doc 581 §VIII — a Resume Vector is functional, not ontological.

A different keeper with a different apparatus could produce a different reading of Bun, and both could be true under their respective accountings. This engagement's reading is one operational instance of Pin-Art on a runtime; other readings are possible.

## IX. Update protocol

**This file changes only when the architecture itself moves.** Examples that warrant update:
- A new pin class is added to the apparatus (currently five; another would justify update).
- The three-tier authority taxonomy is revised.
- The pipeline shape changes (e.g., a new phase is inserted).
- The completion criterion is revised (e.g., a new criterion is added).
- A binding constraint is added, removed, or revised.

Trivial trajectory updates DO NOT propagate here. Per Doc 581 D5: if this seed is changing more than once per few sessions, the architecture has not stabilized yet and the seed should be reconsidered.
