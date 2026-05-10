# Event — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: event-surface-property
  threshold: EVEN1
  interface: [Event]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 31.

## EVEN1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Event** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 31 constraint clauses across 2 test files. Antichain representatives:

- `tests/unit/event_test.ts:8` —  → `assertEquals(event.isTrusted, false)`
- `tests/unit/event_target_test.ts:313` —  → `assertEquals(event.target, null)`
- `tests/unit/event_test.ts:9` —  → `assertEquals(event.target, null)`

