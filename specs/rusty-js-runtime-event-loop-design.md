# rusty-js-runtime — Event Loop Extension Design

[surface] JobQueue + macrotask queue + run-loop driver inside the engine
[reference] ECMA-262 §9.4 (Jobs and Realms), §9.5 (HostEnqueuePromiseJob), §9.6 (HostMakeJobCallback), §9.7 (HostCallJobCallback); WHATWG HTML §8 (event-loop algorithm); QuickJS-NG / Bun / Node-libuv as architectural references
[engagement role] Round Ω.3.f of the rusty-js-runtime pilot. Implements [Doc 714 §VI Consequence 5](/resolve/doc/714-the-rusty-bun-engagement-read-through-the-lattice-extension-basin-expansion-at-the-l2m-saturation-point#consequence-5--the-event-loop-belongs-inside-the-engine-amendment-2026-05-14): the event loop attaches at E5 (engine-realm host-defined behavior) per [Doc 717](/resolve/doc/717-the-apparatus-above-the-engine-boundary-the-three-projections-lifted-to-engine-substrate-and-the-pure-abstraction-point), not as a polyfill split across the engine boundary.

## I. The cut-rung error this design fixes

The current rusty-bun-host architecture (pre-Tier-Ω) places the event loop on the wrong side of the engine boundary:

- **JS side:** `globalThis.__keepAlive` Set + `globalThis.__tickKeepAlive` function + `globalThis.__keepAliveUnref` Set. These hold Promise-like work items the host drives until quiescent.
- **Rust side (rusty-bun-host):** mio reactor with per-thread Poll + per-fd-class tokens (TCP=0x00, TLS=0x40, signalfd=0x50, DNS=0x55, inotify=0x60, spawn=0x70). The reactor's wake events route through host-side Rust code, which in turn calls into JS via FFI.
- **Cross-boundary glue:** the bytecode driver in `eval_esm_module` and `eval_string_async` interleaves microtask drains with `__tickKeepAlive` ticks and mio's `Poll::poll` at idle.

Per [Doc 717 §V](/resolve/doc/717-the-apparatus-above-the-engine-boundary-the-three-projections-lifted-to-engine-substrate-and-the-pure-abstraction-point#v-the-lattice-projection-at-engine-scale), ECMA-262's run-loop machinery (HostEnqueuePromiseJob, HostMakeJobCallback, HostCallJobCallback, HostEnqueueGenericJob) sits at the **E5 cut rung** (realm-level host-defined behavior). The current architecture **smears that cut across the embedding boundary** — half in Rust, half in JS, glued at the FFI line. The libuv/Bun pattern names the right split: the engine owns the run-loop discipline (microtask drain order, Promise settlement, ready-event dispatch); the host supplies OS I/O via callback registration.

## II. The design

The engine gains an explicit event-loop API. Three structural pieces:

### Piece 1 — JobQueue inside Runtime

```rust
pub struct Runtime {
    // existing fields
    pub job_queue: JobQueue,
}

pub struct JobQueue {
    /// Microtasks (Promise reactions per spec §27.2). FIFO.
    microtasks: VecDeque<Job>,
    /// Macrotasks (timer firings, ready I/O completions, etc.).
    /// Each entry is processed at "the next macrotask phase" per HTML's
    /// event-loop algorithm.
    macrotasks: VecDeque<Job>,
    /// Pending Promise jobs from the host's EnqueuePromiseJob hook.
    /// Spec §9.5: realm-scoped queues.
    promise_jobs: VecDeque<Job>,
}

pub enum Job {
    /// Closure to invoke at job time. Captures whatever state the
    /// enqueuer chose to capture.
    Closure(Box<dyn FnOnce(&mut Runtime) -> Result<Value, RuntimeError>>),
    /// A NativeFn + args bundle. Encoded distinctly so the GC's root
    /// enumeration can see through to held Values.
    Native { f: NativeFn, this: Value, args: Vec<Value> },
}
```

### Piece 2 — Host-hook extensions for I/O sources

The existing `HostHook::FinalizeModuleNamespace` (3.d.f) is joined by:

```rust
pub enum HostHook {
    FinalizeModuleNamespace(...),
    /// Host registers a per-fd callback for read-readiness. The engine
    /// invokes the callback (which enqueues a macrotask) when the host
    /// signals readiness from its OS-I/O multiplexer (mio Poll, libuv,
    /// io_uring, etc.).
    WatchReadable(Box<dyn Fn(&mut Runtime, RawFd, Box<dyn FnOnce(&mut Runtime)>) -> Result<(), RuntimeError>>),
    WatchWritable(...),
    /// Host signals "no fd I/O pending; block until next deadline".
    /// Engine calls this at idle.
    PollIo(Box<dyn Fn(&mut Runtime, Option<Duration>) -> Result<(), RuntimeError>>),
    /// Host's time source — when the engine schedules a timer, the host
    /// arranges for the engine to be re-entered after `ms` elapse.
    Timer(Box<dyn Fn(&mut Runtime, u64 /* ms */, Box<dyn FnOnce(&mut Runtime)>) -> Result<(), RuntimeError>>),
}
```

The host supplies its OS-I/O multiplexer via these hooks. The engine never imports mio; the host registers callbacks that the engine invokes.

### Piece 3 — Run-loop driver

```rust
impl Runtime {
    /// HTML event-loop algorithm: run microtasks to quiescence, then
    /// advance one macrotask phase, then loop. Exits when both queues
    /// are empty AND the host's PollIo signals no work pending.
    pub fn run_to_completion(&mut self) -> Result<(), RuntimeError> {
        loop {
            // Phase 1: drain microtasks per spec §9.4.1
            while let Some(job) = self.job_queue.microtasks.pop_front() {
                self.run_job(job)?;
            }
            // Phase 2: advance one macrotask
            if let Some(job) = self.job_queue.macrotasks.pop_front() {
                self.run_job(job)?;
                continue;
            }
            // Phase 3: idle — consult host PollIo for the next deadline.
            // If no deadline + no pending I/O, exit.
            if !self.poll_io_or_exit()? { return Ok(()); }
        }
    }
}
```

`run_to_completion` is the engine's HTML-spec-faithful event-loop driver. It replaces:
- the bytecode driver's `loop { drain microtasks; tick __keepAlive; poll mio }` in rusty-bun-host
- the JS-side `__tickKeepAlive` function
- the JS-side `__keepAlive` Set

All three retire when the host migrates to register I/O sources via `WatchReadable` / `WatchWritable` / `Timer` and call `Runtime::run_to_completion`.

## III. Spec correspondence

Per ECMA-262 §9 + HTML §8:

- **Microtask queue** = ECMA-262 §9.4.1 Promise jobs queue, populated by `HostEnqueuePromiseJob` (which the engine implements as a default that pushes to `job_queue.microtasks`; the host may override per §9.5)
- **Macrotask queue** = HTML's "task source" queues. The engine's macrotask queue is the union of all task sources; the host's I/O readiness callbacks push entries here.
- **Run-to-completion** = HTML's event-loop processing model

The engine implements the spec defaults; the host overrides per the hook API where it has reason to (e.g., redirecting microtask jobs to a custom scheduler for testing).

## IV. Migration of existing reactor work

The nine reactor sub-rounds landed 2026-05-13 (Π2.6.c.a-e + Π2.6.d.a-d) constructed substrate on the wrong side of the cut rung. Their re-location post-Ω.4:

| Pre-Ω substrate | Post-Ω home |
|---|---|
| `host/src/reactor.rs` — mio Poll + thread-local registry | rusty-bun-host installs as `WatchReadable` / `WatchWritable` host hooks |
| `host/src/watchers.rs` — inotify | Same — register inotify fds via WatchReadable |
| `host/src/spawn_async.rs` — child-pipe registry | Same |
| `host/src/signals.rs` — signalfd | Same |
| `host/src/dns_async.rs` — eventfd + worker thread | Same |
| `globalThis.__reactor.register/deregister/...` (JS) | Retired — engine owns the analogous registry internally |
| `globalThis.__keepAlive` Set (JS) | Retired — engine's job_queue replaces it |
| `globalThis.__tickKeepAlive` function (JS) | Retired — engine's run_to_completion replaces it |
| `bytecode driver eval_esm_module` cooperative-yield loop | Retired — engine's run_to_completion is the loop |

Estimated rusty-bun-host LOC delta post-migration: **-1,500 to -2,000 LOC** (the nine reactor sub-rounds' product collapses into a thin registration layer).

## V. Doc 717 closure cross-reference

The event loop attaches at the same E5 rung as the Tuple-A/B closure (HostFinalizeModuleNamespace, already in 3.d.f). Both are realm-level host-defined behaviors per ECMA-262 §16.1. The engine's E5 surface is now:

- `HostFinalizeModuleNamespace` (3.d.f) — Tuple A + B
- `HostHook::WatchReadable / WatchWritable / Timer / PollIo` (this round)
- `HostHook::EnqueueMicrotask` (optional override per spec §9.5)

The engine's E5 cuts are consolidated. The architecture matches Doc 717 §V's pattern: each host-overridable behavior is a named hook; the engine implements a spec-conformant default; the host overrides per its embedding context.

## VI. Job-queue invariants

Per spec §9.4.1:
- **Microtasks run to quiescence between macrotasks.** A microtask enqueued during a microtask run is processed in the same drain.
- **Each macrotask is run "to completion" before the next is dequeued.** A macrotask may enqueue microtasks; those drain before the next macrotask.
- **Promise reactions are microtasks.** Native Promise.then / .catch / .finally enqueue via HostEnqueuePromiseJob.
- **Timers are macrotasks.** `setTimeout(fn, ms)` enqueues a macrotask after the timer fires.
- **I/O readiness is macrotasks.** A ready fd's registered callback runs as a macrotask.

The engine enforces these invariants in `run_to_completion`'s phase ordering.

## VII. Out of scope for round Ω.3.f.b

- **AbortController integration.** Spec §27.2.2.5 ties abort signals to microtask scheduling. v1 of the JobQueue handles it via straightforward job-cancellation (set a flag, drop the job at dequeue). Full AbortSignal AST + reactivity is a follow-on.
- **Generator / async-function suspension.** Per the runtime design spec §IX, generator/async suspension is v2 work. The JobQueue's macrotask processor schedules already-resolved async-function bodies but doesn't support yielding mid-execution.
- **MessagePort / structured-clone scheduling.** Per design spec §IX out-of-scope.
- **Worker threads.** Out of v1 engine scope per design spec §IX.
- **Atomics.wait / SharedArrayBuffer.** Multi-threaded; out of v1 scope.

## VIII. Sub-round roadmap

- **Ω.3.f.a** — substrate-introduction: design spec (this document) + AUDIT.md
- **Ω.3.f.b** — JobQueue struct + microtask + macrotask queues + run_to_completion skeleton + tests against ECMA-262 §9.4.1 ordering invariants
- **Ω.3.f.c** — HostHook extensions: WatchReadable / WatchWritable / Timer / PollIo; default no-op implementations; test suite covering a synthetic host that wires the hooks to a hand-driven event source
- **Ω.3.f.d** — Promise integration: the engine's existing Promise intrinsic (deferred until this round) routes through HostEnqueuePromiseJob; .then / .catch / .finally callbacks run via the microtask queue

## IX. Falsifier

Per Doc 714 §VI Consequence 5: post-Ω.3.f + Ω.4, the host LOC delta required to reach each parity-percentage point should be smaller than the pre-recognition trajectory's. The 2026-05-13 night baseline (88.2%, post-`3f9673ab`) is the anchor. If post-migration the LOC-per-parity-point ratio is higher, the architectural shift didn't deliver the predicted simplification — either the cut-rung diagnosis is wrong (E5 isn't where the run-loop should sit) or the migration's mechanical cost dominated the architectural simplification.

## X. Closing

The event loop is the engine's, not a host polyfill. The shift retires the nine reactor sub-rounds from the host side and consolidates the E5 cut at one rung. The pilot's first round (Ω.3.f.b) builds the JobQueue + run_to_completion; subsequent rounds wire the host-hook API and route the existing Promise intrinsic through it.
