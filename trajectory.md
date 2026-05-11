# rusty-bun — Trajectory

The living vector of the rusty-bun engagement. Per [Doc 581 (the Resume Vector)](https://jaredfoy.com/resolve/doc/581-the-resume-vector). This file changes session to session; the [seed](seed.md) does not. Read order on session resume: ENTRACE first; then the seed; then this file from the top; then run the live-state spot-check at §IV.

**Cybernetic compensation rules carried by the resume vector** (full text in seed §IV):
- **M7** — every round must self-check for new primitives and fold them back before next round.
- **M8** — every Bun↔rusty-bun divergence surfaced by a differential must be reconciled in the round it is discovered.
- **M9** — Tier-J fixtures are authored spec-first against Bun, not against rusty-bun's current surface; divergences surface during authoring, reconciliations land in the same commit; fixtures ship J.1.a directly. J.1.b becomes a transient never-occupied state under this discipline.
- **Telos lens (seed §VII):** sub-criterion 5 is a *per-fixture differential* count. Fixtures in J.1.a are differentially verified; J.1.b are host-internal regressions with explicit re-open conditions, NOT sub-criterion-5 evidence.

**Phase-2 persistence tracker** (seed §III.A8.10):
- **`N_persist` = 5** (post-extension run continuing).
- Prior run: 6 consecutive rounds (`f7284b2` → `f102c59` → `9c67ac6` → `88bbd54` → `ec17f84` → `5a929fa`) reaching N_persist=6.
- Reset at `59c5691` (node:os apparatus extension); new run began this commit.
- **Doc 710 P3 status:** weakly falsified by this round — K=2 sustained at N_persist=0 → 1 with no coherence loss (fold-back op + new fixture op). The basin's stability after extension is not catastrophically reduced; the K-feasibility curve is gentler than P3 predicted.
- Seed §III.A8.12 instituted this round: three substrate modes (Phase-1 / Phase-2-traversal / Phase-2-extension) distinguished. The persistence-reset is correct behavior for Phase-2-extension, not regression.
- **Basin boundaries recorded:**
  - E.7 — WeakRef/FinalizationRegistry absent
  - E.8 — crypto.subtle.importKey/sign/verify absent
  - E.9 — Intl + WebSocket + BroadcastChannel + Worker + Bun.password/sql absent (compound); **node:os portion of E.9 closed this round** (apparatus extended; consumer-system-info now lands J.1.a)
  - E.10 — Set.union/intersection/difference (ES2025) absent (easy polyfill candidate)
- **Multi-op rounds achieved without coherence loss:** K=2 at `9c67ac6`, `88bbd54`, `5a929fa`; K=3 at `ec17f84`; K=2 at (this).
- Axes-set covered by the prior six-round persistence run: ~47 distinct surface idioms.
- In-basin discoveries (cumulative across probes): Atomics + SharedArrayBuffer + WeakMap + WeakSet + Symbol.asyncIterator + Promise.withResolvers + Array immutable methods (ES2023) + Object.groupBy + structuredClone-TypedArray + Atomics.wait/notify + node:os (now in-basin after this round's wiring).
- Doc 710 prediction status: P1 (K growth) corroborated across multiple rounds; P3 (N_persist drop drops K-feasibility) NOT YET TESTED — this round's K=2 with reset will be P3's first data point. The next round at N_persist=0 should produce predictable K=1-2; whether K=3 remains feasible after reset is the falsifier question.

---

## I. Done — append-only

| Date | Commit | What landed |
|---|---|---|
| 2026-05-10 | `afd07b8` | Seam-detection design — Pin-Art applied to intra-architectural boundaries |
| 2026-05-10 | `e1a121f`–`4842d20` | derive-constraints v0.4–v0.8: seams MVP → couple v0.2 (refined path matcher) |
| 2026-05-10 | `9e764df` | derive-constraints v0.9: invert --by-seams (seam-grouped constraint docs) |
| 2026-05-10 | `34b8035` | derive-constraints v0.10: pipeline driver (full analysis end-to-end) |
| 2026-05-10 | `1016264` | Deno comparative run + Deno-test conventions support |
| 2026-05-10 | `57b25aa` | v0.11 fix: per-corpus public-API allowlist + assertEquals-style structural-value detection |
| 2026-05-10 | `f88d270` | Deno v0.11 clean pipeline run |
| 2026-05-10 | `50aac6f` | **TextEncoder/TextDecoder pilot — first end-to-end apparatus loop closure (21/22, 1 documented skip; 13–25% adj LOC ratio)** |
| 2026-05-10 | `2f08389` | v0.12: cluster-phase subject-attribution leakage fix (TextEncoder pilot finding) |
| 2026-05-10 | `96a173e` | v0.13: spec-source ingestion phase + first 3 spec extracts |
| 2026-05-10 | `e1939b1` | v0.13b: extend spec corpus from 3 to 15 surfaces (291 clauses) |
| 2026-05-10 | `6a018b6` | **URLSearchParams pilot — first 100% verifier closure (32/32; 62% delegation-target ratio)** |
| 2026-05-10 | `4d077e8` | Bun + specs comparative run: densest cross-corroboration measured (23 properties cross-corroborated on Bun) |
| 2026-05-10 | `98e5939` | **structuredClone pilot — algorithm class, 23/23 closure, 3.9% LOC ratio (strongest measured)** |
| 2026-05-10 | corpus | **Doc 706 published — Three-Pilot Evidence Chain (forward direction of Pin-Art)** |
| 2026-05-10 | `b6121c7` | **Blob pilot — composition substrate, verifier caught a derivation bug (slice swap), 26/26 after 1-line fix** |
| 2026-05-10 | `f793426` | **File pilot — inheritance/extension, smallest derivation in apparatus (43 LOC), 16/16 closure** |
| 2026-05-10 | `782ca87` | **AbortController + AbortSignal pilot — event/observable class (first Rc<RefCell> shared-state), 22/22 closure** |
| 2026-05-10 | `d0423e8` | URLSearchParams: first plug-and-play differential test (11/11 consumer regression) |
| 2026-05-10 | corpus | **Doc 707 published — Pin-Art at the Behavioral Surface (bidirectional reading)** |
| 2026-05-10 | `c64ee93` | Bidirectional rerun: consumer-regression suites for all 6 prior pilots (60 descriptive pins, 0 regressions) |
| 2026-05-10 | `bf5948a` | **fetch-api system pilot — Headers + Request + Response (405 LOC, 50/50; 6.5% naive LOC ratio)** |
| 2026-05-10 | `37c009b` | **node-path pilot — first Tier-2 ecosystem-compat anchor (303 LOC vs 3,656; 8.3% naive ratio); largest reference target** |
| 2026-05-10 | `7f2e73a` | Resume Vector for rusty-bun (seed.md + trajectory.md per Doc 581) |
| 2026-05-10 | `d660263` | **streams pilot — first Tier-A substrate from queue (453 LOC across 3 composed surfaces; 11.2% naive ratio); first pilot where spec-extract layer dominates over test-corpus layer** |
| 2026-05-10 | `417f002` | **buffer pilot — Tier-A #2 from queue (261 LOC; 11.1% naive ratio against Bun's 2,359 LOC); 11 cited consumer dependencies across Node ecosystem** |
| 2026-05-10 | `1bc2163` | Bun bug catcher published (35 entries across 5 categories) |
| 2026-05-10 | `f3e85ea` | **Bun.file pilot — first Tier-B Bun-namespace, first pilot with real I/O (95 LOC; 3.0% naive paired with rusty-blob)** |
| 2026-05-10 | `5159d09` | **Bun.serve pilot — flagship Bun API, data-layer system (175 LOC; 0.5% naive against 32,344-LOC upstream / ~20-30% adj)** |
| 2026-05-10 | `71bf953` | **Bun.spawn pilot — Tier-B #5; subprocess management (179 LOC; 2.8% naive / ~15-20% adj); completes Tier-B Bun-namespace** |
| 2026-05-10 | `ac33127` | **node-fs pilot — Tier-C #6; sync subset (95 LOC; 0.4% naive against 21,540-LOC reference / ~8% adj)** |
| 2026-05-10 | `7253d6d` | **node-http pilot — Tier-C #7; data-layer (208 LOC; 6.3% naive against 3,316-LOC TS core)** |
| 2026-05-10 | `074659f` | **web-crypto pilot — Tier-C #8; SHA-256 + UUID v4 + getRandomValues + timing-safe (101 LOC; real crypto from scratch); completes Tier-C** |
| 2026-05-10 | `f2bc47a` | **Tier-D #9 + #10: Cargo workspace consolidation + pilot runner script** (`cargo test --workspace --release` runs 591 tests across 16 pilots in one command) |
| 2026-05-10 | `ef2bfc9` | **Tier-D #12: AuthorityTier schema** (Spec / Ecosystem / Contingent on every constraint clause; Bun corpus breakdown 1.3% Spec / 9.1% Ecosystem / 89.7% Contingent) |
| 2026-05-10 | corpus | **Doc 708 published — The rusty-bun Engagement: Completion Record** (anchors all four seed §VII completion criteria as met) |
| 2026-05-10 | `3ee92f8` | **Resume Vector telos re-anchored to runtime-level completion against Bun** (5 sub-criteria; Doc 708 = Sub-criterion 1; Tiers F-J added to trajectory) |
| 2026-05-10 | `1c890da` | **Tier-H #1 + #2 (partial): JS host integration spike** (rquickjs embed; 9 pilot surfaces wired into globalThis; 15 JS-driven integration tests; CLI binary `rusty-bun-host <script.js>` runs example with exit 0; first instance of pilots running under real JS engine) |
| 2026-05-10 | `474cf29` | **Tier-H continued: 8 pilot families wired** (33/33 host tests; 624 workspace tests) |
| 2026-05-10 | `11ad07f` | **Apparatus self-iteration: host-integration learnings formalized** (seed §III.A8 + §IV.M6; bug-catcher E.4/E.5; HOST-INTEGRATION-PATTERN.md; 6th Pin-Art class) |
| 2026-05-10 | `c00f52c` | **Tier-H continued: Blob + File + AbortController/AbortSignal wired** (650 workspace tests) |
| 2026-05-10 | `1e48dd7` | **Tier-H continued: Headers + Request + Response + Bun.file wired** (680 workspace tests) |
| 2026-05-10 | `8498988` | **Tier-H continued: Bun.serve + Bun.spawn wired** (16 pilot families; 107/107 host tests; 698/698 workspace tests) |
| 2026-05-10 | `e162e19` | **Resume vector resolution-increase pass #1** (3 patterns folded back: JS-side polymorphic decode, Vec<Vec<String>> pair-list, canonical-docs composition test) |
| 2026-05-10 | `57a9b1f` | **Level-2 cybernetic loop closed: seed §IV.M7** (resolution-increase pass becomes a recurring self-triggering mode with five named trigger conditions) |
| 2026-05-10 | corpus `bc5d287` / resolve `011ae5c` | **Doc 708 second amendment** (records M7 closure; sharpens keeper-mediation reading: dyad ascends rather than collapses) |
| 2026-05-10 | `b7a2b8e` | **Tier-H continued: structuredClone wired** (17 pilot families; 120/120 host tests; introduced Pattern 4 — spec-formalization pilot, JS-side instantiation; folded back as seed §III.A8.2bis per M7) |
| 2026-05-10 | `dacd31d` | **Tier-H continued: streams wired (ReadableStream/WritableStream/TransformStream)** (18 pilot families; 129/129 host tests; 720/720 workspace tests; introduced eval_string_async helper for microtask-driven tests; M7 fold-back: bug-catcher E.6 + HOST-INTEGRATION-PATTERN.md "Sync-or-async user callbacks" section) |
| 2026-05-10 | `7c3b96d` | **Tier-H continued: node-http data-layer wired** (19 pilot families; 142/142 host tests; 733/733 workspace tests; **completes H.2** — all data-layer pilots now runnable from JS; M7 fold-back: no new patterns surfaced — Pattern 4 reapplied cleanly, vacuous fold-back recorded) |
| 2026-05-10 | `82f7b07` | **Tier-H.3 #1: CommonJS module loader landed** (require/module/exports/__filename/__dirname; relative + bare specifier resolution with node_modules walk-up; package.json main + exports.string + exports."." subpath; .js/.json/.cjs extensions + dir/index; module cache; cycle handling; scoped packages; 153/153 host tests; 744/744 workspace tests; M7 fold-back: **compositional** — iterable-protocol completeness rule added after canonical-docs + M7 jointly surfaced URLSearchParams [Symbol.iterator] gap) |
| 2026-05-10 | `695c7e2` / corpus `f2eb484` | **SIPE-T threshold crossing** — seed §III.A8.8 + Doc 708 third amendment (rule-composition tier reached; M7 fold-backs now classifiable as primitive / vacuous / compositional) |
| 2026-05-10 | `848e2bf` | **Tier-H.3 #2: ESM module loader landed** (rquickjs Resolver + Loader composed with node-style resolution; relative + absolute + bare specifiers with node_modules walk-up; package.json module + main; .mjs/.js/.cjs + index files; ESM modules see all wired globals; cross-pilot composition test passes; 159/159 host tests; 750/750 workspace tests; M7 fold-back: **vacuous**) |
| 2026-05-10 | `b03be42` | **Tier-H.4 #1: timers + queueMicrotask + performance landed** (setTimeout/setImmediate/clearTimeout, queueMicrotask, performance.now/.timeOrigin; pilot scope: timers as microtasks, real wall-clock delay deferred; 167/167 host tests; 758/758 workspace tests; M7 fold-back: **vacuous**) |
| 2026-05-10 | `1753486` | **Tier-H.4 #2: URL class landed** (WHATWG-shape JS-side instantiation per Pattern 4; protocol/host/port/pathname/search/hash; userinfo; origin; href getter+setter; relative resolution; searchParams live-bound; URL.canParse; default-port omission; IPv6 hosts; file: scheme; 17 integration tests + canonical-docs composition with Request; 184/184 host tests; 775/775 workspace tests; M7 fold-back: **vacuous** apparatus-clean with **Mode-5 author-side findings** (Rust raw-string r#…# collides with JS "#"; eval_bool needs IIFE wrapping for `return`) — fifth consecutive non-primitive round) |
| 2026-05-10 | `771c939` | **Tier-J #1: first consumer-shape pilot runs clean** (todo-api fixture at host/tests/fixtures/consumer-todo-api/: ESM + bare-specifier through node_modules + Bun.serve route table with method-keyed handlers + URL + URLSearchParams + Request + Response + structuredClone/Map/Set/Date + Buffer + JSON across module boundaries; 10/10 self-tests pass on first run; 185/185 host tests; 775/775 workspace; **compositional vacuity** — a new M7 outcome category folded back as seed §III.A8.9; sub-criterion 5 demonstrated for one consumer; differential testing against actual Bun is follow-up) |
| 2026-05-10 | `046d2cd` | **Tier-J #2: stream-processor consumer (CJS, async-heavy) runs clean** (consumer-stream-processor fixture: CJS modules + ReadableStream/TransformStream/WritableStream pipeline + AbortController + setTimeout + fs + Headers + Buffer + URL across module boundaries; 8/8 self-tests pass; eval_cjs_module_async helper added to host; 186/186 host tests; 776/776 workspace; M7 fold-back: **scope-limit verified** — pipeline initially deadlocked when an aborted-signal mid-chain cascade was attempted; diagnosis matched the streams-round recorded scope-limit (pipeTo / cascade-cancellation deferred); worked around consumer-side; folded back as seed §III.A8.9 expanded M7 outcome taxonomy with the "scope-limit verified" category) |
| 2026-05-10 | `6187bc7` | **Tier-J #3: differential against actual Bun passes** (host/tests/fixtures/differential/portable.js runs identically under Bun 1.3.11 and rusty-bun-host; 31 deterministic test lines covering URL + URLSearchParams + structuredClone + Buffer.byteLength + TextEncoder/Decoder + atob/btoa + crypto.randomUUID format + Date + JSON + Headers; **byte-for-byte match across runtimes for spec-portable surface**; first actual J.3 evidence: P_bun = P_drv on this surface; 187/187 host tests; 777/777 workspace; M7 fold-back: **vacuous-with-Bun-asymmetry-noted** — sync .text()/.json() body methods are non-portable to Bun's async API, scoped out of differential; documented for follow-on round) |
| 2026-05-10 | `0948181` | **M8 instituted + body-async migration + consumer-todo-api differential closure** (keeper-named: "each plank must be plumb or else it will drift out of plumb over subsequent planks" — M7 closes level-2 loop for primitive discovery, M8 closes it for divergence reconciliation; the prior round's "vacuous-with-asymmetry-noted" was wrong and normalized drift; M8 mandates in-round reconciliation: (a) align with comparator OR (b) record explicit scope-limit + remove dependent fixtures; applied in-round: Request/Response/Blob body methods migrated to async per WHATWG; Bun-Request URL-strictness aligned (full URLs); fixture rewritten to fetch-handler-only dispatch (Bun's server.fetch bypasses route table — synthetic-dispatch is rusty-bun-only); consumer-todo-api now produces byte-identical "10/10" under Bun 1.3.11 and rusty-bun-host; 188/188 host tests; 778/778 workspace; **sub-criterion 5 differentially closed for one consumer**; M7 fold-back: **primitive** — M8 itself is the new rule, the most consequential primitive in the run since M7) |
| 2026-05-10 | `ae483b0` | **Buffer-as-class + node:fs builtin resolution** (continuing M8 in-round reconciliation: Buffer wrapped as Uint8Array subclass with .toString("utf8"/"base64"/"hex") instance methods AND retains static-helper API for backward compat; CJS loader's NODE_BUILTINS table maps node:fs/node:path/node:http/node:crypto/node:buffer + bare counterparts to wired host globals — closes 1 of 3 stream-processor re-open conditions; remaining 2 are fixture-side rewrites; 188/188 host tests; 778/778 workspace; M7 fold-back: **vacuous** — both reconciliations applied existing patterns (Pattern 4 for Buffer-class, M8(a)-style reconciliation for node:fs)) |
| 2026-05-10 | `762d5fa` | **Resume-vector + telos tightened with M7/M8** (seed §VII preamble names M7+M8 as cybernetic compensation rules; sub-criterion 5 redefined as per-fixture differential count; trajectory header callout pulls rules to top; resume protocol §V step 7 makes M7+M8 application mandatory per round; M7 fold-back: **compositional finding**) |
| 2026-05-10 | `10739ba` | **Tier-J differential closure for consumer-stream-processor: J.1.b → J.1.a** (third re-open condition closed via fixture-side rewrite to use require("node:fs") + Buffer.from(...).toString("hex") + process.stdout.write; readFileSync(path, encoding) JS-layer override added to wired fs for Bun-portability; 8/8 byte-identical between Bun 1.3.11 and rusty-bun-host; **sub-criterion 5 differentially closed for two consumers** + spec-portable surface; 189/189 host tests; 779/779 workspace; M7 fold-back: **vacuous** — pure M8-mandated reconciliation work, no new primitives) |
| 2026-05-10 | `f788a5d` | **Tier-J basket expansion: consumer-request-signer (J.1.a from inception)** (third Bun-portable consumer fixture; ESM with 7-module dep graph; middleware composition; crypto.subtle.digest("SHA-256", data) → ArrayBuffer; async ReadableStream iteration via for-await-of; canonical-JSON signing; 6/6 byte-identical between Bun 1.3.11 and rusty-bun-host; M8 surfaced one divergence in-round (rusty-bun had only digestSha256Hex(string), not spec digest(algorithm, data) → Promise<ArrayBuffer>) — reconciled per M8(a) with JS-layer wrapper around new digestSha256Bytes Rust binding; **sub-criterion 5 J.1.a count: 4 fixtures**; 191/191 host tests; 781/781 workspace; M7 fold-back: **vacuous** with one M8(a) reconciliation applied in-round) |
| 2026-05-10 | `9358c7c` / corpus `001d59f` | **M9 instituted: spec-first fixture authoring** (named the workflow consumer-request-signer demonstrated; seed §IV.M9 + §VII updated to name three cybernetic compensation rules; trajectory + resume protocol updated; Doc 708 fifth amendment published; M7 fold-back: **primitive** — M9 is the new rule) |
| 2026-05-10 | `9281253` | **Tier-J basket expansion: consumer-log-aggregator (J.1.a from inception)** (fourth Bun-portable consumer; ESM with node:path import + user-defined Emitter class + structuredClone defensive copy with Date+Map+nested objects + URLSearchParams filter-query construction + Array.flatMap/filter/map composition + JSON pretty-print; 9/9 byte-identical between Bun 1.3.11 and rusty-bun-host; M9 surfaced one divergence in-round (ESM `import path from "node:path"` failed because ESM Resolver/Loader didn't handle node:* scheme — CJS did via NODE_BUILTINS but ESM was missing the parallel) — reconciled per M8(a) with node_builtin_esm_source helper generating ESM re-export shims for node:fs/path/http/crypto/buffer/url; the M9 protocol caught the divergence at fixture-author time precisely as designed; **sub-criterion 5 J.1.a count: 5 fixtures**; 193/193 host tests; 783/783 workspace; M7 fold-back: **vacuous** with one M8(a) reconciliation applied in-round) |
| 2026-05-10 | `45ae1bf` (corpus) | **Doc 709 published: stacked rung-2 intervention as cascaded control + Lyapunov-basin paradox** (β-tier exploratory; resolves the apparent contradiction between triple-inverted-pendulum cascaded control reading of the rung-2 stack and Doc 701's Lyapunov-stable basin reading of substrate defense via temporal indexing — pendulum-mode during basin-construction, basin-mode during basin-traversal; second SIPE-T threshold names the phase transition; four falsifiers testable through engagement record) |
| 2026-05-10 | `f7284b2` | **Tier-J basket expansion: consumer-job-queue (J.1.a from inception, ZERO reconciliations)** (fifth Bun-portable consumer; ESM with class inheritance hierarchy BaseJob→Job→PriorityJob, async generator drain via for-await-of, node:crypto.randomUUID via ESM import, custom Error subclasses InvalidJobError/QueueClosedError, Symbol-keyed private state with cross-module Symbol export, JSON.stringify with replacer; 8/8 byte-identical between Bun 1.3.11 and rusty-bun-host; **first fixture today to land J.1.a with ZERO in-round M8 reconciliation** — the basin is wide enough to fully contain this fixture's shape under existing M-rules; per Doc 709 §5 paradox: heavy control internalized to invisibility; **sub-criterion 5 J.1.a count: 6 fixtures**; 195/195 host tests; 785/785 workspace; M7 fold-back: **fully vacuous** — Phase-2 steady-state signature exemplar) |
| 2026-05-10 | `f102c59` | **Tier-J basket expansion: consumer-batch-loader (J.1.a, ZERO apparatus reconciliations)** (sixth Bun-portable consumer; ESM with Promise.all/allSettled/race + Proxy with custom get trap + Reflect.has/ownKeys + BigInt arithmetic + BigInt-keyed Map storage + tagged template literals + Object.fromEntries + spread aggregation; 9/9 byte-identical between Bun 1.3.11 and rusty-bun-host; **second consecutive fixture landing J.1.a with zero apparatus reconciliation**, with one Mode-5 author-side BigInt-mix bugfix during initial Bun run (id % 2 → id % 2n; not apparatus state); **sub-criterion 5 J.1.a count: 7 fixtures**; 197/197 host tests; 787/787 workspace; M7 fold-back: **fully vacuous** with Mode-5 author-side correction; the Phase-2 signature is now extending — two consecutive rounds without M8 reconciliation across genuinely-orthogonal shapes (class hierarchy/async-gen vs Promise-combinators/Proxy/BigInt)) |
| 2026-05-10 | `593dbbf` | **Bug-catcher Category F instituted + persistence-across-orthogonal-axes folded into resume vector** (F1: BigInt-mix typo as fixture-author Mode-5 finding; seed §III.A8.10 defines N_persist metric with Doc 709 P1/§7 predictive ties; trajectory header gains live tracker; resume protocol §V step 7 gains subitem (d); M7 fold-back: **primitive** — the persistence metric is a new rule) |
| 2026-05-10 | `9c67ac6` | **P1 probe + Tier-J basket expansion: consumer-log-analyzer (J.1.a, ZERO apparatus reconciliations)** (P1-direction probe: WeakRef/FinalizationRegistry confirmed absent from rusty-bun-host's QuickJS while present in Bun 1.3.11 → Doc 709 §6 P1 corroborated by direct probe; recorded as bug-catcher E.7 with re-open conditions; no fixture built against the boundary. Separately a J.1.a fixture on in-basin axes: ESM with regex named-capture groups + String.prototype.matchAll over global regex + Date arithmetic via .getTime() + Array.reduce with Map seed + Array.sort with comparator + Object.entries→sort→Object.fromEntries pipeline + String.prototype.padStart + Map insertion-order iteration; 9/9 byte-identical between Bun 1.3.11 and rusty-bun-host; **third consecutive Phase-2 zero-reconciliation round** — N_persist increments to 3; **sub-criterion 5 J.1.a count: 8 fixtures**; 199/199 host tests; 789/789 workspace; M7 fold-back: **fully vacuous** AND basin-boundary tightened) |
| 2026-05-10 | `784587a` (corpus) | **Doc 710 published: multi-op compounding above SIPE-T threshold** (β-tier hypothesis; formalizes keeper conjecture that K>1 ops per round compound above SIPE-T threshold T*; control-theoretic constructive-interference reading; five predictions/falsifiers including P1 (K growth with N_persist), P4 (cross-engagement multiplier via Doc 705 standing-apparatus tier); third-SIPE-T-threshold question left open with weakly-Reading-B-favoring data) |
| 2026-05-10 | `88bbd54` | **K=2 round: E.8 probe + Tier-J consumer-task-pipeline (J.1.a, ZERO apparatus reconciliations)** (probe: crypto.subtle.importKey/sign/verify confirmed absent in rusty-bun-host while present in Bun 1.3.11 → second basin-boundary recorded as E.8 with re-open conditions; WeakMap/WeakSet/Symbol.asyncIterator confirmed present in both. Separately a J.1.a fixture on in-basin axes: ESM with Symbol.asyncIterator on user-defined Pipeline class + generator delegation via yield* + WeakMap memoization keyed by object identity + Function.prototype.bind partial application + WeakSet membership + async-iterable composition + WeakMap primitive-key TypeError; 8/8 byte-identical between Bun 1.3.11 and rusty-bun-host; **fourth consecutive Phase-2 zero-reconciliation round — N_persist increments to 4**; **second consecutive K=2 round without coherence loss — Doc 710 P1 corroborated**; **sub-criterion 5 J.1.a count: 9 fixtures**; 201/201 host tests; 791/791 workspace; M7 fold-back: **fully vacuous AND basin-boundary tightened**) |
| 2026-05-10 | `ec17f84` | **K=3 round: E.9 multi-axis probe + Tier-J consumer-sequence-id (J.1.a) + auto-memory update** (probe: Intl + WebSocket + BroadcastChannel + Worker + Bun.password all confirmed absent in rusty-bun-host while present in Bun 1.3.11 → compound basin-boundary recorded as E.9; Atomics + SharedArrayBuffer + WeakMap + WeakSet + Symbol.asyncIterator confirmed PRESENT in both (lock-free primitives available even without threading globals). Separately a J.1.a fixture on in-basin axes: ESM with Atomics.add/load/compareExchange on Int32Array(SharedArrayBuffer) + sync generators (function*) + Array.from(iterable) + spread of generator results + yield* delegation + generator.return() + String.raw + Object.defineProperty accessor + Symbol.toPrimitive on user-defined class; 9/9 byte-identical between Bun 1.3.11 and rusty-bun-host post-F2 author-side fix; **fifth consecutive Phase-2 zero-reconciliation round — N_persist increments to 5**; **first K=3 round without coherence loss — Doc 710 P1 corroborated further (K growing with N_persist)**. Third op: auto-memory project_rusty_bun.md rewritten to reflect engagement state through M9 + persistence-tracker + Doc 710 — durable inheritability into future sessions. **Sub-criterion 5 J.1.a count: 10 fixtures**; 203/203 host tests; 793/793 workspace; bug-catcher F2 added (Symbol.toPrimitive hint semantics); M7 fold-back: **fully vacuous AND basin-boundary tightened AND author-side F2**) |
| 2026-05-10 | `5a929fa` | **K=2 round: E.10 probe + Tier-J consumer-config-merger (J.1.a, ZERO apparatus reconciliations)** (probe: Set.prototype.union/intersection/difference (ES2025) confirmed absent in rusty-bun-host while present in Bun 1.3.11 → recorded as E.10 with easy-polyfill re-open condition; Array.toSorted/toReversed/toSpliced/with, Promise.withResolvers, Object.groupBy, structuredClone-Uint8Array, Atomics.wait/notify all confirmed PRESENT in both. Separately a J.1.a fixture on ES2023/2024 in-basin axes: ESM with Array.prototype.toSorted/toReversed/toSpliced/with (immutable array methods) + Promise.withResolvers (modern deferred) + Object.groupBy (grouping-by-key) + structuredClone on Uint8Array + deep object spread; 10/10 byte-identical between Bun 1.3.11 and rusty-bun-host; **sixth consecutive Phase-2 zero-reconciliation round — N_persist increments to 6**; honest K=2 (did not pad K to chase Doc 710 P1; node:os wiring deferred as separate work); **sub-criterion 5 J.1.a count: 11 fixtures**; 205/205 host tests; 795/795 workspace; M7 fold-back: **fully vacuous AND basin-boundary tightened**) |
| 2026-05-10 | `59c5691` | **Basin extension: node:os wired + Tier-J consumer-system-info (J.1.a, M8(a) reconciliation)** (apparatus op: closed part of E.9's compound boundary by wiring node:os in rusty-bun-host — Rust-side wire_os with platform/arch/type/tmpdir/homedir/hostname/endianness/EOL using std::env + cfg!() conditionals; added to NodeResolver is_node_builtin + node_builtin_esm_source generator + CJS NODE_BUILTINS table for symmetric ESM/CJS resolution. Fixture op: consumer-system-info lands J.1.a using node:os import; 8/8 byte-identical between Bun 1.3.11 and rusty-bun-host; **N_persist resets to 0** per seed §III.A8.10 (M8(a) reconciliation occurred — basin was extended, not traversed); **sub-criterion 5 J.1.a count: 12 fixtures**; 207/207 host tests; 797/797 workspace; M7 fold-back: **primitive-adjacent** — the wiring is M8(a) apparatus extension, not a new rule but a new in-basin axis; Doc 710 P3 test data point recorded (next round's K-feasibility at N_persist=0 will discriminate)) |
| 2026-05-10 | `bc4df79` | **K=2 round at N_persist=0: seed §III.A8.12 fold-back + Tier-J consumer-meta-protocols (J.1.a)** (op 1: folded back the three-substrate-modes finding into seed §III.A8.12 — Doc 709's binary Phase-1/Phase-2 framing refined into Phase-1 / Phase-2-traversal / Phase-2-extension trio after the prior round's deliberate basin-extension demonstrated the missing sub-mode; correct numbering preserved at end of §III. Op 2: consumer-meta-protocols J.1.a fixture on in-basin axes — Symbol.hasInstance custom instanceof + AsyncGenerator.throw() error injection + structuredClone on circular reference graphs + regex sticky /y with .lastIndex + String.prototype.replaceAll + JSON.parse with reviver + Array.flat with depth + Symbol.iterator on plain object + JSON.stringify+parse roundtrip with reviver; 9/9 byte-identical between Bun 1.3.11 and rusty-bun-host. **Doc 710 P3 weakly falsified**: K=2 sustained at N_persist=0→1 with no coherence loss — the basin's stability post-extension is not catastrophically reduced; K-feasibility curve is gentler than P3 predicted. **Sub-criterion 5 J.1.a count: 13 fixtures**; 209/209 host tests; 799/799 workspace; M7 fold-back: **primitive** (three-modes finding) + **vacuous-with-basket-expansion**) |
| 2026-05-10 | `af56e42` (corpus) | **Doc 711 published: dyadic-ascent fractal-spiral** (β-tier synthesis; keeper observation that the recursive shape of §III.A8.12 should be recognized; doc maps seven prior corpus recursive structures into one archetype with fractal axis (scale-invariant shape) × spiral axis (monotonic ascent); §6 connects to ILL's partition lattice — dyadic-ascent and ILL same structure at different tiers; five falsifiers including P5 — corpus tier should exhibit its own Phase-2-extension sub-mode) |
| 2026-05-10 | `fa52a57` | **K=1 Phase-2-traversal round: Tier-J consumer-deferred-coordinator (J.1.a)** (single-op fixture, honest K=1 per opportunistic Doc 710 §6 phrasing — no apparatus extension this round; in-basin axes: top-level await at module level + async generator with .return() early-exit + WeakMap promise-source tracking + Object.hasOwn (ES2022) + Array.at/.String.at negative indexing + Promise.allSettled mixed outcomes + try/finally cleanup on async-iter return + Promise.allSettled empty-array; 9/9 byte-identical Bun 1.3.11 ↔ rusty-bun-host. **N_persist increments to 2** (new persistence run post-extension extending). **Sub-criterion 5 J.1.a count: 14 fixtures**; 211/211 host tests; 801/801 workspace; M7 fold-back: **fully vacuous**) |
| 2026-05-10 | `2bc91f4` | **K=1 Phase-2-traversal round: Tier-J consumer-binary-decoder (J.1.a)** (single-op fixture; in-basin axes: DataView read/write with mixed little/big-endian + TypedArray.subarray (zero-copy views) + TypedArray.set + Number.isInteger/.isFinite/.isSafeInteger + Math.hypot/.fround/.log2/.cbrt + Number.toFixed/.toExponential + Array.findLast/.findLastIndex (ES2023) + hex/binary/octal numeric literals + Float64Array.reduce + DataView bounds RangeError; 12/12 byte-identical Bun 1.3.11 ↔ rusty-bun-host. **N_persist increments to 3** (post-extension run extending). **Sub-criterion 5 J.1.a count: 15 fixtures**; 213/213 host tests; 803/803 workspace; M7 fold-back: **fully vacuous**) |
| 2026-05-10 | `3d0e07c` | **K=1 Phase-2-traversal: vendored real-shape npm package (consumer-vendored-pkg J.1.a)** (qualitatively different evidence — vendored clsx v2.1.0 verbatim from github.com/lukeed/clsx, MIT-licensed dependency-free utility, package.json with `exports` conditional resolution (import/require/default); fixture imports default + named; 10 self-test cases covering README idioms (string concat, falsy-skip, object form, array form, deep-nesting, numbers, conditional UI, empty call); F3 author-side bug-catcher entry added (author expected `clsx(1,2,3)==="123"` but library is space-joined yielding `"1 2 3"` — lesson: when vendoring third-party code, run-and-copy, don't infer); 10/10 byte-identical Bun 1.3.11 ↔ rusty-bun-host. **N_persist increments to 4** (post-extension run extending). **Library code was not written for this engagement** — verifies node_modules resolution + package.json exports + a third-party-style library across runtimes. **Sub-criterion 5 J.1.a count: 16 fixtures**; 215/215 host tests; 805/805 workspace; M7 fold-back: **fully vacuous AND author-side F3**) |
| 2026-05-10 | (this) | **K=1 Phase-2-traversal: vendored mri-shape argv parser (consumer-argv-parser J.1.a)** (escalation from clsx ~40 LOC to mri ~150 LOC of real-shape vendored third-party code; tests exercise the full documented argv-parser surface — long flags, equals form, short flags, aliases, booleans, --no- negation, strings with defaults, positional `_` array, `--` separator, mixed shapes, repeated flags accumulating to array, numeric coercion of value-shapes; 12/12 byte-identical Bun 1.3.11 ↔ rusty-bun-host; library uses idiomatic older JS (var, charCodeAt loops, regex-free parsing, `~indexOf` truthy-check pattern, Array.prototype.concat for variadic accumulation) — none of which the engagement-internal author would naturally write but which real npm packages routinely use. **N_persist increments to 5** (post-extension run extending). **Sub-criterion 5 J.1.a count: 17 fixtures**; 217/217 host tests; 807/807 workspace; M7 fold-back: **fully vacuous**) |

**Pilot inventory (16 pilots):**

| # | Pilot | Class | LOC (code-only) | Verifier | Consumer | Aggregate ratio anchor |
|---:|---|---|---:|---:|---:|---|
| 1 | TextEncoder + TextDecoder | data structure | 147 | 21 (1 skip) | 11 | 13–25% adj |
| 2 | URLSearchParams | delegation target | 186 | 32 | 11 | 62% (vs WebKit) |
| 3 | structuredClone | algorithm | 297 | 23 | 10 | 3.9% naive / ~8.5% adj |
| 4 | Blob | composition substrate | 103 | 26 | 10 | 20–35% adj |
| 5 | File | inheritance/extension | 43 | 16 | 8 | 20–30% adj |
| 6 | AbortController + AbortSignal | event/observable | 126 | 22 | 10 | 25–35% adj |
| 7 | fetch-api (Headers + Request + Response) | system / multi-surface | 405 | 34 | 16 | 6.5% naive / ~20% adj |
| 8 | node-path | Tier-2 Node-compat pure-function | 303 | 51 | 11 | 8.3% naive / ~12–15% adj |
| 9 | streams (Readable + Writable + Transform) | substrate / async-state-machine | 453 | 29 | 9 | 11.2% naive / ~12–15% adj |
| 10 | buffer | Tier-2 Node-compat binary type | 261 | 44 | 11 | 11.1% naive / ~17% adj |
| 11 | Bun.file | Tier-2 Bun-namespace + first I/O | 95 | 24 | 8 | 3.0% naive (with Blob) / ~20-30% adj |
| 12 | Bun.serve | Tier-2 Bun-namespace flagship / data-layer system | 175 | 24 | 8 | 0.5% naive / ~20-30% adj |
| 13 | Bun.spawn | Tier-2 Bun-namespace subprocess | 179 | 19 | 8 | 2.8% naive / ~15-20% adj |
| 14 | node-fs | Tier-2 Node-compat fs sync subset | 95 | 28 | 8 | 0.4% naive / ~8% adj |
| 15 | node-http | Tier-2 Node-compat http data-layer | 208 | 21 | 8 | 6.3% naive / ~10-15% adj |
| 16 | web-crypto | Tier-2 Web Crypto subset (real primitives) | 101 | 22 | 8 | N/A (real impl, not delegation) |
|   | **Aggregate** | | **3,177** | **436 (1 skip)** | **155** | **~3.1% naive across ~102,000+ LOC upstream** |

Total tests: **591 verifier + consumer-regression pins. 1 documented skip. 0 regressions.**

Doc-tier corpus output:
- [Doc 704](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) — port-as-translation is a category error
- [Doc 705](https://jaredfoy.com/resolve/doc/705-pin-art-operationalized-for-intra-architectural-seam-detection) — Pin-Art operationalized for architectural seams
- [Doc 706](https://jaredfoy.com/resolve/doc/706-three-pilot-evidence-chain-derivation-from-constraints) — three-pilot evidence chain (forward direction)
- [Doc 707](https://jaredfoy.com/resolve/doc/707-pin-art-at-the-behavioral-surface-bidirectional-probes) — bidirectional Pin-Art at the behavioral surface

---

## II. Queued — priority-ordered

**The next session picks from the top.** Re-prioritization without a stated reason violates the protocol (per Doc 581 D3 / seed M1).

### Tier-A — substrate pilots (unblock other queued items)

1. ~~Streams pilot~~ — **DONE** 2026-05-10 (453 LOC, 38/38 tests, 11.2% naive ratio, first pilot where spec-extract layer dominated)

2. ~~Buffer pilot~~ — **DONE** 2026-05-10 (261 LOC; 11.1% naive ratio; 11 cited consumer dependencies; 44+11 tests)

### Tier-B — flagship Bun-namespace pilots (Tier 2 ecosystem-only; tests apparatus on no-spec target)

3. ~~Bun.file pilot~~ — **DONE** 2026-05-10 (95 LOC paired with rusty-blob; 32/32 tests; first Tier-B Bun-namespace; first pilot with real I/O)

4. ~~Bun.serve pilot~~ — **DONE** 2026-05-10 (175 LOC; 32/32 tests; flagship Bun API at data-layer scope; composes with fetch-api)

5. ~~Bun.spawn pilot~~ — **DONE** 2026-05-10 (179 LOC; 27/27 tests; completes Tier-B; std::process wrapper; 6,389 LOC upstream)

### Tier-C — major Node-compat surfaces

6. ~~Node fs sync subset~~ — **DONE** 2026-05-10 (95 LOC; 36/36 tests; first Tier-C pilot; 21,540 LOC upstream; std::fs wrapper)

7. ~~Node http/https data-layer~~ — **DONE** 2026-05-10 (208 LOC; 29/29 tests; 6.3% naive vs 3,316-LOC TS core)

8. ~~Node crypto.subtle subset~~ — **DONE** 2026-05-10 (101 LOC; 30/30 tests; SHA-256 + UUID v4 + getRandomValues + timing-safe; first real cryptographic primitives from scratch in apparatus); **completes Tier-C**

### Tier-D — apparatus refinements / methodology

9. ~~Workspace consolidation~~ — **DONE** 2026-05-10 (`Cargo.toml` at repo root registers all 16 pilots + derive-constraints + welch; `cargo test --workspace --release` runs 591 tests in one command)

10. ~~Pilot runner script~~ — **DONE** 2026-05-10 (`bin/run-pilots.sh` runs `cargo test --workspace --release` + emits structured summary: suites OK/FAIL + tests passed/failed/ignored)

11. **WPT ingestion (proposed in 2026-05-10 conversation)** — pull web-platform-tests as a third source corpus alongside specs/ and test corpora. Converts every spec extract from "what should be true" to "what is empirically tested." Heavier engineering: ~200 LOC for the WPT extractor + tree-sitter integration. **Re-open condition: a pilot whose constraint coverage from spec extracts alone is operationally insufficient surfaces a derivation gap.**

12. ~~AuthorityTier schema extension~~ — **DONE** 2026-05-10 (Spec / Ecosystem / Contingent enum on every clause; `classify_authority_tier()` default-tagging; spec extracts always Spec, web-platform subjects always Spec, Bun-namespace + Node-API always Ecosystem, default Contingent; Bun corpus breakdown: 1.3% / 9.1% / 89.7%)

### Tier-E — apparatus-saturation-anchoring docs

13. ~~Doc 708 — apparatus saturation record~~ — **DONE** 2026-05-10 (records the four prior-framing criteria as met: coverage of architectural classes / aggregate-ratio / consumer-corpus / doc-tier; folds these into Sub-criterion 1 of the new completion telos at seed §VII)

14. **Cumulative apparatus paper** — academic-style writeup of the apparatus + the bidirectional reading + the saturation-to-completion arc. Audience: external. Defer until completion arc has accumulated more measurement.

---

## TELOS UPDATE 2026-05-10

The seed's §VII has been re-anchored. **Telos: rusty-bun is complete against Bun when a real consumer can swap rusty-bun for Bun and run their JS-using application without regression.** The prior framing (apparatus saturation) is now Sub-criterion 1 of five. Saturation is necessary; completion requires the remaining four sub-criteria. Tiers F through J anchor those.

---

### Tier-F — Surface-API completeness (Sub-criterion 2)

Every Bun runtime API has a pilot anchor. Estimated 50-80 additional pilots beyond the current 16. The list below is a starting partition; per-surface scope is decided at pilot time.

**F.1 — Web Crypto subtle full surface.** generateKey / deriveKey / importKey / exportKey / sign / verify / wrapKey / unwrapKey across HMAC + AES-GCM + AES-CBC + AES-CTR + RSA-OAEP + RSA-PSS + RSA-SSA-PKCS1-v1_5 + ECDSA + ECDH + Ed25519 + X25519 + HKDF + PBKDF2. Plus SHA-1, SHA-384, SHA-512 digests. Big lift; multi-pilot.

**F.2 — Streams full surface.** ReadableByteStream BYOB reads, async iterator (`Symbol.asyncIterator`), transferable streams, pipeTo / pipeThrough automation. Composes with the existing streams pilot.

**F.3 — Node-compat: net, tls, dgram, dns.** Socket-level networking. Each is its own pilot.

**F.4 — Node-compat: zlib, stream (Node-style), events, os.** Compression + Node's stream variant + EventEmitter + OS info.

**F.5 — Node-compat: cluster, worker_threads, vm, perf_hooks, async_hooks.** Process / thread / VM / observability surfaces.

**F.6 — Node-compat: readline, repl, tty, assert, timers, inspector, module.** Interactive + assertion + module surfaces.

**F.7 — Bun-namespace: Bun.password, Bun.SQLite, bun:redis, bun:s3.** Stateful / transactional surfaces.

**F.8 — Bun-namespace: Bun.Cookie, Bun.JSONL, Bun.Image, Bun.Archive, Bun.Glob, Bun.YAML, Bun.CryptoHasher, Bun.deepEquals, Bun.inspect.** Utility surfaces.

**F.9 — Bun-namespace: Bun.connect, Bun.listen, Bun.dns, Bun.write, Bun.fileURLToPath, Bun.pathToFileURL, Bun.Terminal, Bun.cron.** I/O + Bun-specific utilities.

**F.10 — Worker / MessagePort / BroadcastChannel.** Cross-realm message passing. Composes with structuredClone (anchored) + streams (anchored).

**F.11 — WebSocket (server + client).** Bidirectional networking. Composes with streams + transport-layer (Tier-G).

### Tier-G — Transport-layer pilots (Sub-criterion 3)

The data-layer-only pilots lift to wire-format. Each is significantly larger than its data-layer counterpart.

**G.1 — fetch transport.** HTTP/1.1 wire format + connection pooling + TLS handshake + redirect handling + body streaming. Composes with fetch-api (Pilot 7).

**G.2 — Bun.serve transport.** Socket binding + HTTP/1.1 wire parsing + WebSocket upgrade + TLS termination + file streaming. Composes with Bun.serve (Pilot 12).

**G.3 — Bun.spawn transport.** IPC channels + streaming stdio + terminal mode. Composes with Bun.spawn (Pilot 13).

**G.4 — Node http/https transport.** HTTP wire format + cert handling + connection pool. Composes with node-http (Pilot 15).

**G.5 — HTTP/2 + HTTP/3 transport.** Modern protocols. Heavy lift; deferred until G.1 + G.4 land.

### Tier-H — JS host integration (Sub-criterion 4)

~~**H.1 — JS engine selection.**~~ — **DONE** 2026-05-10 (selected rquickjs 0.6; production-tested QuickJS Rust binding; ~150 LOC of FFI glue produces a 1.6 MB binary)

**H.2 — Pilots-to-JS FFI.** ✅ DONE: 19 pilot families wired (atob/btoa, path.*, crypto + crypto.subtle, TextEncoder/TextDecoder, Buffer, URLSearchParams, fs sync subset, Blob, File, AbortController/AbortSignal, Headers, Request, Response, Bun.file, Bun.serve, Bun.spawn, structuredClone, streams). Remaining wirings, **priority-ordered for next session**:

  1. ~~**structuredClone**~~ — DONE 2026-05-10 (Pattern 4 — spec-formalization pilot, JS-side instantiation; 13 integration tests + canonical-docs composition test).
  2. ~~**streams (ReadableStream/WritableStream/TransformStream)**~~ — DONE 2026-05-10 (Pattern 4; 9 integration tests including canonical-docs composition; eval_string_async helper added to host).
  3. ~~**node-http data-layer**~~ — DONE 2026-05-10 (Pattern 4; 13 integration tests including canonical-docs composition + URLSearchParams cross-pilot composition; completes H.2).

  H.2 ALL WIRED. Next H subitems are H.3 (module loader/resolver), H.4 (remaining globals: URL/setTimeout/console-format), H.5 (console + error reporting / source maps). Each wiring ships with (a) integration tests including a **canonical-docs composition test** that mirrors the upstream's flagship usage example verbatim, AND (b) a fold-back commit per M7 if any new patterns surfaced.

**H.3 — Module loader + resolver.** ✅ DONE (CJS + ESM): CommonJS via JS-side bootRequire (`82f7b07`); ESM via Rust-side rquickjs Resolver + Loader with node-style resolution (this round). Both honor relative + absolute + bare specifiers, package.json (main, module, exports."." subpath for CJS), .mjs/.js/.cjs/.json + dir/index resolution, and node_modules walk-up. Real consumer code can require() OR import. Remaining for full sub-criterion 4: import.meta.url/dir, dynamic import(), and the npm-package consumer test harness (a few small real packages from node_modules) — these are H.3 polish items rather than blocking.

**H.4 — globalThis setup.** SUBSTANTIALLY DONE: setTimeout/setImmediate/clearTimeout, queueMicrotask, performance.now/.timeOrigin, URL class all landed 2026-05-10. Remaining gaps: real-time scheduled setInterval (microtask-scoped only), AbortSignal.timeout, performance.mark/measure, URL percent-encoding edge cases (IDN, full byte tables — pilot uses encodeURI/encodeURIComponent), TextEncoder.encodeInto. These are polish; consumer code that touches them is rare enough that closure can wait for a Tier-J consumer to surface them.

**H.5 — Console + error reporting.** Bun's console output format, error stack traces, source maps.

### Tier-I — WPT runner (compliance signal)

**I.1 — WPT runner adapter.** `wpt run` adapter for the integrated rusty-bun runtime. Exposes the rusty-bun executable as a "browser" wpt knows how to drive.

**I.2 — WPT execution per surface.** Run WPT against each piloted surface. Record pass-rate per WHATWG spec area (URL / Encoding / Streams / Fetch / FileAPI / etc.).

**I.3 — WPT pass-rate vs Bun.** Compare rusty-bun's WPT pass-rate against Bun's published WPT pass-rate for the same surfaces. Equivalence = spec-conformance plug-and-play.

### Tier-J — Differential testing against Bun-using applications (Sub-criterion 5)

**J.1 — Application basket.** Curate a representative basket of Bun-using applications: Hono / Elysia frameworks at example-app level; npm packages whose test suites Bun runs cleanly; Cloudflare Workers examples; small but real-world apps.

Sub-categorized by differential portability per M8 (instituted 2026-05-10):

  **J.1.a — Tier-J portable (differentially verified):** fixtures that run identically on Bun and rusty-bun-host.
   - `consumer-todo-api/` (ESM, Bun.serve fetch-handler dispatch, todo CRUD; 10/10 self-tests on both; **byte-identical differential** post-M8 reconciliation).
   - `consumer-stream-processor/` (CJS, ReadableStream/TransformStream/WritableStream pipeline + AbortController + setTimeout + node:fs + Headers + Buffer + URL; 8/8 self-tests on both; **byte-identical differential** after all three M8 re-open conditions closed).
   - `consumer-request-signer/` (ESM, 7-module deep dep graph; middleware composition validate→augment→sign; crypto.subtle.digest("SHA-256",data); async ReadableStream iteration via for-await-of; canonical-JSON signing; 6/6 byte-identical from inception, M8(a) digest API reconciliation in-round).
   - `consumer-log-aggregator/` (ESM, node:path module-import + user-defined Emitter + structuredClone defensive copy + URLSearchParams filter-query + Array.flatMap/filter/map; 9/9 byte-identical from inception, M8(a) ESM node:* resolution reconciliation in-round).
   - `consumer-job-queue/` (ESM, class inheritance BaseJob→Job→PriorityJob + async generator drain + node:crypto.randomUUID import + custom Error subclasses + Symbol-keyed private state + JSON.stringify with replacer; 8/8 byte-identical from inception with **zero reconciliation needed** — Phase-2 steady-state signature per Doc 709).
   - `consumer-batch-loader/` (ESM, Promise.all/allSettled/race + Proxy with get-trap + Reflect.has/ownKeys + BigInt arithmetic + BigInt-keyed Map + tagged template literals + Object.fromEntries + spread aggregation; 9/9 byte-identical from inception with **zero apparatus reconciliation** — second consecutive Phase-2 signature round).
   - `consumer-log-analyzer/` (ESM, regex named-capture + matchAll + Date arithmetic + Array.reduce-into-Map + sort-by-comparator + Object.entries→fromEntries pipeline + padStart + Map insertion-order; 9/9 byte-identical from inception with **zero apparatus reconciliation** — third consecutive Phase-2 round).
   - `consumer-task-pipeline/` (ESM, Symbol.asyncIterator user-class + generator delegation yield* + WeakMap memoization + Function.prototype.bind + WeakSet + async-iterable composition; 8/8 byte-identical from inception with **zero apparatus reconciliation** — fourth consecutive Phase-2 round).
   - `consumer-sequence-id/` (ESM, Atomics + SharedArrayBuffer + sync generators (function*) + generator.return() + Array.from(iterable) + spread of generator + yield* + String.raw + Object.defineProperty accessor + Symbol.toPrimitive; 9/9 byte-identical from inception with **zero apparatus reconciliation** — fifth consecutive Phase-2 round; first K=3 multi-op round).
   - `consumer-config-merger/` (ESM, Array.toSorted/toReversed/toSpliced/with (ES2023 immutable) + Promise.withResolvers (ES2024) + Object.groupBy (ES2024) + structuredClone on Uint8Array + deep object spread; 10/10 byte-identical from inception with **zero apparatus reconciliation** — sixth consecutive Phase-2 round).
   - `consumer-system-info/` (ESM, `import os from "node:os"` for platform/arch/type/tmpdir/homedir/hostname/EOL; 8/8 byte-identical; **landed via M8(a) apparatus extension** — wire_os in host's lib.rs + NODE_BUILTINS table entries for symmetric ESM/CJS resolution; closes the node:os portion of E.9 compound boundary).
   - `consumer-meta-protocols/` (ESM, Symbol.hasInstance custom instanceof + AsyncGenerator.throw() error injection + structuredClone on circular-reference graph + regex sticky /y with .lastIndex anchoring + String.prototype.replaceAll + JSON.parse with reviver + Array.flat with depth + Symbol.iterator on plain object + JSON roundtrip with reviver; 9/9 byte-identical with **zero apparatus reconciliation**).
   - `consumer-deferred-coordinator/` (ESM, top-level await + async generator .return() early-exit + WeakMap promise-source tracking + Object.hasOwn (ES2022) + Array.at / String.at negative indexing + Promise.allSettled mixed outcomes + try/finally cleanup on async-iter return; 9/9 byte-identical with **zero apparatus reconciliation**).
   - `consumer-binary-decoder/` (ESM, DataView read/write mixed endianness + TypedArray subarray/set + Number predicates + Math.hypot/.fround/.log2/.cbrt + toFixed/toExponential + Array.findLast/.findLastIndex (ES2023) + hex/binary/octal literals + Float64Array.reduce + DataView bounds RangeError; 12/12 byte-identical with **zero apparatus reconciliation**).
   - `consumer-vendored-pkg/` (ESM, vendored real-shape npm package — clsx v2.1.0 verbatim from github.com/lukeed/clsx, MIT-licensed; exercises package.json `exports` field with import/require/default conditions; default + named imports; 10 README-idiom test cases; **library code not written for this engagement** — first Tier-J fixture using actual third-party npm code; 10/10 byte-identical).
   - `consumer-argv-parser/` (ESM, vendored mri-shape argv parser ~150 LOC MIT; exercises long flags, equals-form, short flags, aliases, booleans, --no- negation, string flags with defaults, positional _ array, -- separator, mixed shapes, repeated flags accumulating to array, numeric coercion; library uses idiomatic older JS (var, charCodeAt loops, `~indexOf`, Array.concat for variadics) representative of real npm package shape; 12/12 byte-identical).
   - `differential/portable.js` (spec-portable surface: URL, URLSearchParams, structuredClone, Buffer.byteLength, TextEncoder/Decoder, atob/btoa, crypto.randomUUID format, Date, JSON, Headers; 31/31 lines match).

  **J.1.b — rusty-bun-internal (not Tier-J differential):** none currently. All previously-J.1.b fixtures have been reconciled to J.1.a per M8.

**J.2 — Differential runner.** PARTIAL 2026-05-10: differential test landed at `host/tests/integration.rs::js_differential_portable_matches_bun`. Runs `host/tests/fixtures/differential/portable.js` against both Bun (subprocess) and rusty-bun-host (eval_esm_module), captures both outputs, asserts line-by-line equality. Skips cleanly if `bun` not on PATH. First differential pass: 31/31 lines match against Bun 1.3.11.

**J.3 — Per-app regression closure.** SUBSTANTIALLY MET for the current basket (all 3 fixtures J.1.a):
  - Spec-portable surface differentially verified — 31/31 lines match (`6187bc7`).
  - consumer-todo-api differentially verified — byte-identical "10/10" (`0948181`).
  - consumer-stream-processor differentially verified — byte-identical "8/8" (this commit).
  - Remaining: expand basket with additional consumer fixtures (Hono/Elysia at example-app level, npm packages whose test suites Bun runs cleanly). Each new fixture follows M8 in-round: any divergence surfaced is reconciled before commit.

**J.4 — Aggregate basket pass.** N apps × zero regressions × zero crashes = real plug-and-play. The operational completion criterion of the engagement.

## III. Deferred — with explicit re-open conditions

| Item | Re-open condition |
|---|---|
| Wired rederive integration (replacing LLM simulation) | The LLM-simulated derivation has saturated the apparatus' useful pilot space (~15+ pilots) AND keeper directs the integration |
| Bun.build / transpiler / bundler internals | Never (out of scope: compiler, not runtime surface) |
| HTTP/2 + HTTP/3 transport-layer details | A streams pilot lands AND keeper directs HTTP-pilot scope to include transport |
| WPT suite execution against pilots (real, not transcribed) | A JS-host shim (Boa or QuickJS) is on the table AND a pilot's verifier needs operational JS-runtime fidelity |
| Inspector / debugger / DevTools protocol | Never (tooling, not runtime API surface) |
| Bun.SQLite, bun:redis, bun:s3 | After Bun.serve, Bun.file, Bun.spawn (Tier 2 flagship surfaces) anchor the no-spec apparatus pattern |
| Worker / MessagePort / structuredClone-as-transfer | Streams pilot lands (Worker depends on transfer mechanics that involve streams) |
| TLS / DNS / net (raw socket Node-compat) | Bun.serve + Node.http pilots have anchored the data-layer scope; transport scope is then keeper-decision |
| Macro params / Bun-internal hooks | Never (Bun-internal, not external runtime surface) |
| Async iterator protocol (`Symbol.asyncIterator`) for streams | The Streams pilot specifically requires it for completeness |

---

## IV. Live-state spot-check (run on session resume)

Before picking the next queued item, verify:

1. **`cargo test --release` passes in every existing pilot crate.** From the rusty-bun root:

   ```bash
   for p in textencoder urlsearchparams structured-clone blob file abort-controller fetch-api node-path; do
     echo "=== $p ===" && (cd pilots/$p/derived && cargo test --release 2>&1 | grep -E "test result|FAILED")
   done
   ```

   Expected: every line says `ok. N passed; 0 failed`. Any FAILED is a regression that pre-empts queued work.

2. **`derive-constraints` builds.** From `derive-constraints/`: `cargo build --release`. Should complete in <30s.

3. **The latest pipeline run is reproducible.** `runs/2026-05-10-bun-v0.13b-spec-batch/RUN-NOTES.md` records the canonical run. If the keeper has been editing the apparatus, re-run the pipeline before assuming constraint corpus is current.

4. **`/tmp/welch-corpus/target/{bun,deno}` exists** for re-runs. If not, re-clone shallow (commands in any `RUN-NOTES.md`).

5. **Auto-memory points at this pair.** Verify `MEMORY.md` includes a pointer to `seed.md` + `trajectory.md` for rusty-bun. (Update if not.)

---

## V. Resume protocol

Per Doc 581 §III Move 3, this is the four-to-six-step procedure a session reading both seed and trajectory should follow.

1. **Load ENTRACE** (Doc 1).
2. **Read [seed.md](seed.md) once.** It carries the binding constraints, the architecture decisions, and the future-move discipline.
3. **Read this trajectory file from the top.** Skim §I (Done) for context; read §II (Queued) carefully; check §III (Deferred) only if the session's question touches a deferred item.
4. **Run §IV (Live-state spot-check).** Confirms apparatus is in a runnable state before queued work begins.
5. **Pick the topmost queued item from §II that hasn't been blocked by re-prioritization.** Advance the work; commit; update §I (append-only) and revise §II as items move.
6. **Don't re-prioritize §II without a stated reason.** If priorities change, record the change as part of the session's commit message AND the §II edit.
7. **Apply §IV.M7, §IV.M8, and §IV.M9 every round.** Before the round closes: (a) classify the round's M7 fold-back per the §III.A8.11 outcome taxonomy (primitive / vacuous / compositionally vacuous / compositional finding / author-side / scope-limit verified); fold back if primitive or compositional. (b) If the round attempted a Bun differential and surfaced a divergence, reconcile per M8 before commit — either align the apparatus with Bun, or record an explicit scope-limit + remove the dependent fixture from the J.1.a (differentially verified) set. "Noted, will deal with later" is forbidden. (c) When the round adds a Tier-J fixture, author it spec-first against Bun per M9: write Bun-spec idioms, run under Bun for baseline, run under rusty-bun-host, reconcile divergences in the same commit. The fixture ships J.1.a directly; J.1.b is reserved for fixtures whose divergences cannot be reconciled in-round. (d) Update `N_persist` per §III.A8.10: increment if the round landed J.1.a with zero apparatus reconciliation AND covered an axis not in the prior basket; reset to 0 if any M8(a) reconciliation was needed; record the current axis-set in the trajectory header.

Discipline against decoration: if a session skips this protocol and re-derives state from the seed alone or from `git log`, the protocol is failing per [Doc 581 F3](https://jaredfoy.com/resolve/doc/581-the-resume-vector). Note the failure and adjust.

---

## VI. Trajectory metadata

- Created: 2026-05-10
- Last updated: 2026-05-10 (initial creation)
- Update discipline: per Doc 581 D2 (Done is append-only); D3 (Queued is priority-ordered, mutable with reason); D4 (Deferred has re-open conditions); D5 (seed updates only when architecture moves)
- Auto-memory pointer: `MEMORY.md` to be updated alongside this file's first commit
