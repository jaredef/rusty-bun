# events — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: events-surface-property
  threshold: EVEN1
  interface: [events, events, events.filter, events.filter]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 4. Total witnessing constraint clauses: 42.

## EVEN1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**events** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 23)

Witnessed by 23 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/29787.test.ts:159` — stdin stream stays open while concurrent fetch(file://) bodies finish (#29787) → `expect(dataBytes).toBe(totalWrites)`
- `test/regression/issue/14338.test.ts:43` — WebSocket should emit error event before close event on handshake failure (issue #14338) → `expect(events).toEqual(["error", "close"])`
- `test/js/web/broadcastchannel/broadcast-channel.test.ts:61` — messages are delivered in port creation order → `expect(events[0].target).toBe(c2)`

## EVEN2
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**events** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 3 test files. Antichain representatives:

- `test/regression/issue/14338.test.ts:106` — WebSocket successful connection should NOT emit error event → `expect(events).toContain("open")`
- `test/js/web/websocket/websocket-close-connecting.test.ts:39` — fires error + close events and transitions to CLOSED → `expect(events[0].message).toContain("closed before the connection is established")`
- `packages/bun-vscode/src/features/tests/__tests__/socket-integration.test.ts:266` — Socket Integration - Event Handling > test reporter events in correct sequence → `expect(events).toHaveLength(7)`

## EVEN3
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**events.filter** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/3657.test.ts:48` — fs.watch on directory emits 'change' events for files created after watch starts → `expect(testFileEvents.length).toBeGreaterThanOrEqual(2)`
- `packages/bun-vscode/src/features/tests/__tests__/socket-integration.test.ts:612` — Socket Integration - Real Bun Process > real Bun test runner with socket communication → `expect(foundEvents.length).toBeGreaterThanOrEqual(9)`
- `test/regression/issue/3657.test.ts:101` — fs.watch emits multiple 'change' events for repeated modifications → `expect(testFileEvents.length).toBeGreaterThanOrEqual(4)`

## EVEN4
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

