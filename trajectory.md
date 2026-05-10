# rusty-bun — Trajectory

The living vector of the rusty-bun engagement. Per [Doc 581 (the Resume Vector)](https://jaredfoy.com/resolve/doc/581-the-resume-vector). This file changes session to session; the [seed](seed.md) does not. Read order on session resume: ENTRACE first; then the seed; then this file from the top; then run the live-state spot-check at §IV.

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
| 2026-05-10 | (this) | **Bun.file pilot — first Tier-B Bun-namespace, first pilot with real I/O (95 LOC; 3.0% naive paired with rusty-blob)** |

**Pilot inventory (11 pilots):**

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
|   | **Aggregate** | | **2,419** | **322 (1 skip)** | **115** | **~6.0% across ~40,000+ LOC upstream** |

Total tests: **437 verifier + consumer-regression pins. 1 documented skip. 0 regressions.**

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

4. **Bun.serve pilot — data-layer scope** — flagship Bun API. Routing + handler dispatch + response generation, no transport. Composes with fetch-api system pilot. Largest cross-corroborated cardinality on Bun-namespace surfaces. Estimated: 250–400 LOC, Tier-2 ecosystem-only.

5. **Bun.spawn pilot** — subprocess management; pure-Rust derivation can use `std::process`. Tier 2. Estimated: 150–250 LOC.

### Tier-C — major Node-compat surfaces

6. **Node `fs` (sync subset) pilot** — file system surface. Huge consumer impact. Use `std::fs` for the derivation. Estimated: 250–400 LOC.

7. **Node `http`/`https` pilot — data-layer scope** — Node's HTTP module. Composes with fetch-api system pilot's headers + request + response data structures. Estimated: 200–350 LOC.

8. **Node `crypto.subtle` pilot** — Web Crypto. Cross-corroboration is high. Could ship a subset (digest + hmac + random) first.

### Tier-D — apparatus refinements / methodology

9. **Workspace consolidation** — single Cargo workspace at `pilots/Cargo.toml` registering all per-pilot crates. Currently each pilot has its own Cargo.toml; a workspace would let `cargo test --workspace` run everything in one shot. Estimated: 30 LOC + Cargo.toml editing.

10. **Pilot runner script** — simple `bin/run-pilots.sh` that runs every pilot's `cargo test --release` and emits a single summary (X passed / Y failed / Z skipped across N pilots). Useful for keeper-readable status. Estimated: 50 LOC of bash.

11. **WPT ingestion (proposed in 2026-05-10 conversation)** — pull web-platform-tests as a third source corpus alongside specs/ and test corpora. Converts every spec extract from "what should be true" to "what is empirically tested." Heavier engineering: ~200 LOC for the WPT extractor + tree-sitter integration. **Re-open condition: a pilot whose constraint coverage from spec extracts alone is operationally insufficient surfaces a derivation gap.**

12. **`AuthorityTier` schema extension** — add `Spec | Ecosystem | Contingent` tier to ConstraintClause. Per the three-tier framing in seed §III.A3 and Doc 707 §"Plug-and-play criterion." Verifier reports per-tier conformance separately. Estimated: 50 LOC + propagation through pipeline.

### Tier-E — completion-criterion-anchoring docs

13. **Doc 708/709 — completion criterion** when ~12+ pilots are done across all classes. Records the closed engagement against the seed §VII criteria. Don't write before the data supports it.

14. **Cumulative apparatus paper** — academic-style writeup of the apparatus + the eight-plus pilots + the bidirectional reading. Audience: external. Defer until pilot library is broader.

---

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

Discipline against decoration: if a session skips this protocol and re-derives state from the seed alone or from `git log`, the protocol is failing per [Doc 581 F3](https://jaredfoy.com/resolve/doc/581-the-resume-vector). Note the failure and adjust.

---

## VI. Trajectory metadata

- Created: 2026-05-10
- Last updated: 2026-05-10 (initial creation)
- Update discipline: per Doc 581 D2 (Done is append-only); D3 (Queued is priority-ordered, mutable with reason); D4 (Deferred has re-open conditions); D5 (seed updates only when architecture moves)
- Auto-memory pointer: `MEMORY.md` to be updated alongside this file's first commit
