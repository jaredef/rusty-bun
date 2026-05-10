# Atomics — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: atomics-surface-property
  threshold: ATOM1
  interface: [Atomics.load, Atomics.store, Atomics.add, Atomics.isLockFree]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 4. Total witnessing constraint clauses: 60.

## ATOM1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Atomics.load** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 35)

Witnessed by 35 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/atomics.test.ts:10` — Atomics > basic operations > store and load → `expect(Atomics.load(view, 0)).toBe(42)`
- `test/js/web/atomics.test.ts:13` — Atomics > basic operations > store and load → `expect(Atomics.load(view, 1)).toBe(-123)`
- `test/js/web/atomics.test.ts:22` — Atomics > basic operations > add → `expect(Atomics.load(view, 0)).toBe(15)`

## ATOM2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Atomics.store** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 14)

Witnessed by 14 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/atomics.test.ts:9` — Atomics > basic operations > store and load → `expect(Atomics.store(view, 0, 42)).toBe(42)`
- `test/js/web/atomics.test.ts:12` — Atomics > basic operations > store and load → `expect(Atomics.store(view, 1, -123)).toBe(-123)`
- `test/js/web/atomics.test.ts:169` — Atomics > different TypedArray types > Int8Array → `expect(Atomics.store(view, 0, 42)).toBe(42)`

## ATOM3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Atomics.add** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/atomics.test.ts:21` — Atomics > basic operations > add → `expect(Atomics.add(view, 0, 5)).toBe(10)`
- `test/js/web/atomics.test.ts:24` — Atomics > basic operations > add → `expect(Atomics.add(view, 0, -3)).toBe(15)`
- `test/js/web/atomics.test.ts:171` — Atomics > different TypedArray types > Int8Array → `expect(Atomics.add(view, 0, 8)).toBe(42)`

## ATOM4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Atomics.isLockFree** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/atomics.test.ts:96` — Atomics > utility functions > isLockFree → `expect(typeof Atomics.isLockFree(1)).toBe("boolean")`
- `test/js/web/atomics.test.ts:97` — Atomics > utility functions > isLockFree → `expect(typeof Atomics.isLockFree(2)).toBe("boolean")`
- `test/js/web/atomics.test.ts:98` — Atomics > utility functions > isLockFree → `expect(typeof Atomics.isLockFree(4)).toBe("boolean")`

