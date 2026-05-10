# Blob — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: blob-surface-property
  threshold: BLOB1
  interface: [Blob, Blob, Blob]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 46.

## BLOB1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Blob** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 40 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:20` — exists → `expect(typeof Blob !== "undefined").toBe(true)`
- `test/js/web/streams/streams.test.js:670` — ReadableStream for Blob → `expect(await blob.text()).toBe("abdefghijklmnop")`
- `test/js/web/html/FormData.test.ts:222` — FormData > should roundtrip multipart/form-data (${name}) with ${C.name} → `expect(c instanceof FormData).toBe(true)`

## BLOB2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Blob** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/html/FormData.test.ts:50` — FormData > should use the correct filenames → `expect(blob.name).toBeUndefined()`
- `test/js/web/fetch/blob.test.ts:203` — blob: can set name property #10178 → `expect(blob.name).toBeUndefined()`
- `test/js/web/html/FormData.test.ts:53` — FormData > should use the correct filenames → `expect(blob.name).toBeUndefined()`

## BLOB3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Blob** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsc/native-constructor-identity.test.ts:81` — native constructor identity survives ICF > Bun native class constructors remain distinct → `expect(new Blob([])).toBeInstanceOf(Blob)`

