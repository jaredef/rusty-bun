# Bun — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: bun-surface-property
  threshold: BUN1
  interface: [Bun.JSONL.parseChunk, Bun.build, Bun.inspect, Bun.spawn, Bun.Cookie, Bun.Cookie.parse, Bun.Image, Bun.build, Bun.Cookie.parse, Bun.cron, Bun.JSONL.parseChunk, Bun.Terminal, Bun.randomUUIDv5, Bun.readableStreamToArray, Bun.spawn, Bun.Archive]

@imports: []

@pins: []

Surface drawn from 80 candidate properties across the Bun test corpus. Construction-style: 61; behavioral (high-cardinality): 19. Total witnessing constraint clauses: 2453.

## BUN1
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

## BUN2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.build** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 177 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/5344.test.ts:19` — code splitting with re-exports between entry points should not produce duplicate exports → `expect(result.success).toBe(true)`
- `test/regression/issue/26360.test.ts:135` — regular Bun.build (not in macro) still works → `expect(result.success).toBe(true)`
- `test/regression/issue/25785.test.ts:35` — CSS bundler should preserve logical border-radius properties → `expect(result.success).toBe(true)`

## BUN3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.inspect** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 115 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/16007.test.ts:6` — Set is propperly formatted in Bun.inspect() → `expect(formatted).toBe('{ set: Set(2) { "foo", "bar", }, }')`
- `test/js/web/url/url.test.ts:84` — url > prints → `expect(Bun.inspect(new URL("https://example.com"))).toBe('URL { href: "https://example.com/", origin: "https://example.com", protocol: "https:", username: "", password: "", host: "example.com", hostna…`
- `test/js/web/html/URLSearchParams.test.ts:130` — URLSearchParams > non-standard extensions > should support .toJSON → `expect(Bun.inspect(params)).toBe( "URLSearchParams {" + "\n" + ' "foo": [ "bar", "boop" ],' + "\n" + ' "bar": "baz",' + "\n" + "}", )`

## BUN4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.spawn** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 79 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/pe-codesigning-integrity.test.ts:201` — should create valid PE executable with .bun section → `expect(result.exitCode).toBe(0)`
- `test/regression/issue/ctrl-c.test.ts:133` —  → `expect(proc.killed).toBe(false)`
- `test/regression/issue/24387.test.ts:22` — regression: require()ing a module with TLA should error and then wipe the module cache, so… → `expect(await proc.exited).toBe(0)`

## BUN5
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

## BUN6
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

## BUN7
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

## BUN8
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.build** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 28 constraint clauses across 3 test files. Antichain representatives:

- `test/napi/napi.test.ts:135` — napi > `bun build` → `expect(build.logs).toBeEmpty()`
- `test/bundler/metafile.test.ts:46` — bundler metafile > metafile option returns metafile object → `expect(result.metafile).toBeDefined()`
- `test/bundler/bun-build-api.test.ts:436` — Bun.build > warnings do not fail a build → `expect(x.logs[0].position).toBeTruthy()`

## BUN9
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

## BUN10
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.cron** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 16 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/cron/cron.test.ts:172` — @daily nickname registers successfully → `expect(result).toBeUndefined()`
- `test/js/bun/cron/cron.test.ts:182` — @weekly nickname registers successfully → `expect(result).toBeUndefined()`
- `test/js/bun/cron/cron.test.ts:191` — @hourly nickname registers successfully → `expect(result).toBeUndefined()`

## BUN11
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

## BUN12
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.Terminal** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 14 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/terminal/terminal.test.ts:351` — Bun.Terminal > termios flags > flags return 0 on closed terminal → `expect(terminal.inputFlags).toBe(0)`
- `test/js/bun/terminal/terminal-spawn.test.ts:36` — Bun.Terminal subprocess integration > close marks terminal closed and write throws → `expect(terminal.closed).toBe(true)`
- `test/js/bun/terminal/terminal.test.ts:352` — Bun.Terminal > termios flags > flags return 0 on closed terminal → `expect(terminal.outputFlags).toBe(0)`

## BUN13
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

## BUN14
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

## BUN15
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

## BUN16
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

## BUN17
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

## BUN18
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

## BUN19
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

## BUN20
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.Cookie** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 9 constraint clauses across 4 test files. Antichain representatives:

