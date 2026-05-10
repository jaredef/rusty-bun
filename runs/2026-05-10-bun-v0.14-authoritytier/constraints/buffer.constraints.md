# buffer — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: buffer-surface-property
  threshold: BUFF1
  interface: [buffer, buffer.Buffer.isBuffer, buffer.INSPECT_MAX_BYTES]

@imports: []

@pins: []

Surface drawn from 5 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 26.

## BUFF1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**buffer** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 5 constraint clauses across 4 test files. Antichain representatives:

- `test/regression/issue/24575.test.ts:101` — socket._handle.fd should remain consistent during connection lifetime → `expect(buffer).toBe("message1\nmessage2\nmessage3\n")`
- `test/js/valkey/valkey.test.ts:529` — Basic Operations > should get value as Buffer with getBuffer → `expect(buffer?.toString()).toBe("test-value")`
- `test/js/node/fs/fs.test.ts:3260` — fs.read > should work with (fd, callback) → `expect(buffer).toStrictEqual(Buffer.concat([Buffer.from("bun"), Buffer.alloc(16381)]))`

## BUFF2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**buffer.Buffer.isBuffer** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/zlib/zlib.test.js:127` — zlib.gunzip > should be able to unzip a Buffer and return an unzipped Buffer → `expect(buffer.Buffer.isBuffer(data)).toBe(true)`

## BUFF3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**buffer.INSPECT_MAX_BYTES** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/buffer-inspectmaxbytes.test.ts:7` — buffer.INSPECT_MAX_BYTES is a number and not a custom getter/setter → `expect(buffer.INSPECT_MAX_BYTES).toBeNumber()`

## BUFF4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**buffer.toString** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/fetch/fetch.stream.test.ts:406` — fetch() with streaming > can handle multiple simultaneos requests → `expect(buffer.toString("utf8")).toBe(content)`
- `test/js/node/buffer.test.js:446` — UTF-8 write() & slice() → `expect(slice).toBe(testValue)`
- `test/js/bun/shell/commands/yes.test.ts:10` — yes > can pipe to a buffer → `expect(buffer.toString()).toEqual("y\ny\ny\ny\ny\n")`

## BUFF5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**buffer.slice** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:3181` — fs.write > should work with (fd, buffer, offset, length, callback) → `expect(buffer.slice(0, written).toString()).toStrictEqual("bun")`
- `test/js/node/fs/fs.test.ts:3280` — fs.read > should work with (fd, options, callback) → `expect(buffer.slice(0, bytesRead).toString()).toStrictEqual("bun")`
- `test/js/node/fs/fs.test.ts:3300` — fs.read > should work with (fd, buffer, offset, length, position, callback) → `expect(buffer.slice(0, bytesRead).toString()).toStrictEqual("bun")`

