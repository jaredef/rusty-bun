# stream — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: stream-surface-property
  threshold: STRE1
  interface: [stream, stream.json, stream.text, stream.getReader]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 4; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 7.

## STRE1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**stream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process-stdio-invalid-utf16.test.ts:87` — trailing unpaired high surrogate should not duplicate output → `expect(output).toBe("Help�\nTest�\n")`
- `test/js/node/process/process-stdio-invalid-utf16.test.ts:365` — large strings with trailing unpaired surrogates → `expect(output.endsWith("�\n")).toBe(true)`

## STRE2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**stream.json** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/stream-fast-path.test.ts:44` — ByteBlobLoader > json → `expect(result.then).toBeFunction()`
- `test/js/web/fetch/stream-fast-path.test.ts:53` — ByteBlobLoader > returns a rejected Promise for invalid JSON → `expect(result.then).toBeFunction()`

## STRE3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**stream.text** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/fetch/body-clone.test.ts:399` — ReadableStream with mixed content (starting with string) can be converted to text → `expect(typeof text).toBe("string")`
- `test/js/bun/stream/direct-readable-stream.test.tsx:160` — (stream).text() → `expect(text.replaceAll("<!-- -->", "")).toBe(inputString)`

## STRE4
type: specification
authority: derived
scope: module
status: active
depends-on: []

**stream.getReader** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/29225.test.ts:131` — instanceof and prototype identity still work → `expect(reader).toBeInstanceOf(ReadableStreamBYOBReader)`