- `test/regression/issue/22475.test.ts:44` — cookie.isExpired() for various edge cases → `expect(sessionCookie.expires).toBeUndefined()`
- `test/js/bun/cookie/cookie.test.ts:8` — Bun.Cookie validation tests > expires validation > accepts valid Date for expires → `expect(cookie.expires).toBeDefined()`
- `test/js/bun/cookie/cookie-map.test.ts:10` — Bun.Cookie and Bun.CookieMap > can create a basic Cookie → `expect(cookie.domain).toBeNull()`

## BUN21
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

## BUN22
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

## BUN23
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.WebView** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/webview/webview.test.ts:87` — is an EventTarget → `expect(typeof view.addEventListener).toBe("function")`
- `test/js/bun/webview/webview.test.ts:88` — is an EventTarget → `expect(typeof view.removeEventListener).toBe("function")`
- `test/js/bun/webview/webview.test.ts:89` — is an EventTarget → `expect(typeof view.dispatchEvent).toBe("function")`

## BUN24
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.SHA1.hash** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/fetch/body-stream.test.ts:141` — Request.prototoype.${RequestPrototypeMixin.name}() ${
            useRequestObject
       … → `expect(Bun.SHA1.hash(result, "base64")).toBe(Bun.SHA1.hash(input, "base64"))`
- `test/js/bun/s3/s3.test.ts:886` — ${credentials.service} > Bun.s3 > readable stream > should work with large files  → `expect(SHA1).toBe(SHA1_2)`
- `test/js/web/fetch/body-stream.test.ts:405` —  → `expect(Bun.SHA1.hash(await out.arrayBuffer(), "base64")).toBe(expectedHash)`

## BUN25
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.cron.remove** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/cron/cron.test.ts:281` — remove resolves undefined on success → `expect(result).toBeUndefined()`
- `test/js/bun/cron/cron.test.ts:286` — remove non-existent job resolves without error → `expect(result).toBeUndefined()`
- `test/js/bun/cron/cron.test.ts:664` — removing non-existent entry resolves without error → `expect(result).toBeUndefined()`

## BUN26
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.spawnSync** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 5 constraint clauses across 3 test files. Antichain representatives:

- `test/js/bun/spawn/spawnsync-no-microtask-drain.test.ts:87` — spawnSync with timeout still works → `expect(result.exitCode).toBeNull()`
- `test/cli/test/coverage.test.ts:20` — coverage crash → `expect(result.signalCode).toBeUndefined()`
- `test/bake/deinitialization.test.ts:11` — dev server deinitializes itself → `expect(result.signalCode).toBeUndefined()`

## BUN27
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.Cookie** — exposes values of the expected type or class. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/22475.test.ts:32` — cookie.isExpired() for various edge cases → `expect(epochCookie.expires).toBeDate()`
- `test/regression/issue/22475.test.ts:38` — cookie.isExpired() for various edge cases → `expect(negativeCookie.expires).toBeDate()`
- `test/js/bun/cookie/cookie.test.ts:9` — Bun.Cookie validation tests > expires validation > accepts valid Date for expires → `expect(cookie.expires).toBeDate()`

## BUN28
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.Image.hasClipboardImage** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/image/image.test.ts:1450` — Bun.Image clipboard statics > hasClipboardImage / clipboardChangeCount / fromClipboard hav… → `expect(typeof Bun.Image.hasClipboardImage()).toBe("boolean")`
- `test/js/bun/image/image.test.ts:1457` — Bun.Image clipboard statics > hasClipboardImage / clipboardChangeCount / fromClipboard hav… → `expect(Bun.Image.hasClipboardImage()).toBe(img !== null)`
- `test/js/bun/image/image.test.ts:1461` — Bun.Image clipboard statics > hasClipboardImage / clipboardChangeCount / fromClipboard hav… → `expect(Bun.Image.hasClipboardImage()).toBe(false)`

## BUN29
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.RedisClient** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/29925.test.ts:67` — client.connect() recovers after the client enters the failed state → `expect(client.connected).toBe(true)`
- `test/regression/issue/29925.test.ts:86` — client.connect() recovers after the client enters the failed state → `expect(client.connected).toBe(true)`
- `test/regression/issue/29925.test.ts:111` — repeated close()/connect()/send() cycles do not lock up → `expect(client.connected).toBe(true)`

## BUN30
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.WebView** — exposes values of the expected type or class. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/webview/webview.test.ts:66` — backend: 'webkit' throws on non-darwin → `expect(view).toBeInstanceOf(Bun.WebView)`
- `test/js/bun/webview/webview.test.ts:82` — is an EventTarget → `expect(view).toBeInstanceOf(EventTarget)`
- `test/js/bun/webview/webview-chrome.test.ts:130` — backend: chrome constructor returns a WebView → `expect(view).toBeInstanceOf(Bun.WebView)`

## BUN31
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.semver.satisfies** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/26657.test.ts:100` — bun update -i select all with 'A' key > should update packages when 'A' is pressed to sele… → `expect(Bun.semver.satisfies(installedVersion, ">0.1.0")).toBe(true)`
- `test/cli/update_interactive_install.test.ts:96` — bun update --interactive actually installs packages > should update package.json AND insta… → `expect(Bun.semver.satisfies(installedVersion, ">0.1.0")).toBe(true)`
- `test/cli/update_interactive_install.test.ts:174` — bun update --interactive actually installs packages > should work with --latest flag → `expect(Bun.semver.satisfies(updatedPkgJson.version, ">0.1.0")).toBe(true)`

## BUN32
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.$.ShellError** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/shell/bunshell.test.ts:2591` — ShellError constructor > new $.ShellError() → `expect(e).toBeInstanceOf(Bun.$.ShellError)`
- `test/js/bun/shell/bunshell.test.ts:2592` — ShellError constructor > new $.ShellError() → `expect(e).toBeInstanceOf(Error)`

## BUN33
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.cron.parse** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/cron/cron.test.ts:1702` — Bun.cron.parse > impossible expression (Feb 30) returns null → `expect(Bun.cron.parse("0 0 30 2 *", Date.UTC(2025, 0, 1, 0, 0, 0))).toBeNull()`
- `test/js/bun/cron/cron-parse.test.ts:49` — Bun.cron.parse — UTC > impossible day/month (Feb 30) returns null quickly → `expect(Bun.cron.parse("0 0 30 2 *", new Date("2026-01-01T00:00:00Z"))).toBeNull()`

## BUN34
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.file** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/workers/message-channel.test.ts:283` — cloneable and non-transferable equals (BunFile) → `expect(file).toBeInstanceOf(Blob)`
- `test/js/bun/image/image.test.ts:165` — Bun.Image > Bun.file() input chains the async file read into the pipeline → `expect(via).toBeInstanceOf(Bun.Image)`

## BUN35
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.file** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/cli/update_interactive_formatting.test.ts:1394` — bun update --interactive > should handle version ranges with multiple conditions → `expect(packageJson.catalog["no-deps"]).toBeDefined()`
- `test/cli/update_interactive_formatting.test.ts:1395` — bun update --interactive > should handle version ranges with multiple conditions → `expect(packageJson.catalog["dep-with-tags"]).toBeDefined()`

## BUN36
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.randomUUIDv7** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/randomUUIDv7.test.ts:5` — randomUUIDv7 > basic → `expect(Bun.randomUUIDv7()).toBeTypeOf("string")`
- `test/js/bun/util/randomUUIDv7.test.ts:31` — randomUUIDv7 > buffer output encoding → `expect(uuid).toBeInstanceOf(Buffer)`

## BUN37
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.secrets.delete** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/secrets-error-codes.test.ts:20` — delete non-existent returns false without error → `expect(result).toBe(false)`
- `test/js/bun/secrets-error-codes.test.ts:68` — successful operations work correctly → `expect(deleted).toBe(true)`

## BUN38
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.secrets.get** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/secrets-error-codes.test.ts:11` — non-existent secret returns null without error → `expect(result).toBeNull()`
- `test/js/bun/secrets-error-codes.test.ts:72` — successful operations work correctly → `expect(afterDelete).toBeNull()`

## BUN39
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.Archive** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/archive.test.ts:170` — Bun.Archive > new Archive() > converts non-string/buffer values to strings → `expect(archive).toBeDefined()`

## BUN40
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.Image** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/image/image.test.ts:902` — Bun.Image > output-format setters + terminals > .blob() yields a Blob with the right MIME … → `expect(blob).toBeInstanceOf(Blob)`

## BUN41
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.Image.fromClipboard** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/image/image.test.ts:1460` — Bun.Image clipboard statics > hasClipboardImage / clipboardChangeCount / fromClipboard hav… → `expect(Bun.Image.fromClipboard()).toBe(null)`

## BUN42
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.JSONC** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsonc/jsonc.test.ts:5` — Bun.JSONC exists → `expect(typeof Bun.JSONC).toBe("object")`

## BUN43
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.JSONC** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsonc/jsonc.test.ts:4` — Bun.JSONC exists → `expect(Bun.JSONC).toBeDefined()`

## BUN44
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.PostgresError** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11881` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.PostgresError).toBeDefined()`

## BUN45
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.PostgresError.prototype** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11889` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.PostgresError.prototype).toBeInstanceOf(Bun.SQL.SQLError)`

## BUN46
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.SQLError** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11880` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.SQLError).toBeDefined()`

## BUN47
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.SQLError.prototype** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11888` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.SQLError.prototype).toBeInstanceOf(Error)`

## BUN48
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.SQLiteError** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11882` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.SQLiteError).toBeDefined()`

## BUN49
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.SQL.SQLiteError.prototype** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/sql/sql.test.ts:11890` — PostgreSQL tests > Misc > The Bun.SQL.*Error classes exist → `expect(Bun.SQL.SQLiteError.prototype).toBeInstanceOf(Bun.SQL.SQLError)`

## BUN50
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.WebView.closeAll** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/webview/webview-chrome.test.ts:506` — WebView.closeAll is a static function → `expect(typeof Bun.WebView.closeAll).toBe("function")`

## BUN51
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.cron** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/cron/cron.test.ts:100` — Bun.cron API > is a function → `expect(typeof Bun.cron).toBe("function")`

## BUN52
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.cron.parse** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/cron/cron.test.ts:1394` — Bun.cron.parse > is a function that returns a Date → `expect(result).toBeInstanceOf(Date)`

## BUN53
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.cron.remove** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/cron/cron.test.ts:104` — Bun.cron API > has .remove method → `expect(typeof Bun.cron.remove).toBe("function")`

## BUN54
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.fetch.bind** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:1606` — #2794 → `expect(typeof Bun.fetch.bind).toBe("function")`

## BUN55
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.generateHeapSnapshot** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/v8-heap-snapshot.test.ts:49` — v8 heap snapshot arraybuffer → `expect(snapshot).toBeInstanceOf(ArrayBuffer)`

## BUN56
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.inspect** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/third_party/mongodb/mongodb.test.ts:23` — should connect and inpect → `expect(text).toBeDefined()`

## BUN57
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.main** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/bun-main.test.ts:7` — Bun.main > can be overridden → `expect(Bun.main).toBeString()`

## BUN58
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.nanoseconds** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/timers/performance.test.js:36` — performance.now() should be monotonic → `expect(Bun.nanoseconds()).toBeNumber(true)`

## BUN59
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.readableStreamToArrayBuffer** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/body-clone.test.ts:426` — ReadableStream with mixed content (starting with Uint8Array) can be converted to ArrayBuff… → `expect(arrayBuffer).toBeInstanceOf(ArrayBuffer)`

## BUN60
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.readableStreamToBytes** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/body-clone.test.ts:455` — ReadableStream with mixed content (starting with ArrayBuffer) can be converted to Uint8Arr… → `expect(uint8Array).toBeInstanceOf(Uint8Array)`

## BUN61
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.spawn** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/terminal/terminal.test.ts:984` — Bun.spawn with terminal option > creates subprocess with terminal attached → `expect(proc.terminal).toBeInstanceOf(Object)`

## BUN62
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.sliceAnsi** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 267)

Witnessed by 267 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/util/sliceAnsi.test.ts:115` — Bun.sliceAnsi > plain strings > slices ASCII string like String.prototype.slice → `expect(Bun.sliceAnsi("hello world", 0, 5)).toBe("hello")`
- `test/js/bun/util/sliceAnsi-fuzz.test.ts:157` — sliceAnsi adversarial > inputs near SIMD stride boundaries → `expect(Bun.sliceAnsi(s, 0, len)).toBe(s)`
- `test/js/bun/util/sliceAnsi.test.ts:116` — Bun.sliceAnsi > plain strings > slices ASCII string like String.prototype.slice → `expect(Bun.sliceAnsi("hello world", 6, 11)).toBe("world")`

## BUN63
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Bun.file** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 266)

