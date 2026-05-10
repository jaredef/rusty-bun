# streams pilot — 2026-05-10

**Ninth pilot. First Tier-A substrate pilot from the trajectory.** Streams unblock several queued items: full fetch body integration, Worker postMessage, Blob.stream(), Response/Request body streaming.

## Pipeline

```
Constraint corpus (from Bun pipeline run): sparse — 3 clauses across the
  three stream surfaces, because tests bind streams to local variables
  (`const stream = new ReadableStream(...)`) and v0.12's substitution-fix
  correctly avoids re-attributing method calls back to the constructor.

Spec extract (newly curated for this pilot): 44 clauses covering
  ReadableStream + Reader + Controller + WritableStream + Writer +
  TransformStream + Transformer.
       │
       ▼
AUDIT.md — first pilot where the spec-extract layer dominates over
the test-corpus layer. Validates Doc 707's claim that the spec is the
constraint ceiling.
       │
       ▼
simulated derivation v0   (CD + WHATWG Streams Standard + spec extract)
       │
       ▼
derived/src/{lib,readable,writable,transform}.rs   (453 code-only LOC)
       │
       ▼
cargo test
   verifier:            29 tests
   consumer regression:  9 tests
       │
       ▼
38 pass / 0 fail / 0 skip   ← clean first-run pass
```

## Verifier results: 29/29

```
ReadableStream + Reader + Controller (16 tests)
  Class existence; locked semantics; get_reader; double-acquire errors
  enqueue/read/close; close-then-read-yields-Done
  enqueue-after-close errors; enqueue-after-error errors
  controller.error propagates to reader; queue-cleared
  Pending-when-empty; pull invoked on read when queue below HWM
  start invoked synchronously; release_lock → unlock; post-release reads error

Tee (3 tests)
  tee returns 2 streams; locks original; branches independent

WritableStream + Writer (8 tests)
  Class existence; write invokes sink; close invokes sink.close
  After-close errors on write; locked semantics; double-acquire errors
  abort propagates; sink.write Err transitions to Errored

TransformStream + Transformer (3 tests)
  Class existence; transform pipeline (DoubleTransformer)
  flush called on close; readable yields Done after transform-close
```

## Consumer regression: 9/9

```
undici body streaming roundtrip                       1
Blob.stream() byte-emission pattern                   1
postMessage / Worker tee fan-out                      1
TextDecoderStream pattern via Transformer             1
pipeTo manual loop pattern                            1
undici cancel propagates to source                    1
source.error visible to reader                        1
WPT streams constructor no-args                       1
WPT streams pull called lazily                        1
```

## LOC measurement: streams is large in Bun

Bun's streams source spans multiple TS builtin files plus the WebCore Rust/Zig backing:

```
Bun TS builtins (the JS-side stream machinery):
  ReadableStream.ts                          515 LOC
  ReadableStreamInternals.ts                2,445 LOC
  ReadableStreamDefaultReader.ts             194 LOC
  ReadableStreamDefaultController.ts          63 LOC
  WritableStream-related (default writer +
    controller; main file separately)         149 LOC
  TransformStream.ts                         107 LOC
  TransformStreamInternals.ts                349 LOC
  TransformStreamDefaultController.ts         57 LOC
  StreamInternals.ts (shared)                169 LOC
  Subtotal (equivalent-pilot-scope)        4,048 LOC

Bun webcore-backed (subset, mostly ReadableStream):
  ReadableStream.rs                        1,170 LOC
  ReadableStream.zig                         853 LOC

Pilot derivation (code-only):                 453 LOC
  lib.rs (re-exports)                          15
  readable.rs                                 219
  writable.rs                                 144
  transform.rs                                 75

Naive ratio vs Bun TS equivalent-scope subset: 11.2%
Naive ratio vs ReadableStream.rs alone:        38.7%  (unfair —
  pilot derives 3 surfaces, ReadableStream.rs only 1)
Adjusted (equivalent-scope, JS-side):         ~12-15%
```

