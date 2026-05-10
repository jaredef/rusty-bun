# platform-cfg/sync/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: platform-cfg-sync-js-surface-property
  threshold: PLAT1
  interface: [fs.promises.glob]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 22.

## PLAT1
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fs.promises.glob** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/glob.test.ts:176` — fs.promises.glob > returns an AsyncIterable over matched paths → `expect(iter[Symbol.asyncIterator]).toBeDefined()`
- `test/js/node/fs/glob.test.ts:188` — fs.promises.glob > works without providing options → `expect(iter[Symbol.asyncIterator]).toBeDefined()`
- `test/js/node/fs/glob.test.ts:203` — fs.promises.glob > matches directories → `expect(iter[Symbol.asyncIterator]).toBeDefined()`

## PLAT2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.statSync** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 19)

Witnessed by 19 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:1543` — writeFileSync > write file with mode, issue #3740 → `expect(stat.mode).toBe(isWindows ? 33206 : 33188)`
- `test/js/node/fs/fs-mkdir.test.ts:144` — fs.mkdir - recursive > creates nested directories when both top-level and sub-folders don'… → `expect(fs.statSync(pathname).isDirectory()).toBe(true)`
- `test/js/node/fs/fs.test.ts:3036` — utimesSync > works → `expect(newStats.mtime).toEqual(newModifiedTime)`

