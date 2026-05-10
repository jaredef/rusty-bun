# constructor+handle/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: constructor-handle-js-surface-property
  threshold: CONS1
  interface: [Bun.deepMatch, Bun.cron.parse, ReadableStream, Blob, Buffer.prototype.inspect, Buffer.prototype.offset, Buffer.prototype.parent, Buffer.prototype.toLocaleString, Request, Response]

@imports: []

@pins: []

Surface drawn from 27 candidate properties across the Bun test corpus. Construction-style: 10; behavioral (high-cardinality): 17. Total witnessing constraint clauses: 1576.

## CONS1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.deepMatch** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 9 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/test/expect.test.js:3309` — expect() > toMatchObject > with Bun.deepMatch → `expect(Bun.deepMatch({ a: 1, b: 2 }, { a: 1 })).toBe(false)`
- `test/js/bun/bun-object/deep-match.spec.ts:144` — Bun.deepMatch > When comparing same-shape objects with different constructors, returns tru… → `expect(Bun.deepMatch(new Foo(), new Bar())).toBe(true)`
- `test/js/bun/test/expect.test.js:3310` — expect() > toMatchObject > with Bun.deepMatch → `expect(Bun.deepMatch({ a: 1 }, { a: 1, b: 2 })).toBe(true)`

## CONS2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.cron.parse** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/cron/cron.test.ts:1702` — Bun.cron.parse > impossible expression (Feb 30) returns null → `expect(Bun.cron.parse("0 0 30 2 *", Date.UTC(2025, 0, 1, 0, 0, 0))).toBeNull()`
- `test/js/bun/cron/cron-parse.test.ts:49` — Bun.cron.parse — UTC > impossible day/month (Feb 30) returns null quickly → `expect(Bun.cron.parse("0 0 30 2 *", new Date("2026-01-01T00:00:00Z"))).toBeNull()`

## CONS3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ReadableStream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/streams/streams.test.js:472` — exists globally → `expect(typeof ReadableStream).toBe("function")`
- `streams.spec.md:8` — ReadableStream is exposed as a global constructor → `new ReadableStream() returns a default-constructed ReadableStream`

## CONS4
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Blob** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsc/native-constructor-identity.test.ts:81` — native constructor identity survives ICF > Bun native class constructors remain distinct → `expect(new Blob([])).toBeInstanceOf(Blob)`

## CONS5
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Buffer.prototype.inspect** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/buffer.test.js:3013` — inspect() should exist → `expect(Buffer.prototype.inspect).toBeInstanceOf(Function)`

## CONS6
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Buffer.prototype.offset** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/buffer.test.js:1312` — prototype getters should not throw → `expect(Buffer.prototype.offset).toBeUndefined()`

## CONS7
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Buffer.prototype.parent** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/buffer.test.js:1311` — prototype getters should not throw → `expect(Buffer.prototype.parent).toBeUndefined()`

## CONS8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Buffer.prototype.toLocaleString** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/buffer.test.js:1356` — toLocaleString() → `expect(Buffer.prototype.toLocaleString).toBe(Buffer.prototype.toString)`

## CONS9
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Request** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsc/native-constructor-identity.test.ts:77` — native constructor identity survives ICF > Bun native class constructors remain distinct → `expect(new Request("http://x/")).toBeInstanceOf(Request)`

## CONS10
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Response** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsc/native-constructor-identity.test.ts:79` — native constructor identity survives ICF > Bun native class constructors remain distinct → `expect(new Response("")).toBeInstanceOf(Response)`

## CONS11
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Glob** — exhibits the property captured in the witnessing test. (behavioral; cardinality 1221)

