# YAML — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: yaml-surface-property
  threshold: YAML1
  interface: [YAML.parse, YAML.stringify, YAML.stringify, YAML.parse]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 4. Total witnessing constraint clauses: 774.

## YAML1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**YAML.parse** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 506)

Witnessed by 506 constraint clauses across 3 test files. Antichain representatives:

- `test/regression/issue/23489.test.ts:9` — YAML double-quoted strings with ... should not trigger document end error - issue #23489 → `expect(result1).toEqual({ balance_dont_have_wallet: "👛 لا تمتلك محفظة... !", })`
- `test/js/bun/yaml/yaml.test.ts:11` — Bun.YAML > parse > input types > parses from Buffer → `expect(YAML.parse(buffer)).toEqual({ key: "value", number: 42 })`
- `test/js/bun/yaml/yaml-test-suite.test.ts:28` — yaml-test-suite/229Q → `expect(parsed).toEqual(expected)`

## YAML2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**YAML.stringify** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 234)

Witnessed by 234 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/yaml/yaml.test.ts:980` — Bun.YAML > stringify > stringifies null → `expect(YAML.stringify(null)).toBe("null")`
- `test/js/bun/yaml/yaml.test.ts:981` — Bun.YAML > stringify > stringifies null → `expect(YAML.stringify(undefined)).toBe(undefined)`
- `test/js/bun/yaml/yaml.test.ts:985` — Bun.YAML > stringify > stringifies booleans → `expect(YAML.stringify(true)).toBe("true")`

## YAML3
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**YAML.stringify** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 29)

Witnessed by 29 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/yaml/yaml.test.ts:1557` — Bun.YAML > stringify > edge cases > quotes strings ending with colons → `expect(yml).toContain('"tin:"')`
- `test/js/bun/yaml/yaml.test.ts:1714` — Bun.YAML > stringify > edge cases > handles Error objects → `expect(customResult).toContain("code: ERR_TEST")`
- `test/js/bun/yaml/yaml.test.ts:1715` — Bun.YAML > stringify > edge cases > handles Error objects → `expect(customResult).toContain("details:")`

## YAML4
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**YAML.parse** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/yaml/yaml.test.ts:796` — Bun.YAML > parse > issue 22659 → `expect(YAML.parse(input1)).toMatchInlineSnapshot(' [ { "test1": "+", "test2": "next", }, ] ')`
- `test/js/bun/yaml/yaml.test.ts:806` — Bun.YAML > parse > issue 22659 → `expect(YAML.parse(input2)).toMatchInlineSnapshot(' [ { "test1": "+", "test2": "next", }, ] ')`
- `test/js/bun/yaml/yaml.test.ts:822` — Bun.YAML > parse > issue 22392 → `expect(YAML.parse(input)).toMatchInlineSnapshot(' { "foo": "some ... string", } ')`

