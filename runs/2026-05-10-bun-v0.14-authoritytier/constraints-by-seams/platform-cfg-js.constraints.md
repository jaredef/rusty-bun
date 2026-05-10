# platform-cfg/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: platform-cfg-js-surface-property
  threshold: PLAT1
  interface: [URL, Event, File, AbortController, Blob, Bun.cron, Bun.Terminal, URLSearchParams, path.toNamespacedPath, performance.now, Headers, CustomEvent, Bun.WebView, os.userInfo, process.env, Bun.cron.remove]

@imports: []

@pins: []

Surface drawn from 80 candidate properties across the Bun test corpus. Construction-style: 79; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 643.

## PLAT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**URL** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 215 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:6` — exists → `expect(typeof URL !== "undefined").toBe(true)`
- `test/js/web/url/url.test.ts:24` — url > should have correct origin and protocol → `expect(url.protocol).toBe("https:")`
- `test/js/web/fetch/fetch.test.ts:1949` — should allow very long redirect URLS → `expect(url).toBe('${server.url.origin}${Location}')`

## PLAT2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Event** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 30 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:9` — exists → `expect(typeof Event !== "undefined").toBe(true)`
- `test/js/deno/event/event.test.ts:9` —  → `assertEquals(event.isTrusted, false)`
- `test/js/deno/event/event-target.test.ts:190` —  → `assertEquals(event.target, null)`

## PLAT3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**File** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 22 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/workers/structured-clone.test.ts:206` — bun blobs work > dom file > without lastModified → `expect(file.name).toBe("example.txt")`
- `test/js/web/web-globals.test.js:23` — exists → `expect(typeof File !== "undefined").toBe(true)`
- `test/js/web/fetch/blob.test.ts:189` — new File(new Uint8Array()) is supported → `expect(blob.name).toBe("file.txt")`

## PLAT4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortController** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 19 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:11` — exists → `expect(typeof AbortController !== "undefined").toBe(true)`
- `test/js/web/abort/abort.test.ts:81` — AbortSignal > .signal.reason should be a DOMException → `expect(ac.signal.reason.code).toBe(20)`
- `test/js/node/util/test-aborted.test.ts:13` — aborted works when provided a resource that was already aborted → `expect(ac.signal.aborted).toBe(true)`

## PLAT5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Blob** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 17 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:20` — exists → `expect(typeof Blob !== "undefined").toBe(true)`
- `test/js/web/html/FormData.test.ts:235` — FormData > should roundtrip multipart/form-data (${name}) with ${C.name} → `expect(c).toBe(b)`
- `test/js/web/fetch/blob.test.ts:98` — new Blob → `expect(blob.size).toBe(6)`

## PLAT6
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

## PLAT7
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

## PLAT8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**URLSearchParams** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 14 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:7` — exists → `expect(typeof URLSearchParams !== "undefined").toBe(true)`
- `test/js/web/html/URLSearchParams.test.ts:100` — URLSearchParams > non-standard extensions > should support .length → `expect(params.length).toBe(3)`
- `test/js/deno/url/urlsearchparams.test.ts:11` —  → `assertEquals(searchParams, "str=this+string+has+spaces+in+it")`

## PLAT9
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path.toNamespacedPath** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 12 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/path/to-namespaced-path.test.js:11` — path.toNamespacedPath > platform → `assert.strictEqual(path.toNamespacedPath(""), "")`
- `test/js/node/path/to-namespaced-path.test.js:12` — path.toNamespacedPath > platform → `assert.strictEqual(path.toNamespacedPath(null), null)`
- `test/js/node/path/to-namespaced-path.test.js:13` — path.toNamespacedPath > platform → `assert.strictEqual(path.toNamespacedPath(100), 100)`

## PLAT10
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**performance.now** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 12 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/timers/setInterval.test.js:31` — setInterval → `expect(performance.now() - start > 9).toBe(true)`
- `test/js/bun/test/fake-timers/fake-timers.test.ts:300` — performance.now() mocking > performance.now() should be mocked when fake timers are active → `expect(performance.now()).toBe(1000)`
- `test/js/bun/test/fake-timers/fake-timers.test.ts:304` — performance.now() mocking > performance.now() should be mocked when fake timers are active → `expect(performance.now()).toBe(1500)`

## PLAT11
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Headers** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 10 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:14` — exists → `expect(typeof Headers !== "undefined").toBe(true)`
- `test/js/web/fetch/headers.test.ts:495` — Headers > count > can count headers when empty → `expect(headers.count).toBe(0)`
- `test/js/web/fetch/fetch.test.ts:412` — Headers > .getSetCookie() with object → `expect(headers.count).toBe(5)`

## PLAT12
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**CustomEvent** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:13` — exists → `expect(typeof CustomEvent !== "undefined").toBe(true)`
- `test/js/deno/event/custom-event.test.ts:17` —  → `assertEquals(event.bubbles, true)`
- `test/js/deno/event/custom-event.test.ts:18` —  → `assertEquals(event.cancelable, true)`

## PLAT13
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

