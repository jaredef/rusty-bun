# @js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: js-surface-property
  threshold: JS1
  interface: [Bun.JSONL.parseChunk, Bun.Cookie, url.includes, Bun.Cookie.parse, Bun.Image, stream, Bun.Cookie.parse, path.win32.isAbsolute, WebSocket, btoa, Bun.JSONL.parseChunk, expect.any, Bun.randomUUIDv5, Bun.readableStreamToArray, Bun.spawn, expect.not.objectContaining]

@imports: []

@pins: []

Surface drawn from 80 candidate properties across the Bun test corpus. Construction-style: 80; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 852.

## JS1
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

## JS2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.Cookie** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 64 constraint clauses across 5 test files. Antichain representatives:

- `test/js/bun/util/cookie.test.js:6` — Bun.Cookie > can create a cookie → `expect(cookie.name).toBe("name")`
- `test/js/bun/http/bun-serve-cookies.test.ts:396` — Direct usage of Bun.Cookie and Bun.CookieMap > can create a Cookie directly → `expect(cookie.constructor).toBe(Bun.Cookie)`
- `test/js/bun/cookie/cookie.test.ts:10` — Bun.Cookie validation tests > expires validation > accepts valid Date for expires → `expect(cookie.expires).toEqual(futureDate)`

## JS3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**url.includes** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 60 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/s3/s3.test.ts:1306` — ${credentials.service} > S3 static methods > presign > should work → `expect(url.includes("X-Amz-Expires=86400")).toBe(true)`
- `test/js/bun/s3/s3.test.ts:1307` — ${credentials.service} > S3 static methods > presign > should work → `expect(url.includes("X-Amz-Date")).toBe(true)`
- `test/js/bun/s3/s3.test.ts:1308` — ${credentials.service} > S3 static methods > presign > should work → `expect(url.includes("X-Amz-Signature")).toBe(true)`

## JS4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.Cookie.parse** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 58 constraint clauses across 5 test files. Antichain representatives:

- `test/js/bun/util/cookie.test.js:47` — Bun.Cookie > parse a cookie string → `expect(cookie.name).toBe("name")`
- `test/js/bun/http/bun-serve-cookies.test.ts:506` — Direct usage of Bun.Cookie and Bun.CookieMap > can use Cookie.parse to parse cookie string… → `expect(cookie.constructor).toBe(Bun.Cookie)`
- `test/js/bun/cookie/cookie-security-fuzz.test.ts:28` — Bun.Cookie.parse security fuzz tests > resists cookie format injection attacks > additiona… → `expect(cookie.name).toBe("name")`

## JS5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.Image** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 40 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/image/image.test.ts:146` — Bun.Image > constructor exists and is exposed on Bun → `expect(typeof Bun.Image).toBe("function")`
- `test/js/bun/image/image-adversarial.test.ts:166` — format confusion > PNG with valid JPEG appended (polyglot-ish) → `expect(meta.format).toBe("png")`
- `test/js/bun/image/image.test.ts:186` — Bun.Image > metadata() reads PNG dimensions → `expect(img.width).toBe(4)`

## JS6
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

## JS7
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

## JS8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.win32.isAbsolute** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 22 constraint clauses across 3 test files. Antichain representatives:

- `test/js/node/path/zero-length-strings.test.js:30` — path > zero length strings → `assert.strictEqual(path.win32.isAbsolute(""), false)`
- `test/js/node/path/is-absolute.test.js:7` — path.isAbsolute > win32 → `assert.strictEqual(path.win32.isAbsolute("/"), true)`
- `test/js/node/path/browserify.test.js:917` — browserify path tests > isAbsolute > win32 /foo/bar → `expect(path.win32.isAbsolute("/foo/bar")).toBe(true)`

## JS9
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

## JS10
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

## JS11
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

## JS12
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

## JS13
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

## JS14
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

## JS15
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

## JS16
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

## JS17
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

## JS18
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

## JS19
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.cron.parse** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 11 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/cron/cron.test.ts:108` — Bun.cron API > has .parse method → `expect(typeof Bun.cron.parse).toBe("function")`
- `test/js/bun/cron/cron.test.ts:1392` — Bun.cron.parse > is a function that returns a Date → `expect(typeof Bun.cron.parse).toBe("function")`
- `test/js/bun/cron/cron.test.ts:1575` — Bun.cron.parse > full day names match 3-letter abbreviations → `expect(Bun.cron.parse('0 0 * * ${abbr}', from)!.getTime()).toBe( Bun.cron.parse('0 0 * * ${full}', from)!.getTime(), )`

## JS20
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**readline.cursorTo** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 11 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/readline/readline.node.test.ts:236` — readline.cursorTo() > should not throw on undefined or null as stream → `assert.strictEqual(readline.cursorTo(null), true)`
- `test/js/node/readline/readline.node.test.ts:237` — readline.cursorTo() > should not throw on undefined or null as stream → `assert.strictEqual(readline.cursorTo(), true)`
- `test/js/node/readline/readline.node.test.ts:238` — readline.cursorTo() > should not throw on undefined or null as stream → `assert.strictEqual(readline.cursorTo(null, 1, 1, mustCall()), true)`

