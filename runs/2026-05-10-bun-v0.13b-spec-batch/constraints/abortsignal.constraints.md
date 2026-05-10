# AbortSignal — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: abortsignal-surface-property
  threshold: ABOR1
  interface: [AbortSignal.abort, AbortSignal.any, AbortSignal, AbortSignal, AbortSignal, AbortSignal.prototype.throwIfAborted, AbortSignal.timeout]

@imports: []

@pins: []

Surface drawn from 7 candidate properties across the Bun test corpus. Construction-style: 7; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 12.

## ABOR1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal.abort** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 2 test files. Antichain representatives:

- `test/js/deno/abort/abort-controller.test.ts:58` —  → `assertEquals(signal.aborted, true)`
- `abort-controller.spec.md:23` — AbortSignal.abort static method → `AbortSignal.abort() returns an already-aborted AbortSignal with default reason`
- `test/js/deno/abort/abort-controller.test.ts:59` —  → `assertEquals(signal.reason, "hey!")`

## ABOR2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal.any** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/abort/abort.test.ts:60` — AbortSignal > AbortSignal.any() should fire abort event → `expect(signal.aborted).toBe(true)`
- `abort-controller.spec.md:31` — AbortSignal.any static method → `AbortSignal.any(signals) returns an AbortSignal aborted when any signal aborts`
- `abort-controller.spec.md:32` — AbortSignal.any static method → `AbortSignal.any returns an already-aborted signal when any input is already aborted`

## ABOR3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:12` — exists → `expect(typeof AbortSignal !== "undefined").toBe(true)`

## ABOR4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `abort-controller.spec.md:20` — AbortSignal is exposed as a global constructor → `AbortSignal cannot be constructed directly; new AbortSignal() throws TypeError`

## ABOR5
type: specification
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `abort-controller.spec.md:19` — AbortSignal is exposed as a global constructor → `AbortSignal is defined as a global constructor in any execution context with [Exposed=*]`

## ABOR6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal.prototype.throwIfAborted** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `abort-controller.spec.md:41` — AbortSignal.prototype.throwIfAborted → `AbortSignal.prototype.throwIfAborted throws the abort reason when aborted`

## ABOR7
type: specification
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal.timeout** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/abort/abort.test.ts:86` — AbortSignal > .signal.reason should be a DOMException for timeout → `expect(ac.reason).toBeInstanceOf(DOMException)`

