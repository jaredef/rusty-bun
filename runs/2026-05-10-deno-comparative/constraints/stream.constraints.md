# stream — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: stream-surface-property
  threshold: STRE1
  interface: [stream]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 11.

## STRE1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**stream** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 2 test files. Antichain representatives:

- `ext/node/ops/zlib/mod.rs:1531` — zlib_start_write → `assert_eq!(stream . err , Z_OK)`
- `ext/http/reader_stream.rs:85` — success → `assert_eq!(stream . next () . await . unwrap () . unwrap () , Bytes :: from ("hello"))`
- `ext/node/ops/zlib/mod.rs:1532` — zlib_start_write → `assert_eq!(stream . start_write (input , * offset , * len , & mut [] , 0 , 0 , Flush :: None) . is_ok () , * expected)`