## JS21
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

## JS22
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.parse** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 9 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/path/parse-format.test.js:168` — path.parse > general → `assert.strictEqual(typeof output.root, "string")`
- `test/js/node/path/parse-format.test.js:169` — path.parse > general → `assert.strictEqual(typeof output.dir, "string")`
- `test/js/node/path/parse-format.test.js:170` — path.parse > general → `assert.strictEqual(typeof output.base, "string")`

## JS23
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**performance.mark** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 9 constraint clauses across 1 test files. Antichain representatives:

- `test/js/deno/performance/performance.test.ts:37` —  → `assertEquals(mark.detail, null)`
- `test/js/deno/performance/performance.test.ts:38` —  → `assertEquals(mark.name, "test")`
- `test/js/deno/performance/performance.test.ts:39` —  → `assertEquals(mark.entryType, "mark")`

## JS24
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response.redirect** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/fetch/response.test.ts:99` — Response.redirect status code validation → `expect(Response.redirect("url", 301).status).toBe(301)`
- `test/js/web/fetch/fetch.test.ts:1226` — Response > Response.redirect > works → `expect(Response.redirect(input).headers.get("Location")).toBe(input)`
- `response.spec.md:26` — Response.redirect static method → `Response.redirect(url) returns a Response with the Location header set to url`

## JS25
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.exportKey** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/crypto/web-crypto-sha3.test.ts:71` — HMAC with SHA-3 > HMAC-SHA3-384 generateKey default length → `expect(raw.byteLength).toBe(104)`
- `test/js/deno/crypto/webcrypto.test.ts:574` —  → `assertEquals(exportedKey2, jwk)`
- `test/js/web/crypto/web-crypto-sha3.test.ts:92` — RSA with SHA-3 hash > RSA-PSS with SHA3-256: generate, sign, verify, JWK export → `expect(jwk.kty).toBe("RSA")`

## JS26
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.generateKey** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/deno/crypto/webcrypto.test.ts:584` —  → `assertEquals(key.type, "secret")`
- `test/js/deno/crypto/webcrypto.test.ts:585` —  → `assertEquals(key.extractable, true)`
- `test/js/deno/crypto/webcrypto.test.ts:586` —  → `assertEquals(key.usages, [ "sign" ])`

## JS27
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

## JS28
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**readline.clearLine** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/readline/readline.node.test.ts:123` — readline.clearLine() > should clear to the left of cursor when given -1 as direction → `assert.strictEqual(readline.clearLine(writable, -1), true)`
- `test/js/node/readline/readline.node.test.ts:128` — readline.clearLine() > should clear to the right of cursor when given 1 as direction → `assert.strictEqual(readline.clearLine(writable, 1), true)`
- `test/js/node/readline/readline.node.test.ts:133` — readline.clearLine() > should clear whole line when given 0 as direction → `assert.strictEqual(readline.clearLine(writable, 0), true)`

## JS29
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**readline.createInterface** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/readline/readline.node.test.ts:1936` — readline.createInterface() > should respond to home and end sequences for common pttys  → `assert.strictEqual(rl.cursor, 3)`
- `test/js/node/readline/readline.node.test.ts:1959` — readline.createInterface() > should respond to home and end sequences for common pttys  → `assert.strictEqual(rl.cursor, 0)`
- `test/js/node/readline/readline.node.test.ts:1961` — readline.createInterface() > should respond to home and end sequences for common pttys  → `assert.strictEqual(rl.cursor, 3)`

