# @regression — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: regression-surface-property
  threshold: REGR1
  interface: [fetch, crypto.verify, crypto.sign, setTimeout, net.connect, net.connect, setInterval, stream.getReader, tls.TLSSocket, tls.connect, tls.connect, tls.connect]

@imports: []

@pins: []

Surface drawn from 36 candidate properties across the Bun test corpus. Construction-style: 12; behavioral (high-cardinality): 24. Total witnessing constraint clauses: 1546.

## REGR1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fetch** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 455 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/test-21049.test.ts:29` — fetch with Request object respects redirect: 'manual' option → `expect(directResponse.status).toBe(302)`
- `test/regression/issue/server-stop-with-pending-requests.test.ts:48` — server still works normally after jsref changes → `expect(response.status).toBe(200)`
- `test/regression/issue/29371.test.ts:59` — proxy request-line omits default :80 for http:// without explicit port → `expect(res.status).toBe(200)`

## REGR2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.verify** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/11029-crypto-verify-null-algorithm.test.ts:28` — crypto.verify with null algorithm should work for RSA keys → `expect(isVerified).toBe(true)`
- `test/regression/issue/11029-crypto-verify-null-algorithm.test.ts:33` — crypto.verify with null algorithm should work for RSA keys → `expect(isVerifiedWrong).toBe(false)`
- `test/regression/issue/11029-crypto-verify-null-algorithm.test.ts:54` — crypto.verify with undefined algorithm should work for RSA keys → `expect(isVerified).toBe(true)`

## REGR3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto.sign** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/11029-crypto-verify-null-algorithm.test.ts:24` — crypto.verify with null algorithm should work for RSA keys → `expect(signature).toBeInstanceOf(Buffer)`
- `test/regression/issue/11029-crypto-verify-null-algorithm.test.ts:74` — crypto.verify with null algorithm should work for Ed25519 keys → `expect(signature).toBeInstanceOf(Buffer)`

## REGR4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**setTimeout** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/25639.test.ts:12` — setTimeout returns Timeout object with _idleStart property → `expect(typeof timer._idleStart).toBe("number")`
- `test/regression/issue/25639.test.ts:46` — _idleStart is writable (Next.js modifies it to coordinate timers) → `expect(timer._idleStart).toBe(newIdleStart)`

## REGR5
type: specification
authority: derived
scope: module
status: active
depends-on: []

**net.connect** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24575.test.ts:32` — socket._handle.fd should be accessible on TCP sockets → `expect(client._handle.fd).toBeTypeOf("number")`

## REGR6
type: specification
authority: derived
scope: module
status: active
depends-on: []

**net.connect** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24575.test.ts:31` — socket._handle.fd should be accessible on TCP sockets → `expect(client._handle).toBeDefined()`

## REGR7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**setInterval** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/25639.test.ts:27` — setInterval returns Timeout object with _idleStart property → `expect(typeof timer._idleStart).toBe("number")`

## REGR8
type: specification
authority: derived
scope: module
status: active
depends-on: []

**stream.getReader** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/29225.test.ts:131` — instanceof and prototype identity still work → `expect(reader).toBeInstanceOf(ReadableStreamBYOBReader)`

## REGR9
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**tls.TLSSocket** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/25190.test.ts:111` — TLSSocket.isSessionReused > isSessionReused returns false when session not yet established → `expect(typeof socket.isSessionReused).toBe("function")`

## REGR10
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**tls.connect** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24575.test.ts:144` — socket._handle.fd should be accessible on TLS sockets → `expect(typeof client._handle.fd).toBe("number")`

## REGR11
type: specification
authority: derived
scope: module
status: active
depends-on: []

**tls.connect** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24575.test.ts:142` — socket._handle.fd should be accessible on TLS sockets → `expect(client._handle.fd).toBeTypeOf("number")`

## REGR12
type: specification
authority: derived
scope: module
status: active
depends-on: []

**tls.connect** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24575.test.ts:141` — socket._handle.fd should be accessible on TLS sockets → `expect(client._handle).toBeDefined()`

## REGR13
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**JSON.parse** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 210)

Witnessed by 210 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/test-21049.test.ts:77` — fetch with Request object respects redirect: 'manual' option → `expect(bunResult).toEqual({ status: 302, url: '${server.url}/redirect', redirected: false, location: "/target", })`
- `test/regression/issue/28014.test.ts:94` — WebSocket.protocol should not mutate after receiving frames → `expect(result.openProtocol).toBe(PROTOCOL)`
- `test/regression/issue/26657.test.ts:48` — bun update -i select all with 'A' key > should update packages when 'A' is pressed to sele… → `expect(initialPackageJson.dependencies["is-even"]).toBe("0.1.0")`

