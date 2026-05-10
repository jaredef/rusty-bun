# Headers — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: headers-surface-property
  threshold: HEAD1
  interface: [Headers]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 16.

## HEAD1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Headers** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 11 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/headers_test.ts:44` —  → `assertEquals(headers.get(name), String(value))`
- `tests/unit/headers_test.ts:46` —  → `assertEquals(headers.get("length"), null)`
- `tests/unit/headers_test.ts:52` —  → `assertEquals(headers.get(name), String(value))`

## HEAD2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Headers** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/headers_test.ts:83` —  → `assert(headers.has(name), "headers has name " + name)`
- `tests/unit/headers_test.ts:94` —  → `assert(headers.has(name), "headers have a header: " + name)`
- `tests/unit/headers_test.ts:114` —  → `assert(headers.has(key))`

