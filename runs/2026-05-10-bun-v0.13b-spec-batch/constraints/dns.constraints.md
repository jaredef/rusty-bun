# dns — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: dns-surface-property
  threshold: DNS1
  interface: [dns, dns.lookup, dns.lookup, dns.lookupService, dns.promises, dns.promises.lookup, dns.promises.lookupService, dns.promises.resolve, dns.promises.resolve4, dns.promises.resolve6, dns.promises.resolveCaa, dns.promises.resolveCname, dns.promises.resolveMx, dns.promises.resolveNaptr, dns.promises.resolveNs, dns.promises.resolvePtr]

@imports: []

@pins: []

Surface drawn from 31 candidate properties across the Bun test corpus. Construction-style: 31; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 31.

## DNS1
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:15` — it exists → `expect(dns).toBeDefined()`

## DNS2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.lookup** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/dns/resolve-dns.test.ts:171` — dns > lookup with non-object second argument should not crash → `expect(result).toBeArray()`

## DNS3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.lookup** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:16` — it exists → `expect(dns.lookup).toBeDefined()`

## DNS4
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.lookupService** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:17` — it exists → `expect(dns.lookupService).toBeDefined()`

## DNS5
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:31` — it exists → `expect(dns.promises).toBeDefined()`

## DNS6
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.lookup** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:32` — it exists → `expect(dns.promises.lookup).toBeDefined()`

## DNS7
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.lookupService** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:33` — it exists → `expect(dns.promises.lookupService).toBeDefined()`

## DNS8
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolve** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:34` — it exists → `expect(dns.promises.resolve).toBeDefined()`

## DNS9
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolve4** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:35` — it exists → `expect(dns.promises.resolve4).toBeDefined()`

## DNS10
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolve6** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:36` — it exists → `expect(dns.promises.resolve6).toBeDefined()`

## DNS11
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveCaa** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:42` — it exists → `expect(dns.promises.resolveCaa).toBeDefined()`

## DNS12
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveCname** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:45` — it exists → `expect(dns.promises.resolveCname).toBeDefined()`

## DNS13
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveMx** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:41` — it exists → `expect(dns.promises.resolveMx).toBeDefined()`

## DNS14
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveNaptr** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:40` — it exists → `expect(dns.promises.resolveNaptr).toBeDefined()`

## DNS15
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveNs** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:43` — it exists → `expect(dns.promises.resolveNs).toBeDefined()`

## DNS16
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolvePtr** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:44` — it exists → `expect(dns.promises.resolvePtr).toBeDefined()`

## DNS17
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveSoa** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:39` — it exists → `expect(dns.promises.resolveSoa).toBeDefined()`

## DNS18
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveSrv** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:37` — it exists → `expect(dns.promises.resolveSrv).toBeDefined()`

## DNS19
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveTxt** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:38` — it exists → `expect(dns.promises.resolveTxt).toBeDefined()`

## DNS20
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolve** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:18` — it exists → `expect(dns.resolve).toBeDefined()`

## DNS21
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolve4** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:19` — it exists → `expect(dns.resolve4).toBeDefined()`

## DNS22
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolve6** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:20` — it exists → `expect(dns.resolve6).toBeDefined()`

## DNS23
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveCaa** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:26` — it exists → `expect(dns.resolveCaa).toBeDefined()`

## DNS24
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveCname** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:29` — it exists → `expect(dns.resolveCname).toBeDefined()`

## DNS25
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveMx** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:25` — it exists → `expect(dns.resolveMx).toBeDefined()`

## DNS26
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveNaptr** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:24` — it exists → `expect(dns.resolveNaptr).toBeDefined()`

## DNS27
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveNs** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:27` — it exists → `expect(dns.resolveNs).toBeDefined()`

## DNS28
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolvePtr** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:28` — it exists → `expect(dns.resolvePtr).toBeDefined()`

## DNS29
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveSoa** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:23` — it exists → `expect(dns.resolveSoa).toBeDefined()`

## DNS30
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveSrv** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:21` — it exists → `expect(dns.resolveSrv).toBeDefined()`

## DNS31
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveTxt** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:22` — it exists → `expect(dns.resolveTxt).toBeDefined()`