Witnessed by 266 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/patch-bounds-check.test.ts:140` — patch application should work correctly with valid patches → `expect(patchedFile).toMatchInlineSnapshot(' "// Valid patch comment module.exports = require('./lodash');" ')`
- `test/regression/issue/cyclic-imports-async-bundler.test.js:93` — cyclic imports with async dependencies should generate async wrappers → `expect(bundled).toMatchInlineSnapshot(' "var __defProp = Object.defineProperty; var __returnValue = (v) => v; function __exportSetter(name, newValue) { this[name] = __returnValue.bind(null, newValue);…`
- `test/regression/issue/3192.test.ts:43` — issue #3192 > yarn lockfile quotes workspace:* versions correctly → `expect(yarnLock).toContain('"package-b@workspace:*"')`

## BUN64
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.stringWidth** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 266)

Witnessed by 266 constraint clauses across 4 test files. Antichain representatives:

- `test/js/bun/util/stringWidth.test.ts:121` — ambiguousIsNarrow=false → `expect(actual).toBe(npmStringWidth(string, { countAnsiEscapeCodes, ambiguousIsNarrow: false }))`
- `test/js/bun/util/sliceAnsi.test.ts:324` — Bun.sliceAnsi > emoji > counts emoji-style graphemes as fullwidth → `expect(Bun.stringWidth("\u{1F1E6}")).toBe(1)`
- `test/js/bun/util/sliceAnsi-fuzz.test.ts:175` — sliceAnsi adversarial > C1 ST at SIMD boundary positions → `expect(Bun.stringWidth(out)).toBe(pos + 1)`

