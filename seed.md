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

2. **Stateful pilot APIs wire indirectly: stateless Rust helpers + JS-side class.** Rust closures that capture `Rc<RefCell<...>>` and are stored as methods on JS objects break QuickJS' GC (it does not track Rust references). Instead: expose a private `__namespace` of stateless Rust helper functions; install a JS-side class via `ctx.eval()` that holds its own state in pure-JS arrays/objects and calls into the Rust helpers for algorithm work. URLSearchParams + TextEncoder + TextDecoder use this pattern; future stateful types (Blob, File, Headers, Request, Response, AbortController, structuredClone-Heap) MUST follow it.

3. **Optional-arg semantics: JS omits, doesn't pass undefined.** rquickjs `Opt<T>` requires the JS-side to OMIT the argument, not pass `undefined`. JS-side classes that delegate to Rust helpers must branch: `if (val === undefined) call(without arg) else call(with arg)`.

4. **Testing surface:** every wired pilot has at least one JS-side integration test in `host/tests/integration.rs` plus appears in `host/examples/runtime-demo.js`. The workspace runner (`./bin/run-pilots.sh`) covers the host suite alongside per-pilot suites.

5. **Decode polymorphic JS shapes JS-side, not Rust-side.** When a JS API accepts a polymorphic argument shape — e.g., Bun.serve's `routes: { "/x": fn | { GET: fn, POST: fn } }`, fetch's `init: { headers: HeadersInit | Headers, body: BodyInit }` — the JS-side wrapper performs the discrimination and only hands canonical values to the Rust helpers. The Rust pilot stays a pure algorithm (e.g., `match_pattern(pattern, url)`); decoding the user's polymorphic input is JS work. This keeps pilot crates clean of host-encoding concerns and lets a single Rust helper serve many JS surface shapes.

6. **Cross-boundary type translation:** rquickjs does not bind tuples or structs as function args; use `Vec<Vec<String>>` as a pair-list across the FFI when the data is naturally `Vec<(String, String)>`. The JS-side wrapper assembles/disassembles into objects.

7. **Canonical-docs composition test.** Every wired flagship surface ships with at least one integration test that mirrors the upstream's documented usage example *verbatim* (see `js_compose_bun_serve_canonical_pattern`). This test is the smallest unit of "real consumer can swap rusty-bun for Bun" and is the verification of choice for sub-criterion 4. Per-method tests verify the surface; canonical-docs tests verify the **swap-in property**.

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

**M6. Host-wirability is a pilot design constraint.** New pilots' Rust APIs are designed to wire cleanly through the JS host pattern (A8). Concretely: prefer pure-value APIs; avoid `Rc<RefCell<...>>` in public interfaces; stateful types should provide stateless algorithm helpers alongside their owned-state types so the host can wire the helpers without adapting the type's storage. A pilot is "host-wirable" when its public API can be exposed via `host/` with no apparatus refinements — verifying this is a pilot-completion check.

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

The completion telos has five sub-criteria, in dependency order:

**Sub-criterion 1 — Apparatus saturation.** ✓ MET (Doc 708, 2026-05-10).
Sixteen pilots × eight architectural classes × five cybernetic modes × ~3% aggregate LOC ratio. The methodology is empirically anchored and ready for application.

**Sub-criterion 2 — Surface-API completeness.** Every Bun runtime API has a pilot anchor with verifier + consumer-regression closure. Estimated ~50-80 additional pilots beyond the current 16 to cover:
- Web Crypto full (subtle.generateKey/deriveKey/importKey/exportKey/sign/verify, HMAC, AES, RSA, ECDSA, Ed25519, HKDF, PBKDF2, SHA-384/512)
- Streams full (BYOB reads, async iterator protocol, transferable streams, pipeTo/pipeThrough automation)
- Node-compat: net, tls, dgram, dns, zlib, stream (Node), events, os, cluster, worker_threads, vm, perf_hooks, async_hooks, readline, repl, tty, assert, timers, inspector, module
- Bun-namespace: Bun.password, Bun.SQLite, bun:redis, bun:s3, Bun.Cookie, Bun.JSONL, Bun.Image, Bun.Archive, Bun.Terminal, Bun.cron, Bun.Glob, Bun.YAML, Bun.CryptoHasher, Bun.deepEquals, Bun.inspect, Bun.write, Bun.connect, Bun.listen, Bun.dns, Bun.fileURLToPath, Bun.pathToFileURL, etc.

**Sub-criterion 3 — Transport-layer pilots.** The data-layer-only pilots (fetch-api, Bun.serve, Bun.spawn, node-http) lift to wire-format pilots. Includes HTTP/1.1 + HTTP/2 wire parsing, socket binding, TLS handshake, WebSocket upgrade, IPC channels, streaming stdio. Required for any of these surfaces to function as runtime API.

**Sub-criterion 4 — JS host integration.** Embed a JS engine (QuickJS or Boa) and expose all pilots to JS code via FFI. Includes module loader / resolver, console + global setup, the `globalThis` shape Bun provides. Without this, no JS code can execute against the derived runtime; with it, rusty-bun becomes a runtime in the operational sense.

**Sub-criterion 5 — Differential testing against Bun-using applications.** The operational form of plug-and-play. For a representative basket of Bun-using applications (frameworks like Hono / Elysia, real-world apps): run `npm test` under Bun → record P_bun. Run under integrated rusty-bun → record P_drv. Diff. **Zero regressions across the basket** = real plug-and-play.

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
