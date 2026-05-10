# Headers — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: headers-surface-property
  threshold: HEAD1
  interface: [Headers, Headers.prototype.append, Headers, Headers.prototype, Headers.prototype.set]

@imports: []

@pins: []

Surface drawn from 5 candidate properties across the Bun test corpus. Construction-style: 5; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 15.

## HEAD1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Headers** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 10 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:14` — exists → `expect(typeof Headers !== "undefined").toBe(true)`
- `test/js/web/fetch/headers.test.ts:495` — Headers > count > can count headers when empty → `expect(headers.count).toBe(0)`
- `test/js/web/fetch/fetch.test.ts:412` — Headers > .getSetCookie() with object → `expect(headers.count).toBe(5)`

## HEAD2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Headers.prototype.append** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `headers.spec.md:13` — Headers.prototype.append → `Headers.prototype.append throws TypeError on invalid header name`
- `headers.spec.md:14` — Headers.prototype.append → `Headers.prototype.append throws TypeError on invalid header value`

## HEAD3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Headers** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `headers.spec.md:7` — Headers is exposed as a global constructor → `Headers is defined as a global constructor in any execution context with [Exposed=*]`

## HEAD4
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Headers.prototype** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/http/node-fetch.test.js:21` — node-fetch → `expect(Headers.prototype).toBeInstanceOf(globalThis.Headers)`

## HEAD5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Headers.prototype.set** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `headers.spec.md:36` — Headers.prototype.set → `Headers.prototype.set throws TypeError on invalid header name or value`

