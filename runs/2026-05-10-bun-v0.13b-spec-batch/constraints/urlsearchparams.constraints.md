# URLSearchParams — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: urlsearchparams-surface-property
  threshold: URLS1
  interface: [URLSearchParams, URLSearchParams]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 2; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 20.

## URLS1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**URLSearchParams** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 14 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:7` — exists → `expect(typeof URLSearchParams !== "undefined").toBe(true)`
- `test/js/web/html/URLSearchParams.test.ts:100` — URLSearchParams > non-standard extensions > should support .length → `expect(params.length).toBe(3)`
- `test/js/deno/url/urlsearchparams.test.ts:11` —  → `assertEquals(searchParams, "str=this+string+has+spaces+in+it")`

## URLS2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**URLSearchParams** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `url-search-params.spec.md:7` — URLSearchParams is exposed as a global constructor → `URLSearchParams is defined as a global constructor in any execution context with [Exposed=*]`

## URLS3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**URLSearchParams** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `url-search-params.spec.md:9` — URLSearchParams is exposed as a global constructor → `new URLSearchParams(init) accepts a USVString, sequence of pairs, or record`
- `url-search-params.spec.md:12` — URLSearchParams constructor input forms → `URLSearchParams constructor accepts a query-string starting with optional "?"`
- `url-search-params.spec.md:13` — URLSearchParams constructor input forms → `URLSearchParams constructor accepts a sequence of name-value pairs`

