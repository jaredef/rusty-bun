# platform-cfg/sync+async/threaded/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: platform-cfg-sync-async-threaded-js-surface-property
  threshold: PLAT1
  interface: [Worker]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 2.

## PLAT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Worker** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:22` — exists → `expect(typeof Worker !== "undefined").toBe(true)`
- `test/js/node/worker_threads/worker-async-dispose.test.ts:23` — Worker implements Symbol.asyncDispose → `expect(typeof worker[Symbol.asyncDispose]).toBe("function")`

