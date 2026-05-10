# platform-cfg/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: platform-cfg-js-surface-property
  threshold: PLAT1
  interface: [URL, Headers, URLSearchParams, Event, AbortController, Bun.cron, FormData, Bun.Terminal, performance.now, CustomEvent, EventTarget, os.userInfo, TextEncoder, process.env, Bun.cron.remove, path]

@imports: []

@pins: []

Surface drawn from 80 candidate properties across the Bun test corpus. Construction-style: 78; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 847.

## PLAT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**URL** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 219 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:6` — exists → `expect(typeof URL !== "undefined").toBe(true)`
- `test/js/web/url/url.test.ts:24` — url > should have correct origin and protocol → `expect(url.protocol).toBe("https:")`
- `test/js/web/fetch/fetch.test.ts:1949` — should allow very long redirect URLS → `expect(url).toBe('${server.url.origin}${Location}')`

## PLAT2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Headers** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 74 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:14` — exists → `expect(typeof Headers !== "undefined").toBe(true)`
- `test/js/web/fetch/headers.undici.test.ts:99` — Headers append > adds valid header entry to instance → `expect(headers.get(name)).toBe(value)`
- `test/js/web/fetch/headers.test.ts:24` — Headers > constructor > can create headers from object → `expect(headers.get("content-type")).toBe("text/plain")`

## PLAT3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**URLSearchParams** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 68 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:7` — exists → `expect(typeof URLSearchParams !== "undefined").toBe(true)`
- `test/js/web/html/URLSearchParams.test.ts:88` — URLSearchParams > does not crash when calling .toJSON() on a URLSearchParams object with a… → `expect(params.toJSON()).toEqual(props)`
- `test/js/web/html/FormData.test.ts:571` — FormData > URLEncoded > should parse URLSearchParams → `expect(searchParams instanceof URLSearchParams).toBe(true)`

## PLAT4
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

## PLAT5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortController** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 18 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:11` — exists → `expect(typeof AbortController !== "undefined").toBe(true)`
- `test/js/web/abort/abort.test.ts:81` — AbortSignal > .signal.reason should be a DOMException → `expect(ac.signal.reason.code).toBe(20)`
- `test/js/node/util/test-aborted.test.ts:13` — aborted works when provided a resource that was already aborted → `expect(ac.signal.aborted).toBe(true)`

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

**FormData** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 15 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:21` — exists → `expect(typeof FormData !== "undefined").toBe(true)`
- `test/js/web/html/FormData.test.ts:8` — FormData > should be able to append a string → `expect(formData.get("foo")).toBe("bar")`
- `test/js/web/html/FormData.test.ts:9` — FormData > should be able to append a string → `expect(formData.getAll("foo")[0]).toBe("bar")`

## PLAT8
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

## PLAT9
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

## PLAT10
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**CustomEvent** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 9 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:13` — exists → `expect(typeof CustomEvent !== "undefined").toBe(true)`
- `test/js/deno/event/custom-event.test.ts:17` —  → `assertEquals(event.bubbles, true)`
- `test/js/deno/event/custom-event.test.ts:18` —  → `assertEquals(event.cancelable, true)`

## PLAT11
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**EventTarget** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:10` — exists → `expect(typeof EventTarget !== "undefined").toBe(true)`
- `test/js/deno/event/event-target.test.ts:8` —  → `assertEquals(document.addEventListener("x", null, false), undefined)`
- `test/js/deno/event/event-target.test.ts:9` —  → `assertEquals(document.addEventListener("x", null, true), undefined)`

## PLAT12
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.userInfo** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:119` — userInfo → `expect(info.username).toBe(process.env.USER)`
- `test/js/node/os/os.test.js:120` — userInfo → `expect(info.shell).toBe(process.env.SHELL || "unknown")`
- `test/js/node/os/os.test.js:121` — userInfo → `expect(info.uid >= 0).toBe(true)`

## PLAT13
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**TextEncoder** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:18` — exists → `expect(typeof TextEncoder !== "undefined").toBe(true)`
- `test/js/web/encoding/text-encoder.test.js:28` — TextEncoder > should handle undefined → `expect(encoder.encode(undefined).length).toBe(0)`
- `test/js/deno/encoding/encoding.test.ts:287` —  → `assertEquals(encoder.toString(), "[object TextEncoder]")`

## PLAT14
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

## PLAT15
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

## PLAT16
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

## PLAT17
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

## PLAT18
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

## PLAT19
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

## PLAT20
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

## PLAT21
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.hrtime** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/process/process.test.js:125` — process.hrtime() → `expect(end[0]).toBe(0)`
- `test/js/node/process/process.test.js:131` — process.hrtime() → `expect(end2[1] > start[1]).toBe(true)`
- `test/js/node/process/process-array-accessor-crash.test.ts:41` — process.hrtime > does not crash when array has an accessor property → `expect(result.length).toBe(2)`

