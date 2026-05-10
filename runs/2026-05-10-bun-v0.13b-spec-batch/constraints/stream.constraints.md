# stream — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: stream-surface-property
  threshold: STRE1
  interface: [stream, stream.text, stream.json, stream.getReader, stream.locked]

@imports: []

@pins: []

Surface drawn from 5 candidate properties across the Bun test corpus. Construction-style: 5; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 34.

## STRE1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**stream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 27 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:1411` — Response > should consume body correctly > with Bun.file() streams → `expect(stream instanceof ReadableStream).toBe(true)`
- `test/js/node/fs/fs.test.ts:2171` — fs.WriteStream > should be constructable → `expect(stream instanceof fs.WriteStream).toBe(true)`
- `test/js/node/fs/fs.test.ts:2197` — fs.WriteStream > should work if re-exported by name → `expect(stream instanceof WriteStream_).toBe(true)`

## STRE2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**stream.text** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/fetch/utf8-bom.test.ts:131` — UTF-8 BOM should be ignored > readable stream > in ReadableStream.prototype.text() → `expect(await stream.text()).toBe("Hello, World!")`
- `test/js/web/fetch/body.test.ts:528` — body → `expect(await stream.text()).toBe("bun")`
- `test/js/web/fetch/body-clone.test.ts:399` — ReadableStream with mixed content (starting with string) can be converted to text → `expect(typeof text).toBe("string")`

## STRE3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**stream.json** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/stream-fast-path.test.ts:44` — ByteBlobLoader > json → `expect(result.then).toBeFunction()`
- `test/js/web/fetch/stream-fast-path.test.ts:53` — ByteBlobLoader > returns a rejected Promise for invalid JSON → `expect(result.then).toBeFunction()`

## STRE4
type: specification
authority: derived
scope: module
status: active
depends-on: []

**stream.getReader** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/29225.test.ts:131` — instanceof and prototype identity still work → `expect(reader).toBeInstanceOf(ReadableStreamBYOBReader)`

## STRE5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**stream.locked** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/body.test.ts:527` — body → `expect(stream.locked).toBe(false)`