## PLAT14
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.userInfo** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:119` — userInfo → `expect(info.username).toBe(process.env.USER)`
- `test/js/node/os/os.test.js:120` — userInfo → `expect(info.shell).toBe(process.env.SHELL || "unknown")`
- `test/js/node/os/os.test.js:124` — userInfo → `expect(info.username).toBe(process.env.USERNAME)`

## PLAT15
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.env** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:153` — process.env → `expect(process.env["LOL SMILE UTF16 😂"]).toBe("😂")`
- `test/js/node/process/process.test.js:155` — process.env → `expect(process.env["LOL SMILE UTF16 😂"]).toBe(undefined)`
- `test/js/node/process/process.test.js:158` — process.env → `expect(process.env["LOL SMILE latin1 <abc>"]).toBe("<abc>")`

## PLAT16
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

## PLAT17
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**path** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 5 constraint clauses across 4 test files. Antichain representatives:

- `test/js/node/path/path.test.js:58` — path > path.delimiter → `assert.strictEqual(path, path.win32)`
- `test/js/node/fs/glob.test.ts:206` — fs.promises.glob > matches directories → `expect(path).toBe("folder.test")`
- `test/js/bun/resolve/load-same-js-file-a-lot.test.ts:27` — load the same file ${count} times → `expect(path).toBe(meta.path)`

## PLAT18
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.setPriority** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:31` — setPriority → `expect(os.setPriority(0, 10)).toBe(undefined)`
- `test/js/node/os/os.test.js:33` — setPriority → `expect(os.setPriority(0)).toBe(undefined)`
- `test/js/node/os/os.test.js:36` — setPriority → `expect(os.setPriority(0, 2)).toBe(undefined)`

## PLAT19
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

## PLAT20
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

## PLAT21
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process** — exposes values of the expected type or class. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:708` — process.${stub} → `expect(process[stub]()).toBeInstanceOf(Array)`
- `test/js/node/process/process.test.js:724` — process.${stub} → `expect(process[stub]).toBeInstanceOf(Set)`
- `test/js/node/process/process.test.js:731` — process.${stub} → `expect(process[stub]).toBeInstanceOf(Array)`

## PLAT22
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.secrets.delete** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/secrets-error-codes.test.ts:20` — delete non-existent returns false without error → `expect(result).toBe(false)`
- `test/js/bun/secrets-error-codes.test.ts:68` — successful operations work correctly → `expect(deleted).toBe(true)`

## PLAT23
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.secrets.get** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/secrets-error-codes.test.ts:11` — non-existent secret returns null without error → `expect(result).toBeNull()`
- `test/js/bun/secrets-error-codes.test.ts:72` — successful operations work correctly → `expect(afterDelete).toBeNull()`

## PLAT24
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.getuid** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:616` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.getuid).toBeUndefined()`
- `test/js/node/process/process.test.js:617` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.getuid).toBeUndefined()`

## PLAT25
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:12` — exists → `expect(typeof AbortSignal !== "undefined").toBe(true)`

## PLAT26
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.Image.fromClipboard** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/image/image.test.ts:1460` — Bun.Image clipboard statics > hasClipboardImage / clipboardChangeCount / fromClipboard hav… → `expect(Bun.Image.fromClipboard()).toBe(null)`

## PLAT27
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.inspect** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/third_party/mongodb/mongodb.test.ts:23` — should connect and inpect → `expect(text).toBeDefined()`

## PLAT28
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.nanoseconds** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/timers/performance.test.js:36` — performance.now() should be monotonic → `expect(Bun.nanoseconds()).toBeNumber(true)`

## PLAT29
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**EventTarget** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:10` — exists → `expect(typeof EventTarget !== "undefined").toBe(true)`

## PLAT30
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:15` — it exists → `expect(dns).toBeDefined()`

## PLAT31
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.lookup** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:16` — it exists → `expect(dns.lookup).toBeDefined()`

## PLAT32
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.lookupService** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:17` — it exists → `expect(dns.lookupService).toBeDefined()`

## PLAT33
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:31` — it exists → `expect(dns.promises).toBeDefined()`

## PLAT34
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.lookup** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:32` — it exists → `expect(dns.promises.lookup).toBeDefined()`

## PLAT35
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.lookupService** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:33` — it exists → `expect(dns.promises.lookupService).toBeDefined()`

## PLAT36
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolve** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:34` — it exists → `expect(dns.promises.resolve).toBeDefined()`

## PLAT37
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolve4** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:35` — it exists → `expect(dns.promises.resolve4).toBeDefined()`

## PLAT38
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolve6** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:36` — it exists → `expect(dns.promises.resolve6).toBeDefined()`

## PLAT39
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveMx** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:41` — it exists → `expect(dns.promises.resolveMx).toBeDefined()`

## PLAT40
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveNaptr** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:40` — it exists → `expect(dns.promises.resolveNaptr).toBeDefined()`

## PLAT41
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveSoa** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:39` — it exists → `expect(dns.promises.resolveSoa).toBeDefined()`

