# sync+async/threaded/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: sync-async-threaded-js-surface-property
  threshold: SYNC1
  interface: [AsyncLocalStorage]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 70.

## SYNC1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AsyncLocalStorage** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 70)

Witnessed by 70 constraint clauses across 4 test files. Antichain representatives:

- `test/js/node/process/process-nexttick.test.js:1028` — process.nextTick and AsyncLocalStorage.enterWith don't conflict → `expect(storage.getStore()).toBe("hello")`
- `test/js/node/async_hooks/async_hooks.node.test.ts:10` — node async_hooks.AsyncLocalStorage enable disable → `assert.strictEqual(asyncLocalStorage.getStore()!.get("foo"), "bar")`
- `test/js/node/async_hooks/async-local-storage-thenable.test.ts:15` — node.js test test-async-local-storage-no-mix-contexts.js → `assert.strictEqual(asyncLocalStorage.getStore().get("a"), 1)`

