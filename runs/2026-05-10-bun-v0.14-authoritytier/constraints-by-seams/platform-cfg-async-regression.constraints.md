# platform-cfg/async/@regression — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: platform-cfg-async-regression-surface-property
  threshold: PLAT1
  interface: [Bun.spawn]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 209.

## PLAT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.spawn** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 79 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/pe-codesigning-integrity.test.ts:201` — should create valid PE executable with .bun section → `expect(result.exitCode).toBe(0)`
- `test/regression/issue/ctrl-c.test.ts:133` —  → `expect(proc.killed).toBe(false)`
- `test/regression/issue/24387.test.ts:22` — regression: require()ing a module with TLA should error and then wipe the module cache, so… → `expect(await proc.exited).toBe(0)`

## PLAT2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**proc.exited** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 130)

Witnessed by 130 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/compile-outfile-subdirs.test.ts:87` — creates parent directories if they don't exist → `expect(exitCode).toBe(0)`
- `test/regression/issue/29298.test.ts:46` — standalone HTML inlines file-loader assets imported from JS as data URIs → `expect(exitCode).toBe(0)`
- `test/regression/issue/04893.test.ts:24` — issue/04893 > correctly handles CRLF multiline string in CRLF terminated files → `expect(await proc.exited).toBe(0)`