## BUN65
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.file** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 198)

Witnessed by 198 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/26647.test.ts:40` — Bun.file().stat() should handle UTF-8 paths with Japanese characters → `expect(bunStat.size).toBe(Buffer.byteLength(content))`
- `test/regression/issue/14029.test.ts:41` — snapshots will recognize existing entries → `expect(newSnapshot).toBe(await Bun.file(join(testDir, "__snapshots__", "test.test.js.snap")).text())`
- `test/js/web/workers/message-channel.test.ts:284` — cloneable and non-transferable equals (BunFile) → `expect(file.name).toEqual(import.meta.filename)`

## BUN66
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.JSONL.parse** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 90)

Witnessed by 90 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsonl/jsonl-parse.test.ts:11` — Bun.JSONL > parse > complete input > objects separated by newlines → `expect(Bun.JSONL.parse('{"a":1}\n{"b":2}\n{"c":3}\n')).toStrictEqual([{ a: 1 }, { b: 2 }, { c: 3 }])`
- `test/js/bun/jsonl/jsonl-parse.test.ts:15` — Bun.JSONL > parse > complete input > single value with trailing newline → `expect(Bun.JSONL.parse('{"key":"value"}\n')).toStrictEqual([{ key: "value" }])`
- `test/js/bun/jsonl/jsonl-parse.test.ts:19` — Bun.JSONL > parse > complete input > single value without trailing newline → `expect(Bun.JSONL.parse('{"key":"value"}')).toStrictEqual([{ key: "value" }])`

