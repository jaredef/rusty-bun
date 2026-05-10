# Headers — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: headers-surface-property
  threshold: HEAD1
  interface: [Headers.prototype.append, Headers, Headers.prototype.set]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 4.

## HEAD1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Headers.prototype.append** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `headers.spec.md:13` — Headers.prototype.append → `Headers.prototype.append throws TypeError on invalid header name`
- `headers.spec.md:14` — Headers.prototype.append → `Headers.prototype.append throws TypeError on invalid header value`

## HEAD2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Headers** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `headers.spec.md:7` — Headers is exposed as a global constructor → `Headers is defined as a global constructor in any execution context with [Exposed=*]`

## HEAD3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Headers.prototype.set** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `headers.spec.md:36` — Headers.prototype.set → `Headers.prototype.set throws TypeError on invalid header name or value`

