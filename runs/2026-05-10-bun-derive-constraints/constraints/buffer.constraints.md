# buffer — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: buffer-surface-property
  threshold: BUFF1
  interface: [buffer.Buffer.isBuffer, buffer.INSPECT_MAX_BYTES]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 2; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 2.

## BUFF1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**buffer.Buffer.isBuffer** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/zlib/zlib.test.js:127` — zlib.gunzip > should be able to unzip a Buffer and return an unzipped Buffer → `expect(buffer.Buffer.isBuffer(data)).toBe(true)`

## BUFF2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**buffer.INSPECT_MAX_BYTES** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/buffer-inspectmaxbytes.test.ts:7` — buffer.INSPECT_MAX_BYTES is a number and not a custom getter/setter → `expect(buffer.INSPECT_MAX_BYTES).toBeNumber()`

