# events — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: events-surface-property
  threshold: EVEN1
  interface: [events.find]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 12.

## EVEN1
type: specification
authority: derived
scope: module
status: active
depends-on: []

**events.find** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/29787.test.ts:152` — stdin stream stays open while concurrent fetch(file://) bodies finish (#29787) → `expect(err).toBeUndefined()`

## EVEN2
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**events.filter** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/3657.test.ts:48` — fs.watch on directory emits 'change' events for files created after watch starts → `expect(testFileEvents.length).toBeGreaterThanOrEqual(2)`
- `packages/bun-vscode/src/features/tests/__tests__/socket-integration.test.ts:612` — Socket Integration - Real Bun Process > real Bun test runner with socket communication → `expect(foundEvents.length).toBeGreaterThanOrEqual(9)`
- `test/regression/issue/3657.test.ts:101` — fs.watch emits multiple 'change' events for repeated modifications → `expect(testFileEvents.length).toBeGreaterThanOrEqual(4)`

## EVEN3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**events.filter** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `packages/bun-vscode/src/features/tests/__tests__/socket-integration.test.ts:708` — Socket Integration - Real Bun Process > real Bun test runner with socket communication → `expect(startEvents.length).toBe(expectedStartCount)`
- `packages/bun-vscode/src/features/tests/__tests__/socket-integration.test.ts:711` — Socket Integration - Real Bun Process > real Bun test runner with socket communication → `expect(endEvents.length).toBe(allTests.length)`
- `packages/bun-vscode/src/features/tests/__tests__/socket-integration.test.ts:1899` — Socket Integration - Complex Edge Cases > test discovery with .gitignore patterns → `expect(foundTests.length).toBe(1)`

