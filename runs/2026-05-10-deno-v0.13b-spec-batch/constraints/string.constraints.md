# String — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: string-surface-property
  threshold: STRI1
  interface: [String]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 30.

## STRI1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**String** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 30)

Witnessed by 30 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit/url_test.ts:29` —  → `assertEquals( String(url), "https://foo:bar@baz.qat:8000/qux/quux?foo=bar&baz=12#qat", )`
- `libs/eszip/v2.rs:2513` — from_graph_relative_base → `assert_eq!(String :: from_utf8_lossy (& source) , "import './sub_dir/mod.ts';\n")`
- `libs/eszip/lib.rs:320` — test_eszip_v1_iterator → `assert_eq!(String :: from_utf8_lossy (& got_module . source () . await . unwrap ()) , expected . source)`

