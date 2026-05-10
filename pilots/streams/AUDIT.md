# streams pilot — coverage audit

**Ninth pilot. First Tier-A substrate pilot from the trajectory.** Streams unblock several queued items: full fetch body integration, Worker postMessage, Blob.stream(), Response/Request body streaming. Per the Resume Vector trajectory §II.

## Constraint inputs

| Surface | CD properties | CD clauses | Spec extract clauses |
|---|---:|---:|---:|
| ReadableStream | 1 | 1 | 28 |
| WritableStream | (sparse) | ~1 | 11 |
| TransformStream | (sparse) | ~1 | 5 |
| **Total** | | ~3 | **44 (newly curated for this pilot)** |

The constraint corpus from Bun's tests is sparse on direct-attribution to ReadableStream (1 clause). This is expected: stream tests typically construct via `const stream = new ReadableStream(...)` then operate on `stream`. The v0.12 binding-substitution-fix correctly avoids re-attributing `stream.cancel(...)` back to ReadableStream when methods are called.

**The pilot is therefore primarily spec-driven.** The newly-curated `specs/streams.spec.md` (44 clauses across the three surfaces) carries most of the constraint material. This is the **first pilot where the spec-extract layer dominates over the test-corpus layer**, validating Doc 707's claim that the spec layer is the *ceiling* on what the apparatus can constrain.

## Pilot scope

Three composed surfaces in a single crate:

### ReadableStream<T>
- Constructor with `UnderlyingSource<T>` providing start/pull/cancel callbacks
- `.locked()` — has a reader been acquired
- `.cancel(reason)` — cancel + propagate to source
- `.get_reader()` → `Reader<T>` — locks the stream
- `.tee()` → `(ReadableStream<T>, ReadableStream<T>)` — split + lock original
- States: readable, closed, errored
- Internal queue + high-water-mark; pull called when queue drops below

### Reader<T>
- `.read()` → `ReadResult<T>` (an enum: `Chunk(T)`, `Done`, `Pending`, `Error(...)`)
- `.cancel(reason)` — cancel underlying + release lock
- `.release_lock()` — unlocks; subsequent reads error

### Controller<T> (passed to source callbacks)
- `.enqueue(chunk)` — add to queue; errors if closed/errored
- `.close()` — transition to closed
- `.error(reason)` — transition to errored

### WritableStream<T>
- Constructor with `UnderlyingSink<T>` providing start/write/close/abort callbacks
- `.locked()`, `.abort(reason)`, `.close()`, `.get_writer()` → `Writer<T>`

### Writer<T>
- `.write(chunk)` — feed to sink
- `.close()`, `.abort(reason)`, `.release_lock()`

### TransformStream<I, O>
- Constructor with `Transformer<I, O>` providing start/transform/flush callbacks
- `.readable()` → reference to internal ReadableStream
- `.writable()` → reference to internal WritableStream

## Out of pilot scope

- **Async / Promise model.** Pure-Rust analog uses synchronous poll-based reads. `read()` returns `ReadResult` (an enum with `Chunk | Done | Pending | Error`), not `Future<Result<...>>`. AOT: the apparatus' value claim doesn't depend on async-runtime fidelity; the data-layer claim suffices.
- ReadableByteStreamController (BYOB reads — bring-your-own-buffer)
- Async iterator protocol (`Symbol.asyncIterator`)
- Transferable streams (cross-realm)
- pipeTo / pipeThrough automation (manual loops in the verifier)

## Approach: shared state via Rc<RefCell>, callbacks as trait objects

Like the AbortController pilot, streams need shared state between the stream (held by the consumer) and the controller (passed to the source's start/pull callbacks). The pattern is the same: `Rc<RefCell<StreamInner<T>>>` shared across the stream-handle, the reader-handle, and the controller-handle.

Source/sink/transformer callbacks are stored as `Box<dyn FnOnce(...)>` or `Box<dyn FnMut(...)>` depending on whether they fire once or multiple times.

## Ahead-of-time hypotheses

1. **The pilot will be the largest derivation in the apparatus** outside fetch-api, given three composed surfaces. Estimated 350-500 LOC.
2. **Verifier-caught derivation bug expected** (first since Pilot 4): streams have many invariants — backpressure semantics, lock release timing, error propagation order — and the spec is dense. AOT prediction: at least one bug surfaces.
3. **Tee semantics is the most likely bug site.** Spec mandates: tee'd streams emit the same chunks in the same order; cancellation of one branch propagates to the source only when both branches cancel. Easy to get wrong.
4. **The synchronous-poll model will deviate from JS in observable ways.** Pilot will document where the synchronous derivation differs from a real async stream consumer's expectations. AOT: at least one consumer-regression test will need a documented skip for "requires async runtime."

## Verifier strategy

~30-40 verifier tests across the three surfaces:
- ReadableStream: enqueue / read / close / error / tee / cancel / lock
- WritableStream: write / close / abort / lock
- TransformStream: transform pipeline / paired read+write
- State transitions: readable → closed; readable → errored; locked invariants

Consumer regression: 8-10 tests citing real consumers (undici body streaming, Blob.stream, Worker postMessage transfer expectations).
