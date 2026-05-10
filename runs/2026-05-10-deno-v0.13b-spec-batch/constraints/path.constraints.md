# path — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: path-surface-property
  threshold: PATH1
  interface: [path]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 16.

## PATH1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 16)

Witnessed by 16 constraint clauses across 5 test files. Antichain representatives:

- `libs/dotenv/lib.rs:1062` — find_path_and_content_reads_file_directly → `assert_eq!(path , Path :: new ("/project/.env"))`
- `libs/config/deno_json/mod.rs:3192` — test_to_import_map_import_map_entry → `assert_eq!(path , deno_path_util :: url_to_file_path (& root_url ()) . unwrap () . join ("import_map.json"))`
- `libs/cache_dir/local.rs:1242` — test_url_to_local_sub_path → `assert_eq!(path , test_caches . local_temp . join (& parts))`

