# @js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: js-surface-property
  threshold: JS1
  interface: [Buffer.from, Bun.JSONL.parseChunk, Bun.Cookie.parse, Bun.Image, Bun.Cookie.parse, WebSocket, Bun.cron.parse, btoa, Bun.JSONL.parseChunk, expect.any, Bun.randomUUIDv5, Bun.readableStreamToArray, Bun.spawn, expect.not.objectContaining, Bun.Archive, Bun.JSONL.parseChunk]

@imports: []

@pins: []

Surface drawn from 80 candidate properties across the Bun test corpus. Construction-style: 80; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 959.

## JS1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Buffer.from** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 333 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/fetch/fetch.stream.test.ts:993` — fetch() with streaming > Content-Length response works (multiple parts) with ${compression… → `expect(contentBuffer.compare(value, undefined, undefined, currentRange, currentRange + value.length)).toEqual( 0, )`
- `test/js/web/fetch/fetch-http3-client.test.ts:220` — fetch protocol: http3 > large response body (multi-packet) → `expect(Buffer.from(buf).equals(big)).toBe(true)`
- `test/js/web/fetch/fetch-gzip.test.ts:37` — fetch() with a buffered gzip response works (one chunk) → `expect(second.equals(clone)).toBe(true)`

## JS2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.JSONL.parseChunk** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 180 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsonl/jsonl-parse.test.ts:205` — Bun.JSONL > parse > error handling > parseChunk: error at every position reports correct r… → `expect(result.values.length).toBe(errPos)`
- `test/js/bun/jsonl/jsonl-parse.test.ts:207` — Bun.JSONL > parse > error handling > parseChunk: error at every position reports correct r… → `expect(result.done).toBe(false)`
- `test/js/bun/jsonl/jsonl-parse.test.ts:211` — Bun.JSONL > parse > error handling > parseChunk: error at every position reports correct r… → `expect(result.read).toBe(validPart.length)`

## JS3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.Cookie.parse** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 61 constraint clauses across 5 test files. Antichain representatives:

- `test/js/bun/util/cookie.test.js:47` — Bun.Cookie > parse a cookie string → `expect(cookie.name).toBe("name")`
- `test/js/bun/http/bun-serve-cookies.test.ts:506` — Direct usage of Bun.Cookie and Bun.CookieMap > can use Cookie.parse to parse cookie string… → `expect(cookie.constructor).toBe(Bun.Cookie)`
- `test/js/bun/cookie/cookie-security-fuzz.test.ts:28` — Bun.Cookie.parse security fuzz tests > resists cookie format injection attacks > additiona… → `expect(cookie.name).toBe("name")`

## JS4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.Image** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 42 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/image/image.test.ts:146` — Bun.Image > constructor exists and is exposed on Bun → `expect(typeof Bun.Image).toBe("function")`
- `test/js/bun/image/image-adversarial.test.ts:166` — format confusion > PNG with valid JPEG appended (polyglot-ish) → `expect(meta.format).toBe("png")`
- `test/js/bun/image/image.test.ts:186` — Bun.Image > metadata() reads PNG dimensions → `expect(img.width).toBe(4)`

## JS5
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.Cookie.parse** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 22 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/cookie/cookie-security-fuzz.test.ts:197` — Bun.Cookie.parse security fuzz tests > handles malicious MaxAge and Expires combinations → `expect(cookie).toBeDefined()`
- `test/js/bun/cookie/cookie-exotic-inputs.test.ts:18` — Bun.Cookie.parse with exotic inputs > handles cookies with various special characters in n… → `expect(cookie).toBeDefined()`
- `test/js/bun/cookie/cookie-security-fuzz.test.ts:224` — Bun.Cookie.parse security fuzz tests > handles SQL injection attempts in cookie values → `expect(cookie).toBeDefined()`

## JS6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**WebSocket** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 18 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/websocket/websocket.test.js:380` — WebSocket > nodebuffer > should support 'nodebuffer' binaryType → `expect(ws.binaryType).toBe("nodebuffer")`
- `test/js/web/websocket/websocket-unix.test.ts:53` — ws+unix:// echoes through Bun.serve({ unix }) → `expect(ws.url).toBe('ws+unix://${unix}')`
- `test/js/web/websocket/websocket-proxy.test.ts:98` — WebSocket proxy API > accepts proxy option as string (HTTP proxy) → `expect(ws.readyState).toBe(WebSocket.CONNECTING)`

