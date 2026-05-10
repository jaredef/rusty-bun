# sync/@regression — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: sync-regression-surface-property
  threshold: SYNC1
  interface: [Bun.spawnSync]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 76.

## SYNC1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.spawnSync** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 76)

Witnessed by 76 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/25432.test.ts:26` — process.stdout.end() flushes pending writes before callback > large write followed by end(… → `expect(result.exitCode).toBe(0)`
- `test/regression/issue/22199.test.ts:27` — plugin onResolve returning undefined should not crash → `expect(result.exitCode).toBe(0)`
- `test/regression/issue/19652.test.ts:18` — bun build --production does not crash (issue #19652) → `expect(result.exitCode).toBe(0)`

