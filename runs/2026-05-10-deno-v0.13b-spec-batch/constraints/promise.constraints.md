# Promise — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: promise-surface-property
  threshold: PROM1
  interface: [Promise, Promise.withResolvers]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 27.

## PROM1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Promise** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 17)

Witnessed by 17 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit_node/crypto/crypto_hash_test.ts:105` — [node/crypto.Hash] streaming usage → `assertEquals( result, Buffer.from([ 0x1f, 0x8a, 0xc1, 0xf, 0x23, 0xc5, 0xb5, 0xbc, 0x11, 0x67, 0xbd, 0xa8, 0x4b, 0x83, 0x3e, 0x5c, 0x5, 0x7a, 0x77, 0xd2, ]), )`
- `tests/unit_node/_fs/_fs_writeFile_test.ts:128` — Data is written to correct file → `assertEquals(res, null)`
- `tests/unit_node/_fs/_fs_realpath_test.ts:29` — realpath → `assertEquals(realPath, realSymLinkPath)`

## PROM2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Promise.withResolvers** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 10)

Witnessed by 10 constraint clauses across 2 test files. Antichain representatives:

- `tests/unit_node/async_hooks_test.ts:79` —  → `assertEquals(await deferred.promise, { x: 1 })`
- `tests/unit/streams_test.ts:206` —  → `assertEquals(await cancel.promise, "resource closed")`
- `tests/unit_node/async_hooks_test.ts:80` —  → `assertEquals(await deferred1.promise, null)`

