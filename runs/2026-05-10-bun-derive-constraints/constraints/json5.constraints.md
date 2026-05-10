# JSON5 — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: json5-surface-property
  threshold: JSON1
  interface: [JSON5.parse, JSON5.stringify]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 322.

## JSON1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**JSON5.parse** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 246)

Witnessed by 246 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/json5/json5.test.ts:11` — escape sequences > \\v vertical tab → `expect(parsed).toEqual(expected)`
- `test/js/bun/json5/json5-test-suite.test.ts:11` — arrays > empty array → `expect(parsed).toEqual(expected)`
- `test/js/bun/json5/json5.test.ts:18` — escape sequences > \\0 null character → `expect(parsed).toEqual(expected)`

## JSON2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**JSON5.stringify** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 76)

Witnessed by 76 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/json5/json5.test.ts:672` — stringify > stringifies null → `expect(JSON5.stringify(null)).toEqual("null")`
- `test/js/bun/json5/json5.test.ts:676` — stringify > stringifies booleans → `expect(JSON5.stringify(true)).toEqual("true")`
- `test/js/bun/json5/json5.test.ts:677` — stringify > stringifies booleans → `expect(JSON5.stringify(false)).toEqual("false")`

