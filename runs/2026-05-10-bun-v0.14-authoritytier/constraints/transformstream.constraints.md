# TransformStream — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: transformstream-surface-property
  threshold: TRAN1
  interface: [TransformStream, TransformStream]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 2; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 2.

## TRAN1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**TransformStream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/streams/streams.test.js:477` — exists globally → `expect(typeof TransformStream).toBe("function")`

## TRAN2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**TransformStream** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `streams.spec.md:87` — TransformStream is exposed as a global constructor → `TransformStream is defined as a global constructor in any execution context with [Exposed=*]`

