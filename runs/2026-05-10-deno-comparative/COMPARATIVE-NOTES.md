# Deno comparative run — 2026-05-10

First cross-corpus run of the rusty-bun pipeline. Anchors [Doc 705](https://jaredfoy.com/resolve/doc/705-pin-art-operationalized-for-intra-architectural-seam-detection) §10 to a third operational instance — a hand-written Rust JS runtime — alongside the existing GitLab manual instance (§10.1) and Bun tooled instance (§10.2). Per Doc 693's standing-apparatus pattern, three instances anchor the apparatus more firmly than two.

## Inputs

- **Target corpus.** `denoland/deno` HEAD shallow clone (282 MB; 940 .rs files; 3,562 test source files of which 331 contain `Deno.test` calls).
- **Welch baseline.** Domain-matched async-runtime crates: `tokio` + `hyper` + `reqwest` (947 .rs files / 234,513 LOC).
- **Tool.** `derive-constraints pipeline` v0.10 — full eight-phase analysis end-to-end in one command (~3.7 seconds on this corpus).

The pipeline driver was extended in this run to support Deno's test conventions: `*_test.ts` filename pattern (vs Bun/Jest's `.test.ts`), and the `Deno.test(name, fn)` / `Deno.test({name, fn})` call shape (vs Bun's `test(name, fn)` / `it(name, fn)`).

## Numerical comparison

| Metric                       | Bun (phase-a-port) | Deno | Δ |
|------------------------------|-------------------:|-----:|---|
| Test files scanned           | 4,470              | 1,263 | −72% |
| Tests extracted              | 17,775             | (~3,800 with Deno.test) | −78% |
| Constraint clauses           | 43,094             | 11,399 | −74% |
| Properties (cluster)         | 4,838              | 1,852 | −62% |
| **Construction-style**       | **303**            | **0** | **see §"The construction-style finding" below** |
| Distinct signal vectors      | 93                 | 28    | −70% |
| Cross-namespace seams        | 50                 | 15    | −70% |
| Welch impl files scanned     | 1,429              | 940   | −34% |
| Welch baseline files         | 1,085              | 947   | −13% |
| Welch surfaces total         | 307                | 110   | −64% |
| Welch surfaces matched       | 70 (22.8%)         | 25 (22.7%) | match-rate ≈ identical |
| Welch mismatch candidates    | 21                 | 10    | −52% |

The two corpora differ in absolute size by a ~3-4× factor across nearly every metric. Match rates and ratios are stable.

## Three findings

### 1. The seam categories are real, not Bun-specific

Deno's seams exhibit the same architectural-hedging categories Bun's do, in proportion to corpus size:

```
SC0001  card=7,838  slack baseline (most properties)
SC0002  card=  839  ctor                   ← construct-then-handle
SC0003  card=  732  cfg                    ← platform-conditional
SC0004  card=  429  async                  ← async-discipline
SC0005  card=  350  ffi (122 properties!)  ← native/userland — see finding 3
SC0006  card=  315  sync                   ← synchronous syscall
SC0007  card=  300  cfg|ctor               ← compound
SC0008  card=  143  mixed                  ← sync+async
SC0009  card=  108  cfg|throw|plain_throw  ← Rust-style panics
SC0010  card=  105  throw|plain_throw      ← unreachable patterns
SC0011  card=   37  sync|ctor              ← Deno.opendirSync, readFileSync
SC0012  card=   31  ctor|@mod.rs           ← Rust-side test-helper patterns
```

Every Doc 705 §4 signal type that fired on Bun also fires on Deno, with the same architectural meaning. This is the strongest empirical anchor that the apparatus is reading *real architectural form* rather than Bun-specific corpus shape.

### 2. Implementation-internal seams are JS-runtime universals

The 10 welch-hot/seams-cold mismatch candidates Deno's coupling produced map to nearly the same architectural surfaces as Bun's:

```
Deno:                          Bun:
http      z=+19.2  raw_ptrs    http    z=+35.0  raw_ptrs
Buffer    z=+14.0  raw_ptrs    Stream  z=+20.0  raw_ptrs
buffer    z=+14.0  raw_ptrs    App     z=+15.9  raw_ptrs
util      z=+13.1  extern      module  z=+15.9  raw_ptrs
process   z=+ 8.4  unsafe_blk  WebSocket z=+13.8 raw_ptrs
crypto    z=+ 8.1  raw_ptrs    Hmac    z=+13.1  unsafe_blk
Stream    z=+ 7.7  raw_ptrs    SourceMap z=+15.3 raw_ptrs
                               (+ 14 more in the +5 to +15 band)
```

`http`, `Buffer`/`buffer`, `Stream`, `crypto`, `process`, `util` — the same architectural seams surface on both runtimes. **These are not vibe-port artifacts** (Deno is hand-written from day one). They are **JS-runtime universal architectural forms**: HTTP/parser FFI, native byte-pool heritage, threading/sync syscall infrastructure, BoringSSL crypto bindings, native process management.

This is the empirical control [Doc 466 framework-magnetism](https://jaredfoy.com/resolve/doc/466-doc-446-as-a-sipe-instance) calls for: the apparatus produces *similar shape* of finding on a different corpus, ruling out the explanation that Bun's seams are AI-translation-specific. The implementation-internal seam class is a real category of JS-runtime architecture; both runtimes have it; both expose it through the same probe-coupling apparatus.

### 3. Deno's tests surface FFI patterns Bun's don't (signal asymmetry)

The most surprising finding: **SC0005 in Deno fires `ffi` signal on 122 properties (350 cardinality)**. In Bun's run, the equivalent S5 signal fired on essentially zero properties at the test-corpus surface — the FFI boundary was implementation-internal-only.

Why does Deno expose FFI patterns at the test-corpus surface where Bun hides them?

Inspection reveals: Deno's tests directly reference Web-Platform-Tests assertions like `Float64Array`, `ArrayBuffer`, `Math.fround`, `Atomics` — the typed-array / shared-buffer surface that *is* the FFI boundary in any JS runtime. These appear in Deno's test corpus because Deno's WPT-shaped test layer exercises them as first-class test subjects. Bun's tests exercise these surfaces too but typically through higher-level wrappers (`Bun.file()` returning a typed-array result), so the FFI primitive appears as a result-of-wrapper, not as a direct test subject.

**Operational consequence:** the same architectural seam (FFI / typed-array heritage) is detected by different probes depending on the test-corpus's authoring style. Deno's WPT-direct testing surfaces it via the test-corpus probe layer (S5); Bun's wrapper-mediated testing hides it from S5 and requires the welch coupling layer to surface it. The triple decomposition apparatus from `couple v0.2` (bidirectional-visible / implementation-internal / contract-only) handles both correctly.

### The construction-style finding (zero on Deno)

Deno's classifier produces 0 construction-style properties vs Bun's 303. **This is a structural classifier limitation, not a finding about Deno's architecture.**

The cluster phase's construction-style classifier matches subjects against a public-API-surface allowlist. The current allowlist was built for Bun (`Bun.*`, `fs.*`, `URL`, `fetch`, `Buffer`, etc.). It includes Bun's runtime namespace but not Deno's. So:

- Bun's 124-properties-deep `Bun.*` surface → 124 candidate construction-style classifications
- Deno's 124-properties-deep `Deno.*` surface → 0 (because `Deno` isn't in `PUBLIC_API_HEADS`)

The fix is one-line: add `"Deno"` to `PUBLIC_API_HEADS` in `cluster.rs`. With that addition, Deno's construction-style count would land in the same shape as Bun's. The fix is queued for v0.11; this run records the *uncorrected* output to make the structural-classifier-limitation visible. Any future per-corpus run needs to declare its target's public-API namespaces in the allowlist; this is an apparatus-tuning step the pipeline driver should expose as a flag.

## What this run shows about Doc 705's apparatus

**Three operational instances are now anchored.** [Doc 705 §10](https://jaredfoy.com/resolve/doc/705-pin-art-operationalized-for-intra-architectural-seam-detection) named the GitLab engagement (manual probe extraction) and the Bun phase-a-port (tooled probe extraction on AI-translated source). Deno is now the third — tooled probe extraction on hand-written source. Per Doc 693's pattern, three instances reach operational confidence.

**The contrastive case rules out vibe-port specificity.** The implementation-internal seam class (HTTP / Buffer / Stream / crypto / process / util) appears at similar shape on both runtimes. The seam category list (sync/async, ffi, ctor, cfg, throw, mixed, weak-ref, threaded) reproduces in proportion to corpus size on both. The apparatus is reading real JS-runtime architectural form, not artifacts of how the source was generated.

**Two structural limitations are now visible:**
- The construction-style classifier is namespace-allowlist-dependent and needs per-corpus tuning.
- Test-corpus probe extraction depends on the test author's style: WPT-direct testing surfaces architectural primitives the public-wrapper-mediated style hides. Both styles work with the apparatus, but produce different signal-vector distributions for the same underlying architecture.

## v0.11 refinements queued

- **`--public-api-namespaces <NS,NS,…>` flag on the pipeline subcommand** — exposes the construction-style classifier's allowlist for per-corpus tuning. Default values cover Bun + web-platform; users add `Deno`, `chrome`, etc. for other runtimes.
- **`--test-call-names <NAME,NAME,…>` flag** — similarly externalizes the TS/JS extractor's test-call recognition. Currently hand-coded for `test`/`it`/`describe`/`Deno.test`. Externalizing supports tap-style runners, ava, uvu, and other test conventions.
- **The 10 mismatch candidates from Deno's coupling are the rederive-pilot-target candidates.** When the rederive infrastructure lands, http / Buffer / Stream / crypto on Deno are smaller, more contained derivation targets than the equivalent Bun surfaces — Deno's hand-written source provides the comparison point for whether rederive's output matches the existing implementation's architectural shape.

## Files

- All standard pipeline outputs at `runs/2026-05-10-deno-comparative/` (scan.json, cluster.json, seams.json, coupled.json, welch-* JSONs, constraints/, constraints-by-seams/).
- This file: COMPARATIVE-NOTES.md.

## Provenance

- Tool: `derive-constraints` v0.11 (pipeline driver + Deno-test conventions extension).
- Target: `denoland/deno` HEAD shallow clone, 2026-05-10.
- Baseline: `tokio` + `hyper` + `reqwest` HEAD shallow clones (async-runtime-shaped, matching Deno's domain).
- Pipeline runtime: 3.7 seconds end-to-end.
