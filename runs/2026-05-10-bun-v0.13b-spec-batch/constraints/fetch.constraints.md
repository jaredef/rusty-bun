# fetch — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: fetch-surface-property
  threshold: FETC1
  interface: [fetch, fetch, fetch]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 467.

## FETC1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fetch** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 455 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/test-21049.test.ts:29` — fetch with Request object respects redirect: 'manual' option → `expect(directResponse.status).toBe(302)`
- `test/regression/issue/server-stop-with-pending-requests.test.ts:48` — server still works normally after jsref changes → `expect(response.status).toBe(200)`
- `test/regression/issue/29371.test.ts:59` — proxy request-line omits default :80 for http:// without explicit port → `expect(res.status).toBe(200)`

## FETC2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fetch** — exposes values of the expected type or class. (construction-style)

Witnessed by 4 constraint clauses across 3 test files. Antichain representatives:

- `test/js/node/http/node-fetch-primordials.test.ts:29` — fetch, Response, Request can be overriden → `expect(response).toBeInstanceOf(Response)`
- `test/js/node/http/node-fetch-cjs.test.js:12` — require('node-fetch') fetches → `expect(await fetch("http://" + server.hostname + ":" + server.port)).toBeInstanceOf(Response)`
- `test/js/bun/http/serve.test.ts:2055` — allow requestIP after async operation → `expect(ip.address).toBeString()`

## FETC3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fetch** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:623` — fetch > redirect: "error" #2819 → `expect(response).toBeUndefined()`

## FETC4
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**fetch** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/http/bun-serve-html.test.ts:261` — serve html → `expect(await (await fetch('http://${hostname}:${port}/a-different-url')).text()).toMatchInlineSnapshot( '"Hello World"', )`
- `test/js/bun/http/bun-serve-html-entry.test.ts:474` — bun *.html → `expect(homeJs).toContain('document.getElementById("counter")')`
- `test/js/bun/http/bun-serve-html-entry.test.ts:475` — bun *.html → `expect(homeJs).toContain("Click me:")`

