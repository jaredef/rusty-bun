# atob — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: atob-surface-property
  threshold: ATOB1
  interface: [atob, atob]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 2; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 4.

## ATOB1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**atob** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `atob-btoa.spec.md:11` — atob input validation → `atob throws DOMException InvalidCharacterError on non-Base64 characters`
- `atob-btoa.spec.md:20` — btoa input validation → `btoa throws DOMException InvalidCharacterError on input with code points above 0xFF`

## ATOB2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**atob** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `atob-btoa.spec.md:7` — atob is exposed as a global function → `atob is defined as a global function in any execution context with [Exposed=*]`
- `atob-btoa.spec.md:16` — btoa is exposed as a global function → `btoa is defined as a global function in any execution context with [Exposed=*]`