## JS30
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.Cookie.from** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/http/bun-serve-cookies.test.ts:522` — Direct usage of Bun.Cookie and Bun.CookieMap > can use Cookie.from to create cookies → `expect(cookie.constructor).toBe(Bun.Cookie)`
- `test/js/bun/cookie/cookie-expires-validation.test.ts:123` — Bun.Cookie expires validation > Constructors and methods > handles valid expires in Cookie… → `expect(cookie.expires).toEqual(new Date(futureTimestamp * 1000))`
- `test/js/bun/http/bun-serve-cookies.test.ts:523` — Direct usage of Bun.Cookie and Bun.CookieMap > can use Cookie.from to create cookies → `expect(cookie.name).toBe("name")`

## JS31
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.posix.toNamespacedPath** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/path/to-namespaced-path.test.js:74` — path.toNamespacedPath > posix → `assert.strictEqual(path.posix.toNamespacedPath("/foo/bar"), "/foo/bar")`
- `test/js/node/path/to-namespaced-path.test.js:75` — path.toNamespacedPath > posix → `assert.strictEqual(path.posix.toNamespacedPath("foo/bar"), "foo/bar")`
- `test/js/node/path/to-namespaced-path.test.js:76` — path.toNamespacedPath > posix → `assert.strictEqual(path.posix.toNamespacedPath(null), null)`

## JS32
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

## JS33
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

## JS34
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

## JS35
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**readline.moveCursor** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/readline/readline.node.test.ts:189` — readline.moveCursor() > shouldn't write when moveCursor(0, 0) is called → `assert.strictEqual(readline.moveCursor(writable, set[0], set[1]), true)`
- `test/js/node/readline/readline.node.test.ts:192` — readline.moveCursor() > shouldn't write when moveCursor(0, 0) is called → `assert.strictEqual(readline.moveCursor(writable, set[0], set[1], mustCall()), true)`
- `test/js/node/readline/readline.node.test.ts:211` — readline.moveCursor() > should not throw on null or undefined stream → `assert.strictEqual(readline.moveCursor(null, 1, 1), true)`

## JS36
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Request** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:1467` — Request > clone → `expect(body.signal).toBeDefined()`
- `request.spec.md:7` — Request is exposed as a global constructor → `Request is defined as a global constructor in any execution context with [Exposed=*]`
- `test/js/web/fetch/fetch.test.ts:1557` — body nullable → `expect(req.body).toBeNull()`

## JS37
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**vm.runInNewContext** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/util/node-inspect-tests/parallel/util-inspect.test.js:3013` — no assertion failures 3 → `assert.strictEqual(target.ctx, undefined)`
- `test/js/bun/jsc/domjit.test.ts:152` — DOMJIT > in NodeVM > vm.runInNewContext → `expect(vm.runInNewContext(code, { crypto, performance, TextEncoder, TextDecoder, dirStats })).toBe("success")`
- `test/js/node/util/node-inspect-tests/parallel/util-inspect.test.js:3018` — no assertion failures 3 → `assert.strictEqual(typeof target.ctx, "object")`

## JS38
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal.abort** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 2 test files. Antichain representatives:

- `test/js/deno/abort/abort-controller.test.ts:58` —  → `assertEquals(signal.aborted, true)`
- `abort-controller.spec.md:23` — AbortSignal.abort static method → `AbortSignal.abort() returns an already-aborted AbortSignal with default reason`
- `test/js/deno/abort/abort-controller.test.ts:59` —  → `assertEquals(signal.reason, "hey!")`

## JS39
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

## JS40
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.deriveKey** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 2 test files. Antichain representatives:

- `test/js/deno/crypto/webcrypto.test.ts:828` —  → `assertEquals(derivedKey.type, "secret")`
- `test/js/bun/crypto/x25519-derive-bits.test.ts:114` — X25519 deriveKey produces an AES-GCM key from the shared secret → `expect(key.algorithm.name).toBe("AES-GCM")`
- `test/js/deno/crypto/webcrypto.test.ts:829` —  → `assertEquals(derivedKey.extractable, true)`

## JS41
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.importKey** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/deno/crypto/webcrypto.test.ts:627` —  → `assertEquals(key.type, "private")`
- `test/js/deno/crypto/webcrypto.test.ts:628` —  → `assertEquals(key.extractable, true)`
- `test/js/deno/crypto/webcrypto.test.ts:629` —  → `assertEquals(key.usages, [ "sign" ])`

## JS42
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

## JS43
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

## JS44
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**performance.measure** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/deno/performance/performance.test.ts:86` —  → `assertEquals(measure1.detail, null)`
- `test/js/deno/performance/performance.test.ts:87` —  → `assertEquals(measure1.name, measureName1)`
- `test/js/deno/performance/performance.test.ts:88` —  → `assertEquals(measure1.entryType, "measure")`

## JS45
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**readline.clearScreenDown** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/readline/readline.node.test.ts:86` — readline.clearScreenDown() > should put clear screen sequence into writable when called → `assert.strictEqual(readline.clearScreenDown(writable), true)`
- `test/js/node/readline/readline.node.test.ts:88` — readline.clearScreenDown() > should put clear screen sequence into writable when called → `assert.strictEqual(readline.clearScreenDown(writable, mustCall()), true)`
- `test/js/node/readline/readline.node.test.ts:104` — readline.clearScreenDown() > should that clearScreenDown() does not throw on null or undef… → `assert.strictEqual( readline.clearScreenDown( null, mustCall(err => { assert.strictEqual(err, null); }), ), true, )`