## REGR14
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**stdout.trim** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 203)

Witnessed by 203 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/hashbang-still-works.test.ts:24` — hashbang-still-works > hashbang still works after bounds check fix → `expect(stdout.trim()).toBe("hello")`
- `test/regression/issue/comma-operator-this-binding.test.ts:41` — comma operator should strip 'this' binding in function calls → `expect(lines[0]).toBe("beans")`
- `test/regression/issue/5344.test.ts:51` — code splitting with re-exports between entry points should not produce duplicate exports → `expect(stdout.trim()).toBe("function function true")`

## REGR15
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**response.text** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 127)

Witnessed by 127 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/server-stop-with-pending-requests.test.ts:47` — server still works normally after jsref changes → `expect(text).toBe("Hello World")`
- `test/regression/issue/26387.test.ts:39` — Request.text() should work after many requests → `expect(responseText).toBe('ok:${body.length}')`
- `test/regression/issue/20053.test.ts:35` — issue #20053 - multi-frame zstd responses should be fully decompressed → `expect(text.length).toBe(part1.length + part2.length)`

## REGR16
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Uint8Array** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 125)

Witnessed by 125 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/8254.test.ts:38` — Bun.write() should write past 2GB boundary without corruption → `expect(buf[0]).toBe(expected)`
- `test/regression/issue/27478.test.ts:31` — multipart formdata preserves null bytes in small binary files → `expect(parsed.byteLength).toBe(source.byteLength)`
- `test/regression/issue/23723.test.js:2` — doesn't crash → `expect(typeof Uint8Array !== undefined + "").toBe(true)`

## REGR17
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Promise.all** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 75)

Witnessed by 75 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/25869.test.ts:77` — user-event style wait pattern does not hang → `expect(result).toEqual(["timeout", "advanced"])`
- `test/regression/issue/20875.test.ts:284` — gRPC streaming calls > rapid successive streaming calls → `expect(results[i][2]).toEqual({ value: 'batch${i}', value2: i })`
- `test/regression/issue/14477/14477.test.ts:22` — JSXElement with mismatched closing tags produces a syntax error → `expect(exited).toEqual(Array.from({ length: fixtures.length }, () => 1))`

## REGR18
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**response.headers.get** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 75)

Witnessed by 75 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/test-21049.test.ts:111` — fetch with Request object respects redirect: 'manual' for external URLs → `expect(response.headers.get("location")).toBe("/target")`
- `test/regression/issue/18547.test.ts:26` — 18547 → `expect(response.headers.get("set-cookie")).toEqual("sessionToken=123456; Path=/; SameSite=Lax")`
- `test/regression/issue/07397.test.ts:9` — Response.redirect clones string from Location header → `expect(response.headers.get("Location")).toBe(href)`

## REGR19
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Array.from** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 68)

Witnessed by 68 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/s3-signature-performance.test.ts:47` — S3 presigned URL performance test with stack allocator → `expect(params).toEqual(sortedParams)`
- `test/regression/issue/s3-signature-order.test.ts:26` — S3 presigned URL should have correct query parameter order → `expect(params).toEqual(expected)`
- `test/regression/issue/27478.test.ts:30` — multipart formdata preserves null bytes in small binary files → `expect(Array.from(parsed)).toEqual(Array.from(source))`

## REGR20
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**stderr.toString** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 49)

Witnessed by 49 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/invalid-escape-sequences.test.ts:20` — Invalid escape sequence \\x in identifier shows helpful error message → `expect(err).toContain("const \\x41 = 1;")`
- `test/regression/issue/15276.test.ts:15` — parsing npm aliases without package manager does not crash → `expect(stderr.toString()).toContain("error: bunbunbunbunbun@npm:another-bun@1.0.0 failed to resolve")`
- `test/regression/issue/026039.test.ts:52` — frozen lockfile should use scope-specific registry for scoped packages → `expect(stderrText).toContain("npm.pkg.github.com")`

## REGR21
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**result.exited** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 27)

Witnessed by 27 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/5961.test.ts:19` — 5961 → `expect(exitCode).toBe(0)`
- `test/regression/issue/5738.test.ts:16` — 5738 → `expect(exitCode).toBe(0)`
- `test/regression/issue/23077/23077.test.ts:13` — 23077 → `expect(exitCode).toBe(0)`

