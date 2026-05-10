# Buffer — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: buffer-surface-property
  threshold: BUFF1
  interface: [Buffer.isBuffer]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 9.

## BUFF1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Buffer.isBuffer** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/_fs/_fs_readFile_test.ts:167` — fs.readFile returns Buffer when encoding is not provided → `assertEquals(Buffer.isBuffer(data), true)`
- `tests/unit_node/_fs/_fs_readFile_test.ts:188` — fs.readFileSync returns Buffer when encoding is not provided → `assertEquals(Buffer.isBuffer(data), true)`

## BUFF2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Buffer.from** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `tests/unit_node/crypto/crypto_hkdf_test.ts:108` — crypto.hkdfSync - TypedArray byte representation fix → `assertEquals( resultHex, expected, '${name} should produce Node.js-compatible result', )`
- `tests/unit_node/_fs/_fs_raw_fd_test.ts:247` — [node/fs] readFileSync with fd reads entire file → `assertEquals(Buffer.from(data).toString(), "file content here")`
- `tests/unit_node/crypto/crypto_hkdf_test.ts:133` — crypto.hkdfSync - TypedArray byte representation fix → `assertEquals( resultHex, bufferHex, '${name} should match its underlying ArrayBuffer representation', )`

