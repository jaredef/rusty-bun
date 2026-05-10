# TextDecoder — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: textdecoder-surface-property
  threshold: TEXT1
  interface: [TextDecoder, TextDecoder]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 82.

## TEXT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**TextDecoder** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 69)

Witnessed by 69 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit_node/tls_test.ts:525` — [node/tls] tls.Server.unref() works → `assertEquals(new TextDecoder().decode(stdout), "")`
- `tests/unit_node/net_test.ts:103` — [node/net] net.connect().unref() works → `assertEquals(new TextDecoder().decode(stdout), "connected\n")`
- `tests/unit_node/http_test.ts:2736` — [node/http] IncomingMessage as Request body supports BYOB reader → `assertEquals(new TextDecoder().decode(value), "hello world")`

## TEXT2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**TextDecoder** — satisfies the documented invariant. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit/websocket_test.ts:1210` — WebSocket close with reason but no code doesn't send 1005 → `assert(headerEnd > 0)`
- `tests/unit/serve_test.ts:988` — httpServerUrl${name} → `assert(response.startsWith('HTTP/1.1 ${expected}'))`
- `tests/unit/os_test.ts:241` —  → `assert(hostname.length > 0)`

