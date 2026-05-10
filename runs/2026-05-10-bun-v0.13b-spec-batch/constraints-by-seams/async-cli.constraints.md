# async/@cli — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: async-cli-surface-property
  threshold: ASYN1
  interface: [install.exited]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 23.

## ASYN1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**install.exited** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 23)

Witnessed by 23 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/update_interactive_formatting.test.ts:517` — bun update --interactive > should handle catalog updates correctly → `expect(await install.exited).toBe(0)`
- `test/cli/update_interactive_formatting.test.ts:585` — bun update --interactive > should work correctly when run from inside a workspace director… → `expect(await install.exited).toBe(0)`
- `test/cli/update_interactive_formatting.test.ts:643` — bun update --interactive > should handle basic interactive update with select all → `expect(await install.exited).toBe(0)`

