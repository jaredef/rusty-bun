# fetch — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: fetch-surface-property
  threshold: FETC1
  interface: [fetch, fetch, fetch]

@imports: []

@pins: []

Surface drawn from 5 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 886.

## FETC1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fetch** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 806 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/test-21049.test.ts:29` — fetch with Request object respects redirect: 'manual' option → `expect(directResponse.status).toBe(302)`
- `test/regression/issue/server-stop-with-pending-requests.test.ts:48` — server still works normally after jsref changes → `expect(response.status).toBe(200)`
- `test/regression/issue/29371.test.ts:59` — proxy request-line omits default :80 for http:// without explicit port → `expect(res.status).toBe(200)`

## FETC2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fetch** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 9 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/22353.test.ts:26` — issue #22353 - server should handle oversized request without crashing → `expect(await resp.text()).toBeEmpty()`
- `test/js/web/fetch/fetch.test.ts:623` — fetch > redirect: "error" #2819 → `expect(response).toBeUndefined()`
- `test/js/bun/util/zstd.test.ts:389` — Zstandard HTTP compression > doesn't use zstd when not in Accept-Encoding → `expect(response.headers.get("Content-Encoding")).toBeNull()`

## FETC3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fetch** — exposes values of the expected type or class. (construction-style)

Witnessed by 5 constraint clauses across 4 test files. Antichain representatives:

- `test/js/web/fetch/fetch.upgrade.test.ts:40` — fetch upgrade > should upgrade to websocket → `expect(res.headers.get("sec-websocket-accept")).toBeString()`
- `test/js/node/http/node-fetch-primordials.test.ts:29` — fetch, Response, Request can be overriden → `expect(response).toBeInstanceOf(Response)`
- `test/js/node/http/node-fetch-cjs.test.js:12` — require('node-fetch') fetches → `expect(await fetch("http://" + server.hostname + ":" + server.port)).toBeInstanceOf(Response)`

## FETC4
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**fetch** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 44)

Witnessed by 44 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:1973` — 304 not modified with missing content-length does not cause a request timeout → `expect(await response.arrayBuffer()).toHaveLength(0)`
- `test/js/web/fetch/fetch-http3-client.test.ts:207` — fetch protocol: http3 > JSON + query string → `expect(res.headers.get("content-type")).toContain("application/json")`
- `test/js/bun/http/serve-if-none-match.test.ts:45` — If-None-Match Support > ETag Generation > should automatically generate ETag for static re… → `expect(res.headers.get("ETag")).toMatch(/^"[a-f0-9]+"$/)`

## FETC5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fetch** — exhibits the property captured in the witnessing test. (behavioral; cardinality 22)

Witnessed by 22 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/fetch/body-stream.test.ts:77` — Should not crash when not returning a promise when stream is in progress → `expect(await fetch(url).then(res => res.text())).toBeOneOf(["hey", ""])`
- `test/js/bun/s3/s3.test.ts:283` — ${credentials.service} > fetch > should be able to set content-type → `expect(response.headers.get("content-type")).toStartWith("application/json")`
- `test/js/bun/http/serve.test.ts:1449` — #5859 json → `expect(response.ok).toBeFalse()`