## REGR22
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**X509Certificate** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/27025.test.ts:30` — X509Certificate properties should not crash on valid certificates → `expect(cert.ca).toBe(false)`
- `test/regression/issue/21274.test.ts:10` — #21274 → `expect(cert.subjectAltName).toEqual( "DNS:*.lifecycle-prober-prod-89308e4e-9927-4280-9e14-3330f6900396.asia-northeast1.managedkafka.gmk-lifecycle-prober-prod-1.cloud.goog", )`
- `test/regression/issue/21274.test.ts:13` — #21274 → `expect(cert.issuer).toEqual("C=US\nO=Google Trust Services\nCN=WR1")`

## REGR23
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**file.presign** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 12)

Witnessed by 12 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/25750.test.ts:23` — issue #25750 - S3 presign contentDisposition and type > should include response-content-di… → `expect(url).toContain("response-content-disposition=")`
- `test/regression/issue/25750.test.ts:24` — issue #25750 - S3 presign contentDisposition and type > should include response-content-di… → `expect(url).toContain("attachment")`
- `test/regression/issue/25750.test.ts:25` — issue #25750 - S3 presign contentDisposition and type > should include response-content-di… → `expect(url).toContain("quarterly-report.txt")`

## REGR24
type: specification
authority: derived
scope: module
status: active
depends-on: []

**X509Certificate** — is defined and resolves to a non-nullish value at the documented call site. (behavioral; cardinality 10)

Witnessed by 10 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/27025.test.ts:14` — issuerCertificate should return undefined for directly-parsed certificates without crashin… → `expect(cert.issuerCertificate).toBeUndefined()`
- `test/regression/issue/21274.test.ts:9` — #21274 → `expect(cert.subject).toBeUndefined()`
- `test/regression/issue/27025.test.ts:21` — X509Certificate properties should not crash on valid certificates → `expect(cert.subject).toBeDefined()`

## REGR25
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**file.text** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/26851.test.ts:33` — --bail writes JUnit reporter outfile → `expect(xml).toContain("<?xml")`
- `test/regression/issue/26851.test.ts:34` — --bail writes JUnit reporter outfile → `expect(xml).toContain("<testsuites")`
- `test/regression/issue/26851.test.ts:35` — --bail writes JUnit reporter outfile → `expect(xml).toContain("</testsuites>")`

## REGR26
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**err.code** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/18413-truncation.test.ts:108` — truncated gzip stream should throw error → `expect(err.code || err.name || err.message).toMatch(/ZlibError|ShortRead/)`
- `test/regression/issue/18413-deflate-semantics.test.ts:199` — truncated zlib-wrapped deflate should fail → `expect(err.code).toMatch(/ZlibError|ShortRead/)`
- `test/regression/issue/18413-truncation.test.ts:121` — truncated brotli stream should throw error → `expect(err.code || err.name || err.message).toMatch(/BrotliDecompressionError/)`

## REGR27
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**scriptNodes.filter** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/29240.test.ts:77` — cpu-prof callFrame.lineNumber/columnNumber point at function definition, not sample positi… → `expect(fibNodes.length).toBeGreaterThan(0)`
- `test/regression/issue/29240.test.ts:97` — cpu-prof callFrame.lineNumber/columnNumber point at function definition, not sample positi… → `expect(fibNodes.length).toBeLessThan(40)`
- `test/regression/issue/29240.test.ts:102` — cpu-prof callFrame.lineNumber/columnNumber point at function definition, not sample positi… → `expect(doWorkNodes.length).toBeGreaterThan(0)`

## REGR28
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**decompressed.toString** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 3 test files. Antichain representatives:

- `test/regression/issue/23314/zstd-large-input.test.ts:14` — zstd compression with larger inputs > should handle larger strings → `expect(decompressed.toString()).toBe(input)`
- `test/regression/issue/23314/zstd-large-decompression.test.ts:19` — should handle large data decompression safely → `expect(decompressed.toString()).toBe(input)`
- `test/regression/issue/23314/zstd-async-compress.test.ts:11` — zstd compression compatibility > should decompress data compressed with zlib.zstdCompressS… → `expect(decompressed.toString()).toBe(input)`

