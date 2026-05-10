# URLSearchParams — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: urlsearchparams-surface-property
  threshold: URLS1
  interface: [URLSearchParams]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 80.

## URLS1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**URLSearchParams** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 68 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:7` — exists → `expect(typeof URLSearchParams !== "undefined").toBe(true)`
- `test/js/web/html/URLSearchParams.test.ts:88` — URLSearchParams > does not crash when calling .toJSON() on a URLSearchParams object with a… → `expect(params.toJSON()).toEqual(props)`
- `test/js/web/html/FormData.test.ts:571` — FormData > URLEncoded > should parse URLSearchParams → `expect(searchParams instanceof URLSearchParams).toBe(true)`

## URLS2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**URLSearchParams** — satisfies the documented invariant. (behavioral; cardinality 12)

Witnessed by 12 constraint clauses across 1 test files. Antichain representatives:

- `test/js/deno/url/urlsearchparams.test.ts:68` —  → `assert(params != null, "constructor returned non-null value.")`
- `test/js/deno/url/urlsearchparams.test.ts:69` —  → `assert(params.has("id"), 'Search params object has name "id"')`
- `test/js/deno/url/urlsearchparams.test.ts:70` —  → `assert(params.has("value"), 'Search params object has name "value"')`

