# String — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: string-surface-property
  threshold: STRI1
  interface: [String.fromCharCode, String, String, String]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 4. Total witnessing constraint clauses: 31.

## STRI1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**String.fromCharCode** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 12)

Witnessed by 12 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/webview/webview-chrome.test.ts:197` — chrome: screenshot format options produce the right magic bytes → `expect(String.fromCharCode(wb[0], wb[1], wb[2], wb[3])).toBe("RIFF")`
- `test/js/bun/image/image.test.ts:193` — Bun.Image > PNG → PNG round-trip preserves every pixel → `expect(String.fromCharCode(out[1], out[2], out[3])).toBe("PNG")`
- `test/js/bun/webview/webview-chrome.test.ts:198` — chrome: screenshot format options produce the right magic bytes → `expect(String.fromCharCode(wb[8], wb[9], wb[10], wb[11])).toBe("WEBP")`

## STRI2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**String** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/25231.test.ts:17` — Bun.FFI.CString is callable with new → `expect(String(result)).toBe("hello")`
- `test/js/node/stream/node-stream-uint8array.test.ts:29` — Writable > should perform simple operations → `expect(String(chunk)).toBe("ABC")`
- `test/js/deno/url/url.test.ts:25` —  → `assertEquals(String(url), "https://foo:bar@baz.qat:8000/qux/quux?foo=bar&baz=12#qat")`

## STRI3
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**String** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/fetch/fetch-http2-adversarial.test.ts:219` — fetch() HTTP/2 adversarial > server that closes without sending SETTINGS fails the request… → `expect(String(code)).toMatch(/Connection|ECONNRESET|HTTP2|SocketClosed/i)`
- `test/js/bun/test/snapshot-tests/snapshots/snapshot.test.ts:56` — most types → `expect(s).toMatchSnapshot("String with property")`
- `test/js/bun/test/snapshot-tests/snapshots/more.test.ts:21` — d0 > snapshot serialize edgecases → `expect(new String()).toMatchSnapshot()`

## STRI4
type: specification
authority: derived
scope: module
status: active
depends-on: []

**String** — exposes values of the expected type or class. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/test/jest-extended.test.js:559` — jest-extended > toBeString() → `expect(new String()).toBeString()`
- `test/js/bun/test/expect.test.js:3991` — expect() > toBeString() → `expect(new String()).toBeString()`
- `test/js/bun/test/jest-extended.test.js:560` — jest-extended > toBeString() → `expect(new String("123")).toBeString()`

