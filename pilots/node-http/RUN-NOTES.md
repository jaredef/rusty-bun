# node-http pilot — 2026-05-10

**Fifteenth pilot. Tier-C #7 from the trajectory queue.** Node's `http`/`https` module — data-layer only. Composes naturally with the existing fetch-api pilot but uses Node's flat-object header representation rather than WHATWG Headers.

## Pipeline

```
v0.13b enriched constraint corpus (http2 partial; node http core via Bun's
  TS implementations at js/node/http.ts + _http_*.ts)
       │
       ▼
AUDIT.md
       │
       ▼
simulated derivation v0   (CD + Node.js docs §http + Bun TS reference shape)
       │
       ▼
derived/src/lib.rs   (208 code-only LOC)
       │
       ▼
cargo test
   verifier:            21 tests
   consumer regression:  8 tests
       │
       ▼
29 pass / 0 fail / 0 skip   ← clean first-run pass
```

## LOC measurement

```
Bun reference (js-side node-compat http TS):
  http.ts                                 71 LOC
  _http_server.ts                      1,925
  _http_outgoing.ts                      588
  _http_incoming.ts                      479
  _http_common.ts                        253
  Total Bun TS http core               3,316 LOC

Plus llhttp parser machinery (out of scope):
  llhttp.c + llhttp.h + api.c          11,945 LOC

Pilot derivation (code-only):              208 LOC
Naive ratio vs Bun TS core:               6.3%
Adjusted (data-layer scope):           ~10-15%
```

The 6.3% naive ratio is consistent with the apparatus' system-pilot range. Pilot covers IncomingMessage + ServerResponse + ClientRequest + Server + NodeHeaders shape; Bun's TS source includes streaming-body integration with node:streams, full HTTP-state-machine wiring, error-recovery paths.

## Findings

1. **AOT #1 confirmed.** 208 LOC, in predicted 150-250 range.
2. **AOT #2 confirmed.** First-run clean closure. **Four consecutive pilots first-run clean** (Bun.serve, Bun.spawn, node-fs, node-http).
3. **AOT #3 confirmed.** Body-as-Vec<u8> simplification works for the data-layer; documented in AUDIT as the scope choice. Real Node uses streaming Readable/Writable; pilot accumulates.

## Updated 15-pilot table

| Pilot | Class | LOC | Adj. ratio |
|---|---|---:|---:|
| (...prior 14...) | | 2,868 | (aggregate ~5-7%) |
| **node-http** | **Tier-2 Node-compat http data-layer** | **208** | **6.3% naive / ~10-15% adj** |

Fifteen-pilot aggregate: **3,076 LOC** of derived Rust against ~102,000+ LOC of upstream reference targets. **Aggregate naive ratio: ~3.0%.**

## Trajectory advance

Tier-C #7 done. Next queued: **Tier-C #8 — crypto.subtle pilot**.

## Files

```
pilots/node-http/
├── AUDIT.md
├── RUN-NOTES.md
└── derived/
    ├── Cargo.toml
    ├── src/lib.rs            (286 LOC, 208 code-only)
    └── tests/{verifier.rs, consumer_regression.rs}
```
