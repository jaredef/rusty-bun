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
| 2026-05-10 | (this) | **Tier-H continued: structuredClone wired** (17 pilot families; 120/120 host tests; introduced Pattern 4 — spec-formalization pilot, JS-side instantiation; folded back as seed §III.A8.2bis per M7) |

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

**H.2 — Pilots-to-JS FFI.** SUBSTANTIALLY DONE: 16 pilot families wired (atob/btoa, path.*, crypto + crypto.subtle, TextEncoder/TextDecoder, Buffer, URLSearchParams, fs sync subset, Blob, File, AbortController/AbortSignal, Headers, Request, Response, Bun.file, Bun.serve, Bun.spawn). Remaining wirings, **priority-ordered for next session**:

  1. ~~**structuredClone**~~ — DONE 2026-05-10 (Pattern 4 — spec-formalization pilot, JS-side instantiation; 13 integration tests + canonical-docs composition test).
  2. **streams (ReadableStream/WritableStream/TransformStream)** — stateful (Pattern 3), but the pilot is already shipped. Wiring blocks Response.body() and fetch() composition; high downstream leverage.
  3. **node-http data-layer** — last data-layer surface; Pattern 3 with method-keyed dispatch (same shape as Bun.serve route table). Use Bun.serve's `routes:` decoder as the template; the JS-side polymorphic-shape decode is the canonical move.

  Pick #2 next unless the keeper specifies otherwise. Each wiring ships with (a) integration tests including a **canonical-docs composition test** that mirrors the upstream's flagship usage example verbatim, AND (b) a fold-back commit per M7 if any new patterns surfaced.

**H.3 — Module loader + resolver.** ESM + CommonJS resolution. `import`, `require`, `import.meta`, package.json semantics, node_modules resolution.

**H.4 — globalThis setup.** Wire all the globals (URL, fetch, console, setTimeout, structuredClone, etc.) into the JS host's `globalThis`.

**H.5 — Console + error reporting.** Bun's console output format, error stack traces, source maps.

### Tier-I — WPT runner (compliance signal)

**I.1 — WPT runner adapter.** `wpt run` adapter for the integrated rusty-bun runtime. Exposes the rusty-bun executable as a "browser" wpt knows how to drive.

**I.2 — WPT execution per surface.** Run WPT against each piloted surface. Record pass-rate per WHATWG spec area (URL / Encoding / Streams / Fetch / FileAPI / etc.).

**I.3 — WPT pass-rate vs Bun.** Compare rusty-bun's WPT pass-rate against Bun's published WPT pass-rate for the same surfaces. Equivalence = spec-conformance plug-and-play.

### Tier-J — Differential testing against Bun-using applications (Sub-criterion 5)

**J.1 — Application basket.** Curate a representative basket of Bun-using applications: Hono / Elysia frameworks at example-app level; npm packages whose test suites Bun runs cleanly; Cloudflare Workers examples; small but real-world apps.

**J.2 — Differential runner.** Tooling to run `npm test` under both Bun and rusty-bun and diff outcomes per test.

**J.3 — Per-app regression closure.** For each app in the basket: P_bun ⊆ P_drv (every test passing on Bun also passes on rusty-bun). Zero regressions per app.

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

Discipline against decoration: if a session skips this protocol and re-derives state from the seed alone or from `git log`, the protocol is failing per [Doc 581 F3](https://jaredfoy.com/resolve/doc/581-the-resume-vector). Note the failure and adjust.

---

## VI. Trajectory metadata

- Created: 2026-05-10
- Last updated: 2026-05-10 (initial creation)
- Update discipline: per Doc 581 D2 (Done is append-only); D3 (Queued is priority-ordered, mutable with reason); D4 (Deferred has re-open conditions); D5 (seed updates only when architecture moves)
- Auto-memory pointer: `MEMORY.md` to be updated alongside this file's first commit
