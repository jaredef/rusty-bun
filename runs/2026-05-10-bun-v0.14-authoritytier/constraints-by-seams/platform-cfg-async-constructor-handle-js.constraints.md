# platform-cfg/async/constructor+handle/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: platform-cfg-async-constructor-handle-js-surface-property
  threshold: PLAT1
  interface: [Buffer]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 19.

## PLAT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Buffer** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 3 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:1555` — writeFileSync > returning Buffer works → `expect(buffer[i]).toBe(out[i])`
- `test/js/node/buffer.test.js:1409` — Buffer → `expect(new Buffer(input).toString("utf8")).toBe(inputs[i])`
- `test/js/bun/io/bun-write.test.js:60` — large file > write large file (bytes) → `expect(new Buffer(await Bun.file(filename + ".bytes").arrayBuffer()).equals(bytes)).toBe(true)`

## PLAT2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.write** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/io/bun-write.test.js:26` — Bun.write blob → `expect(await Bun.write(new TextEncoder().encode(tmpbase + "response-file.test.txt"), new Uint32Array(1024))).toBe( new Uint32Array(1024).byteLength, )`
- `test/js/bun/bun-object/write.spec.ts:74` — Bun.write() on file paths > Given a path to a file in an existing directory > When the fil… → `expect(result).toBe(content.length)`
- `test/js/bun/io/bun-write.test.js:59` — large file > write large file (bytes) → `expect(written).toBe(bytes.byteLength)`

