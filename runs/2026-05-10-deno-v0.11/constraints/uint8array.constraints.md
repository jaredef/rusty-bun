# Uint8Array — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: uint8array-surface-property
  threshold: UINT1
  interface: [Uint8Array]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 39.

## UINT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Uint8Array** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 39)

Witnessed by 39 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit_node/zlib_test.ts:160` — Brotli quality 10 doesn't panic → `assertEquals( new Uint8Array(e.buffer, e.byteOffset, e.byteLength), new Uint8Array([11, 1, 128, 97, 98, 99, 3]), )`
- `tests/unit_node/crypto/crypto_misc_test.ts:27` — [node/crypto.randomFillSync] array buffer view → `assertEquals(view.length, 16)`
- `tests/unit_node/child_process_test.ts:136` — [node/child_process spawn] stdin and stdout with binary data → `assertEquals(new Uint8Array(data!), buffer)`