## PLAT22
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.Image.fromClipboard** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/image/image.test.ts:1456` — Bun.Image clipboard statics > hasClipboardImage / clipboardChangeCount / fromClipboard hav… → `expect(img === null || img instanceof Bun.Image).toBe(true)`
- `test/js/bun/image/image.test.ts:1460` — Bun.Image clipboard statics > hasClipboardImage / clipboardChangeCount / fromClipboard hav… → `expect(Bun.Image.fromClipboard()).toBe(null)`

## PLAT23
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.secrets.delete** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/secrets-error-codes.test.ts:20` — delete non-existent returns false without error → `expect(result).toBe(false)`
- `test/js/bun/secrets-error-codes.test.ts:68` — successful operations work correctly → `expect(deleted).toBe(true)`

## PLAT24
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.secrets.get** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/secrets-error-codes.test.ts:11` — non-existent secret returns null without error → `expect(result).toBeNull()`
- `test/js/bun/secrets-error-codes.test.ts:72` — successful operations work correctly → `expect(afterDelete).toBeNull()`

## PLAT25
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.getuid** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:616` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.getuid).toBeUndefined()`
- `test/js/node/process/process.test.js:617` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.getuid).toBeUndefined()`

## PLAT26
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**AbortSignal** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:12` — exists → `expect(typeof AbortSignal !== "undefined").toBe(true)`

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
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:15` — it exists → `expect(dns).toBeDefined()`

## PLAT30
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.lookup** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:16` — it exists → `expect(dns.lookup).toBeDefined()`

## PLAT31
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.lookupService** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:17` — it exists → `expect(dns.lookupService).toBeDefined()`

## PLAT32
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:31` — it exists → `expect(dns.promises).toBeDefined()`

## PLAT33
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.lookup** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:32` — it exists → `expect(dns.promises.lookup).toBeDefined()`

## PLAT34
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.lookupService** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:33` — it exists → `expect(dns.promises.lookupService).toBeDefined()`

## PLAT35
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolve** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:34` — it exists → `expect(dns.promises.resolve).toBeDefined()`

## PLAT36
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolve4** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:35` — it exists → `expect(dns.promises.resolve4).toBeDefined()`

## PLAT37
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolve6** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:36` — it exists → `expect(dns.promises.resolve6).toBeDefined()`

## PLAT38
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveMx** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:41` — it exists → `expect(dns.promises.resolveMx).toBeDefined()`

## PLAT39
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveNaptr** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:40` — it exists → `expect(dns.promises.resolveNaptr).toBeDefined()`

## PLAT40
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveSoa** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:39` — it exists → `expect(dns.promises.resolveSoa).toBeDefined()`

## PLAT41
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveSrv** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:37` — it exists → `expect(dns.promises.resolveSrv).toBeDefined()`

## PLAT42
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolveTxt** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:38` — it exists → `expect(dns.promises.resolveTxt).toBeDefined()`

## PLAT43
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolve** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:18` — it exists → `expect(dns.resolve).toBeDefined()`

## PLAT44
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolve4** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:19` — it exists → `expect(dns.resolve4).toBeDefined()`

## PLAT45
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolve6** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:20` — it exists → `expect(dns.resolve6).toBeDefined()`

## PLAT46
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveCaa** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:26` — it exists → `expect(dns.resolveCaa).toBeDefined()`

## PLAT47
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveCname** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:29` — it exists → `expect(dns.resolveCname).toBeDefined()`

## PLAT48
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveMx** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:25` — it exists → `expect(dns.resolveMx).toBeDefined()`

## PLAT49
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveNaptr** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:24` — it exists → `expect(dns.resolveNaptr).toBeDefined()`

## PLAT50
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveNs** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:27` — it exists → `expect(dns.resolveNs).toBeDefined()`

## PLAT51
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolvePtr** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:28` — it exists → `expect(dns.resolvePtr).toBeDefined()`