## BUN67
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.spawnSync** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 76)

Witnessed by 76 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/25432.test.ts:26` — process.stdout.end() flushes pending writes before callback > large write followed by end(… → `expect(result.exitCode).toBe(0)`
- `test/regression/issue/22199.test.ts:27` — plugin onResolve returning undefined should not crash → `expect(result.exitCode).toBe(0)`
- `test/regression/issue/19652.test.ts:18` — bun build --production does not crash (issue #19652) → `expect(result.exitCode).toBe(0)`

## BUN68
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.wrapAnsi** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 71)

Witnessed by 71 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/util/wrapAnsi.test.ts:6` — Bun.wrapAnsi > basic wrapping > wraps text at word boundaries → `expect(Bun.wrapAnsi("hello world", 5)).toBe("hello\nworld")`
- `test/js/bun/util/wrapAnsi.npm.test.ts:52` — wraps string at 20 characters → `expect(result).toBe( "The quick brown \u001B[31mfox\u001B[39m\n\u001B[31mjumped over \u001B[39mthe lazy\n\u001B[32mdog and then ran\u001B[39m\n\u001B[32maway with the\u001B[39m\n\u001B[32municorn.\u00…`
- `test/js/bun/util/wrapAnsi.test.ts:10` — Bun.wrapAnsi > basic wrapping > handles empty string → `expect(Bun.wrapAnsi("", 10)).toBe("")`

