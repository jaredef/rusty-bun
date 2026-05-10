# JSON — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: json-surface-property
  threshold: JSON1
  interface: [JSON.parse]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 13.

## JSON1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**JSON.parse** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit_node/http_test.ts:1655` — [node/http] http.request() post streaming body works → `assertEquals(response.bytes, contentLength)`
- `tests/unit/read_text_file_test.ts:16` —  → `assertEquals(pkg.name, "deno")`
- `tests/unit/read_file_test.ts:17` —  → `assertEquals(pkg.name, "deno")`