## PLAT42
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveSrv** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:37` — it exists → `expect(dns.promises.resolveSrv).toBeDefined()`

## PLAT43
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveTxt** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:38` — it exists → `expect(dns.promises.resolveTxt).toBeDefined()`

## PLAT44
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolve** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:18` — it exists → `expect(dns.resolve).toBeDefined()`

## PLAT45
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolve4** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:19` — it exists → `expect(dns.resolve4).toBeDefined()`

## PLAT46
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolve6** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:20` — it exists → `expect(dns.resolve6).toBeDefined()`

## PLAT47
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveCaa** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:26` — it exists → `expect(dns.resolveCaa).toBeDefined()`

## PLAT48
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveCname** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:29` — it exists → `expect(dns.resolveCname).toBeDefined()`

## PLAT49
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveMx** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:25` — it exists → `expect(dns.resolveMx).toBeDefined()`

## PLAT50
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveNaptr** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:24` — it exists → `expect(dns.resolveNaptr).toBeDefined()`

## PLAT51
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveNs** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:27` — it exists → `expect(dns.resolveNs).toBeDefined()`

## PLAT52
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolvePtr** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:28` — it exists → `expect(dns.resolvePtr).toBeDefined()`

## PLAT53
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveSoa** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:23` — it exists → `expect(dns.resolveSoa).toBeDefined()`

## PLAT54
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveSrv** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:21` — it exists → `expect(dns.resolveSrv).toBeDefined()`

## PLAT55
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveTxt** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:22` — it exists → `expect(dns.resolveTxt).toBeDefined()`

## PLAT56
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.glob** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/glob.test.ts:33` — fs.glob > has a length of 3 → `expect(typeof fs.glob).toEqual("function")`

## PLAT57
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.promises.glob** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/glob.test.ts:165` — fs.promises.glob > has a length of 2 → `expect(typeof fs.promises.glob).toBe("function")`

## PLAT58
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**globalThis.navigator** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:233` — navigator → `expect(globalThis.navigator !== undefined).toBe(true)`

## PLAT59
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.homedir** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:61` — homedir → `expect(os.homedir() !== "unknown").toBe(true)`

## PLAT60
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.hostname** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:88` — hostname → `expect(os.hostname() !== "unknown").toBe(true)`

## PLAT61
type: specification
authority: derived
scope: module
status: active
depends-on: []

**os.loadavg** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:52` — loadavg → `expect(actual).toBeArrayOfSize(3)`

## PLAT62
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.release** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:96` — release → `expect(os.release().length > 1).toBe(true)`

## PLAT63
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.uptime** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:104` — uptime → `expect(os.uptime() > 0).toBe(true)`

## PLAT64
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.version** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:108` — version → `expect(typeof os.version() === "string").toBe(true)`

## PLAT65
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:700` — process.${stub} → `expect(process[stub]()).toBeUndefined()`

## PLAT66
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.constrainedMemory** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:762` — process.constrainedMemory() → `expect(process.constrainedMemory() >= 0).toBe(true)`

## PLAT67
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.execve** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process-execve.test.ts:6` — process.execve > is a function → `expect(typeof process.execve).toBe("function")`

## PLAT68
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.getegid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:595` — process.getegid → `expect(typeof process.getegid()).toBe("number")`

## PLAT69
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.getegid** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:612` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.getegid).toBeUndefined()`

## PLAT70
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.geteuid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:598` — process.geteuid → `expect(typeof process.geteuid()).toBe("number")`

## PLAT71
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.geteuid** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:613` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.geteuid).toBeUndefined()`

## PLAT72
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.getgid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:601` — process.getgid → `expect(typeof process.getgid()).toBe("number")`

## PLAT73
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.getgid** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:614` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.getgid).toBeUndefined()`

## PLAT74
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.getgroups** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:604` — process.getgroups → `expect(process.getgroups()).toBeInstanceOf(Array)`

## PLAT75
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.getgroups** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:615` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.getgroups).toBeUndefined()`

## PLAT76
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.getuid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:608` — process.getuid → `expect(typeof process.getuid()).toBe("number")`

## PLAT77
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.kill** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:653` — signal > process.kill(2) works → `expect(ret).toBe(true)`

## PLAT78
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.version.startsWith** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:216` — process.version starts with v → `expect(process.version.startsWith("v")).toBeTruthy()`

## PLAT79
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**url.href.endsWith** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/fileUrl.test.js:13` — pathToFileURL > should handle relative paths longer than PATH_MAX → `expect(url.href.endsWith("/" + long)).toBe(true)`

## PLAT80
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**glob.match** — exhibits the property captured in the witnessing test. (behavioral; cardinality 152)

Witnessed by 152 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/glob/match.test.ts:40` — Glob.match > single wildcard → `expect(glob.match("foo")).toBeTrue()`
- `test/js/bun/glob/match.test.ts:41` — Glob.match > single wildcard → `expect(glob.match("lmao.ts")).toBeTrue()`
- `test/js/bun/glob/match.test.ts:42` — Glob.match > single wildcard → `expect(glob.match("")).toBeTrue()`

