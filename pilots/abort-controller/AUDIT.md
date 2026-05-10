# AbortController + AbortSignal pilot — coverage audit

Sixth pilot. **Event/observable class** — first pilot in this class. Pilots 1-5 covered data structures, delegation, algorithm, composition substrate, and inheritance. AbortController/AbortSignal exercise a different pattern: a controller-and-signal pair where mutation on one triggers callbacks on the other, with shared state across two value-typed handles.

## Constraint inputs

From `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/`:

| Surface | Property | Cardinality | Class | Source |
|---|---|---:|---|---|
| AbortController | ABOR1 | 19 | construction-style | tests + spec extract |
| AbortController | ABOR2 | 1 | construction-style | tests |
| AbortController | ABOR3 | 1 | construction-style | spec |
| AbortSignal | ABOR1 (.abort) | 4 | construction-style | tests + spec extract |
| AbortSignal | ABOR2 (.any) | 3 | construction-style | tests + spec extract |
| AbortSignal | ABOR3 (.timeout) | 2 | behavioral | spec |
| AbortSignal | + 4 other props | 1-2 | construction-style | spec |

Cross-corroborated tier (Bun + spec): AbortController, AbortSignal.abort, AbortSignal.any.

Antichain reps drawn from real Bun tests:
- `expect(typeof AbortController !== "undefined").toBe(true)` — global existence
- `expect(ac.signal.aborted).toBe(true)` — abort-state propagation through controller
- `expect(ac.signal.reason).toBeInstanceOf(DOMException)` — default reason class
- `expect(ac.signal.reason.code).toBe(20)` — DOMException AbortError code is 20
- `assertEquals(signal.aborted, true)` after `AbortSignal.abort()` — Deno test
- `expect(signal.aborted).toBe(true)` after `AbortSignal.any([s1])` with one already aborted

## Pilot scope

Implement AbortController + AbortSignal per DOM §3.3:
- `AbortController::new()` — non-aborted signal
- `controller.signal()` getter — returns `&AbortSignal` (shared state with controller)
- `controller.abort(reason?)` — aborts the signal; idempotent
- `AbortSignal::abort(reason?)` static — already-aborted signal
- `AbortSignal::any(signals)` — combinator; aborts when any input aborts
- `AbortSignal::timeout(ms)` — out of scope (requires platform timer)
- `signal.aborted()`, `signal.reason()`, `signal.throw_if_aborted()`
- Listener registration via `signal.add_event_listener(callback)` — fires on abort transition

Out of pilot scope:
- Timer integration (timeout)
- Event-target full machinery (the spec's EventTarget surface — pilot models only the abort event)
- DOMException as a full type — pilot's reason is `Reason::DomExceptionAbortError` enum variant or arbitrary user-supplied value

## Approach: shared state via Rc<RefCell>

The controller-and-signal pair share state. JS does this via the controller holding a private reference to the signal it created and mutating its internal slot. Rust analog: `Rc<RefCell<SignalInner>>` shared between AbortController and AbortSignal.

This is the **first pilot to need interior mutability + shared ownership**. The pilot tests whether the apparatus's value claim survives the more-complex Rust pattern. Rc<RefCell> is straightforward Rust — not borrow-checker-hostile in the way naive object-graph cloning is — but it's a different idiom than the prior pilots' value-type composition.

## Listener model

AbortSignal supports `addEventListener("abort", callback)`. Pilot models this as `signal.on_abort(F)` where `F: FnOnce(&Reason)`. Listeners stored in `Vec<Box<dyn FnOnce(&Reason)>>`. On abort, all listeners fire and are drained. `FnOnce` not `FnMut` because the spec says listeners fire exactly once (the signal's abort transition is one-shot).

## Ahead-of-time hypotheses

1. **Rc<RefCell<>> shared-state pattern is the smallest deviation from prior pilots' pure value semantics.** No `Arc`, no async, no `Sync` requirements. Pilot is single-threaded.
2. **AbortSignal::any composes cleanly via listener registration.** Each input signal registers a listener that aborts the result signal; if any input is already aborted at construction time, the result is constructed already-aborted. No cycle issues because listeners drop after firing.
3. **`signal.reason` defaults to a DOMException-AbortError analog when none provided.** Pilot models this with a `Reason` enum: `Reason::AbortError` (default DOMException) or `Reason::Custom(String)`. Avoids needing a full DOMException implementation.
4. **First-run clean closure expected.** Spec is concrete; constraint corpus is rich (cross-corroborated on three properties); no semantic ambiguity comparable to Pilot 4's slice-swap. AOT prediction: 100% pass on first run, no fixes.

## Verifier strategy

CD antichain reps + spec extracts. Target ~25 tests. Pilot succeeds if 100% pass with 0 documented skips. The interesting test cases:
- Listener fires exactly once on abort, never on subsequent abort calls (idempotency)
- AbortSignal.any with one already-aborted input returns already-aborted signal
- AbortSignal.any with no aborted inputs aborts when any input later aborts
- throwIfAborted is no-op when not aborted, throws the reason when aborted
- controller.abort() is idempotent — repeated calls don't re-fire listeners or change the reason