## JS46
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

## JS47
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal.any** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/abort/abort.test.ts:60` — AbortSignal > AbortSignal.any() should fire abort event → `expect(signal.aborted).toBe(true)`
- `abort-controller.spec.md:31` — AbortSignal.any static method → `AbortSignal.any(signals) returns an AbortSignal aborted when any signal aborts`
- `abort-controller.spec.md:32` — AbortSignal.any static method → `AbortSignal.any returns an already-aborted signal when any input is already aborted`

## JS48
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.timingSafeEqual** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:156` — crypto.timingSafeEqual → `expect(crypto.timingSafeEqual(uuid, uuid)).toBe(true)`
- `test/js/web/web-globals.test.js:157` — crypto.timingSafeEqual → `expect(crypto.timingSafeEqual(uuid, uuid.slice())).toBe(true)`
- `test/js/web/web-globals.test.js:171` — crypto.timingSafeEqual → `expect(crypto.timingSafeEqual(uuid, crypto.randomUUID())).toBe(false)`

## JS49
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

## JS50
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

## JS51
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

## JS52
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Blob** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/fetch/blob.test.ts:203` — blob: can set name property #10178 → `expect(blob.name).toBeUndefined()`
- `blob.spec.md:7` — Blob is exposed as a global constructor → `Blob is defined as a global constructor in any execution context with [Exposed=*]`

## JS53
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Buffer.allocUnsafeSlow** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/buffer.test.js:1339` — unpooled buffer (replaces SlowBuffer) → `expect(ubuf).toBeTruthy()`
- `test/js/node/buffer.test.js:1340` — unpooled buffer (replaces SlowBuffer) → `expect(ubuf.buffer).toBeTruthy()`

## JS54
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.$.ShellError** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/shell/bunshell.test.ts:2591` — ShellError constructor > new $.ShellError() → `expect(e).toBeInstanceOf(Bun.$.ShellError)`
- `test/js/bun/shell/bunshell.test.ts:2592` — ShellError constructor > new $.ShellError() → `expect(e).toBeInstanceOf(Error)`

## JS55
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.randomUUIDv7** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/randomUUIDv7.test.ts:5` — randomUUIDv7 > basic → `expect(Bun.randomUUIDv7()).toBeTypeOf("string")`
- `test/js/bun/util/randomUUIDv7.test.ts:31` — randomUUIDv7 > buffer output encoding → `expect(uuid).toBeInstanceOf(Buffer)`

## JS56
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.randomInt** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:14` — crypto.randomInt should return a number → `expect(typeof result).toBe("number")`
- `test/js/node/crypto/node-crypto.test.js:25` — crypto.randomInt with one argument → `expect(typeof result).toBe("number")`

## JS57
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/crypto/web-crypto.test.ts:38` — Web Crypto > has globals → `expect(crypto.subtle !== undefined).toBe(true)`
- `test/js/web/crypto/web-crypto.test.ts:134` — Web Crypto > unwrapKey JWK error handling > rejects when wrapped bytes are not valid JSON → `expect(err.name).toBe("DataError")`

## JS58
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/crypto/web-crypto.test.ts:133` — Web Crypto > unwrapKey JWK error handling > rejects when wrapped bytes are not valid JSON → `expect(err).toBeInstanceOf(DOMException)`
- `test/js/web/crypto/web-crypto.test.ts:148` — Web Crypto > unwrapKey JWK error handling > rejects when wrapped bytes are valid JSON but … → `expect(err).toBeInstanceOf(TypeError)`

## JS59
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fs.promises.statfs** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:3660` — fs.promises.statfs should work → `expect(stats).toBeDefined()`
- `test/js/node/fs/fs.test.ts:3665` — fs.promises.statfs should work with bigint → `expect(stats).toBeDefined()`

## JS60
type: specification
authority: derived
scope: module
status: active
depends-on: []

**inspector.console** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/inspector/inspector.test.ts:9` — inspector.console → `expect(inspector.console).toBeObject()`
- `test/js/node/inspector/inspector-profiler.test.ts:331` — node:inspector > exports > console is exported → `expect(inspector.console).toBeObject()`

## JS61
type: specification
authority: derived
scope: module
status: active
depends-on: []

**inspector.url** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/inspector/inspector.test.ts:5` — inspector.url() → `expect(inspector.url()).toBeUndefined()`
- `test/js/node/inspector/inspector-profiler.test.ts:327` — node:inspector > exports > url() returns undefined → `expect(inspector.url()).toBeUndefined()`

