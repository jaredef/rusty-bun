# RedisClient — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: redisclient-surface-property
  threshold: REDI1
  interface: [RedisClient]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 15.

## REDI1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**RedisClient** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 15)

Witnessed by 15 constraint clauses across 3 test files. Antichain representatives:

- `test/js/valkey/valkey.test.ts:6881` — duplicate() > should duplicate client that failed to connect → `expect(failedRedis.connected).toBe(false)`
- `test/js/valkey/reliability/recovery.test.ts:37` — client.connect() recovers after the client enters the failed state → `expect(await client.get("recovery:k")).toBe("before")`
- `test/js/valkey/reliability/connection-failures.test.ts:46` — Connection Failure Handling > should reject commands with appropriate errors when disconne… → `expect(client.connected).toBe(false)`