## REGR29
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**parser.validatePE** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/pe-codesigning-integrity.test.ts:212` — should create valid PE executable with .bun section → `expect(validation.dos.signature).toBe(0x5a4d)`
- `test/regression/issue/pe-codesigning-integrity.test.ts:217` — should create valid PE executable with .bun section → `expect(validation.pe.signature).toBe(0x00004550)`
- `test/regression/issue/pe-codesigning-integrity.test.ts:218` — should create valid PE executable with .bun section → `expect(validation.pe.machine).toBe(isArm64 ? 0xaa64 : 0x8664)`

## REGR30
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**transpiler.scanImports** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/13251.test.ts:10` — scanImports respects trimUnusedImports → `expect(transpiler.scanImports('import { Component } from "./Component";')).toEqual([])`
- `test/regression/issue/13251.test.ts:13` — scanImports respects trimUnusedImports → `expect(transpiler.scanImports('import Foo from "./Foo";')).toEqual([])`
- `test/regression/issue/13251.test.ts:16` — scanImports respects trimUnusedImports → `expect(transpiler.scanImports('import * as Utils from "./Utils";')).toEqual([])`

## REGR31
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**parser.validatePE** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/pe-codesigning-integrity.test.ts:213` — should create valid PE executable with .bun section → `expect(validation.dos.e_lfanew).toBeGreaterThan(0)`
- `test/regression/issue/pe-codesigning-integrity.test.ts:214` — should create valid PE executable with .bun section → `expect(validation.dos.e_lfanew).toBeLessThan(0x1000)`
- `test/regression/issue/pe-codesigning-integrity.test.ts:219` — should create valid PE executable with .bun section → `expect(validation.pe.numberOfSections).toBeGreaterThan(0)`

## REGR32
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**promise.bodyLength** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/27061.test.ts:59` — node:http ClientRequest preserves explicit Content-Length > with multiple req.write() call… → `expect(result.bodyLength).toBe(200)`
- `test/regression/issue/27061.test.ts:113` — node:http ClientRequest preserves explicit Content-Length > with req.write() + req.end(dat… → `expect(result.bodyLength).toBe(200)`
- `test/regression/issue/27061.test.ts:170` — node:http ClientRequest preserves explicit Content-Length > with three req.write() calls → `expect(result.bodyLength).toBe(300)`

## REGR33
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**s3.presign** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/s3-signature-performance.test.ts:37` — S3 presigned URL performance test with stack allocator → `expect(url).toContain("test-file-")`
- `test/regression/issue/s3-signature-performance.test.ts:38` — S3 presigned URL performance test with stack allocator → `expect(url).toContain("X-Amz-Algorithm=AWS4-HMAC-SHA256")`
- `test/regression/issue/s3-signature-performance.test.ts:39` — S3 presigned URL performance test with stack allocator → `expect(url).toContain("X-Amz-Credential=")`

## REGR34
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**db.unsafe** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/28004.test.ts:220` — MySQL-compatible server without CLIENT_DEPRECATE_EOF returns rows correctly → `expect(rows.length).toBe(2)`
- `test/regression/issue/28004.test.ts:221` — MySQL-compatible server without CLIENT_DEPRECATE_EOF returns rows correctly → `expect(rows[0].id).toBe("1")`
- `test/regression/issue/28004.test.ts:222` — MySQL-compatible server without CLIENT_DEPRECATE_EOF returns rows correctly → `expect(rows[0].name).toBe("hello")`

## REGR35
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**n.callFrame.lineNumber** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/29240.test.ts:85` — cpu-prof callFrame.lineNumber/columnNumber point at function definition, not sample positi… → `expect(n.callFrame.lineNumber).toBe(0)`
- `test/regression/issue/29240.test.ts:104` — cpu-prof callFrame.lineNumber/columnNumber point at function definition, not sample positi… → `expect(n.callFrame.lineNumber).toBe(5)`
- `test/regression/issue/29240.test.ts:110` — cpu-prof callFrame.lineNumber/columnNumber point at function definition, not sample positi… → `expect(n.callFrame.lineNumber).toBe(13)`

## REGR36
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**server.url.toString** — exhibits the property captured in the witnessing test. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/21792.test.ts:23` — SNI TLS array handling (issue #21792) > should accept empty TLS array (no TLS) → `expect(server.url.toString()).toStartWith("http://")`
- `test/regression/issue/21792.test.ts:34` — SNI TLS array handling (issue #21792) > should accept single TLS config in array → `expect(server.url.toString()).toStartWith("https://")`
- `test/regression/issue/21792.test.ts:50` — SNI TLS array handling (issue #21792) > should accept multiple TLS configs for SNI → `expect(server.url.toString()).toStartWith("https://")`

