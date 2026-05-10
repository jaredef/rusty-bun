# atob — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: atob-surface-property
  threshold: ATOB1
  interface: [atob]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 22.

## ATOB1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**atob** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 22)

Witnessed by 22 constraint clauses across 4 test files. Antichain representatives:

- `test/js/web/websocket/websocket-custom-headers.test.ts:118` — WebSocket custom headers > should reject invalid Sec-WebSocket-Key and generate a valid on… → `expect(keyBytes.length).toBe(16)`
- `test/js/web/util/atob.test.js:8` — atob → `expect(atob("YQ==")).toBe("a")`
- `test/js/deno/encoding/encoding.test.ts:13` —  → `assertEquals(decoded, "hello world")`