## BUN69
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.stripANSI** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 48)

Witnessed by 48 constraint clauses across 4 test files. Antichain representatives:

- `test/regression/issue/27014.test.ts:11` — stripANSI does not hang on non-escape control characters → `expect(result).toBe(s)`
- `test/js/bun/util/stripANSI.test.ts:20` — Bun.stripANSI > returns new string when ANSI sequences are removed → `expect(result).toBe("hello world")`
- `test/js/bun/util/sliceAnsi.test.ts:641` — Bun.sliceAnsi > control sequences > keeps C1 SGR CSI behavior → `expect(Bun.stripANSI(Bun.sliceAnsi(input, 0, 3))).toBe("red")`

## BUN70
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.randomUUIDv5** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 31)

Witnessed by 31 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/randomUUIDv5.test.ts:14` — randomUUIDv5 > basic functionality → `expect(result[14]).toBe("5")`
- `test/js/bun/util/randomUUIDv5.test.ts:22` — randomUUIDv5 > deterministic output → `expect(uuid1).toBe(uuid2)`
- `test/js/bun/util/randomUUIDv5.test.ts:30` — randomUUIDv5 > compatibility with uuid library → `expect(bunUuid).toBe(uuidLibUuid)`

## BUN71
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.CookieMap** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 29)

Witnessed by 29 constraint clauses across 4 test files. Antichain representatives:

- `test/regression/issue/isArray-proxy-crash.test.ts:72` — isArray + Proxy crash fixes > new Bun.CookieMap does not crash with Proxy-wrapped array → `expect(map.size).toBe(0)`
- `test/js/bun/util/cookie.test.js:76` — Bun.CookieMap > can create an empty cookie map → `expect(cookieMap.size).toBe(0)`
- `test/js/bun/http/bun-serve-cookies.test.ts:425` — Direct usage of Bun.Cookie and Bun.CookieMap > can create a CookieMap directly → `expect(cookieMap.constructor).toBe(Bun.CookieMap)`

## BUN72
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Bun.inspect** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 24)

Witnessed by 24 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/hashbang-still-works.test.ts:41` — hashbang-still-works > lexer handles single # character without bounds error → `expect(errorMessage).toContain("error: Syntax Error")`
- `test/regression/issue/23022-stack-trace-iterator.test.ts:31` — V8StackTraceIterator handles frames without parentheses (issue #23022) → `expect(inspected).toContain("at unknown")`
- `test/js/web/websocket/error-event.test.ts:15` — WebSocket error event snapshot → `expect(Bun.inspect(error)).toMatchInlineSnapshot(' "ErrorEvent { type: "error", message: "WebSocket connection to 'ws://127.0.0.1:8080/' failed: Failed to connect", error: error: WebSocket connection …`

## BUN73
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Bun.build** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 19)

Witnessed by 19 constraint clauses across 3 test files. Antichain representatives:

- `test/internal/int_from_float.test.ts:36` — bun.intFromFloat function > handles normal finite values within range → `expect(result.logs).toHaveLength(0)`
- `test/bundler/bun-build-compile.test.ts:66` — Bun.build compile > compile with relative outfile paths → `expect(result1.outputs[0].path).toContain(join("output", "nested", isWindows ? "app1.exe" : "app1"))`
- `test/bundler/bun-build-api.test.ts:30` — Bun.build > css works → `expect(build.outputs).toHaveLength(1)`

## BUN74
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.hash** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 18)

