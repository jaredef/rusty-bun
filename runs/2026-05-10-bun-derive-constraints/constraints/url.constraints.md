# URL — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: url-surface-property
  threshold: URL1
  interface: [URL, URL, URL.createObjectURL]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 221.

## URL1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**URL** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 219 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:6` — exists → `expect(typeof URL !== "undefined").toBe(true)`
- `test/js/web/url/url.test.ts:24` — url > should have correct origin and protocol → `expect(url.protocol).toBe("https:")`
- `test/js/web/fetch/fetch.test.ts:1949` — should allow very long redirect URLS → `expect(url).toBe('${server.url.origin}${Location}')`

## URL2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**URL** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/s3/s3-requester-pays.test.ts:249` — s3 - Requester Pays > should NOT include x-amz-request-payer in presigned URLs when reques… → `expect(url.searchParams.get("x-amz-request-payer")).toBeNull()`

## URL3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**URL.createObjectURL** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/buffer-resolveObjectURL.test.ts:11` — buffer.resolveObjectURL → `expect(id).toBeString()`

