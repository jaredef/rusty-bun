# URLSearchParams — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: urlsearchparams-surface-property
  threshold: URLS1
  interface: [URLSearchParams, URLSearchParams]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 55.

## URLS1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**URLSearchParams** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 43)

Witnessed by 43 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/url_search_params_test.ts:7` —  → `assertEquals(searchParams, "str=this+string+has+spaces+in+it")`
- `tests/unit/url_search_params_test.ts:15` —  → `assertEquals(searchParams, "str=hello%2C+world%21")`
- `tests/unit/url_search_params_test.ts:23` —  → `assertEquals(searchParams, "str=%27hello+world%27")`

## URLS2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**URLSearchParams** — satisfies the documented invariant. (behavioral; cardinality 12)

Witnessed by 12 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/url_search_params_test.ts:65` —  → `assert(params != null, "constructor returned non-null value.")`
- `tests/unit/url_search_params_test.ts:66` —  → `assert(params.has("id"), 'Search params object has name "id"')`
- `tests/unit/url_search_params_test.ts:67` —  → `assert(params.has("value"), 'Search params object has name "value"')`

