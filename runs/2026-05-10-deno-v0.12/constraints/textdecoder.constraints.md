# TextDecoder — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: textdecoder-surface-property
  threshold: TEXT1
  interface: [TextDecoder]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 61.

## TEXT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**TextDecoder** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 61)

Witnessed by 61 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit_node/tls_test.ts:525` — [node/tls] tls.Server.unref() works → `assertEquals(new TextDecoder().decode(stdout), "")`
- `tests/unit_node/net_test.ts:103` — [node/net] net.connect().unref() works → `assertEquals(new TextDecoder().decode(stdout), "connected\n")`
- `tests/unit_node/http_test.ts:2736` — [node/http] IncomingMessage as Request body supports BYOB reader → `assertEquals(new TextDecoder().decode(value), "hello world")`

