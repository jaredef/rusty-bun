# platform-cfg/async/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: platform-cfg-async-js-surface-property
  threshold: PLAT1
  interface: [Buffer.from]

@imports: []

@pins: []

Surface drawn from 5 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 4. Total witnessing constraint clauses: 307.

## PLAT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Buffer.from** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 213 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/fetch/fetch-http3-client.test.ts:220` — fetch protocol: http3 > large response body (multi-packet) → `expect(Buffer.from(buf).equals(big)).toBe(true)`
- `test/js/web/fetch/client-fetch.test.ts:50` — request arrayBuffer → `expect(Buffer.from(JSON.stringify(obj))).toEqual(Buffer.from(await body.arrayBuffer()))`
- `test/js/valkey/unit/buffer-operations.test.ts:34` — getBuffer returns binary data as Uint8Array → `expect(stringBuffer.length).toBe(binaryData.length)`

## PLAT2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**view.evaluate** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 49)

Witnessed by 49 constraint clauses across 3 test files. Antichain representatives:

- `test/js/bun/webview/webview.test.ts:147` — navigate + evaluate round-trip → `expect(result).toBe("hi")`
- `test/js/bun/webview/webview-chrome.test.ts:141` — chrome: navigate + evaluate round-trip → `expect(result).toBe("chrome")`
- `test/js/bun/webview/webview-chrome-ws.test.ts:106` — connect via full ws:// URL → `expect(await view.evaluate("document.getElementById('t').textContent")).toBe("ws-direct")`

## PLAT3
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**proc.stderr.text** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 23)

Witnessed by 23 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/fetch/fetch.tls.test.ts:422` — fetch-tls > fetch should ignore invalid NODE_EXTRA_CA_CERTS → `expect(await proc.stderr.text()).toContain("DEPTH_ZERO_SELF_SIGNED_CERT")`
- `test/js/node/tls/node-tls-cert.test.ts:536` — tls.connect should ignore invalid NODE_EXTRA_CA_CERTS → `expect(stderr).toContain("UNABLE_TO_GET_ISSUER_CERT_LOCALLY")`
- `test/js/node/process/process.test.js:814` — aborts when the uncaughtException handler throws → `expect(await proc.stderr.text()).toContain("bar")`

## PLAT4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**files.get** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 16)

Witnessed by 16 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/s3/s3.test.ts:1618` — writes archive to S3 via S3Client.write() → `expect(await files.get("hello.txt")!.text()).toBe("Hello from Archive!")`
- `test/js/bun/archive.test.ts:1200` — Bun.Archive > archive.files() > handles nested directory structure → `expect(files.get("root.txt")!.name).toBe("root.txt")`
- `test/js/bun/s3/s3.test.ts:1619` — writes archive to S3 via S3Client.write() → `expect(await files.get("data.json")!.text()).toBe(JSON.stringify({ test: true }))`

## PLAT5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**file.text** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/fetch/blob-write.test.ts:15` — Bun.file(path).write() does not throw → `expect(await file.text()).toBe("Hello, world!!")`
- `test/js/bun/shell/bunshell.test.ts:401` — bunshell > redirect Bun.File → `expect(await file.text()).toEqual(thisFileText)`
- `test/js/bun/s3/s3.test.ts:84` — basic operations → `expect(text).toBe("Hello Bun!")`

