# URL — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: url-surface-property
  threshold: URL1
  interface: [URL]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 170.

## URL1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**URL** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 170)

Witnessed by 170 constraint clauses across 3 test files. Antichain representatives:

- `tests/unit/url_test.ts:13` —  → `assertEquals(url.hash, "#qat")`
- `tests/unit/serve_test.ts:452` —  → `assertEquals(new URL(request.url).href, 'http://127.0.0.1:${servePort}/')`
- `tests/unit/http_test.ts:67` —  → `assertEquals(new URL(request.url).href, 'http://127.0.0.1:${listenPort}/')`