Witnessed by 18 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/09555.test.ts:39` — #09555 > fetch() Response body → `expect(out).toBe(sha)`
- `test/js/web/fetch/fetch-http3-client.test.ts:269` — fetch protocol: http3 > type:direct large response → `expect(Bun.hash(buf)).toBe(Bun.hash(Buffer.concat(Array.from({ length: 8 }, () => big.subarray(0, 32 * 1024)))))`
- `test/js/web/fetch/fetch-http3-adversarial.test.ts:138` — POST /echo (Uint8Array) → `expect(Bun.hash(got)).toBe(want)`

## BUN75
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Bun.stringWidth** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 3 test files. Antichain representatives:

- `test/js/bun/util/stringWidth.test.ts:595` — stringWidth extended > fuzzer-like stress tests > CSI without final byte (unterminated) → `expect(Bun.stringWidth(input)).toBeGreaterThanOrEqual(1)`
- `test/js/bun/util/sliceAnsi.test.ts:919` — Bun.sliceAnsi > stress tests > mixed content performance → `expect(Bun.stringWidth(result)).toBeLessThanOrEqual(50)`
- `test/js/bun/util/sliceAnsi-fuzz.test.ts:544` — sliceAnsi ambiguousIsNarrow fuzz > narrow slice ⊆ wide slice visibly (narrow chars are s… → `expect(Bun.stringWidth(Bun.stripANSI(narrow), { ambiguousIsNarrow: true })).toBeLessThanOrEqual(budget + 1)`

## BUN76
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.write** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/io/bun-write.test.js:26` — Bun.write blob → `expect(await Bun.write(new TextEncoder().encode(tmpbase + "response-file.test.txt"), new Uint32Array(1024))).toBe( new Uint32Array(1024).byteLength, )`
- `test/js/bun/bun-object/write.spec.ts:74` — Bun.write() on file paths > Given a path to a file in an existing directory > When the fil… → `expect(result).toBe(content.length)`
- `test/js/bun/io/bun-write.test.js:59` — large file > write large file (bytes) → `expect(written).toBe(bytes.byteLength)`

## BUN77
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.build** — exhibits the property captured in the witnessing test. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/22317.test.ts:21` — issue 22317: build with CSS file entry points mixed with JS should not crash → `expect(result.success).toBeTrue()`
- `test/integration/svelte/client-side.test.ts:31` — generating client-side code > Bundling Svelte components → `expect(result.success).toBeTrue()`
- `test/bundler/native-plugin.test.ts:117` — native-plugins > works in a basic case → `expect(result.success).toBeTrue()`

## BUN78
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Bun.wrapAnsi** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 10)

Witnessed by 10 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/wrapAnsi.test.ts:65` — Bun.wrapAnsi > ANSI escape codes > preserves simple color code → `expect(result).toContain("\x1b[31m")`
- `test/js/bun/util/wrapAnsi.test.ts:66` — Bun.wrapAnsi > ANSI escape codes > preserves simple color code → `expect(result).toContain("hello")`
- `test/js/bun/util/wrapAnsi.test.ts:73` — Bun.wrapAnsi > ANSI escape codes > preserves color across line break → `expect(result).toContain("\x1b[39m\n")`

## BUN79
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Bun.JSONL.parseChunk** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsonl/jsonl-parse.test.ts:1048` — Bun.JSONL > fuzz-like stress tests > garbage input > null bytes in input → `expect(result.values.length).toBeGreaterThanOrEqual(1)`
- `test/js/bun/jsonl/jsonl-parse.test.ts:1793` — Bun.JSONL > fuzz-like stress tests > adversarial input > start/end boundary security > eve… → `expect(result.read).toBeGreaterThanOrEqual(0)`
- `test/js/bun/jsonl/jsonl-parse.test.ts:1794` — Bun.JSONL > fuzz-like stress tests > adversarial input > start/end boundary security > eve… → `expect(result.read).toBeLessThanOrEqual(buf.length)`

## BUN80
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.CryptoHasher.hash** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/09555.test.ts:98` — #09555 > Bun.serve() Request body streaming → `expect(out).toEqual(sha)`
- `test/js/bun/util/bun-cryptohasher.test.ts:126` — Hash is consistent > base64 → `expect(Bun.CryptoHasher.hash(algorithm, buffer, "base64")).toEqual( Bun.CryptoHasher.hash(algorithm, buffer, "base64"), )`
- `test/regression/issue/09555.test.ts:124` — #09555 > Bun.serve() Request body buffered → `expect(out).toEqual(sha)`

