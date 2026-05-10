# URLPattern — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: urlpattern-surface-property
  threshold: URLP1
  interface: [URLPattern]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 12.

## URLP1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**URLPattern** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 12)

Witnessed by 12 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/urlpattern/urlpattern.test.ts:99` — URLPattern > WPT tests → `expect(pattern[component]).toBe(expected)`
- `test/js/web/urlpattern/urlpattern.test.ts:110` — URLPattern > WPT tests → `expect(pattern.test(...(entry.inputs ?? []))).toBe(!!entry.expected_match)`
- `test/js/web/urlpattern/urlpattern.test.ts:166` — URLPattern > hasRegExpGroups > match-everything pattern → `expect(new URLPattern({}).hasRegExpGroups).toBe(false)`

