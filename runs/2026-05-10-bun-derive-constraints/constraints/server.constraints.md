# Server — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: server-surface-property
  threshold: SERV1
  interface: [Server]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 22.

## SERV1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Server** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 22)

Witnessed by 22 constraint clauses across 4 test files. Antichain representatives:

- `test/js/third_party/socket.io/socket.io.test.ts:1258` — socket.io > should leave all rooms joined after a middleware failure → `expect(io.of("/").adapter.rooms.size).toStrictEqual(0)`
- `test/js/third_party/socket.io/socket.io-server-attachment.test.ts:195` — server attachment > http.Server > should work with #attach (and merge options) → `expect(server.eio.opts.pingTimeout).toBe(6000)`
- `test/js/third_party/socket.io/socket.io-namespaces.test.ts:16` — namespaces > should be aliased → `expect(typeof io.use).toBe("function")`

