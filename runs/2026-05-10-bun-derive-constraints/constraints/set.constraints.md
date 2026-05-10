# Set — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: set-surface-property
  threshold: SET1
  interface: [Set, Set]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 38.

## SET1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Set** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 32)

Witnessed by 32 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/29240.test.ts:91` — cpu-prof callFrame.lineNumber/columnNumber point at function definition, not sample positi… → `expect(fibColumns.size).toBe(1)`
- `test/js/valkey/valkey.test.ts:2635` — Set Operations > should scan set members with SSCAN → `expect(new Set(allMembers).size).toBe(20)`
- `test/js/node/watch/fs.watchFile.test.ts:197` — fs.watchFile > StatWatcherScheduler stress test (1000 watchers with random times) → `expect(set.size).toBe(1000)`

## SET2
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Set** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 4 test files. Antichain representatives:

- `test/regression/issue/27358.test.ts:39` — mTLS SSLConfig keepalive (#27358) > fetch with custom TLS reuses keepalive connections → `expect(uniquePorts.size).toBeLessThanOrEqual(2)`
- `test/js/bun/image/image-kernels.test.ts:249` — Floyd–Steinberg dither > 2-colour dithered ramp tracks source mean inside palette gamut → `expect(seen.size).toBeLessThanOrEqual(2)`
- `test/js/bun/http/tls-keepalive.test.ts:33` — TLS keepalive for custom SSL configs > keepalive reuses connections with same TLS config → `expect(uniquePorts.size).toBeLessThanOrEqual(2)`

