# @internal — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: internal-surface-property
  threshold: INTE1
  interface: [bindgen.add, queue.size, bindgen.requiredAndOptionalArg, queue.peek, queue.shift]

@imports: []

@pins: []

Surface drawn from 5 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 5. Total witnessing constraint clauses: 58.

## INTE1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**bindgen.add** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 23)

Witnessed by 23 constraint clauses across 1 test files. Antichain representatives:

- `test/internal/bindgen.test.ts:5` — bindgen add example → `expect(bindgen.add(5, 3)).toBe(8)`
- `test/internal/bindgen.test.ts:6` — bindgen add example → `expect(bindgen.add(-2, 7)).toBe(5)`
- `test/internal/bindgen.test.ts:7` — bindgen add example → `expect(bindgen.add(0, 0)).toBe(0)`

## INTE2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**queue.size** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 14)

Witnessed by 14 constraint clauses across 1 test files. Antichain representatives:

- `test/internal/fifo.test.ts:58` — Given an empty queue > has a size of 0 → `expect(queue.size()).toBe(0)`
- `test/internal/fifo.test.ts:68` — Given an empty queue > shift() returns undefined → `expect(queue.size()).toBe(0)`
- `test/internal/fifo.test.ts:86` — Given an empty queue > When an element is pushed > has a size of 1 → `expect(queue.size()).toBe(1)`

## INTE3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**bindgen.requiredAndOptionalArg** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 1 test files. Antichain representatives:

- `test/internal/bindgen.test.ts:54` — optional arguments / default arguments → `expect(bindgen.requiredAndOptionalArg(false)).toBe(123498)`
- `test/internal/bindgen.test.ts:55` — optional arguments / default arguments → `expect(bindgen.requiredAndOptionalArg(false, 10)).toBe(52)`
- `test/internal/bindgen.test.ts:56` — optional arguments / default arguments → `expect(bindgen.requiredAndOptionalArg(true, 10)).toBe(-52)`

## INTE4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**queue.peek** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/internal/fifo.test.ts:90` — Given an empty queue > When an element is pushed > can be peeked without removing it → `expect(queue.peek()).toBe(42)`
- `test/internal/fifo.test.ts:130` — grow boundary conditions > can shift() ${n} times → `expect(queue.peek()).toBe(i)`
- `test/internal/fifo.test.ts:209` — adding and removing items > when 1k items are pushed, then removed > when new items are ad… → `expect(queue.peek()).toBe(expected.peek())`

## INTE5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**queue.shift** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/internal/fifo.test.ts:67` — Given an empty queue > shift() returns undefined → `expect(queue.shift()).toBe(undefined)`
- `test/internal/fifo.test.ts:101` — Given an empty queue > When an element is pushed > can be shifted out → `expect(el).toBe(42)`
- `test/internal/fifo.test.ts:131` — grow boundary conditions > can shift() ${n} times → `expect(queue.shift()).toBe(i)`

