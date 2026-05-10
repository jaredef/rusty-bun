# structuredClone — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: structuredclone-surface-property
  threshold: STRU1
  interface: [structuredClone, structuredClone, structuredClone]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 30.

## STRU1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**structuredClone** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 12 constraint clauses across 2 test files. Antichain representatives:

- `tests/unit/structured_clone_test.ts:72` — structuredClone CryptoKey → `assertEquals(aesClone.type, aesKey.type)`
- `structured-clone.spec.md:8` — structuredClone is exposed as a global function → `structuredClone(value) returns a deep clone of value`
- `tests/unit/structured_clone_test.ts:73` — structuredClone CryptoKey → `assertEquals(aesClone.extractable, aesKey.extractable)`

## STRU2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**structuredClone** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `structured-clone.spec.md:23` — structuredClone unsupported types → `structuredClone throws DataCloneError on functions`
- `structured-clone.spec.md:24` — structuredClone unsupported types → `structuredClone throws DataCloneError on DOM nodes outside the supported list`
- `structured-clone.spec.md:25` — structuredClone unsupported types → `structuredClone throws DataCloneError on values containing non-cloneable references`

## STRU3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**structuredClone** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `structured-clone.spec.md:7` — structuredClone is exposed as a global function → `structuredClone is defined as a global function in any execution context with [Exposed=*]`

## STRU4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**structuredClone** — satisfies the documented invariant. (behavioral; cardinality 14)

Witnessed by 14 constraint clauses across 1 test files. Antichain representatives:

- `structured-clone.spec.md:9` — structuredClone is exposed as a global function → `structuredClone(value, options) accepts a StructuredSerializeOptions with transfer`
- `structured-clone.spec.md:12` — structuredClone supported types → `structuredClone clones primitives by value`
- `structured-clone.spec.md:13` — structuredClone supported types → `structuredClone clones Date with the same time value`