## PLAT52
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveSoa** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:23` — it exists → `expect(dns.resolveSoa).toBeDefined()`

## PLAT53
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveSrv** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:21` — it exists → `expect(dns.resolveSrv).toBeDefined()`

## PLAT54
type: specification
authority: derived
scope: module
status: active
depends-on: []

**dns.resolveTxt** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:22` — it exists → `expect(dns.resolveTxt).toBeDefined()`

## PLAT55
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.glob** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/glob.test.ts:33` — fs.glob > has a length of 3 → `expect(typeof fs.glob).toEqual("function")`

## PLAT56
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.promises.glob** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/glob.test.ts:165` — fs.promises.glob > has a length of 2 → `expect(typeof fs.promises.glob).toBe("function")`

## PLAT57
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**globalThis.navigator** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:233` — navigator → `expect(globalThis.navigator !== undefined).toBe(true)`

## PLAT58
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.homedir** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:61` — homedir → `expect(os.homedir() !== "unknown").toBe(true)`

## PLAT59
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.hostname** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:88` — hostname → `expect(os.hostname() !== "unknown").toBe(true)`

## PLAT60
type: specification
authority: derived
scope: module
status: active
depends-on: []

**os.loadavg** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:52` — loadavg → `expect(actual).toBeArrayOfSize(3)`

## PLAT61
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.release** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:96` — release → `expect(os.release().length > 1).toBe(true)`

## PLAT62
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.uptime** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:104` — uptime → `expect(os.uptime() > 0).toBe(true)`

## PLAT63
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**os.version** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/os/os.test.js:108` — version → `expect(typeof os.version() === "string").toBe(true)`

## PLAT64
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:700` — process.${stub} → `expect(process[stub]()).toBeUndefined()`

## PLAT65
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.constrainedMemory** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:762` — process.constrainedMemory() → `expect(process.constrainedMemory() >= 0).toBe(true)`

## PLAT66
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.execve** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process-execve.test.ts:6` — process.execve > is a function → `expect(typeof process.execve).toBe("function")`

## PLAT67
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.getegid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:595` — process.getegid → `expect(typeof process.getegid()).toBe("number")`

## PLAT68
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.getegid** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:612` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.getegid).toBeUndefined()`

## PLAT69
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.geteuid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:598` — process.geteuid → `expect(typeof process.geteuid()).toBe("number")`

## PLAT70
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.geteuid** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:613` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.geteuid).toBeUndefined()`

## PLAT71
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.getgid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:601` — process.getgid → `expect(typeof process.getgid()).toBe("number")`

## PLAT72
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.getgid** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:614` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.getgid).toBeUndefined()`

## PLAT73
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.getgroups** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:604` — process.getgroups → `expect(process.getgroups()).toBeInstanceOf(Array)`

## PLAT74
type: specification
authority: derived
scope: module
status: active
depends-on: []

**process.getgroups** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:615` — process.getegid, process.geteuid, process.getgid, process.getgroups, process.getuid, proce… → `expect(process.getgroups).toBeUndefined()`

## PLAT75
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.getuid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:608` — process.getuid → `expect(typeof process.getuid()).toBe("number")`

## PLAT76
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.hrtime.bigint** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/process/process.test.js:137` — process.hrtime.bigint() → `expect(end > start).toBe(true)`

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

**glob.match** — exhibits the property captured in the witnessing test. (behavioral; cardinality 151)

Witnessed by 151 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/glob/match.test.ts:40` — Glob.match > single wildcard → `expect(glob.match("foo")).toBeTrue()`
- `test/js/bun/glob/match.test.ts:41` — Glob.match > single wildcard → `expect(glob.match("lmao.ts")).toBeTrue()`
- `test/js/bun/glob/match.test.ts:42` — Glob.match > single wildcard → `expect(glob.match("")).toBeTrue()`

## PLAT80
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ctx.redis.send** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 106)

Witnessed by 106 constraint clauses across 5 test files. Antichain representatives:

- `test/js/valkey/unit/list-operations.test.ts:27` — Basic List Operations > LPUSH and RPUSH commands → `expect(lpushResult).toBe(1)`
- `test/js/valkey/unit/hash-operations.test.ts:28` — Basic Hash Commands > HSET and HGET commands → `expect(setResult).toBe(1)`
- `test/js/valkey/unit/basic-operations.test.ts:85` — String Commands > APPEND command → `expect(newLength).toBe(initialValue.length + appendValue.length)`

