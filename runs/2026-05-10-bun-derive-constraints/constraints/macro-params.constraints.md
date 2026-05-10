# MACRO_PARAMS — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: macro-params-surface-property
  threshold: MACR1
  interface: [MACRO_PARAMS]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 12.

## MACR1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**MACRO_PARAMS** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 12)

Witnessed by 12 constraint clauses across 1 test files. Antichain representatives:

- `src/clap/lib.rs:928` — parse_param_macro_static_tables → `assert_eq!(MACRO_PARAMS [0] . names . short , Some (b's'))`
- `src/clap/lib.rs:929` — parse_param_macro_static_tables → `assert_eq!(MACRO_PARAMS [0] . names . long , Some (b"long" as & [u8]))`
- `src/clap/lib.rs:930` — parse_param_macro_static_tables → `assert_eq!(MACRO_PARAMS [0] . id . value , b"value")`

