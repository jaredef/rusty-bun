# Headers — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: headers-surface-property
  threshold: HEAD1
  interface: [Headers, Headers, Headers.prototype]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 90.

## HEAD1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Headers** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 74 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:14` — exists → `expect(typeof Headers !== "undefined").toBe(true)`
- `test/js/web/fetch/headers.undici.test.ts:99` — Headers append > adds valid header entry to instance → `expect(headers.get(name)).toBe(value)`
- `test/js/web/fetch/headers.test.ts:24` — Headers > constructor > can create headers from object → `expect(headers.get("content-type")).toBe("text/plain")`

## HEAD2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Headers** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 10 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/fetch/headers.undici.test.ts:134` — Headers delete > deletes valid header entry from instance → `expect(headers.get(name)).toBeNull()`
- `test/js/web/fetch/headers.test.ts:36` — Headers > constructor > deleted key in header constructor is not kept → `expect(headers.get("content-type")).toBeNull()`
- `test/js/web/fetch/headers.undici.test.ts:158` — Headers delete > `Headers#delete` returns undefined → `expect(headers.delete("test")).toBeUndefined()`

## HEAD3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Headers.prototype** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/http/node-fetch.test.js:21` — node-fetch → `expect(Headers.prototype).toBeInstanceOf(globalThis.Headers)`

## HEAD4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Headers** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/deno/fetch/headers.test.ts:71` —  → `assert(headers.has(name), "headers has name " + name)`
- `test/js/deno/fetch/headers.test.ts:78` —  → `assert(headers.has(name), "headers have a header: " + name)`
- `test/js/deno/fetch/headers.test.ts:96` —  → `assert(headers.has(key))`

