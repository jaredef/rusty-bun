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

  This pattern was empirically observed **five times** across the 2026-05-11 engagement run:
  - **Bigint substrate → RSA family**: `fb71d2d` (bigint primitives, no host wiring, no fixture — math-layer-only Phase-2-extension) → `2b86462` (RSA-OAEP closure, 4 hashes, fixture) → `660f94d` (RSA-PSS closure, fixture).
  - **EC substrate → EC family**: `8cc2ac5` (P-256 substrate + ECDSA-P-256 closure together, ~250 LOC) → `aae8dc2` (ECDH-P-256 closure, ~30 LOC) → `5a6ab71` (curve-parameterization refactor → ECDSA + ECDH over P-384 and P-521, four surfaces in one round).
  - **DER substrate → X.509 + TLS family** (Π1.4 sequence): `fac8fd9` (ASN.1/DER reader, 340 LOC, no host wiring) → `327dfc5` (X.509 v3 parsing + signature verification, 520 LOC) → `c6a67c1` (TLS record layer + trust store + chain walk, 480 LOC) → `c824ada` (TLS 1.3 handshake framing + key schedule + AEAD record wrap, 350 LOC, RFC 8448 byte vectors verified) → `25a267e` (ClientHello + ServerHello + extensions, 300 LOC) → `efd353a` (driver skeleton + ECDH ephemeral, 190 LOC) → `3f2ae9a` (full handshake state machine + CertificateVerify dispatcher, 210 LOC) → `cd683ef` (TcpTlsTransport + live slow test) → `1ca5d1c` (host wiring globalThis.__tls) → `57505e5` (Tier-J consumer-tls-namespace-suite) → `8d02c5f` (middlebox-compat fix, live handshake against openssl passes) → `2006cd7` (fetch() HTTPS routing). Eleven sub-rounds; substrate-amortization at its largest observed scale.
  - **WebSocket frame codec → JS WebSocket class** (Π1.5 sequence): `7664438` (RFC 6455 frame codec + handshake key derivation pilot, 280 LOC, RFC 6455 §1.3 + §5.7 vectors verified) → `cfe55cb` (host wiring globalThis.__ws + Tier-J consumer-ws-primitives-suite) → `4cd18d3` (JS-side WHATWG WebSocket class) → `9d2ea44` (live ws:// round-trip against Bun-spawned echo server). Four sub-rounds.
  - **Blake2b substrate → Argon2id family** (Π4.14, in-flight): `1e5eb09` (Blake2b per RFC 7693, RFC §A "abc" + empty-input vectors verified, 100 LOC added to rusty-web-crypto). Argon2id closure + Bun.password JS wiring is the queued continuation.

  Doc 710 P1 is fully corroborated by these five runs: K-feasibility curve becomes gentler once a shared substrate is in place because the marginal cost of an additional closure has dropped from "introduce primitives + apply them" to "thread existing primitives through one more padding/dispatch rule." The TLS run extends the corroboration to large-scale substrate sequences (11 sub-rounds composing on a single DER substrate); the K-multiplier per substrate-introduction round scales sub-linearly with the substrate's structural depth.

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

16. **Harness-tier process-global state requires a serial guard, DI threading, or explicit non-shared isolation.** Any harness (test, seed, build) that mutates process-global resources during execution under parallel scheduling will silently deadlock or produce cross-test contamination unless one of three patterns holds: (a) a static `Mutex<()>` serializes the critical section across parallel threads; (b) the resource is threaded through as a per-test config and not shared; (c) an explicit isolation contract makes single-threading the documented invariant. Process-global resources include but are not limited to: environment variables, the working directory, well-known port numbers, signal handlers, file paths used as IPC/locks, on-disk SQLite databases.

  **Why visual review fails.** Each test or harness reads correctly in isolation. The race is invisible without thinking about parallel scheduling. Single-threaded execution (`cargo test -- --test-threads=1`, or running scripts one at a time) passes; default parallel execution hangs or fails. F9-recovery instincts (re-check the accept bound, re-check the test's apparatus) come up empty because the test-internal logic is correct; the bug lives at the harness/apparatus tier.

  **Catching mechanism.** If a test or pipeline (a) passes single-threaded but hangs/fails parallel, OR (b) fails intermittently when other processes are running concurrently, OR (c) shows `futex_wait_queue` waits on /proc inspection while no progress is visible, the suspect is a shared mutable resource at the harness tier. Add the rule's pattern (a), (b), or (c) and re-run.

  **Empirical record.** Four corroborating incidents on 2026-05-11:
  - **F9** (integration-test server-thread accept-loop deadlock-at-join): well-known-port contention (the listener) plus accept-count mismatch between harness and fixture. First instance.
  - **F12** (parallel cargo tests racing on a process-global env var route fixture traffic to the wrong server): two parallel tests overwriting `FETCH_TEST_PORT` env var; one test's fixture connected to the other test's server. Fixed by `static Mutex<()>` (pattern a) inside `with_fetch_target_server`. Second instance.
  - **Seed-corpus DB race**: a re-run of `bun run seed` while a prior seed's `inject-links.ts` subprocess still held `app/data/corpus.sqlite` open caused `DROP TABLE IF EXISTS content` to fail with a SQLite-busy error. The shared resource was the DB file. Retried sequentially and succeeded. Third instance.
  *(A provisionally-entered fourth corroboration — a 72-minute host-suite hang on 2026-05-11 — was reclassified to §A8.17 after diagnosis showed the worker threads were runnable, not blocked, and the apparent "hang" was the cumulative cost of the bigint/EC/RSA test cluster under parallel scheduling. §A8.16's empirical record stands at three correctness-class incidents.)*

  **Generalization.** The bug-catcher's F-series accretes individual incidents; this seed-tier rule names the cross-incident pattern. Once stated as §A8.16, future rounds must apply one of patterns (a), (b), or (c) to any harness work that mutates process-global state. Per [Doc 685](https://jaredfoy.com/resolve/doc/685-the-self-reinforcing-boundary)'s recurrence equation, the rule self-reinforces: substrate cannot drift back into the pattern without crossing a named boundary. Per [Doc 707](https://jaredfoy.com/resolve/doc/707-pin-art-at-the-behavioral-surface-bidirectional-probes)'s bidirectional reading, each incident's forward fix (the specific guard) and backward invariant (this rule) are now both recorded.

  **Phase classification.** §A8.16's institution is a Phase-2-extension micro-round (per §A8.12): apparatus-tier work, not within-basin traversal. N_persist resets to 0 on this commit, deliberately. The basin is now wider by one named boundary.

17. **Test-cost stratification: inner-loop tests vs scheduled-burst tests.** The engagement's reference hardware (Raspberry Pi) sets an inner-loop wall-clock budget. Tests whose individual cost exceeds 30 seconds on this hardware are marked `#[ignore]` (Rust's built-in stratification primitive) so default `cargo test` runs the inner-loop set only; full coverage runs via `cargo test -- --ignored` or `./bin/run-pilots.sh --slow`. Target default-suite wall-clock: under 3 minutes.

  **Why visual review fails.** Each test passes when run. Nothing flags that running it inside an iteration loop turns a 10-minute development round into a 60-minute round. The cost only surfaces at the loop's outer edge — when the apparatus running its own diagnostic at full-suite scope blocks substantive work.

  **Catching mechanism.** Cargo's built-in `running for over 60 seconds` slow-test warning is the signal. Any test that trips it is a candidate for `#[ignore]`. Exception: a test that is the canonical evidence for a primitive (substrate-introduction round's verifier) is run once on its introducing commit and `#[ignore]`'d thereafter.

  **Empirical record 2026-05-11 (fourteen corroborating instances).** All from the bigint/EC/RSA arithmetic cluster:
  - Consumer pass: `js_consumer_jwks_verifier_suite_runs_clean`, `js_consumer_jwt_rs256_suite_runs_clean`, `js_consumer_ec_curves_suite_runs_clean`, `js_consumer_ecdh_p256_suite_runs_clean`, `js_consumer_ecdsa_p256_suite_runs_clean`, `js_consumer_rsa_pss_suite_runs_clean`, `js_consumer_rsa_oaep_suite_runs_clean`.
  - Differential pass: the seven `js_differential_consumer_<surface>_matches_bun` analogues of the above. Each runs the EC/bigint work twice (once under rusty-bun-host, once under Bun comparator), doubling the cost.

  The earlier-suspected "full-suite deadlock" was diagnostic confusion between this rule's regime and §A8.16's: process-state inspection (`State: S (sleeping)`, threads on `futex_wait_queue`) was misread as a lock; the actual signature was main-thread waiting on workers that were CPU-bound, not blocked. The misreading was caught when the rerun's worker threads showed `wchan: 0` (runnable), not `futex_wait_queue`.

  **Composition with C7 and the engagement's commit discipline.** The `#[ignore]` annotation is a working-file edit, not a substantive change to apparatus behavior — the tests still exist, still pass, are still authoritative evidence. The commit lands at the keeper's authorization per C7. Full-suite slow-test runs happen at engagement-closure milestones (Tier-Π phase closure, host-iteration completion, doc-tier completion records). Inner-loop rounds run the fast set.

  **Bidirectional Pin-Art reading per [Doc 707](https://jaredfoy.com/resolve/doc/707-pin-art-at-the-behavioral-surface-bidirectional-probes).** Forward: the fourteen `#[ignore]` annotations. Backward: the inner-loop cost budget is itself a Pin-Art constraint on apparatus operability. The reference-hardware budget sets which tests live at the inner-loop tier versus the scheduled-burst tier. Tests that exceed it are not bad tests — they are tests that belong at a different stratum.

  **Phase classification.** §A8.17 is a Phase-2-extension micro-round bundled with §A8.16 in the same apparatus-tier commit. N_persist already reset to 0 by §A8.16; no additional reset for §A8.17 (it lands in the same Phase-2-extension transition). The basin is now wider by two named boundaries.

  **`./bin/run-pilots.sh --slow` flag.** Added in the same commit. Pipes `-- --include-ignored` through to cargo. The pilot runner emits identical output shape with or without `--slow`; only the test set differs.

18. **Fourth SIPE-T threshold: substrate-standing-in-production.** The first three SIPE-T thresholds (§A8.8, §A8.9, §A8.15) named transitions in *rule* dynamics: primitive-discovery → rule-composition → rule-standing-in-production → author-side-bug-dominance. The fourth threshold, observed during the post-compaction continuation session 2026-05-12 (commits `a24ea63` → `0427f68`, 28 sub-rounds), is the next tier: **substrate-standing-in-production** — the substrate set doing the work that previously required apparatus-extension per consumer.

  **Empirical signature.** Of seventeen Π5 real-OSS differentials in the session, **nine had zero apparatus reconciliation** (zod, valibot, uuid, ms, yaml, composed-stack, composed-mini-app, dayjs-after-its-esm-heuristic, **koa**). The marginal cost-per-lib has flattened to zero on the basin's interior. Express required nine substrate edges; koa immediately after required none of them. The substrate gains landed for one consumer carried every subsequent consumer that depends on overlapping surface without further intervention.

  **Distinction from §A8.15 (third threshold).** Third threshold: M-rules do the cognitive work that previously required keeper rung-2 input per round (regime-tier internalization of *rules*). Fourth threshold: substrate gains do the apparatus-extension work that previously required per-consumer M8(a) reconciliation (regime-tier internalization of *substrate*). The two thresholds compose: third closes per-round meta-work; fourth closes per-consumer apparatus-work. The dyad-ascends pattern continues — rule-discovery → rule-composition → rule-standing → substrate-standing.

  **Three markers of the crossing.** (1) Zero-reconciliation rounds are the modal case, not exceptional (9/17 = 53%). (2) Two canonical web frameworks separated by ~15 years of design evolution (express ^4 + koa ^2) — distinct authors, distinct internal architectures, distinct dep trees — both close on the same substrate without per-framework adaptation; the substrate proved invariant against the framework axis. (3) Keeper-mediation shifts again, naming the *position on the SIPE-T curve* rather than naming individual rules or regimes: *"This seems like it is getting to the top of the SIPE-T curve."*

  **The express → koa transition as the threshold moment.** Express dragged ten substrate gains with it across this session's drilling: E.13 CJS-in-ESM bridge → destructure-export rewrite → node:tty/zlib/child_process stubs → Error.captureStackTrace polyfill with structured CallSite array → util.inherits + Stream-as-class-with-statics → ESM strict-reserved filter → path.resolve+family → Buffer.isBuffer → crypto.createHash/Hmac. The substrate gains were extracted via per-dep probes drilling through 30+ transitive CJS dependencies. Koa was tested immediately after express landed; it loaded and dispatched and responded byte-identically to Bun on first attempt. **The substrate's invariance against the framework axis is the threshold's defining property.**

  **Per M12 (basin-expansion-at-L2M-saturation), the next productive surface moves up the lattice.** Round-tier work has exhausted its marginal value for consumers in the existing basin. Artifact-tier (this seed entry; trajectory done-log + Status block) and corpus-tier (Doc 708 ninth amendment) consolidation is the coherent move. New rounds at this point either (a) deliberately widen the basin against a recorded boundary (Phase-2-extension, §A8.12), or (b) operate against a new consumer axis the basin has not yet been tested against (Phase-2-traversal of the not-yet-validated surface).

  **Composition with Doc 710 P1.** Doc 710 P1 predicts K growth with N_persist. The fourth threshold is the empirical signature of P1 saturating: K does not just grow but plateaus at the basin width as N_persist increases past the substrate-recursion depth of the consumer corpus. The basin width is now wider than any single npm consumer's dependency depth in the surface area tested.

  **Phase classification.** §A8.18's institution is itself the M12 move per [Doc 714 §VI.3](https://jaredfoy.com/resolve/doc/714-the-rusty-bun-engagement-read-through-the-lattice-extension-basin-expansion-at-the-l2m-saturation-point): apparatus-tier consolidation at L2M-saturation. N_persist resets is not applicable — this is a meta-rule about rule-dynamics, not a basin-expansion against a specific surface. The basin is named, not extended. Reference: trajectory Status block at session-close; Doc 708 ninth amendment; rusty-bun commit `0427f68` (koa landing); rusty-bun commit `7b0750e` (status-block + SIPE-T marker added).

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

**M12. Basin-expansion-at-L2M-saturation.** When a long-running engagement session reaches the L2M-saturation point at the session tier (per [Doc 700 Appendix C](https://jaredfoy.com/resolve/doc/700-l2m-resolved-against-the-corpus-bipartite-mutual-information-scaling-as-empirical-grounding-for-the-pin-art-channel-ensemble) and the diminishing-per-round-productive-surface signal), the next move is not "push another substrate round." It is to re-read the engagement at the lattice-extended scope per [Doc 572](https://jaredfoy.com/resolve/doc/572-the-lattice-extension-of-the-ontological-ladder) and concentrate productive surface at the artifact and corpus tiers where the L2M-bound does not constrain in the same way.

  Operationally:
  1. Consolidate the operating seed (this file) — fold cross-incident bug-catcher entries into seed §A8 where the generalization has stabilized; re-organize §A8 entries by the lattice they participate in rather than by chronological accretion.
  2. Summarize the trajectory's done-log into a corpus-tier completion-record amendment (per the Doc 708 amendment pattern).
  3. Articulate the engagement's contribution to the corpus's *standing apparatus* in a corpus-tier doc (per [Doc 714](https://jaredfoy.com/resolve/doc/714-the-rusty-bun-engagement-read-through-the-lattice-extension-basin-expansion-at-the-l2m-saturation-point)'s shape for the rusty-bun case).
  4. The seed-and-trajectory pair preserves the engagement's constraint structure across the boundary per [Doc 713](https://jaredfoy.com/resolve/doc/713-the-operating-seed-schema-an-efficient-compaction-strategy-from-the-joint-mi-lattice-reading)'s operating-seed schema; the next session resumes at the lattice-extended engagement tier rather than re-deriving against the saturating session tier.

  Per [Doc 714 §VI.3](https://jaredfoy.com/resolve/doc/714-the-rusty-bun-engagement-read-through-the-lattice-extension-basin-expansion-at-the-l2m-saturation-point) Fal-714.3: track future long-session engagements for the predicted artifact-tier consolidation pattern. This rule is the candidate apparatus discipline the rusty-bun thirtieth-round saturation produced; it is offered for the corpus's standing apparatus, not just this engagement.

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

**Telos: full plug-and-play parity.** An arbitrary real-world Bun consumer (npm packages, frameworks, applications) can swap `bun` for `rusty-bun-host` in their command line and run unchanged, against the cited consumer corpus + against Web Platform Tests for spec'd surfaces. Per C1 (plug-and-play interoperability with no regressions), at the **runtime level**, not just the surface-API level.

The engagement run through 2026-05-11 (commit `fbe6e41`) achieved a strictly weaker form: **curated-corpus parity**. For 52 J.1.a fixtures of growing diversity (production-shape vendored libraries + real HTTP server + real WebCrypto + full algorithmic surface), rusty-bun-host's output is byte-identical to Bun 1.3.11. The curated corpus demonstrates the apparatus works; it does not demonstrate that any arbitrary Bun app drops in.

The gap from curated-corpus parity to full plug-and-play parity is **substantive**, not polish. The remaining hard blockers — TLS, real-network fetch, compression, DNS, WebSocket, async-runtime-model auto-listen, node:* breadth — collectively require roughly 12-19 substantial rounds (another engagement's worth of work, comparable in scope to what's been done so far).

Sub-criteria below carry both their **current status** (against curated parity, in parens) and their **status against full parity** (the trailing assessment). The trajectory's queue is updated to chart the trajectory toward full parity.

This is a much larger commitment than the prior framing. The engagement's prior milestone — apparatus saturation at 16 pilots / 8 architectural classes (Doc 708) — is a **necessary** precondition for completion but not sufficient. Saturation establishes that the apparatus' methodology works; completion requires applying the methodology across Bun's full runtime API surface, integrating with a JS engine, and demonstrating differential equivalence against actual Bun-using applications.

Three cybernetic compensation rules govern progress toward completion and prevent drift:
  - **§IV.M7** closes the level-2 loop for primitive-discovery: every round must self-check for new patterns and fold them back before the next round begins.
  - **§IV.M8** closes the level-2 loop for divergence-reconciliation: every divergence between rusty-bun and the comparator runtime must be reconciled in the round it is discovered, not deferred.
  - **§IV.M9** prevents the divergence in the first place: Tier-J fixtures are authored spec-first against Bun, not against rusty-bun's current surface, so divergences surface during authoring and reconciliations land in the same commit.
Without M7 the substrate accumulates work without consolidating its primitives; without M8 fixtures inherit misalignment from prior fixtures and the differential count never converges; without M9 every fixture round transits J.1.b before reaching J.1.a, doubling the work and creating windows where drift can compound. M7 and M8 were instituted under keeper rung-2 intervention; M9 was operationalized after a fixture (consumer-request-signer) shipped J.1.a from inception under spec-first authoring, demonstrating the workflow as a single coherent unit. All three are self-triggering.

The completion telos has five sub-criteria, in dependency order:

**Sub-criterion 1 — Apparatus saturation.** ✓ MET (Doc 708, 2026-05-10).
Sixteen pilots × eight architectural classes × five cybernetic modes × ~3% aggregate LOC ratio. The methodology is empirically anchored and ready for application.

**Sub-criterion 2 — Surface-API completeness.** Every Bun runtime API has a pilot anchor with verifier + consumer-regression closure.
- **Current state (curated):** WebCrypto fully closed (SHA-1/256/384/512 digest + HMAC + PBKDF2 + HKDF + AES-GCM/CBC/CTR/KW + RSA-OAEP/PSS/PKCS1-v1_5 + ECDSA + ECDH over P-256/P-384/P-521); node:fs/path/os/crypto/http(data)/process/buffer/url wired; Bun.serve (in-process + listen-extension)/file/spawn wired; URLSearchParams/TextEncoder/TextDecoder/Blob/Request/Response/Headers/AbortController wired; ReadableStream/WritableStream with tee; Set ES2025 polyfill; Atomics/SharedArrayBuffer.
- **Full-parity remaining (hard blockers):**
  - **TLS / HTTPS** — no pilot. Blocks every HTTPS fetch + every HTTPS server. ASN.1/DER → X.509 cert validation → TLS 1.2/1.3 record + handshake. Estimated 4-5 rounds substrate-amortization.
  - **Real fetch()** — current fetch-api pilot wires Request/Response/Headers classes in-process; `fetch(externalUrl)` does not currently traverse the sockets + http-codec stack to the network. 1-2 rounds to wire properly atop the just-landed Tier-G substrate.
  - **Compression** — no gzip/deflate/brotli. ~80% of HTTP responses are gzipped; consumers see raw compressed bytes. 1-2 rounds (zlib bindings + Content-Encoding negotiation).
  - **DNS** — no Bun.dns or node:dns. fetch on hostnames requires OS resolver integration. 1 round (std::net or trust-dns wrapper).
  - **WebSocket** — Bun.serve's `websocket: {message, open, close}` upgrade path not implemented. 1-2 rounds (HTTP Upgrade handshake + RFC 6455 frame codec + Bun.serve integration).
  - **node:* breadth** — currently wired: fs/path/os/crypto/http(data)/process/buffer/url. Missing: events / stream (full) / util / tls / net / dns / querystring / zlib / child_process / readline / repl / tty / assert / module / vm / perf_hooks / async_hooks. Estimated 3-5 rounds for the load-bearing subset.
  - **Bun-namespace breadth** — beyond Bun.serve/file/spawn currently wired: Bun.password / Bun.SQLite / bun:redis / bun:s3 / Bun.Cookie / Bun.JSONL / Bun.Image / Bun.Archive / Bun.Terminal / Bun.cron / Bun.Glob / Bun.YAML / Bun.CryptoHasher / Bun.deepEquals / Bun.inspect / Bun.write / Bun.connect / Bun.listen-async-shape / Bun.dns / Bun.fileURLToPath / Bun.pathToFileURL. Most are opt-in for specific consumer classes; 2-3 rounds for the commonly-used subset.
- **Out-of-scope deferrals (for full-parity v1):** ECDSA/ECDH keypair *generation* (import is supported); WebAssembly (would need a separate WASM engine — QuickJS doesn't include one); HTTP/2 (HTTP/1.1 fallback works for almost all servers); Workers / BroadcastChannel; Bun.bundle / Bun.build.

**Sub-criterion 3 — Transport-layer pilots.** The data-layer-only pilots lift to wire-format pilots.
- **Current state (~98%):** http-codec pilot (RFC 7230 HTTP/1.1 wire format); sockets pilot (TCP primitives + async-listener via thread-per-listener + mpsc + main-thread poll, std-only, matches Bun's architectural pattern per deepwiki research); Bun.serve facade with listen()/tick()/serve() composing both. End-to-end real HTTP-over-TCP demonstrated byte-identical to Bun (consumer-http-over-tcp-suite, consumer-async-http-server-suite, consumer-bun-serve-facade-suite).
- **Full-parity remaining:**
  - **TLS substrate** (overlaps with sub-2) — the substantial missing piece.
  - **HTTP/2 multiplexing** — separate frame-layer binary protocol; lower-priority since HTTP/1.1 fallback works.
  - **WebSocket frame codec** — overlaps with sub-2 websocket entry.
  - **IPC channels / Bun.spawn full stdio streaming** — partial; some consumers depend on stdin streaming, signal-driven coordination.

**Sub-criterion 4 — JS host integration.** Embed a JS engine (QuickJS or Boa) and expose all pilots to JS code via FFI.
- **Current state (~85%):** rquickjs embedded; 20+ pilot families wired; CommonJS + ESM module loaders both honor relative + bare-specifier resolution with node_modules walk-up + `node:*` builtin scheme; timers + queueMicrotask + performance + URL globals wired; Buffer wrapped as Bun-portable Uint8Array subclass.
- **Full-parity remaining:**
  - **Async-runtime model divergence (BLOCKING).** Real Bun: `Bun.serve({port:3000, fetch})` returns a server that listens in the background; the consumer's main script returns naturally; the process keeps running because the runtime tracks pending async work. Ours: `Bun.serve()` returns; without explicit `await server.serve()`, the process exits before serving. This is a structural API divergence that breaks the canonical Bun program shape (one of the most common). Two paths to close: (a) integrate a "keep alive while pending async" loop into the host (substantial, requires the host to know about pending listeners + timers + outstanding promises), or (b) Bun.serve auto-spawns a setInterval-driven serve loop on construction (lighter, std-only). 1-2 rounds.
  - **WebAssembly** — QuickJS doesn't natively support WASM; needs a separate WASM engine (wasmtime / wasmer integration) or a different JS engine entirely. Out of scope for full-parity v1.
  - **import.meta.* completeness** — most properties wired; some Bun-specific ones (import.meta.dir, .main, .resolveSync, .require) need verification.
  - **process.* completeness** — argv, env, exit, cwd, hrtime wired; stdin streaming, signal handlers (SIGINT etc.), process.on, child_process integration not yet.

**Sub-criterion 5 — Differential testing against Bun-using applications.** The operational form of plug-and-play. For a representative basket of Bun-using applications (frameworks like Hono / Elysia, real-world apps): run `npm test` under Bun → record P_bun. Run under integrated rusty-bun → record P_drv. Diff. **Zero regressions across the basket** = real plug-and-play.

Closure of this sub-criterion is **per-fixture differential**: a fixture counts toward sub-criterion 5 only when it produces byte-identical output under Bun and rusty-bun-host (the J.1.a category in the trajectory's Tier-J basket). Fixtures that exercise the apparatus but depend on rusty-bun-only shapes (J.1.b) are host-internal regression tests, not sub-criterion-5 evidence — they must either be reconciled per §IV.M8 to enter J.1.a, or remain explicitly out of the differential count with a re-open condition.

This is what makes the criterion non-deferrable. Each fixture not yet in J.1.a is a permanent ratchet against the eventual count; new fixtures cannot be built atop unreconciled divergences without inheriting the misalignment (the plank metaphor in §IV.M8). M8 enforces the rule at the round level; sub-criterion 5 enforces it at the telos level.

- **Current state (~40% against full parity):** 52 J.1.a fixtures byte-identical to Bun 1.3.11 across the major non-network production-library shape catalog (8 orthogonal vendored-library axes + WebCrypto closure + Tier-G HTTP/TCP/async-bridge). All differential-verified; sub-criterion 5 J.1.a count is the live progress metric.
- **Full-parity remaining:** the basket needs to extend to real third-party OSS packages (not engagement-authored shims): hono, elysia, drizzle-orm, prisma-client, jose (the real library), pino, etc. Each adds another set of axes the apparatus hasn't yet been exercised against. Once the hard-blocker sub-2/sub-3/sub-4 items above land, the basket-expansion is largely mechanical at ~1 fixture/round. Estimated 30-50 additional fixtures to reach "any of the top-200 Bun-compatible npm packages drops in unchanged."

A complementary signal: run Web Platform Tests against the integrated runtime via `wpt run` adapter. WPT pass-rate per surface is a published number for browser engines and Bun itself; rusty-bun's WPT pass-rate becomes an operational comparison.

**The trajectory holds the per-surface pilot list, the transport-layer queue, the JS-host integration plan, and the differential-test basket.** The seed names the criterion; the trajectory holds the work.

### VII.A. Trajectory toward full parity (concrete roadmap)

Estimated ~12-19 substantial M10-staged rounds to close the hard-blocker list above. Approximate ordering reflects dependency + leverage:

**Phase Π1 — Network completion (5-7 rounds, highest leverage).**
1. **Real fetch() wiring** — ✅ DONE 2026-05-11 (single round). globalThis.fetch composes http-codec + sockets + URL parsing into a `fetch(url, init) → Promise<Response>` that traverses the Tier-G stack. http:// only (https: → explicit ENOTLS pointing to Π1.4). IPv4 literals + "localhost" only (other hostnames → explicit ENODNS pointing to Π1.2). consumer-real-fetch-suite Tier-J fixture 8/8 byte-identical to Bun.
2. **DNS resolution** — std::net::ToSocketAddrs wrapper exposed via `Bun.dns` + `node:dns` minimum surface (resolve / lookup). Unlocks hostname-based fetch + connect. 1 round.
3. **Compression** — gzip/deflate/brotli encode + decode pilot; wire Content-Encoding negotiation in fetch + response serialization. Most HTTP responses are gzipped; this is bottleneck-class. 1-2 rounds. (flate2 crate or hand-rolled DEFLATE — keeper decides std-only vs new-dep policy.)
4. **TLS substrate** — the largest single piece. ASN.1/DER parser → X.509 cert validation against system root store → TLS 1.2/1.3 record layer + handshake. Heaviest substrate-introduction round of the engagement. Unlocks every HTTPS interaction. 4-5 rounds (substrate-amortization: ASN.1 → certs → TLS records → handshake → integration with sockets pilot).
5. **WebSocket** — HTTP Upgrade handshake + RFC 6455 frame codec + Bun.serve `websocket:` integration. Composes on http-codec + sockets + crypto.subtle (for Sec-WebSocket-Accept SHA-1). 1-2 rounds.

**Phase Π2 — Runtime-model completion (2-3 rounds).**
6. **Async-runtime auto-keep-alive** — host learns to track pending async work (active listeners + timers + outstanding promises) and only exits when all are settled. OR Bun.serve auto-spawns a setInterval-driven serve loop on construction. Closes the program-structure divergence. 1-2 rounds.
7. **process.* completeness** — stdin streaming, signal handlers (SIGINT/SIGTERM via signal-hook crate or std::signal_hook), process.on('exit'/'beforeExit'). 1 round.

**Phase Π3 — node:* breadth (3-5 rounds).**
8. **node:events** — EventEmitter class. Universal dependency for npm packages. 1 round.
9. **node:stream full** — Readable / Writable / Duplex / Transform with backpressure. Many packages depend on this. 1-2 rounds.
10. **node:util** — promisify, callbackify, format, inspect, types. 1 round.
11. **node:querystring + node:url full** — partial; need full surface. 1 round (often bundled).
12. **node:tls / node:net** — node-style wrappers over the Tier-G + TLS substrate from Π1.4. 1 round once Π1.4 lands.
13. **Optional based on consumer-corpus needs:** node:zlib (folded into Π1.3), node:child_process (atop Bun.spawn), node:dns (folded into Π1.2), node:readline / repl / tty / vm / perf_hooks / async_hooks / assert.

**Phase Π4 — Bun-namespace breadth (2-3 rounds).**
14. **Bun.password** — Argon2id wrapper. Pure Rust; one focused pilot.
15. **Bun.SQLite** — wraps sqlite via rusqlite or hand-rolled SQLite ABI. Substantial; defer unless consumer-corpus requires.
16. **Bun-namespace small utilities** — Bun.write, Bun.connect, Bun.listen-async-shape, Bun.dns, Bun.fileURLToPath, Bun.pathToFileURL, Bun.deepEquals, Bun.inspect, Bun.Glob, Bun.YAML, Bun.CryptoHasher. Many are thin wrappers; 1-2 rounds for the load-bearing subset.

**Phase Π5 — Real-OSS differential basket (3-5 rounds, opportunistic).**
17. **First real package: hono** — micro web framework, native Bun support. Vendor it unchanged, run its tests under rusty-bun-host vs Bun. Any divergence is sub-2 work.
18. **Production JWT library: jose** — the canonical JOSE library. Heavy crypto.subtle user. Already-closed surface; should drop in cleanly.
19. **A real database driver / SQLite app** — depends on Π4 Bun.SQLite.
20. **WPT runner adapter** — `wpt run` against the integrated runtime; track pass-rate per surface.

**Estimated cumulative cost.** ~12-19 substantial rounds for Phase Π1-Π4 hard-blockers; ~3-5 additional for Π5 real-OSS demonstration. Total roughly an engagement's worth of work comparable in scope to what's been done so far (the 2026-05-10/05-11 run produced ~12 substantial rounds across crypto/Tier-G).

**Self-update discipline.** After each round in Phase Π1-Π5, update this section's percentages + check off the completed item + adjust estimates for downstream items based on what was learned. The trajectory section of trajectory.md mirrors this list and tracks actual round-by-round progress.

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
