# platform-cfg/sync+async/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: platform-cfg-sync-async-js-surface-property
  threshold: PLAT1
  interface: [ctx.redis.get]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 12.

## PLAT1
type: specification
authority: derived
scope: module
status: active
depends-on: []

**ctx.redis.get** — is defined and resolves to a non-nullish value at the documented call site. (behavioral; cardinality 12)

Witnessed by 12 constraint clauses across 4 test files. Antichain representatives:

- `test/js/valkey/valkey.test.ts:794` — Basic Operations > should unlink one or more keys asynchronously with UNLINK → `expect(await redis.get("unlink-key1")).toBeNull()`
- `test/js/valkey/unit/basic-operations.test.ts:35` — String Commands > SET and GET commands → `expect(nullResult).toBeNull()`
- `test/js/valkey/reliability/protocol-handling.test.ts:106` — RESP3 Data Type Handling > should handle RESP3 Null type → `expect(nullResult).toBeNull()`

