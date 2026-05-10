# module — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: module-surface-property
  threshold: MODU1
  interface: [module]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 4.

## MODU1
type: specification
authority: derived
scope: module
status: active
depends-on: []

**module** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/module/require-extensions.test.ts:85` — wrapping an existing extension with no logic → `expect(module).toBeDefined()`
- `test/js/node/module/require-extensions.test.ts:101` — wrapping an existing extension with mutated compile function → `expect(module).toBeDefined()`
- `test/js/node/module/require-extensions.test.ts:124` — wrapping an existing extension with mutated compile function ts → `expect(module).toBeDefined()`