Witnessed by 1221 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/glob/match.test.ts:88` — Glob.match > no early globstar lock-in → `expect(new Glob('**/*abc*').match('a/abc')).toBeTrue()`
- `test/js/bun/glob/match.test.ts:89` — Glob.match > no early globstar lock-in → `expect(new Glob('**/*.js').match('a/b.c/c.js')).toBeTrue()`
- `test/js/bun/glob/match.test.ts:90` — Glob.match > no early globstar lock-in → `expect(new Glob("/**/*a").match("/a/a")).toBeTrue()`

## CONS12
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

## CONS13
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Glob** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 86)

Witnessed by 86 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/glob/match.test.ts:400` — Glob.match > ported from micromatch / glob-match / globlin tests > basic → `expect(new Glob("abc").match("abc")).toBe(true)`
- `test/js/bun/glob/match.test.ts:401` — Glob.match > ported from micromatch / glob-match / globlin tests > basic → `expect(new Glob("*").match("abc")).toBe(true)`
- `test/js/bun/glob/match.test.ts:402` — Glob.match > ported from micromatch / glob-match / globlin tests > basic → `expect(new Glob("*").match("")).toBe(true)`

## CONS14
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Buffer.byteLength** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 48)

Witnessed by 48 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/buffer.test.js:1413` — Buffer → `expect(Buffer.byteLength(input)).toBe(good[i].length)`
- `test/js/node/buffer.test.js:1420` — Buffer.byteLength → `expect(Buffer.byteLength("😀😃😄😁😆😅😂🤣☺️😊😊😇")).toBe( new TextEncoder().encode("😀😃😄😁😆😅😂🤣☺️😊😊😇").byteLength, )`
- `test/js/node/buffer.test.js:2742` — Buffer.byteLength() → `expect(Buffer.byteLength("", undefined, true)).toBe(0)`

## CONS15
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**WebAssembly.Global** — exhibits the property captured in the witnessing test. (behavioral; cardinality 16)

Witnessed by 16 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/jest-extended.test.js:410` — jest-extended > toBeEven() → `expect(new WebAssembly.Global({ value: "i32", mutable: false }, 4).value).toBeEven()`
- `test/js/bun/test/jest-extended.test.js:412` — jest-extended > toBeEven() → `expect(new WebAssembly.Global({ value: "i32", mutable: true }, 2).value).toBeEven()`
- `test/js/bun/test/jest-extended.test.js:415` — jest-extended > toBeEven() → `expect(new WebAssembly.Global({ value: "i64", mutable: true }, -9223372036854775808n).value).toBeEven()`

## CONS16
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**extracted.slice** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 12)

Witnessed by 12 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/archive.test.ts:1364` — Bun.Archive > sparse files > extracts sparse file with small hole (< 1 tar block) → `expect(extracted.slice(0, 64)).toEqual(new Uint8Array(64).fill(0x41))`
- `test/js/bun/archive.test.ts:1365` — Bun.Archive > sparse files > extracts sparse file with small hole (< 1 tar block) → `expect(extracted.slice(64, 320)).toEqual(new Uint8Array(256).fill(0))`
- `test/js/bun/archive.test.ts:1366` — Bun.Archive > sparse files > extracts sparse file with small hole (< 1 tar block) → `expect(extracted.slice(320, 384)).toEqual(new Uint8Array(64).fill(0x42))`

## CONS17
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**URLPattern** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/urlpattern/urlpattern.test.ts:99` — URLPattern > WPT tests → `expect(pattern[component]).toBe(expected)`
- `test/js/web/urlpattern/urlpattern.test.ts:166` — URLPattern > hasRegExpGroups > match-everything pattern → `expect(new URLPattern({}).hasRegExpGroups).toBe(false)`
- `test/js/web/urlpattern/urlpattern.test.ts:171` — URLPattern > hasRegExpGroups > wildcard in ${component} → `expect(new URLPattern({ [component]: "*" }).hasRegExpGroups).toBe(false)`

