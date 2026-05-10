# FormData — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: formdata-surface-property
  threshold: FORM1
  interface: [FormData, FormData]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 2; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 16.

## FORM1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**FormData** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 15 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:21` — exists → `expect(typeof FormData !== "undefined").toBe(true)`
- `test/js/web/html/FormData.test.ts:8` — FormData > should be able to append a string → `expect(formData.get("foo")).toBe("bar")`
- `test/js/web/html/FormData.test.ts:9` — FormData > should be able to append a string → `expect(formData.getAll("foo")[0]).toBe("bar")`

## FORM2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**FormData** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/html/FormData.test.ts:40` — FormData > should get filename from file → `expect(formData.get("foo").name).toBeUndefined()`

