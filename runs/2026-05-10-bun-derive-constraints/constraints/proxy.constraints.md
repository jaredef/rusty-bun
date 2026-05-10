# Proxy — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: proxy-surface-property
  threshold: PROX1
  interface: [Proxy]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 13.

## PROX1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Proxy** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/isArray-proxy-crash.test.ts:81` — isArray + Proxy crash fixes > expect.arrayContaining does not crash with Proxy receiver → `expect(proxy).toEqual(expect.arrayContaining([1]))`
- `test/js/bun/test/expect.test.js:578` — expect() > deepEquals works with proxies → `expect(p1).toEqual(p2)`
- `test/js/bun/test/expect.test.js:579` — expect() > deepEquals works with proxies → `expect(p1).toStrictEqual(p2)`

