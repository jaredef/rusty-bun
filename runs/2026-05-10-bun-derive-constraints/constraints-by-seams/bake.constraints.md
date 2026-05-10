# @bake — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: bake-surface-property
  threshold: BAKE1
  interface: [Bun.$]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 23.

## BAKE1
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Bun.$** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 23)

Witnessed by 23 constraint clauses across 2 test files. Antichain representatives:

- `test/bake/dev/response-to-bake-response.test.ts:47` — Response -> import { Response } from 'bun:app' transform in server components → `expect(serverResult).toContain('import { Response } from "bun:app"')`
- `test/bake/dev/production.test.ts:103` — production > import.meta properties are inlined in production build → `expect(distFiles).toContain("index.html")`
- `test/bake/dev/response-to-bake-response.test.ts:49` — Response -> import { Response } from 'bun:app' transform in server components → `expect(serverResult).toContain("new import_bun_app.Response")`

