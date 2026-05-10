# BroadcastChannel — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: broadcastchannel-surface-property
  threshold: BROA1
  interface: [BroadcastChannel, BroadcastChannel, BroadcastChannel]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 8.

## BROA1
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

## BROA2
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

## BROA3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**BroadcastChannel** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/worker_threads/worker_threads.test.ts:63` — all worker_threads module properties are present → `expect(BroadcastChannel).toBeDefined()`

