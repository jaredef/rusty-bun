# assert — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: assert-surface-property
  threshold: ASSE1
  interface: [assert.CallTracker, assert.default.CallTracker]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 2; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 3.

## ASSE1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**assert.CallTracker** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/assert_test.ts:29` — [node/assert] CallTracker correctly exported → `assert.strictEqual(typeof assert.CallTracker, "function")`
- `tests/unit_node/assert_test.ts:31` — [node/assert] CallTracker correctly exported → `assert.strictEqual(assert.CallTracker, assert.default.CallTracker)`

## ASSE2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**assert.default.CallTracker** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/assert_test.ts:30` — [node/assert] CallTracker correctly exported → `assert.strictEqual(typeof assert.default.CallTracker, "function")`