## CONS18
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**MessageEvent** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/workers/message-event.test.ts:17` — MessageEvent constructor > has the right defaults → `expect(new MessageEvent("custom type")).toMatchObject(expected)`
- `test/js/web/workers/message-event.test.ts:18` — MessageEvent constructor > has the right defaults → `expect(new MessageEvent("custom type", undefined)).toMatchObject(expected)`
- `test/js/web/workers/message-event.test.ts:19` — MessageEvent constructor > has the right defaults → `expect(new MessageEvent("custom type", {})).toMatchObject(expected)`

## CONS19
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**s.rooms** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 2 test files. Antichain representatives:

- `test/js/third_party/socket.io/socket.io-namespaces.test.ts:267` — namespaces > should fire a `disconnecting` event just before leaving all rooms → `expect(s.rooms).toStrictEqual(new Set([s.id, "a"]))`
- `test/js/third_party/socket.io/socket.io-messaging-many.test.ts:349` — messaging many > keeps track of rooms → `expect(s.rooms).toStrictEqual(new Set([s.id, "a"]))`
- `test/js/third_party/socket.io/socket.io-messaging-many.test.ts:351` — messaging many > keeps track of rooms → `expect(s.rooms).toStrictEqual(new Set([s.id, "a", "b"]))`

## CONS20
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Date** — exposes values of the expected type or class. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/test/jest-extended.test.js:258` — jest-extended > toBeDate() → `expect(new Date()).toBeDate()`
- `test/js/bun/test/expect.test.js:3972` — expect() > toBeDate() → `expect(new Date()).toBeDate()`
- `test/js/bun/test/jest-extended.test.js:259` — jest-extended > toBeDate() → `expect(new Date(0)).toBeDate()`

## CONS21
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ArrayBuffer.isView** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/buffer.test.js:2728` — ArrayBuffer.isView() → `expect(ArrayBuffer.isView(new Buffer(10))).toBe(true)`
- `test/js/node/buffer.test.js:2729` — ArrayBuffer.isView() → `expect(ArrayBuffer.isView(new SlowBuffer(10))).toBe(true)`
- `test/js/node/buffer.test.js:2730` — ArrayBuffer.isView() → `expect(ArrayBuffer.isView(Buffer.alloc(10))).toBe(true)`

## CONS22
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Map** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/test/expect.test.js:1237` — expect() > deepEquals set and map → `expect(e).toEqual(d)`
- `test/js/bun/jsc/native-constructor-identity.test.ts:38` — native constructor identity survives ICF > expect.any distinguishes builtin constructors w… → `expect(new Map()).toEqual(expect.any(Map))`
- `test/js/bun/test/expect.test.js:1238` — expect() > deepEquals set and map → `expect(d).toEqual(e)`

## CONS23
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

## CONS24
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Array** — exposes values of the expected type or class. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/test/jest-extended.test.js:198` — jest-extended > toBeArray() → `expect(new Array()).toBeArray()`
- `test/js/bun/test/expect.test.js:3874` — expect() > toBeObject() → `expect(new Array(0)).toBeObject()`
- `test/js/bun/test/jest-extended.test.js:199` — jest-extended > toBeArray() → `expect(new Array(1, 2, 3)).toBeArray()`

## CONS25
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/deno/fetch/response.test.ts:104` —  → `assert(response.bodyUsed)`
- `response.spec.md:9` — Response is exposed as a global constructor → `new Response(body) wraps body as the response body`
- `test/js/deno/fetch/response.test.ts:106` —  → `assert(response.bodyUsed)`

## CONS26
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

## CONS27
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**results.map** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/sql/sqlite-sql.test.ts:748` — Data Types & Values > handles INTEGER values → `expect(results.map(r => r.value)).toEqual(values)`
- `test/js/bun/cron/cron.test.ts:1434` — Bun.cron.parse > 0 0 * * MON,WED,FRI produces correct weekday sequence → `expect(results.map(t => new Date(t).getUTCDay())).toEqual([3, 5, 1, 3, 5])`
- `test/js/sql/sqlite-sql.test.ts:784` — Data Types & Values > handles TEXT values → `expect(results.map(r => r.value)).toEqual(values)`

