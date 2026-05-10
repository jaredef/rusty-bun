# threaded/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: threaded-js-surface-property
  threshold: THRE1
  interface: [Worker, Worker, BroadcastChannel, BroadcastChannel, BroadcastChannel, MessageChannel, MessagePort]

@imports: []

@pins: []

Surface drawn from 12 candidate properties across the Bun test corpus. Construction-style: 7; behavioral (high-cardinality): 5. Total witnessing constraint clauses: 121.

## THRE1
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

## THRE2
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

## THRE3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**BroadcastChannel** — exposes values of the expected type or class. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/broadcastchannel/broadcast-channel.test.ts:27` — broadcast channel properties → `expect(c1.close).toBeInstanceOf(Function)`
- `test/js/web/broadcastchannel/broadcast-channel.test.ts:28` — broadcast channel properties → `expect(c1.postMessage).toBeInstanceOf(Function)`
- `test/js/web/broadcastchannel/broadcast-channel.test.ts:29` — broadcast channel properties → `expect(c1.ref).toBeInstanceOf(Function)`

## THRE4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**BroadcastChannel** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/broadcastchannel/broadcast-channel.test.ts:24` — broadcast channel properties → `expect(c1.name).toBe("props")`
- `test/js/web/broadcastchannel/broadcast-channel.test.ts:25` — broadcast channel properties → `expect(c1.onmessage).toBe(null)`
- `test/js/web/broadcastchannel/broadcast-channel.test.ts:26` — broadcast channel properties → `expect(c1.onmessageerror).toBe(null)`

## THRE5
type: specification
authority: derived
scope: module
status: active
depends-on: []

**BroadcastChannel** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/worker_threads/worker_threads.test.ts:63` — all worker_threads module properties are present → `expect(BroadcastChannel).toBeDefined()`

## THRE6
type: specification
authority: derived
scope: module
status: active
depends-on: []

**MessageChannel** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/worker_threads/worker_threads.test.ts:64` — all worker_threads module properties are present → `expect(MessageChannel).toBeDefined()`

## THRE7
type: specification
authority: derived
scope: module
status: active
depends-on: []

**MessagePort** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/worker_threads/worker_threads.test.ts:65` — all worker_threads module properties are present → `expect(MessagePort).toBeDefined()`

## THRE8
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

## THRE9
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

## THRE10
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

## THRE11
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

## THRE12
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

