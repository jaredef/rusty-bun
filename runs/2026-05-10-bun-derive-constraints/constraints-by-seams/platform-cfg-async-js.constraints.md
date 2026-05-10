# platform-cfg/async/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: platform-cfg-async-js-surface-property
  threshold: PLAT1
  interface: [Blob, File, Bun.WebView]

@imports: []

@pins: []

Surface drawn from 9 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 6. Total witnessing constraint clauses: 178.

## PLAT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Blob** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 40 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:20` — exists → `expect(typeof Blob !== "undefined").toBe(true)`
- `test/js/web/streams/streams.test.js:670` — ReadableStream for Blob → `expect(await blob.text()).toBe("abdefghijklmnop")`
- `test/js/web/html/FormData.test.ts:222` — FormData > should roundtrip multipart/form-data (${name}) with ${C.name} → `expect(c instanceof FormData).toBe(true)`

## PLAT2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**File** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 26 constraint clauses across 4 test files. Antichain representatives:

- `test/js/web/workers/structured-clone.test.ts:206` — bun blobs work > dom file > without lastModified → `expect(file.name).toBe("example.txt")`
- `test/js/web/web-globals.test.js:23` — exists → `expect(typeof File !== "undefined").toBe(true)`
- `test/js/web/fetch/blob.test.ts:188` — new File(new Uint8Array()) is supported → `expect(await blob.text()).toBe("1234")`

## PLAT3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.WebView** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/webview/webview.test.ts:87` — is an EventTarget → `expect(typeof view.addEventListener).toBe("function")`
- `test/js/bun/webview/webview-chrome-ws.test.ts:106` — connect via full ws:// URL → `expect(await view.evaluate("document.getElementById('t').textContent")).toBe("ws-direct")`
- `test/js/bun/webview/webview.test.ts:88` — is an EventTarget → `expect(typeof view.removeEventListener).toBe("function")`

## PLAT4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ctx.redis.hget** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 25)

Witnessed by 25 constraint clauses across 2 test files. Antichain representatives:

- `test/js/valkey/valkey.test.ts:5458` — Hash Operations > should get and delete multiple hash fields using hgetdel → `expect(await redis.hget(key, "age")).toBe("30")`
- `test/js/valkey/unit/hash-operations.test.ts:48` — Basic Hash Commands > HGET native method → `expect(result).toBe("johndoe")`
- `test/js/valkey/valkey.test.ts:5569` — Hash Operations > should set multiple hash fields with expiration using hsetex → `expect(await redis.hget(key, "name")).toBe("John")`

## PLAT5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fetch** — exhibits the property captured in the witnessing test. (behavioral; cardinality 22)

Witnessed by 22 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/fetch/body-stream.test.ts:77` — Should not crash when not returning a promise when stream is in progress → `expect(await fetch(url).then(res => res.text())).toBeOneOf(["hey", ""])`
- `test/js/bun/s3/s3.test.ts:283` — ${credentials.service} > fetch > should be able to set content-type → `expect(response.headers.get("content-type")).toStartWith("application/json")`
- `test/js/bun/http/serve.test.ts:1449` — #5859 json → `expect(response.ok).toBeFalse()`

## PLAT6
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Bun.spawn** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 18)

Witnessed by 18 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/fetch/fetch.tls.test.ts:422` — fetch-tls > fetch should ignore invalid NODE_EXTRA_CA_CERTS → `expect(await proc.stderr.text()).toContain("DEPTH_ZERO_SELF_SIGNED_CERT")`
- `test/js/node/process/process.test.js:814` — aborts when the uncaughtException handler throws → `expect(await proc.stderr.text()).toContain("bar")`
- `test/js/node/process/process-stdin.test.ts:28` — file does the right thing → `expect(await result.stdout.text()).toMatchInlineSnapshot(' "undefined " ')`

## PLAT7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**readArchive.files** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 17)

Witnessed by 17 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/s3/s3.test.ts:1617` — writes archive to S3 via S3Client.write() → `expect(files.size).toBe(2)`
- `test/js/bun/archive.test.ts:1648` — Bun.Archive > Bun.write with Archive > writes archive to local file → `expect(files.size).toBe(2)`
- `test/js/bun/s3/s3.test.ts:1618` — writes archive to S3 via S3Client.write() → `expect(await files.get("hello.txt")!.text()).toBe("Hello from Archive!")`

## PLAT8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**RedisClient** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 15)

Witnessed by 15 constraint clauses across 3 test files. Antichain representatives:

- `test/js/valkey/valkey.test.ts:6881` — duplicate() > should duplicate client that failed to connect → `expect(failedRedis.connected).toBe(false)`
- `test/js/valkey/reliability/recovery.test.ts:37` — client.connect() recovers after the client enters the failed state → `expect(await client.get("recovery:k")).toBe("before")`
- `test/js/valkey/reliability/connection-failures.test.ts:46` — Connection Failure Handling > should reject commands with appropriate errors when disconne… → `expect(client.connected).toBe(false)`

## PLAT9
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**response.blob** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/streams/streams.test.js:1025` — new Response(stream).blob() (bytes) → `expect(await blob.text()).toBe('{"hello":true}')`
- `test/js/web/fetch/fetch.test.ts:928` — ${jsonObject.hello === true ? "latin1" : "utf16"} blob${withGC ? " (with gc) " : ""} → `expect(blobed instanceof Blob).toBe(true)`
- `test/js/bun/stream/direct-readable-stream.test.tsx:152` — Response.blob() → `expect(text.replaceAll("<!-- -->", "")).toBe(inputString)`

