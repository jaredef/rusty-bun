# module — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: module-surface-property
  threshold: MODU1
  interface: [module, module]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 55.

## MODU1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**module** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 50)

Witnessed by 50 constraint clauses across 4 test files. Antichain representatives:

- `libs/eszip/v2.rs:2216` — test_graph_external → `assert_eq!(module . specifier , "file:///external.ts")`
- `libs/eszip/v1.rs:169` — file_format_parse → `assert_eq!(module . specifier , specifier)`
- `libs/eszip/lib.rs:259` — take_source_v1 → `assert_eq!(module . specifier , specifier)`

## MODU2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**module** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `libs/eszip/v2.rs:2416` — from_graph_dynamic → `assert!(module . is_some ())`
- `libs/eszip/lib.rs:264` — take_source_v1 → `assert!(module . source_map () . await . is_none ())`
- `libs/eszip/lib.rs:284` — take_source_v2 → `assert!(module . source () . await . is_none ())`