## JS7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.cron.parse** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 17 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/cron/cron.test.ts:108` — Bun.cron API > has .parse method → `expect(typeof Bun.cron.parse).toBe("function")`
- `test/js/bun/cron/cron.test.ts:1392` — Bun.cron.parse > is a function that returns a Date → `expect(typeof Bun.cron.parse).toBe("function")`
- `test/js/bun/cron/cron.test.ts:1575` — Bun.cron.parse > full day names match 3-letter abbreviations → `expect(Bun.cron.parse('0 0 * * ${abbr}', from)!.getTime()).toBe( Bun.cron.parse('0 0 * * ${full}', from)!.getTime(), )`

## JS8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**btoa** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 16 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/util/atob.test.js:51` — btoa → `expect(btoa("a")).toBe("YQ==")`
- `test/js/deno/encoding/encoding.test.ts:8` —  → `assertEquals(encoded, "aGVsbG8gd29ybGQ=")`
- `test/js/web/util/atob.test.js:52` — btoa → `expect(btoa("ab")).toBe("YWI=")`

## JS9
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.JSONL.parseChunk** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 15 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsonl/jsonl-parse.test.ts:221` — Bun.JSONL > parse > error handling > parseChunk: error vs incomplete distinction → `expect(incomplete.error).toBeNull()`
- `test/js/bun/jsonl/jsonl-parse.test.ts:351` — Bun.JSONL > parseChunk > complete input > returns values, read, done, error → `expect(result.error).toBeNull()`
- `test/js/bun/jsonl/jsonl-parse.test.ts:359` — Bun.JSONL > parseChunk > complete input > single value without trailing newline → `expect(result.error).toBeNull()`

## JS10
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**expect.any** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 15 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/expect.test.js:4027` — expect() > asymmetric matchers > expect.any → `expect(expect.any(String)).toEqual("jest")`
- `test/js/bun/test/expect.test.js:4028` — expect() > asymmetric matchers > expect.any → `expect(expect.any(Number)).toEqual(1)`
- `test/js/bun/test/expect.test.js:4029` — expect() > asymmetric matchers > expect.any → `expect(expect.any(Function)).toEqual(() => {})`

## JS11
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.randomUUIDv5** — exposes values of the expected type or class. (construction-style)

Witnessed by 13 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/randomUUIDv5.test.ts:10` — randomUUIDv5 > basic functionality → `expect(result).toBeTypeOf("string")`
- `test/js/bun/util/randomUUIDv5.test.ts:48` — randomUUIDv5 > empty name → `expect(result).toBeTypeOf("string")`
- `test/js/bun/util/randomUUIDv5.test.ts:55` — randomUUIDv5 > long name → `expect(result).toBeTypeOf("string")`

## JS12
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.readableStreamToArray** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 13 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/encoding/text-encoder-stream.test.ts:136` —  → `expect(chunkArray.length, "number of chunks should match").toBe(output.length)`
- `test/js/web/encoding/text-decoder-stream.test.ts:28` —  → `expect(array, "the output should be in one chunk").toEqual([expectedOutputString])`
- `test/js/web/encoding/encode-bad-chunks.test.ts:71` — input of type ${input.name} should be converted correctly to string → `expect(output.length, "output should contain one chunk").toBe(1)`

## JS13
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.spawn** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 13 constraint clauses across 3 test files. Antichain representatives:

