# derive-constraints seams v0.2 — 2026-05-10 — Bun phase-a-port

Tightening pass on the [v0.1 seams run](./SEAMS-NOTES.md). Implements the four queued v0.2 refinements: extended signal catalog adding S7–S10 to the original Doc 705 §4 six probes.

## Refinements

| Signal | Description | Detection |
|--------|-------------|-----------|
| **S7 — Ownership / reference-cycle** | Detects WeakRef, WeakMap, WeakSet, FinalizationRegistry, structuredClone, `.deref()` | Pattern match in subject + antichain raw text |
| **S8 — Error-shape distinction** | Refines S4 binary throw/return-error into Result / OkErrorsArray / SuccessErrors / PlainThrow / Mixed shapes | Compound-shape detection in raw text |
| **S9 — Allocator-discipline awareness** | Detects arena, bumpalo, slab, MimallocArena, ArrayList, BabyList, MultiArrayList | Pattern match in antichain raw text |
| **S10 — Threading-model awareness** | Detects Worker, MessageChannel, MessagePort, BroadcastChannel, Atomics, SharedArrayBuffer, AsyncLocalStorage | Subject head + antichain pattern match |

## Numerical delta

| Metric | v0.1 | v0.2 | Δ |
|--------|-----:|-----:|---|
| Distinct signal vectors | 63 | 76 | +13 |
| Cross-namespace seams | 32 | 36 | +4 |

## v0.2 new-signal property counts (across all 76 clusters)

```
weak_ref properties:           14
allocator_aware properties:     1
threaded properties:           35
error_shape distribution:
  plain_throw:   14 properties
  success_errors: 2 properties
  result:         2 properties
  ok_errors_array: 4 properties
```

The error_shape distribution is the most operationally consequential single finding: **Bun's runtime surface is overwhelmingly throw-discipline at the test-corpus probe layer.** 14 throw-shaped properties vs 8 compound-shape properties (across Result + SuccessErrors + OkErrorsArray). This matches Bun's Web-platform inheritance (URL throws; JSON.parse throws; fetch rejects); a Rust port should use the throw-equivalent (panic or runtime error) for most surfaces, matching the JS contract, rather than systematically converting to `Result<T, E>`.

## New substantive clusters surfaced by v0.2

```
SC0004  card=1938  v2-signals=err=plain_throw    surfaces=<anonymous>, panic, testing…
SC0007  card=1556  v2-signals=err=plain_throw    surfaces=<anonymous>…
SC0011  card= 511  v2-signals=thr                surfaces=Atomics, BroadcastChannel, MessageChannel, MessagePort, Promise…
SC0020  card= 244  v2-signals=weak               surfaces=releaseWeakRefs, structuredClone, structuredCloneFn
SC0023  card= 182  v2-signals=err=success_errors surfaces=Bun
SC0024  card= 141  v2-signals=err=plain_throw    surfaces=async
SC0027  card= 101  v2-signals=err=plain_throw    surfaces=async
SC0031  card=  77  v2-signals=thr                surfaces=AsyncLocalStorage, Atomics, Worker
```

Two genuinely new architectural seams emerge:

**Threading-model seam (S10).** SC0011 (`signal=thr/@js`, card=511) cleanly isolates Atomics + BroadcastChannel + MessageChannel + MessagePort + Promise — the multi-threaded-coordination surface. SC0031 (`signal=thr`, card=77) covers AsyncLocalStorage + Atomics + Worker — the address-space-sharing surface. **This is a new architectural seam not detected in v0.1.** It is distinct from the sync/async (S3) seam: S3 is execution-discipline; S10 is address-space-sharing.

**Weak-reference / deep-copy seam (S7).** SC0020 (`signal=weak/@js`, card=244) covers releaseWeakRefs + structuredClone + structuredCloneFn. This is the *reference-structure* boundary — properties whose contract requires explicit handling of shared references (deep-copy to break sharing; weak-ref to allow GC). For a Rust port this is structurally significant: Rust's `Clone::clone` is per-type; `structuredClone` is universal-deep-copy across arbitrary object graphs. Derivation must handle this seam explicitly rather than treating it as a single method.

## What v0.2 does not yet detect

**Allocator-discipline (S9) fired on only 1 property.** Bun's tests don't surface arena/bumpalo/slab patterns in expect-bodies. Like the v0.1 P2 (native byte-pool) finding: this is an implementation-internal boundary invisible at the test-corpus probe layer. Refinement requires probing the implementation source, not the test corpus — coupling to welch's per-file diagnostic over `phase-a-port` source might surface it.

**Platform-conditional meta-seam (P4 from Doc 705 §10.2) still localized.** S1's pattern catalog still doesn't catch the platform-conditional setup-code patterns Bun uses (the conditionals live in `beforeEach` / `describe` blocks, not in `expect` clauses). Refinement requires probing the surrounding test-fn body, not just the antichain's raw text. Queued for v0.3.

## Doc 705 §10.2 prediction outcomes (after v0.2)

| Prediction | v0.1 | v0.2 |
|------------|------|------|
| P1 sync/async split on fs/crypto/child_process | ✅ HELD | ✅ HELD (stable) |
| P2 native byte-pool merge | ⚠️ NOT SURFACING | ⚠️ NOT SURFACING (boundary impl-internal) |
| P3 Bun.* split into 4-6 architectural surfaces | ✅ HELD in shape | ✅ HELD in shape |
| P4 platform-conditional meta-seam | ⚠️ PARTIAL | ⚠️ PARTIAL (S1 still narrow) |
| P5 throw vs return-error seam | ✅ HELD | ✅ **HELD with shape** — S8 refines: 14 throw vs 8 compound = throw-dominant |
| P6 construct-then-handle seam | ✅ HELD | ✅ HELD (stable) |

P5 sharpened from "binary held" to "specific shape distribution". The other predictions are stable; v0.2 surfaces additional architectural seams not in the original prediction set:

- **S10 threading-model seam** — Atomics / Worker / MessageChannel / SharedArrayBuffer cluster of 511 + 77 cardinality. Not a Doc 705 §10.2 prediction; a *discovered* seam.
- **S7 weak-reference / deep-copy seam** — structuredClone / WeakRef / FinalizationRegistry cluster of 244 cardinality. Discovered seam.

These are the apparatus operating in its proper mode: predictions check the apparatus's reach against expected outcomes; non-predicted clusters surface architectural forms the keeper's prior decomposition didn't anticipate.

## v0.3 refinements queued

- **Test-fn-body probing** — open antichain representative source files from disk during seam analysis; scan ±20 lines around the cited line for setup conditionals (`beforeEach`, `describe`-scope guards). Fixes P4 platform-conditional partial.
- **Couple to welch's per-file diagnostic** — surface allocator/native-byte-pool patterns by cross-referencing per-file welch unsafe-density anomalies with seams' property antichain files. Implementation-internal seams become visible at the implementation-source layer rather than test-corpus layer.
- **Resistance-as-boundary verification via rederive** — Doc 705 Step 4. The candidate seams are stable; running rederive validation against them is the next concrete move once rederive infrastructure lands.

## Files

- `bun-seams-v2.json` (~800KB) — v0.2 cluster + cross-namespace-seam catalog.
- v0.1 artifacts (`bun-seams.json`, `seams-summary.txt`, `SEAMS-NOTES.md`) preserved alongside for historical comparison.

## Provenance

- Tool: `derive-constraints` v0.4 (seams v0.2).
- Source: `bun-cluster-v2.json` from the cluster v0.2 run on 2026-05-10.
- Tool runtime: well under a second.
