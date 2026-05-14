# rusty-js-runtime — Event Loop Extension Audit

**Round Ω.3.f.a of the rusty-js-runtime pilot.** Substrate-introduction sub-round for the JobQueue + run-loop extension. Per [the design spec](../../specs/rusty-js-runtime-event-loop-design.md) + [Doc 714 §VI Consequence 5](/resolve/doc/714-the-rusty-bun-engagement-read-through-the-lattice-extension-basin-expansion-at-the-l2m-saturation-point#consequence-5--the-event-loop-belongs-inside-the-engine-amendment-2026-05-14).

## Engagement role

Brings the event loop inside the engine. Replaces the current architecture's split (mio + JS-side `__keepAlive` + `__tickKeepAlive` polyfill across the rquickjs FFI boundary) with a single E5-rung run-loop owned by rusty-js-runtime. The host (post-Ω.4 rusty-bun-host) supplies OS I/O via host hooks rather than as a parallel substrate.

The architectural shift is named at the corpus tier in Doc 714 §VI Consequence 5 (amendment 2026-05-14). This pilot is the corresponding substrate work.

## Pilot scope (round Ω.3.f)

Four composed surfaces, distributed across sub-rounds:

### JobQueue (round Ω.3.f.b)
- `Job` enum: Closure-style and NativeFn-style entries
- `JobQueue` with microtasks / macrotasks / promise_jobs FIFOs
- `Runtime::enqueue_microtask` / `enqueue_macrotask` API
- Ordering invariants per ECMA-262 §9.4.1 + HTML §8

### Run-loop driver (round Ω.3.f.b)
- `Runtime::run_to_completion` — HTML event-loop algorithm
- Phase 1: drain microtasks to quiescence
- Phase 2: advance one macrotask
- Phase 3: consult host PollIo for next deadline; exit if quiescent
- Replaces rusty-bun-host's `eval_esm_module` cooperative-yield loop

### Host hooks for I/O sources (round Ω.3.f.c)
- `HostHook::WatchReadable(fd, cb)` — host wires its OS-I/O multiplexer
- `HostHook::WatchWritable(fd, cb)` — same for writes
- `HostHook::Timer(ms, cb)` — host wires its time source
- `HostHook::PollIo(deadline)` — host blocks waiting for fd readiness
- `HostHook::EnqueueMicrotask(fn)` — optional override per §9.5

Default no-op implementations mean the engine is fully functional without a host (useful for testing); the host supplies hooks for production deployment.

### Promise integration (round Ω.3.f.d)
- Promise intrinsic (deferred from 3.d.e)
- `.then` / `.catch` / `.finally` route reactions through the microtask queue
- `Promise.resolve` / `Promise.reject` / `Promise.all` / `Promise.race` minimum surface

## Out of scope for the round

Per design spec §VII:
- AbortController spec compliance (basic abort-flag handling only)
- Generator/async-function suspension (per runtime design spec §IX)
- MessagePort / structured-clone scheduling
- Worker threads
- Atomics.wait / SharedArrayBuffer

## Test corpus

Three layers:

1. **JobQueue ordering invariants** — golden tests per ECMA-262 §9.4.1: microtasks drain between macrotasks; microtasks enqueued during drain process in same phase; macrotasks process one-at-a-time with microtask drain between.

2. **Run-loop driver** — synthetic host installing WatchReadable hooks against a hand-driven event source; verifying the engine drives to completion correctly under various readiness patterns.

3. **Promise round-trip** — `Promise.resolve(42).then(x => x * 2).then(x => log(x))` produces 84 via the engine's job queue; verifies HostEnqueuePromiseJob integration.

## First-round scope (this commit — Ω.3.f.a)

Substrate-introduction only:
- specs/rusty-js-runtime-event-loop-design.md
- pilots/rusty-js-runtime/derived/EVENT-LOOP-AUDIT.md (this file)

No Rust code yet. Next round (Ω.3.f.b) lands the JobQueue + run_to_completion skeleton.

## Estimated pilot size

Per design spec §IV migration estimate:
- JobQueue + Job enum: ~200 LOC
- Run-loop driver: ~150 LOC
- Host-hook extensions + defaults: ~200 LOC
- Promise intrinsic + reaction routing: ~400 LOC
- Tests: ~600 LOC

Total ~1,550 LOC across 3-4 sub-rounds.

Post-migration: rusty-bun-host's reactor work product (Π2.6.c.a-e + Π2.6.d.a-d, ~1,500-2,000 LOC) collapses into a thin registration layer over the host-hook API. Net engine + host LOC change: **negative** (the engine adds ~1,550 LOC; the host sheds ~1,500-2,000 LOC; the engagement's total Tier-Ω.4 work product shrinks substantially).

## Doc 717 cross-reference

The event loop attaches at the **E5 rung** (realm-level host-defined behavior) per design spec §V. Same rung as Tuple-A/B's HostFinalizeModuleNamespace (already in 3.d.f). The engine's E5 cuts are consolidated; the architecture matches Doc 717 §V's pattern.

## Doc 714 §VI Consequence 5 cross-reference

This pilot is the substrate corresponding to Consequence 5's architectural recognition. The falsifier in Consequence 5 — "post-Ω.3.f + Ω.4, host LOC delta per parity-point should decrease" — is the operational test of the cut-rung diagnosis.
