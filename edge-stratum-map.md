# Edge → Substrate Stratum Map (revision 2)

Pin-Art bilateral reading. Edges cluster around K substrate primitives;
each stratum decomposes into L hierarchical *constraint layers* that
the closure arc must lift in dependency order. The recorded edge is
the topmost-revealed layer; everything above is invisible until you
lift the lower one.

## Constraint hierarchy (L axis)

| Layer | Question | Visible only when … |
|---|---|---|
| L0 parse | Does the engine accept the source? | — |
| L1 load | Does the loader run in bounded time? | L0 passes |
| L2 namespace | Are required APIs present? | L1 passes |
| L3 surface | Do APIs have correct shape? | L2 passes |
| L4 idiom | Do shapes support consumer call patterns? | L3 passes |
| L5 semantics | Do correct ops produce correct bytes? | L4 passes |
| L6 timing | Does scheduling match? | L5 passes |

A single recorded edge is a **stratum × layer** coordinate. The
consumer hits its lowest-failing layer and stops; everything above is
silent. This is why one substrate fix can retire 2–10 consumers
transitively: the consumers were all stalled at the SAME layer of the
SAME stratum, and the fix lifts them all to the next layer (which is
often already closed for the simpler consumers).

## Strata × layers (S6 illustrative)

The S6 (http / fastify) closure arc proceeded in layer order:

| Stratum | L0 | L1 | L2 | L3 | L4 | L5 | L6 |
|---|---|---|---|---|---|---|---|
| **S6 http** | `\-` in /u | preprocessor backtracking | `process.nextTick` | `require('node:url').URL` undefined | ES6 class vs `.call(this)` | (req/res byte parity) | (real socket loop) |
| S1 regex /u | shared with S6 L0 | | | | | | |
| S2 timers | | | shared with S6 L2 | | | | wall-clock fidelity |
| S3 node:* | | | 12-stub gap | | | | |
| S4 exports | | | | conditional-exports recurse | | | |
| S5 server | | | | http.Server two-arg + setTimeout | EE shape + assignSocket | | |

S6 spanned L0–L4 because fastify is a deep consumer of the http
stratum. Most observed strata only manifest at one or two layers
because the consumer set hasn't reached deeper.

## Refined work-to-closure bound

- **K** = substrate strata (engine primitives) ~ 18
- **L** = layers per stratum, bounded ~ 7
- **L̄ exercised** = mean layers a real consumer actually exercises per
  stratum on the in-basin axis, ~ 2–5
- **Total measured fix points to telos** ≈ K × L̄
- Still much smaller than N (recorded edges) because edges collapse on
  TWO axes: many edges → one (stratum, layer); many (stratum, layer)
  → one substrate widening.

## Consumer-depth prediction

For an unprobed consumer, expected fail-layer is predictable from its
idioms:

| Consumer shape | Expected deepest layer |
|---|---|
| pure-data utility (lodash-like) | L2 |
| CJS lib with require chain | L3 |
| framework using Node-style inheritance | L4 |
| framework with custom output formatting | L5 |
| framework with async I/O / real socket | L6 |

This is the canonicalization. A stratum-closure arc is bounded by L;
its depth equals max-layer-exercised. §A8.18's substrate-standing
geometry is exactly the layered nature: each consumer riding the same
stratum lands at zero cost up to the deepest layer the prior consumer
already lifted.

---

(Earlier revision-1 content — flat edge-to-stratum bucketing — was
the same map without the L axis. The L hierarchy is the structural
finding that makes the map predictive instead of descriptive.)
