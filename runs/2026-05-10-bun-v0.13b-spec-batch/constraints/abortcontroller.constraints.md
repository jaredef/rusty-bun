# AbortController — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: abortcontroller-surface-property
  threshold: ABOR1
  interface: [AbortController, AbortController, AbortController]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 21.

## ABOR1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortController** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 19 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:11` — exists → `expect(typeof AbortController !== "undefined").toBe(true)`
- `test/js/web/abort/abort.test.ts:81` — AbortSignal > .signal.reason should be a DOMException → `expect(ac.signal.reason.code).toBe(20)`
- `test/js/node/util/test-aborted.test.ts:13` — aborted works when provided a resource that was already aborted → `expect(ac.signal.aborted).toBe(true)`

## ABOR2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**AbortController** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/abort/abort.test.ts:79` — AbortSignal > .signal.reason should be a DOMException → `expect(ac.signal.reason).toBeInstanceOf(DOMException)`

## ABOR3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**AbortController** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `abort-controller.spec.md:7` — AbortController is exposed as a global constructor → `AbortController is defined as a global constructor in any execution context with [Exposed=*]`