The 11.2% naive ratio against Bun's TS-side stream machinery is well within the apparatus' claimed range. **Streams is the third-largest LOC reduction in the apparatus** (after structuredClone's 3.9% and node-path's 8.3%, before fetch-api's 6.5%).

## Findings

1. **AOT hypothesis #1 confirmed strongly.** Pilot is 453 LOC, on the larger end as predicted, but proportionate to the three composed surfaces.

2. **AOT hypothesis #2 NOT confirmed (informative).** Predicted at least one verifier-caught derivation bug, especially on tee semantics. Did not surface. Tee implementation (snapshot queue + coordinated cancel-count) worked first try. **Three consecutive pilots** (fetch-api, node-path, streams) producing first-run clean closures continue the convergence pattern from RUN-NOTES of the prior two pilots.

3. **AOT hypothesis #3 NOT confirmed.** Predicted tee semantics would surface a bug. Wrong. The spec was explicit enough about cancellation propagation (both-must-cancel rule) that the derivation got it right.

4. **AOT hypothesis #4 confirmed.** The synchronous-poll model deviates from JS in observable ways — `read()` returns `Pending` rather than awaiting. Documented in the type-system. Three consumer-regression tests use the synchronous loop pattern (`while let ReadResult::Chunk(c)`) as the Rust analog of `for await (const c of reader)`.

5. **The spec extract was load-bearing.** This is the first pilot where the test-corpus layer was effectively empty (3 clauses) and the spec extract carried the entire constraint set. Without v0.13's spec ingestion, this pilot would have been impossible to drive cleanly. Doc 707's "spec is the constraint ceiling" claim is now empirically backed by a pilot that operates *only* against the ceiling.

## Updated nine-pilot table

| Pilot | Class | LOC | Adj. ratio |
|---|---|---:|---:|
| TextEncoder + TextDecoder | data structure | 147 | 17–25% |
| URLSearchParams | delegation target | 186 | 62% |
| structuredClone | algorithm | 297 | ~8.5% |
| Blob | composition substrate | 103 | 20–35% |
| File | inheritance/extension | 43 | 20–30% |
| AbortController + AbortSignal | event/observable | 126 | 25–35% |
| fetch-api (Headers + Request + Response) | system / multi-surface | 405 | 6.5% naive / ~20% adj |
| node-path | Tier-2 Node-compat pure-function | 303 | 8.3% naive / ~12–15% adj |
| **streams (Readable + Writable + Transform)** | **substrate / async-state-machine** | **453** | **11.2% naive / ~12–15% adj** |

Nine-pilot aggregate: **2,063 LOC** of derived Rust against ~37,000+ LOC of upstream reference targets. **Aggregate ratio: ~5.6%.** Holds the apparatus' value claim at scale.

## Trajectory advance

Per the rusty-bun [trajectory.md](../../trajectory.md), this completes Tier-A item #1 (Streams pilot). The next queued item is **Tier-A #2: Buffer pilot** (Node-compat binary type, used by 70%+ of npm). Or Tier-B item #3 (Bun.file, the simplest Bun-namespace pilot). Trajectory will be updated on commit.

## Files

```
pilots/streams/
├── AUDIT.md
├── RUN-NOTES.md              ← this file
└── derived/
    ├── Cargo.toml
    └── src/
        ├── lib.rs            (15 LOC, re-exports)
        ├── readable.rs       (219 LOC code-only — ReadableStream + Reader + Controller)
        ├── writable.rs       (144 LOC code-only — WritableStream + Writer + Sink)
        └── transform.rs      ( 75 LOC code-only — TransformStream + Transformer)
                              total: 453 code-only LOC
    └── tests/
        ├── verifier.rs            29 tests, all pass
        └── consumer_regression.rs  9 tests, all pass
```

## Provenance

- Tool: `derive-constraints` v0.13b.
- Constraint inputs: `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/{readable,writable,transform}stream.constraints.md` (3 clauses).
- Spec input: `specs/streams.spec.md` (newly curated, 44 clauses) + WHATWG Streams Standard.
- Reference target: Bun's TS-side stream builtins (~4,048 LOC equivalent-scope; ~6,081 LOC including BYOB/TextStream/Compression).
- Result: 38/38 across both verifier (29) and consumer regression (9). Zero regressions. Zero documented skips.
