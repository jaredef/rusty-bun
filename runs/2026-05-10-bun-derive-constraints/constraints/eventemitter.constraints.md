# EventEmitter — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: eventemitter-surface-property
  threshold: EVEN1
  interface: [EventEmitter]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 45.

## EVEN1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**EventEmitter** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 45)

Witnessed by 45 constraint clauses across 4 test files. Antichain representatives:

- `test/regression/issue/24147.test.ts:46` — removeAllListeners() with actual listeners to remove → `assert.strictEqual(emitter.listenerCount("foo"), 0)`
- `test/regression/issue/014187.test.ts:18` — issue-14187 → `expect(ee.listenerCount("beep")).toBe(1)`
- `test/js/node/http/node-http.test.ts:1282` — server.address should be valid IP > ServerResponse assign assignSocket → `expect(socket).toBe(socket)`

