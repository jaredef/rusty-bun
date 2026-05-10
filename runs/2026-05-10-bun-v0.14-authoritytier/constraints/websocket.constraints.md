# WebSocket — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: websocket-surface-property
  threshold: WEBS1
  interface: [WebSocket, WebSocket.CLOSED, WebSocket.CLOSING, WebSocket.CONNECTING, WebSocket.OPEN]

@imports: []

@pins: []

Surface drawn from 5 candidate properties across the Bun test corpus. Construction-style: 5; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 22.

## WEBS1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**WebSocket** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 18 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/websocket/websocket.test.js:380` — WebSocket > nodebuffer > should support 'nodebuffer' binaryType → `expect(ws.binaryType).toBe("nodebuffer")`
- `test/js/web/websocket/websocket-unix.test.ts:53` — ws+unix:// echoes through Bun.serve({ unix }) → `expect(ws.url).toBe('ws+unix://${unix}')`
- `test/js/web/websocket/websocket-proxy.test.ts:98` — WebSocket proxy API > accepts proxy option as string (HTTP proxy) → `expect(ws.readyState).toBe(WebSocket.CONNECTING)`

## WEBS2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**WebSocket.CLOSED** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/first_party/ws/ws.test.ts:253` — WebSocket > sets static properties correctly → `expect(WebSocket.CLOSED).toBeDefined()`

## WEBS3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**WebSocket.CLOSING** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/first_party/ws/ws.test.ts:254` — WebSocket > sets static properties correctly → `expect(WebSocket.CLOSING).toBeDefined()`

## WEBS4
type: specification
authority: derived
scope: module
status: active
depends-on: []

**WebSocket.CONNECTING** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/first_party/ws/ws.test.ts:255` — WebSocket > sets static properties correctly → `expect(WebSocket.CONNECTING).toBeDefined()`

## WEBS5
type: specification
authority: derived
scope: module
status: active
depends-on: []

**WebSocket.OPEN** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/first_party/ws/ws.test.ts:256` — WebSocket > sets static properties correctly → `expect(WebSocket.OPEN).toBeDefined()`