## JS62
type: specification
authority: derived
scope: module
status: active
depends-on: []

**it.each** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/jest-each.test.ts:11` — jest-each > check types → `expect(it.each).toBeTypeOf("function")`
- `test/js/bun/test/jest-each.test.ts:12` — jest-each > check types → `expect(it.each([])).toBeTypeOf("function")`

## JS63
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.hasUncaughtExceptionCaptureCallback** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:828` — process.hasUncaughtExceptionCaptureCallback → `expect(process.hasUncaughtExceptionCaptureCallback()).toBe(false)`
- `test/js/node/process/process.test.js:830` — process.hasUncaughtExceptionCaptureCallback → `expect(process.hasUncaughtExceptionCaptureCallback()).toBe(true)`

## JS64
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.stdout.json** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/spawn/readablestream-helpers.test.ts:106` — ReadableStream conversion methods > Bun.spawn() process.stdout.json() should throw on inva… → `expect(result).toBeInstanceOf(Promise)`
- `test/js/bun/spawn/readablestream-helpers.test.ts:127` — ReadableStream conversion methods > Bun.spawn() process.stdout.json() should throw on inva… → `expect(result).toBeInstanceOf(Promise)`

## JS65
type: specification
authority: derived
scope: module
status: active
depends-on: []

**stream.json** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/stream-fast-path.test.ts:44` — ByteBlobLoader > json → `expect(result.then).toBeFunction()`
- `test/js/web/fetch/stream-fast-path.test.ts:53` — ByteBlobLoader > returns a rejected Promise for invalid JSON → `expect(result.then).toBeFunction()`

## JS66
type: specification
authority: derived
scope: module
status: active
depends-on: []

**tls.createSecureContext** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/tls/tls-connect-socket-churn.test.ts:70` — createSecureContext memoises the native SSL_CTX (not the wrapper) by config → `expect(a.servername).toBeUndefined()`
- `test/js/node/tls/ssl-ctx-cache.test.ts:105` — SSL_CTX is freed once no owners remain (weak cache, not strong) → `expect(sc.context).toBeTruthy()`

## JS67
type: specification
authority: derived
scope: module
status: active
depends-on: []

**AbortController** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/abort/abort.test.ts:79` — AbortSignal > .signal.reason should be a DOMException → `expect(ac.signal.reason).toBeInstanceOf(DOMException)`

## JS68
type: specification
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal.timeout** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/abort/abort.test.ts:86` — AbortSignal > .signal.reason should be a DOMException for timeout → `expect(ac.reason).toBeInstanceOf(DOMException)`

## JS69
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.Archive** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/archive.test.ts:170` — Bun.Archive > new Archive() > converts non-string/buffer values to strings → `expect(archive).toBeDefined()`

## JS70
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.Image** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/image/image.test.ts:902` — Bun.Image > output-format setters + terminals > .blob() yields a Blob with the right MIME … → `expect(blob).toBeInstanceOf(Blob)`

## JS71
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.JSONC** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsonc/jsonc.test.ts:5` — Bun.JSONC exists → `expect(typeof Bun.JSONC).toBe("object")`

## JS72
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.JSONC** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsonc/jsonc.test.ts:4` — Bun.JSONC exists → `expect(Bun.JSONC).toBeDefined()`

## JS73
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.PostgresError** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11881` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.PostgresError).toBeDefined()`

## JS74
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.PostgresError.prototype** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11889` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.PostgresError.prototype).toBeInstanceOf(Bun.SQL.SQLError)`

## JS75
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.SQLError** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11880` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.SQLError).toBeDefined()`

## JS76
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.SQLError.prototype** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11888` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.SQLError.prototype).toBeInstanceOf(Error)`

## JS77
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.SQLiteError** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11882` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.SQLiteError).toBeDefined()`

## JS78
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.SQLiteError.prototype** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11890` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.SQLiteError.prototype).toBeInstanceOf(Bun.SQL.SQLError)`

## JS79
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.WebView.closeAll** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/webview/webview-chrome.test.ts:506` — WebView.closeAll is a static function → `expect(typeof Bun.WebView.closeAll).toBe("function")`

## JS80
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.cron** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/cron/cron.test.ts:100` — Bun.cron API > is a function → `expect(typeof Bun.cron).toBe("function")`

