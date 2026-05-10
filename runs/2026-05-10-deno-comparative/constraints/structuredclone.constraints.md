# structuredClone — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: structuredclone-surface-property
  threshold: STRU1
  interface: [structuredClone]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 11.

## STRU1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**structuredClone** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/structured_clone_test.ts:72` — structuredClone CryptoKey → `assertEquals(aesClone.type, aesKey.type)`
- `tests/unit/structured_clone_test.ts:73` — structuredClone CryptoKey → `assertEquals(aesClone.extractable, aesKey.extractable)`
- `tests/unit/structured_clone_test.ts:74` — structuredClone CryptoKey → `assertEquals(aesClone.algorithm, aesKey.algorithm)`

