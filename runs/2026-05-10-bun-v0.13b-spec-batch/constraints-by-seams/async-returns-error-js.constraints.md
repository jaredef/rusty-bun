# async/returns-error/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: async-returns-error-js-surface-property
  threshold: ASYN1
  interface: [res.json]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 13.

## ASYN1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**res.json** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 4 test files. Antichain representatives:

- `test/js/web/fetch/fetch-keepalive.test.ts:23` — keepalive → `expect(headers.connection).toBe("keep-alive")`
- `test/js/web/fetch/fetch-http3-client.test.ts:206` — fetch protocol: http3 > JSON + query string → `expect(await res.json()).toEqual({ ok: true, q: "h3" })`
- `test/js/bun/http/bun-serve-routes.test.ts:37` — path parameters > handles single parameter → `expect(data).toEqual({ id: "123", method: "GET", })`

