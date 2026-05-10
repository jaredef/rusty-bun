# sync/@cli — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: sync-cli-surface-property
  threshold: SYNC1
  interface: [fs.readFileSync]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 85.

## SYNC1
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**fs.readFileSync** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 85)

Witnessed by 85 constraint clauses across 5 test files. Antichain representatives:

- `test/cli/install/migration/yarn-lock-migration.test.ts:51` — yarn.lock migration basic > simple yarn.lock migration produces correct bun.lock → `expect(bunLockContent).toMatchSnapshot("simple-yarn-migration")`
- `test/cli/install/migration/pnpm-migration-complete.test.ts:89` — PNPM Migration Complete Test Suite > comprehensive PNPM migration with all edge cases → `expect(basicLockfile).toContain('"lodash": "^4.17.21"')`
- `test/cli/install/migration/pnpm-lock-migration.test.ts:81` — pnpm-lock.yaml migration > simple pnpm lockfile migration produces correct bun.lock → `expect(bunLockContent).toMatchSnapshot("simple-pnpm-migration")`

