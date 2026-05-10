# AbortSignal — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: abortsignal-surface-property
  threshold: ABOR1
  interface: [AbortSignal.abort, AbortSignal, AbortSignal, AbortSignal.prototype.throwIfAborted]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 4; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 7.

## ABOR1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal.abort** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 2 test files. Antichain representatives:

- `tests/unit/abort_controller_test.ts:62` —  → `assertEquals(signal.aborted, true)`
- `abort-controller.spec.md:23` — AbortSignal.abort static method → `AbortSignal.abort() returns an already-aborted AbortSignal with default reason`
- `tests/unit/abort_controller_test.ts:63` —  → `assertEquals(signal.reason, "hey!")`

## ABOR2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `abort-controller.spec.md:20` — AbortSignal is exposed as a global constructor → `AbortSignal cannot be constructed directly; new AbortSignal() throws TypeError`

## ABOR3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `abort-controller.spec.md:19` — AbortSignal is exposed as a global constructor → `AbortSignal is defined as a global constructor in any execution context with [Exposed=*]`

## ABOR4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal.prototype.throwIfAborted** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `abort-controller.spec.md:41` — AbortSignal.prototype.throwIfAborted → `AbortSignal.prototype.throwIfAborted throws the abort reason when aborted`

