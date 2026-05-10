# Event — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: event-surface-property
  threshold: EVEN1
  interface: [Event]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 30.

## EVEN1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Event** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 30 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:9` — exists → `expect(typeof Event !== "undefined").toBe(true)`
- `test/js/deno/event/event.test.ts:9` —  → `assertEquals(event.isTrusted, false)`
- `test/js/deno/event/event-target.test.ts:190` —  → `assertEquals(event.target, null)`

