# async/@regression — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: async-regression-surface-property
  threshold: ASYN1
  interface: [Bun.file, Response]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 2; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 405.

## ASYN1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.file** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 247 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/26851.test.ts:30` — --bail writes JUnit reporter outfile → `expect(await file.exists()).toBe(true)`
- `test/regression/issue/26647.test.ts:39` — Bun.file().stat() should handle UTF-8 paths with Japanese characters → `expect(bunStat.isFile()).toBe(true)`
- `test/regression/issue/14029.test.ts:41` — snapshots will recognize existing entries → `expect(newSnapshot).toBe(await Bun.file(join(testDir, "__snapshots__", "test.test.js.snap")).text())`

## ASYN2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 158 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/19850/19850.test.ts:22` — when beforeEach callback throws > test name is not garbled → `expect(err).toBe(' err-in-hook-and-multiple-tests.ts: 1 | import { beforeEach, test } from "bun:test"; 2 | 3 | beforeEach(() => { 4 | throw new Error("beforeEach"); ^ error: beforeEach at <anonymous> …`
- `test/regression/issue/09555.test.ts:154` — #09555 > Readable.fromWeb consumes the ReadableStream → `expect(response.bodyUsed).toBe(false)`
- `test/regression/issue/02368.test.ts:14` — can clone a response → `expect(await response.text()).toBe("bun")`

