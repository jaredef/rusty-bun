# TextDecoder — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: textdecoder-surface-property
  threshold: TEXT1
  interface: [TextDecoder, TextDecoder, TextDecoder.prototype.decode]

@imports: []

@pins: []

Surface drawn from 7 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 4. Total witnessing constraint clauses: 146.

## TEXT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**TextDecoder** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `text-decoder.spec.md:13` — TextDecoder constructor label resolution → `TextDecoder constructor with unknown label throws RangeError`

## TEXT2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**TextDecoder** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `text-decoder.spec.md:7` — TextDecoder is exposed as a global constructor → `TextDecoder is defined as a global constructor in any execution context with [Exposed=*]`

## TEXT3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**TextDecoder.prototype.decode** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `text-decoder.spec.md:35` — TextDecoder.prototype.decode method → `TextDecoder.prototype.decode with fatal true throws TypeError on invalid byte sequence`

## TEXT4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**TextDecoder** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 99)

Witnessed by 99 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/streams/streams.test.js:611` — readableStreamToArrayBuffer (bytes) → `expect(new TextDecoder().decode(new Uint8Array(buffer))).toBe("abdefgh")`
- `test/js/web/encoding/text-encoder.test.js:158` — TextEncoder > should encode long latin1 text → `expect(decoded).toBe(text)`
- `test/js/web/encoding/text-decoder.test.js:41` — TextDecoder > should decode ascii text → `expect(decoder.encoding).toBe("windows-1252")`

## TEXT5
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**TextDecoder** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 34)

Witnessed by 34 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/pe-codesigning-integrity.test.ts:241` — should create valid PE executable with .bun section → `expect(embeddedText).toContain("B:/~BUN/root/")`
- `test/regression/issue/23474.test.ts:40` — request.cookies.set() should set websocket upgrade response cookie - issue #23474 → `expect(response).toContain("HTTP/1.1 101")`
- `test/js/web/fetch/body-clone.test.ts:428` — ReadableStream with mixed content (starting with Uint8Array) can be converted to ArrayBuff… → `expect(text).toContain("Hello")`

## TEXT6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**TextDecoder** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `text-decoder.spec.md:10` — TextDecoder is exposed as a global constructor → `new TextDecoder(label, options) accepts a TextDecoderOptions dictionary`
- `text-decoder.spec.md:14` — TextDecoder constructor label resolution → `TextDecoder constructor accepts label "utf-8" and resolves to "utf-8"`
- `text-decoder.spec.md:15` — TextDecoder constructor label resolution → `TextDecoder constructor accepts label "utf8" and resolves to "utf-8"`

## TEXT7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**TextDecoder.prototype.decode** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `text-decoder.spec.md:36` — TextDecoder.prototype.decode method → `TextDecoder.prototype.decode with fatal false replaces invalid byte sequences with U+FFFD`
- `text-decoder.spec.md:37` — TextDecoder.prototype.decode method → `TextDecoder.prototype.decode with ignoreBOM false consumes a leading UTF-8 BOM EF BB BF`
- `text-decoder.spec.md:38` — TextDecoder.prototype.decode method → `TextDecoder.prototype.decode with ignoreBOM true preserves a leading UTF-8 BOM as U+FEFF`

