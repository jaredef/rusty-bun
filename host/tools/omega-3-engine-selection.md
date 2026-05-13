# Tier-Ω.3 — Engine Selection Decision

**Date:** 2026-05-13 night
**Status:** Decision artifact, derived from [Doc 717](https://jaredfoy.com/resolve/doc/717-the-apparatus-above-the-engine-boundary-the-three-projections-lifted-to-engine-substrate-and-the-pure-abstraction-point) + the P3 classification (`host/tools/p3-classification.md`).
**Keeper directive (2026-05-13 19:53Z):** "It becomes apparent that a hand roll quickjs is imperative."

## I. Decision space

Candidate engines, evaluated against the three Doc 717 tuples that account for 12/14 of the residual parity failures.

| Candidate | Tuple A: Module Namespace cut rung | Tuple B: realm host-hooks for named-export synthesis | Tuple C: parser ES edition | Architectural fit | Derivation-discipline alignment |
|---|---|---|---|---|---|
| **rquickjs / QuickJS (current)** | E2 (frozen at construction) — does NOT match Bun | None exposed — does NOT match Bun | ~ES2020 + selective patches — does NOT match Bun | Embedded-friendly, FFI overhead present | Modifies third-party — derivation not owned |
| **rquickjs / QuickJS-NG (upstream-fork upgrade)** | E2 same as QuickJS — does NOT match Bun | None — does NOT match Bun | ES2022+ — MATCHES Bun | Embedded-friendly, same FFI as current | Same — derivation not owned |
| **Boa (Rust-native engine)** | Conformant-spec (Module Namespace frozen per ECMA-262) — does NOT match Bun out of box | Limited; would require Boa-internal contribution | ES2024 in progress — MATCHES Bun | Native Rust, no FFI penalty, closer architectural fit | Adopting upstream + contributing patches — derivation not fully owned |
| **Patched QuickJS-NG (vendor + diff)** | E5 if we patch [[OwnPropertyKeys]] for Module Namespace — MATCHES | Yes if we patch — MATCHES | ES2022+ inherited — MATCHES | Embedded-friendly | We own the diff; engine derivation still upstream |
| **Hand-roll Rust JS engine (per engagement discipline)** | E5 by construction — MATCHES | Yes by construction — MATCHES | Implement what we need — MATCHES | Native Rust, no FFI, architectural fit | Full derivation ownership per Doc 581 |

## II. Selection

**Decision: hand-roll a Rust JS engine following the rusty-bun derivation discipline, with QuickJS as the architectural reference.**

QuickJS as architectural reference means: small AST + bytecode-interpreter, no JIT, embedded-first, single-pass compiler. This is the architecture Bellard chose for QuickJS and that the keeper's directive named. The engagement's derivation discipline (per [Doc 581](https://jaredfoy.com/resolve/doc/581-the-resume-vector)) means: implement against ECMA-262 + WHATWG as the pure abstraction point per [Doc 717 §VII](https://jaredfoy.com/resolve/doc/717-the-apparatus-above-the-engine-boundary-the-three-projections-lifted-to-engine-substrate-and-the-pure-abstraction-point), with our specific cuts chosen against Bun's cut-profile to minimize migration distance.

The other candidates lose on the same axis: they do not match Bun on Tuple A and Tuple B without internal patches, and adopting + patching a third-party engine carries the cost of long-term divergence-tracking without delivering the derivation-discipline payoff. Vendoring QuickJS-NG and patching it would close the parity gap faster (months not quarters) but at the cost of the engagement's core epistemic stance. The keeper's directive is decisive against that shortcut.

**Architectural reference summary (the "QuickJS" in "hand-roll QuickJS"):**

- AST built by a hand-rolled recursive-descent parser; no parser generator
- Single-pass bytecode compiler (AST → bytecode in one walk)
- Interpreter with computed-goto-style dispatch
- Conservative mark-sweep GC, no generational/incremental complexity in v1
- Module-loading via a host-defined hook layer at E5 (Doc 717's realm rung) — this is where Bun-parity for Tuples A/B is achieved
- Embedding API directly in Rust (no FFI through C)

## III. Implementation roadmap derived from P3's tuple classification

The P3 verification produced an implementation priority. Each tuple corresponds to a sub-round of Tier-Ω.4.

**Ω.4.a — Module Namespace augmentation hooks (Tuple A, closes 7 packages).**
- Implement Module Namespace exotic object [[OwnPropertyKeys]] with post-init augmentation permitted via a realm-level host hook
- Hook fires after ParseModule's NamedExports table is constructed but before the namespace freezes
- Default behavior: if NamedExports lacks `default`, synthesize `default` = the namespace itself (the Bun behavior that retires yup/io-ts/superstruct/neverthrow/jsonc-parser/fp-ts + yargs's y18n cascade)

**Ω.4.b — Realm post-init named-export synthesis (Tuple B, closes 3 packages).**
- Hook fires after `export default X` evaluates and X is bound
- Default behavior: enumerate X's own properties; for each, if no name-collision with an existing named export, synthesize one. Includes `export default function NAME(...)` → expose NAME.
- Retires dayjs, date-fns, node-fetch

**Ω.4.c — Parser grammar refresh (Tuple C, closes 2 packages).**
- Implement ES2022 module-export grammar: string-literal export aliases (`export { x as 'm-search' }`)
- Implement modern class-field forms (arrow-fn-init variants, reserved-name class fields where currently we preprocess)
- Robust grammar acceptance on minified-ESM (the E.60 elysia SIGSEGV class)
- Retires superagent, ora; also retires basket boundary E.60, E.12 hono ^4 arrow-fn variant, E.62 yargs

**Ω.4.d — Cleanup of compensating polyfills (no parity delta; line-count reduction).**
- The CJS→ESM bridge's `synthesize-via-getOwnPropertyNames` preprocessor becomes obsolete once Ω.4.a/b land natively
- Source-rewriting helpers `strip_reserved_class_field_decls`, `rewrite_destructure_exports`, `rewrite_regex_u_class_escapes` become obsolete once Ω.4.c parser handles their patterns natively
- The K1/K2/K3 stub catalogue (Doc 716 §VI) becomes the audit checklist: any K1/K2 that compensated for a now-resolved engine cut retires
- Estimated line reduction in host/src/lib.rs: ~800 LOC of preprocessors + bridge logic

**Ω.4.e — Substrate migration of the existing pilot wirings.**
- ~26 Rust pilots currently FFI-bridged via rquickjs need to expose through the new engine's Rust-native embedding API
- Conceptually mechanical; per-pilot ~30-50 LOC of wiring change
- The five sub-rounds above happen before this so we don't migrate twice

**Ω.4.f — Bootstrap minimum: enough engine to load the parity test fixture.**
- Defines "done" for Ω.4: the parity-measurement tool (host/tools/parity-measure.sh) runs against the new engine and produces a measurement
- Predicted post-migration baseline per P3 §predictions: 117/119 ≈ 98.3% if Ω.4.a/b/c all match Bun cleanly

## IV. Pre-implementation prerequisites

**Ω.3.a — ECMA-262 corpus seeding.** Pull ECMA-262 + WHATWG (Streams, URL, Encoding, Fetch) into the engagement's spec corpus alongside the existing Bun corpus. Per the existing seed §III.A2 derivation discipline, pilot work derives from specs. The engine's derivation will read against ECMA-262 the way pilots read against Bun.

**Ω.3.b — Architectural tier-1 pilots.** Before the engine itself, build the load-bearing substrate pilots in rusty-bun's existing tier:
- `rusty-js-parser` (recursive-descent ES2022 module-aware grammar)
- `rusty-js-ast` (typed AST nodes)
- `rusty-js-bytecode` (instruction set + compiler from AST)
- `rusty-js-runtime` (Value representation, intrinsic objects, execution-context records, realm)
- `rusty-js-gc` (conservative mark-sweep)

Each is a tier-A or tier-G pilot in the engagement's existing pilot grammar.

**Ω.3.c — Engine assembly.** The pilots compose into the engine. Host wiring is a thin layer atop the runtime's embedding API.

**Estimated round count:** 15-25 rounds for Ω.3.a-c (the engine), then 3-5 rounds for Ω.4.a-f (migration). Substantially larger than any prior phase of the engagement. Per [Doc 581 D6](https://jaredfoy.com/resolve/doc/581-the-resume-vector), this is the engagement-bounded scope at maximum credible cost.

## V. The decision artifact's audit trail

Per Doc 717 §VIII, this decision artifact is the Tier-Ω.3 deliverable. It cites:

- Doc 717 (the apparatus generalization that made this decision legible)
- The P3 classification (the empirical evidence the decision is built on)
- The keeper directive (the authority that named the path)
- The candidate analysis (the alternative space the decision rules out)

The decision is falsifiable: if the engine is built, Ω.4 runs, and the predicted 98.3% baseline does not materialize, the decision's premises (Doc 717's tuple analysis + P3's classification) were wrong about the residual's structure. The first round of Ω.4.a will be the first falsifier.

## VI. Open questions deferred to first implementation round

1. **GC strategy precision.** Conservative mark-sweep is the v1 commitment; whether to add incremental marking is deferred to a successor round.
2. **WeakRef/FinalizationRegistry.** Currently K1-stubbed (basin boundary E.7). The hand-rolled engine can either implement them or keep the K1 stub; deferred.
3. **WebAssembly.** Permanent out-of-scope per Doc 716 §VI; the engine will not include a WASM execution engine.
4. **Promise/async-await scheduling.** Build atop the existing mio reactor (Π2.6.c.a). The engine's microtask queue interleaves with the reactor.
5. **Optimization passes.** Single-pass compile + naive interpreter for v1; any optimization is a successor-engagement scope.

## VII. Next move

**The first round of Tier-Ω.3 proper is the ECMA-262 corpus seeding (Ω.3.a) + the rusty-js-parser pilot (first sub-pilot of Ω.3.b).** ECMA-262's parser grammar is the smallest piece that opens the engine substrate; its derivation against the spec follows the engagement's existing tier-A pilot discipline.

The parity-measurement tool stays in place as the regression detector during Ω.4. Until Ω.4.f produces a measurement on the new engine, the current 88.2% baseline holds as the engagement's headline metric.
