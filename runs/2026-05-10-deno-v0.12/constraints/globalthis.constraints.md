# globalThis — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: globalthis-surface-property
  threshold: GLOB1
  interface: [globalThis.crypto.subtle.generateKey, globalThis.name]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 2; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 9.

## GLOB1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**globalThis.crypto.subtle.generateKey** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/webcrypto_test.ts:189` —  → `assertEquals(key.extractable, true)`

## GLOB2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**globalThis.name** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/globals_test.ts:114` —  → `assertEquals(typeof globalThis.name, "string")`

## GLOB3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**globalThis.crypto.subtle** — satisfies the documented invariant. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/webcrypto_test.ts:14` —  → `assert(subtle)`
- `tests/unit/webcrypto_test.ts:34` —  → `assert(subtle)`
- `tests/unit/webcrypto_test.ts:106` —  → `assert(subtle)`

