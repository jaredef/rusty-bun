# AbortSignal — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: abortsignal-surface-property
  threshold: ABOR1
  interface: [AbortSignal.abort]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 2.

## ABOR1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal.abort** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/abort_controller_test.ts:62` —  → `assertEquals(signal.aborted, true)`
- `tests/unit/abort_controller_test.ts:63` —  → `assertEquals(signal.reason, "hey!")`

