# FormData — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: formdata-surface-property
  threshold: FORM1
  interface: [FormData, FormData]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 2; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 8.

## FORM1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**FormData** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 7 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:21` — exists → `expect(typeof FormData !== "undefined").toBe(true)`
- `test/js/web/html/FormData.test.ts:536` — FormData > non-standard extensions > should support .length → `expect(formData.length).toBe(3)`
- `form-data.spec.md:8` — FormData is exposed as a global constructor → `new FormData() returns an empty FormData instance`

## FORM2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**FormData** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `form-data.spec.md:7` — FormData is exposed as a global constructor → `FormData is defined as a global constructor in any execution context with [Exposed=*]`

