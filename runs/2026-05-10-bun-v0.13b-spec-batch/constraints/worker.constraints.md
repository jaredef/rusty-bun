# Worker — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: worker-surface-property
  threshold: WORK1
  interface: [Worker, Worker, Worker]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 53.

## WORK1
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Worker** — exposes values of the expected type or class. (construction-style)

Witnessed by 21 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/worker_threads/worker_threads.test.ts:107` — all worker_threads worker instance properties are present → `expect(worker.threadId).toBeNumber()`
- `test/js/node/worker_threads/worker_threads.test.ts:108` — all worker_threads worker instance properties are present → `expect(worker.ref).toBeFunction()`
- `test/js/node/worker_threads/worker_threads.test.ts:109` — all worker_threads worker instance properties are present → `expect(worker.unref).toBeFunction()`

## WORK2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Worker** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/worker_threads/worker_threads.test.ts:66` — all worker_threads module properties are present → `expect(Worker).toBeDefined()`
- `test/js/node/worker_threads/worker_threads.test.ts:110` — all worker_threads worker instance properties are present → `expect(worker.stdin).toBeNull()`
- `test/js/node/worker_threads/worker_threads.test.ts:111` — all worker_threads worker instance properties are present → `expect(worker.stdout).toBeNull()`

## WORK3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Worker** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:22` — exists → `expect(typeof Worker !== "undefined").toBe(true)`
- `test/js/node/worker_threads/worker-async-dispose.test.ts:23` — Worker implements Symbol.asyncDispose → `expect(typeof worker[Symbol.asyncDispose]).toBe("function")`

## WORK4
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Worker** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 25)

Witnessed by 25 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/worker_threads/worker_threads.test.ts:81` — all worker_threads worker instance properties are present → `expect(worker).toHaveProperty("threadId")`
- `test/js/node/worker_threads/worker_threads.test.ts:82` — all worker_threads worker instance properties are present → `expect(worker).toHaveProperty("ref")`
- `test/js/node/worker_threads/worker_threads.test.ts:83` — all worker_threads worker instance properties are present → `expect(worker).toHaveProperty("unref")`