- `test/js/bun/terminal/terminal.test.ts:983` — Bun.spawn with terminal option > creates subprocess with terminal attached → `expect(proc.terminal).toBeDefined()`
- `test/js/bun/terminal/terminal-spawn.test.ts:121` — Bun.Terminal subprocess integration > Bun.spawn with inline terminal option → `expect(proc.terminal).toBeDefined()`
- `test/js/bun/terminal/terminal-platform-gaps.test.ts:319` — Bun.Terminal platform behaviour > SAME: exit callback fires after child exits (inline term… → `expect(proc.terminal).toBeDefined()`

## JS14
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**expect.not.objectContaining** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 13 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/expect.test.js:4253` — expect() > asymmetric matchers > ObjectNotContaining matches → `expect(expect.not.objectContaining({})).toEqual("jest")`
- `test/js/bun/test/expect.test.js:4254` — expect() > asymmetric matchers > ObjectNotContaining matches → `expect(expect.not.objectContaining({})).toEqual(null)`
- `test/js/bun/test/expect.test.js:4255` — expect() > asymmetric matchers > ObjectNotContaining matches → `expect(expect.not.objectContaining({})).toEqual(undefined)`

## JS15
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.Archive** — exposes values of the expected type or class. (construction-style)

Witnessed by 11 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/archive.test.ts:45` — Bun.Archive > new Archive() > creates archive from object with string values → `expect(archive).toBeInstanceOf(Bun.Archive)`
- `test/js/bun/archive.test.ts:54` — Bun.Archive > new Archive() > creates archive from object with Blob values → `expect(archive).toBeInstanceOf(Bun.Archive)`
- `test/js/bun/archive.test.ts:64` — Bun.Archive > new Archive() > creates archive from object with Uint8Array values → `expect(archive).toBeInstanceOf(Bun.Archive)`

## JS16
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.JSONL.parseChunk** — exposes values of the expected type or class. (construction-style)

Witnessed by 11 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsonl/jsonl-parse.test.ts:206` — Bun.JSONL > parse > error handling > parseChunk: error at every position reports correct r… → `expect(result.error).toBeInstanceOf(SyntaxError)`
- `test/js/bun/jsonl/jsonl-parse.test.ts:226` — Bun.JSONL > parse > error handling > parseChunk: error vs incomplete distinction → `expect(errored.error).toBeInstanceOf(SyntaxError)`
- `test/js/bun/jsonl/jsonl-parse.test.ts:245` — Bun.JSONL > parse > error handling > typed array: error at various positions → `expect(result.error).toBeInstanceOf(SyntaxError)`

## JS17
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.JSONC.parse** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 10 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsonc/jsonc.test.ts:6` — Bun.JSONC exists → `expect(typeof Bun.JSONC.parse).toBe("function")`
- `test/js/bun/jsonc/jsonc.test.ts:11` — Bun.JSONC.parse handles basic JSON → `expect(result).toEqual({ name: "test", value: 42 })`
- `test/js/bun/jsonc/jsonc.test.ts:23` — Bun.JSONC.parse handles comments → `expect(result).toEqual({ name: "test", value: 42 })`

## JS18
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Headers** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 10 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/fetch/headers.undici.test.ts:134` — Headers delete > deletes valid header entry from instance → `expect(headers.get(name)).toBeNull()`
- `test/js/web/fetch/headers.test.ts:36` — Headers > constructor > deleted key in header constructor is not kept → `expect(headers.get("content-type")).toBeNull()`
- `test/js/web/fetch/headers.undici.test.ts:158` — Headers delete > `Headers#delete` returns undefined → `expect(headers.delete("test")).toBeUndefined()`

## JS19
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.Cookie.from** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/http/bun-serve-cookies.test.ts:522` — Direct usage of Bun.Cookie and Bun.CookieMap > can use Cookie.from to create cookies → `expect(cookie.constructor).toBe(Bun.Cookie)`
- `test/js/bun/cookie/cookie-expires-validation.test.ts:123` — Bun.Cookie expires validation > Constructors and methods > handles valid expires in Cookie… → `expect(cookie.expires).toEqual(new Date(futureTimestamp * 1000))`
- `test/js/bun/http/bun-serve-cookies.test.ts:523` — Direct usage of Bun.Cookie and Bun.CookieMap > can use Cookie.from to create cookies → `expect(cookie.name).toBe("name")`

## JS20
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.posix.isAbsolute** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 3 test files. Antichain representatives:

- `test/js/node/path/zero-length-strings.test.js:29` — path > zero length strings → `assert.strictEqual(path.posix.isAbsolute(""), false)`
- `test/js/node/path/is-absolute.test.js:28` — path.isAbsolute > posix → `assert.strictEqual(path.posix.isAbsolute("/home/foo"), true)`
- `test/js/node/path/browserify.test.js:918` — browserify path tests > isAbsolute > posix /foo/bar → `expect(path.posix.isAbsolute("/foo/bar")).toBe(true)`

## JS21
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.FileSystemRouter** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/filesystem_router.test.ts:366` — reload() works → `expect(router.match("/posts")!.name).toBe("/posts")`
- `test/js/bun/util/filesystem_router.test.ts:368` — reload() works → `expect(router.match("/posts")!.name).toBe("/posts")`
- `test/js/bun/util/filesystem_router.test.ts:381` — reload() works with new dirs/files → `expect(router.match("/posts")!.name).toBe("/posts")`

## JS22
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**expect.anything** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/expect.test.js:4101` — expect() > asymmetric matchers > Anything matches any type → `expect(expect.anything()).toEqual("jest")`
- `test/js/bun/test/expect.test.js:4102` — expect() > asymmetric matchers > Anything matches any type → `expect(expect.anything()).toEqual(1)`
- `test/js/bun/test/expect.test.js:4103` — expect() > asymmetric matchers > Anything matches any type → `expect(expect.anything()).toEqual(() => {})`

## JS23
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**globalThis** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:53` — self.${name} → `expect(globalThis[name]).toBe(callback)`
- `test/js/node/vm/vm.test.ts:465` — can modify global context → `expect(globalThis[props[1]]).toBe("baz")`
- `test/js/bun/globals.test.js:72` — writable → `expect(globalThis[name]).toBe(123)`

## JS24
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**http2.connect** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/http2/node-http2.test.js:599` — ${path.basename(nodeExecutable)} ${paddingStrategyName(paddingStrategy)} > Client Basics >… → `expect(client.destroyed).toBe(true)`
- `test/js/node/http2/node-http2.test.js:620` — ${path.basename(nodeExecutable)} ${paddingStrategyName(paddingStrategy)} > Client Basics >… → `expect(client.destroyed).toBe(true)`
- `test/js/node/http2/node-http2.test.js:690` — ${path.basename(nodeExecutable)} ${paddingStrategyName(paddingStrategy)} > Client Basics >… → `expect(client.destroyed).toBe(true)`

## JS25
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

## JS26
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.getRandomValues** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:121` — crypto.getRandomValues → `expect(array).toBe(foo)`
- `test/js/web/web-globals.test.js:122` — crypto.getRandomValues → `expect(array.reduce((sum, a) => (sum += a === 0), 0) != foo.length).toBe(true)`
- `test/js/web/web-globals.test.js:130` — crypto.getRandomValues → `expect(array).toBe(foo)`

## JS27
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Request** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:1467` — Request > clone → `expect(body.signal).toBeDefined()`
- `test/js/web/fetch/fetch.test.ts:1557` — body nullable → `expect(req.body).toBeNull()`
- `test/js/web/fetch/fetch.test.ts:1562` — body nullable → `expect(req.body).toBeNull()`

## JS28
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.deriveBits** — exposes values of the expected type or class. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/crypto/x25519-derive-bits.test.ts:23` — X25519 deriveBits with known test vector → `expect(bits).toBeInstanceOf(ArrayBuffer)`
- `test/js/bun/crypto/x25519-derive-bits.test.ts:33` — X25519 deriveBits with null length returns full output → `expect(bits).toBeInstanceOf(ArrayBuffer)`
- `test/js/bun/crypto/x25519-derive-bits.test.ts:43` — X25519 deriveBits with zero length returns full output → `expect(bits).toBeInstanceOf(ArrayBuffer)`

## JS29
type: specification
authority: derived
scope: module
status: active
depends-on: []

**http2.connect** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/http2/node-http2.test.js:818` — ${path.basename(nodeExecutable)} ${paddingStrategyName(paddingStrategy)} > Client Basics >… → `expect(client.alpnProtocol).toBeUndefined()`
- `test/js/node/http2/node-http2.test.js:825` — ${path.basename(nodeExecutable)} ${paddingStrategyName(paddingStrategy)} > Client Basics >… → `expect(client.remoteSettings).toBeNull()`
- `test/js/node/http2/node-http2.test.js:1698` — http2 server handles multiple concurrent requests → `expect(client.encrypted).toBeFalsy()`

## JS30
type: specification
authority: derived
scope: module
status: active
depends-on: []

**module** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/module/require-extensions.test.ts:85` — wrapping an existing extension with no logic → `expect(module).toBeDefined()`
- `test/js/node/module/require-extensions.test.ts:101` — wrapping an existing extension with mutated compile function → `expect(module).toBeDefined()`
- `test/js/node/module/require-extensions.test.ts:124` — wrapping an existing extension with mutated compile function ts → `expect(module).toBeDefined()`

## JS31
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**vm.runInContext** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/transpiler/repl-transform.test.ts:108` — Bun.Transpiler replMode > REPL session with node:vm > await with variable → `expect(result).toEqual({ value: 20 })`
- `test/js/bun/bun-object/deep-equals.spec.ts:76` — global object > main global object is not equal to vm global objects → `expect(areEqual).toBe(false)`
- `test/js/bun/transpiler/repl-transform.test.ts:191` — Bun.Transpiler replMode > object literal detection > {foo: await bar()} parsed as object l… → `expect(result.value).toEqual({ foo: 42 })`

## JS32
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.CryptoHasher** — exposes values of the expected type or class. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/bun-cryptohasher.test.ts:125` — Hash is consistent > base64 → `expect(hasher.update(buffer, "base64")).toBeInstanceOf(Bun.CryptoHasher)`
- `test/js/bun/util/bun-cryptohasher.test.ts:145` — Hash is consistent > hex → `expect(hasher.update(buffer, "hex")).toBeInstanceOf(Bun.CryptoHasher)`
- `test/js/bun/util/bun-cryptohasher.test.ts:165` — Hash is consistent > blob → `expect(hasher.update(buffer)).toBeInstanceOf(Bun.CryptoHasher)`

## JS33
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.allocUnsafe** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/unsafe.test.js:47` — Bun.allocUnsafe → `expect(buffer instanceof Uint8Array).toBe(true)`
- `test/js/bun/util/unsafe.test.js:48` — Bun.allocUnsafe → `expect(buffer.length).toBe(1024)`
- `test/js/bun/util/unsafe.test.js:50` — Bun.allocUnsafe → `expect(buffer[0]).toBe(0)`

## JS34
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**globalThis.Buffer** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/buffer.test.js:217` — Buffer global is settable → `expect(globalThis.Buffer).toBe(42)`
- `test/js/node/buffer.test.js:219` — Buffer global is settable → `expect(globalThis.Buffer).toBe(BufferModule.Buffer)`
- `test/js/node/buffer.test.js:220` — Buffer global is settable → `expect(globalThis.Buffer).toBe(prevBuffer)`

## JS35
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.binding** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/process-binding.test.ts:5` — process.binding > process.binding('constants') → `expect(constants).toBeDefined()`
- `test/js/node/process-binding.test.ts:15` — process.binding > process.binding('uv') → `expect(uv).toBeDefined()`
- `test/js/node/nodettywrap.test.ts:8` — process.binding('tty_wrap') → `expect(tty_wrap).toBeDefined()`

## JS36
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.execArgv** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/workers/worker.test.ts:187` — web worker > argv / execArgv options → `expect(process.execArgv).toEqual(original_execArgv)`
- `test/js/web/workers/worker.test.ts:389` — worker_threads > worker with argv/execArgv → `expect(process.execArgv).toEqual(original_execArgv)`
- `test/js/node/process/process.test.js:306` — process.execArgv → `expect(process.execArgv instanceof Array).toBe(true)`

## JS37
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Buffer.allocUnsafeSlow** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/buffer.test.js:1339` — unpooled buffer (replaces SlowBuffer) → `expect(ubuf).toBeTruthy()`
- `test/js/node/buffer.test.js:1340` — unpooled buffer (replaces SlowBuffer) → `expect(ubuf.buffer).toBeTruthy()`

## JS38
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.$.ShellError** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/shell/bunshell.test.ts:2591` — ShellError constructor > new $.ShellError() → `expect(e).toBeInstanceOf(Bun.$.ShellError)`
- `test/js/bun/shell/bunshell.test.ts:2592` — ShellError constructor > new $.ShellError() → `expect(e).toBeInstanceOf(Error)`

## JS39
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.escapeHTML** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsc-stress/fixtures/simd-baseline.test.ts:16` — escapeHTML — @Vector(16, u8) gated by enableSIMD > clean passthrough → `expect(Bun.escapeHTML(ascii256)).toBe(ascii256)`
- `test/js/bun/jsc-stress/fixtures/simd-baseline.test.ts:24` — escapeHTML — @Vector(16, u8) gated by enableSIMD > ampersand in middle → `expect(escaped.replaceAll("&amp;", "").includes("&")).toBe(false)`

## JS40
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.randomUUIDv7** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/randomUUIDv7.test.ts:5` — randomUUIDv7 > basic → `expect(Bun.randomUUIDv7()).toBeTypeOf("string")`
- `test/js/bun/util/randomUUIDv7.test.ts:31` — randomUUIDv7 > buffer output encoding → `expect(uuid).toBeInstanceOf(Buffer)`

## JS41
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.randomInt** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:14` — crypto.randomInt should return a number → `expect(typeof result).toBe("number")`
- `test/js/node/crypto/node-crypto.test.js:25` — crypto.randomInt with one argument → `expect(typeof result).toBe("number")`

## JS42
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/crypto/web-crypto.test.ts:38` — Web Crypto > has globals → `expect(crypto.subtle !== undefined).toBe(true)`
- `test/js/web/crypto/web-crypto.test.ts:134` — Web Crypto > unwrapKey JWK error handling > rejects when wrapped bytes are not valid JSON → `expect(err.name).toBe("DataError")`

## JS43
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/crypto/web-crypto.test.ts:133` — Web Crypto > unwrapKey JWK error handling > rejects when wrapped bytes are not valid JSON → `expect(err).toBeInstanceOf(DOMException)`
- `test/js/web/crypto/web-crypto.test.ts:148` — Web Crypto > unwrapKey JWK error handling > rejects when wrapped bytes are valid JSON but … → `expect(err).toBeInstanceOf(TypeError)`

## JS44
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fs.promises.statfs** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:3660` — fs.promises.statfs should work → `expect(stats).toBeDefined()`
- `test/js/node/fs/fs.test.ts:3665` — fs.promises.statfs should work with bigint → `expect(stats).toBeDefined()`

## JS45
type: specification
authority: derived
scope: module
status: active
depends-on: []

**inspector.console** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/inspector/inspector.test.ts:9` — inspector.console → `expect(inspector.console).toBeObject()`
- `test/js/node/inspector/inspector-profiler.test.ts:331` — node:inspector > exports > console is exported → `expect(inspector.console).toBeObject()`

## JS46
type: specification
authority: derived
scope: module
status: active
depends-on: []

**inspector.url** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/inspector/inspector.test.ts:5` — inspector.url() → `expect(inspector.url()).toBeUndefined()`
- `test/js/node/inspector/inspector-profiler.test.ts:327` — node:inspector > exports > url() returns undefined → `expect(inspector.url()).toBeUndefined()`

## JS47
type: specification
authority: derived
scope: module
status: active
depends-on: []

**it.each** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/jest-each.test.ts:11` — jest-each > check types → `expect(it.each).toBeTypeOf("function")`
- `test/js/bun/test/jest-each.test.ts:12` — jest-each > check types → `expect(it.each([])).toBeTypeOf("function")`

## JS48
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.hasUncaughtExceptionCaptureCallback** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:828` — process.hasUncaughtExceptionCaptureCallback → `expect(process.hasUncaughtExceptionCaptureCallback()).toBe(false)`
- `test/js/node/process/process.test.js:830` — process.hasUncaughtExceptionCaptureCallback → `expect(process.hasUncaughtExceptionCaptureCallback()).toBe(true)`

## JS49
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.stdout.json** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/spawn/readablestream-helpers.test.ts:106` — ReadableStream conversion methods > Bun.spawn() process.stdout.json() should throw on inva… → `expect(result).toBeInstanceOf(Promise)`
- `test/js/bun/spawn/readablestream-helpers.test.ts:127` — ReadableStream conversion methods > Bun.spawn() process.stdout.json() should throw on inva… → `expect(result).toBeInstanceOf(Promise)`

## JS50
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**stream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process-stdio-invalid-utf16.test.ts:87` — trailing unpaired high surrogate should not duplicate output → `expect(output).toBe("Help�\nTest�\n")`
- `test/js/node/process/process-stdio-invalid-utf16.test.ts:365` — large strings with trailing unpaired surrogates → `expect(output.endsWith("�\n")).toBe(true)`

## JS51
type: specification
authority: derived
scope: module
status: active
depends-on: []

**stream.json** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/stream-fast-path.test.ts:44` — ByteBlobLoader > json → `expect(result.then).toBeFunction()`
- `test/js/web/fetch/stream-fast-path.test.ts:53` — ByteBlobLoader > returns a rejected Promise for invalid JSON → `expect(result.then).toBeFunction()`

## JS52
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**stream.text** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/fetch/body-clone.test.ts:399` — ReadableStream with mixed content (starting with string) can be converted to text → `expect(typeof text).toBe("string")`
- `test/js/bun/stream/direct-readable-stream.test.tsx:160` — (stream).text() → `expect(text.replaceAll("<!-- -->", "")).toBe(inputString)`

## JS53
type: specification
authority: derived
scope: module
status: active
depends-on: []

**tls.createSecureContext** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/tls/tls-connect-socket-churn.test.ts:70` — createSecureContext memoises the native SSL_CTX (not the wrapper) by config → `expect(a.servername).toBeUndefined()`
- `test/js/node/tls/ssl-ctx-cache.test.ts:105` — SSL_CTX is freed once no owners remain (weak cache, not strong) → `expect(sc.context).toBeTruthy()`

## JS54
type: specification
authority: derived
scope: module
status: active
depends-on: []

**AbortController** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/abort/abort.test.ts:79` — AbortSignal > .signal.reason should be a DOMException → `expect(ac.signal.reason).toBeInstanceOf(DOMException)`

## JS55
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal.any** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/abort/abort.test.ts:60` — AbortSignal > AbortSignal.any() should fire abort event → `expect(signal.aborted).toBe(true)`

## JS56
type: specification
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal.timeout** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/abort/abort.test.ts:86` — AbortSignal > .signal.reason should be a DOMException for timeout → `expect(ac.reason).toBeInstanceOf(DOMException)`

## JS57
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.Archive** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/archive.test.ts:170` — Bun.Archive > new Archive() > converts non-string/buffer values to strings → `expect(archive).toBeDefined()`

## JS58
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.Image** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/image/image.test.ts:902` — Bun.Image > output-format setters + terminals > .blob() yields a Blob with the right MIME … → `expect(blob).toBeInstanceOf(Blob)`

## JS59
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.JSONC** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsonc/jsonc.test.ts:5` — Bun.JSONC exists → `expect(typeof Bun.JSONC).toBe("object")`

## JS60
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.JSONC** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsonc/jsonc.test.ts:4` — Bun.JSONC exists → `expect(Bun.JSONC).toBeDefined()`

## JS61
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.PostgresError** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11881` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.PostgresError).toBeDefined()`

## JS62
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.PostgresError.prototype** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11889` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.PostgresError.prototype).toBeInstanceOf(Bun.SQL.SQLError)`

## JS63
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.SQLError** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11880` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.SQLError).toBeDefined()`

## JS64
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.SQLError.prototype** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11888` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.SQLError.prototype).toBeInstanceOf(Error)`

## JS65
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.SQLiteError** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11882` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.SQLiteError).toBeDefined()`

## JS66
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.SQLiteError.prototype** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11890` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.SQLiteError.prototype).toBeInstanceOf(Bun.SQL.SQLError)`

## JS67
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.WebView.closeAll** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/webview/webview-chrome.test.ts:506` — WebView.closeAll is a static function → `expect(typeof Bun.WebView.closeAll).toBe("function")`

## JS68
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.connect** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/net/socket.test.ts:927` — getServername on a closed TLS socket should not crash → `expect(client.getServername()).toBeUndefined()`

## JS69
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.cron** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/cron/cron.test.ts:100` — Bun.cron API > is a function → `expect(typeof Bun.cron).toBe("function")`

## JS70
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.cron.parse** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/cron/cron.test.ts:1394` — Bun.cron.parse > is a function that returns a Date → `expect(result).toBeInstanceOf(Date)`

## JS71
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.cron.remove** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/cron/cron.test.ts:104` — Bun.cron API > has .remove method → `expect(typeof Bun.cron.remove).toBe("function")`

## JS72
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.fetch.bind** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:1606` — #2794 → `expect(typeof Bun.fetch.bind).toBe("function")`

## JS73
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.generateHeapSnapshot** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/v8-heap-snapshot.test.ts:49` — v8 heap snapshot arraybuffer → `expect(snapshot).toBeInstanceOf(ArrayBuffer)`

## JS74
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.main** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/bun-main.test.ts:7` — Bun.main > can be overridden → `expect(Bun.main).toBeString()`

## JS75
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.readableStreamToArrayBuffer** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/body-clone.test.ts:426` — ReadableStream with mixed content (starting with Uint8Array) can be converted to ArrayBuff… → `expect(arrayBuffer).toBeInstanceOf(ArrayBuffer)`

## JS76
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.readableStreamToBytes** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/body-clone.test.ts:455` — ReadableStream with mixed content (starting with ArrayBuffer) can be converted to Uint8Arr… → `expect(uint8Array).toBeInstanceOf(Uint8Array)`

## JS77
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.spawn** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/terminal/terminal.test.ts:984` — Bun.spawn with terminal option > creates subprocess with terminal attached → `expect(proc.terminal).toBeInstanceOf(Object)`

## JS78
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ByteLengthQueuingStrategy** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/streams/streams.test.js:482` — exists globally → `expect(typeof ByteLengthQueuingStrategy).toBe("function")`

## JS79
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**CountQueuingStrategy** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/streams/streams.test.js:483` — exists globally → `expect(typeof CountQueuingStrategy).toBe("function")`

## JS80
type: specification
authority: derived
scope: module
status: active
depends-on: []

**FormData** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/html/FormData.test.ts:40` — FormData > should get filename from file → `expect(formData.get("foo").name).toBeUndefined()`

