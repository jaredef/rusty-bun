# AbortSignal — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: abortsignal-surface-property
  threshold: ABOR1
  interface: [AbortSignal, AbortSignal.any, AbortSignal.timeout]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 3.

## ABOR1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:12` — exists → `expect(typeof AbortSignal !== "undefined").toBe(true)`

## ABOR2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal.any** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/abort/abort.test.ts:60` — AbortSignal > AbortSignal.any() should fire abort event → `expect(signal.aborted).toBe(true)`

## ABOR3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal.timeout** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/abort/abort.test.ts:86` — AbortSignal > .signal.reason should be a DOMException for timeout → `expect(ac.reason).toBeInstanceOf(DOMException)`

