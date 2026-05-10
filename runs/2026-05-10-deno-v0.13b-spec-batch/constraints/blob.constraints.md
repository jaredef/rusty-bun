# Blob — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: blob-surface-property
  threshold: BLOB1
  interface: [Blob]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 7.

## BLOB1
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Blob** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `blob.spec.md:7` — Blob is exposed as a global constructor → `Blob is defined as a global constructor in any execution context with [Exposed=*]`

## BLOB2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Blob** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `tests/unit/blob_test.ts:9` —  → `assertEquals(b2.size, b1.size + str.length)`
- `blob.spec.md:8` — Blob is exposed as a global constructor → `new Blob() returns a zero-byte Blob with empty type`
- `tests/unit/blob_test.ts:17` —  → `assertEquals(b1.size, 2 * u8.length)`

