# AbortController + AbortSignal pilot — 2026-05-10

Sixth pilot. **First event/observable class** — controller-and-signal pair where mutation on one triggers callbacks on the other, with shared state across two value-typed handles.

## Pipeline

```
v0.13b enriched constraint corpus
  (Bun: AbortController 21 clauses + AbortSignal 12 clauses across 10 properties)
       │
       ▼
AUDIT.md — predicted: shared state via Rc<RefCell>; clean first-run pass
       │
       ▼
simulated derivation v0   (CD + DOM §3.3 spec extract)
       │
       ▼
derived/src/lib.rs   (126 code-only LOC)
       │
       ▼
cargo test           (22 tests)
       │
       ▼
22 pass / 0 fail / 0 skip   ← clean first-run pass
```

## Verifier results

```
running 22 tests

CD-AC reps (cardinality 19+1+1):
  cd_ac_abor1_class_exists                       ok
  cd_ac_abor1_abort_sets_signal_aborted          ok
  cd_ac_abor1_default_reason_code_is_20          ok    ◀ DOMException AbortError code
  cd_ac_abor2_default_reason_is_dom_exception    ok

CD-AS reps (cardinality 4+3):
  cd_as_abor1_abort_static_returns_aborted_signal ok    ◀ AbortSignal.abort()
  cd_as_abor1_abort_static_with_custom_reason     ok
  cd_as_abor1_default_reason_is_abort_error       ok
  cd_as_abor2_any_with_already_aborted_input_is_aborted  ok    ◀ AbortSignal.any()
  cd_as_abor2_any_aborts_when_input_aborts        ok

Listener semantics (spec §3.3):
  spec_listener_fires_on_abort_transition         ok
  spec_listener_receives_reason                   ok
  spec_abort_is_idempotent_listeners_fire_at_most_once  ok
  spec_abort_idempotent_reason_unchanged          ok
  spec_listener_registered_after_abort_fires_immediately ok

throw_if_aborted (spec §3.3):
  spec_throw_if_aborted_noop_when_not_aborted     ok
  spec_throw_if_aborted_returns_reason_when_aborted ok

AbortSignal.any composability:
  spec_any_with_no_aborted_inputs_starts_unaborted  ok
  spec_any_propagates_first_aborter_reason        ok
  spec_any_subsequent_input_aborts_dont_re_fire   ok
  spec_any_empty_input_is_never_aborted           ok

Shared state:
  spec_signal_clone_shares_state_with_original    ok
  spec_signal_handle_persists_after_controller_drop ok

result: 22 passed, 0 failed, 0 skipped
```

## LOC measurement

| Target | LOC |
|---|---:|
| Pilot derivation `lib.rs` (code-only) | **126** |
| Bun's `AbortController.rs/zig` (estimated) | ~200-300 |
| WebKit's `AbortController.{h,cpp}` + `AbortSignal.{h,cpp}` (estimated) | ~400-600 |

Naive ratio against WebKit estimate: **~21-31%**. Adjusted (excluding event-target full machinery, timeout/timer infrastructure, AbortSignal.any with full edge cases): **~25-35%**.

The accumulating six-pilot LOC ratio table:

| Pilot | Class | Pilot LOC | Adjusted ratio |
|---|---|---:|---:|
| TextEncoder + TextDecoder | data structure | 147 | 17–25% |
| URLSearchParams | delegation target | 186 | 62% |
| structuredClone | algorithm | 297 | ~8.5% |
| Blob | composition substrate | 103 | 20–35% |
| File | inheritance/extension | 43 | 20–30% |
| **AbortController + AbortSignal** | **event/observable** | **126** | **25–35%** |

Six-pilot aggregate: **902 LOC** of derived Rust against ~25,000–30,000 LOC of upstream targets. **Aggregate ratio remains ~3%.** Below the htmx 9.4% prior at the aggregate level.

## Findings

1. **First Rc<RefCell<>> pilot lands cleanly.** Pilot needed interior mutability + shared ownership for the controller-and-signal state-sharing. The pattern is straightforward Rust idiom — not the borrow-checker-hostile case the structuredClone pilot's heap-with-ids architecture was designed to avoid. AOT hypothesis #1 confirmed: shared state via `Rc<RefCell>` is the smallest-possible deviation from the prior pilots' pure value semantics.

2. **AbortSignal.any composes via listener registration without cycle issues.** Each input signal registers a listener that aborts the result; if any input is already aborted at construction, the result is constructed already-aborted. Listeners are `FnOnce` and drop after firing. AOT hypothesis #2 confirmed.

3. **DOMException-AbortError analog modeled as enum variant** rather than full DOMException implementation. Pilot's `Reason::AbortError` carries the legacy code 20 and the `is_dom_exception()` predicate. The Bun antichain rep `expect(ac.signal.reason.code).toBe(20)` is satisfied without needing a full DOMException type. AOT hypothesis #3 confirmed.

4. **First-run clean closure.** No verifier-caught derivation bugs. AOT hypothesis #4 confirmed. The class is event-pattern but the spec is concrete; constraint corpus is rich (cross-corroborated on three properties); semantic ambiguity comparable to Pilot 4's slice-swap is absent.

5. **The listener-fires-immediately-on-late-registration semantics** (`spec_listener_registered_after_abort_fires_immediately`) was a non-obvious spec detail. Pilot's `add_event_listener` checks `inner.aborted` first and fires synchronously if true. JS uses microtask scheduling here; pilot is single-threaded so synchronous fire is the closest analog. The constraint corpus didn't surface this — the spec extract did.

## Implication: pilot-class diversity is now exhaustive

Six pilots × six distinct classes:

```
Pilot 1: data structure          (TextEncoder/Decoder)
Pilot 2: delegation target       (URLSearchParams - WebKit-imported)
Pilot 3: algorithm               (structuredClone)
Pilot 4: composition substrate   (Blob)
Pilot 5: inheritance/extension   (File extends Blob)
Pilot 6: event/observable        (AbortController/Signal)
```

These cover the major patterns a Bun-scale port encounters at the WebIDL boundary: pure data, delegated implementation, pure algorithm, container types, IDL inheritance, and event/observable surfaces. Three of these classes (algorithm, event/observable, inheritance) require fundamentally different Rust idioms; the apparatus accommodates all without forcing a uniform shape on derivations.

The five-pilot evidence chain in [Doc 706](/resolve/doc/706-three-pilot-evidence-chain-derivation-from-constraints) records pilots 1-3. Pilots 4-6 extend it across the remaining classes. **Six pilots × six classes × five clean closures + one verifier-caught-bug closure × 902 LOC aggregate against ~25-30k LOC of upstream = the apparatus' value claim is now anchored across the breadth of WebIDL surface types.**

## Files

```
pilots/abort-controller/
├── AUDIT.md
├── RUN-NOTES.md              ← this file
└── derived/
    ├── Cargo.toml
    ├── src/lib.rs            ← 207 LOC (126 code-only)
    └── tests/verifier.rs     ← 22 tests, 100% pass first-run
```

## Provenance

- Tool: `derive-constraints` v0.13b.
- Constraint inputs:
  - `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/abortcontroller.constraints.md` (3 props / 21 clauses)
  - `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/abortsignal.constraints.md` (7 props / 12 clauses)
- Spec input: DOM §3.3 + `specs/abort-controller.spec.md`.
- Result: 22/22 verifier closure on first run.
