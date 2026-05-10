# @regression — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: regression-surface-property
  threshold: REGR1
  interface: [fetch, fs.Dirent, crypto.verify, dns.promises.resolve, tls.connect, crypto.sign, tls.TLSSocket, events.find, net.connect, net.connect, setInterval, stream.getReader, tls.connect, tls.connect]

@imports: []

@pins: []

Surface drawn from 35 candidate properties across the Bun test corpus. Construction-style: 14; behavioral (high-cardinality): 21. Total witnessing constraint clauses: 1828.

## REGR1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fetch** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 806 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/test-21049.test.ts:29` — fetch with Request object respects redirect: 'manual' option → `expect(directResponse.status).toBe(302)`
- `test/regression/issue/server-stop-with-pending-requests.test.ts:48` — server still works normally after jsref changes → `expect(response.status).toBe(200)`
- `test/regression/issue/29371.test.ts:59` — proxy request-line omits default :80 for http:// without explicit port → `expect(res.status).toBe(200)`

## REGR2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.Dirent** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 10 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24129.test.ts:17` — Dirent with unknown type should return false for all type checks → `expect(dirent.isFile()).toBe(false)`
- `test/regression/issue/24129.test.ts:18` — Dirent with unknown type should return false for all type checks → `expect(dirent.isDirectory()).toBe(false)`
- `test/regression/issue/24129.test.ts:19` — Dirent with unknown type should return false for all type checks → `expect(dirent.isSymbolicLink()).toBe(false)`

## REGR3
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

## REGR4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**dns.promises.resolve** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/22712.test.ts:39` — dns.promises.resolve returns array of strings → `expect(result.every((addr: any) => typeof addr === "string")).toBe(true)`
- `test/regression/issue/22712.test.ts:45` — dns.promises.resolve with A record returns array of strings → `expect(result.every((addr: any) => typeof addr === "string")).toBe(true)`
- `test/regression/issue/22712.test.ts:51` — dns.promises.resolve with AAAA record returns array of strings → `expect(result.every((addr: any) => typeof addr === "string")).toBe(true)`

## REGR5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**tls.connect** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/25190.test.ts:42` — TLSSocket.isSessionReused > returns false for fresh connection without session reuse → `expect(socket.isSessionReused()).toBe(false)`
- `test/regression/issue/25190.test.ts:77` — TLSSocket.isSessionReused > returns true when session is successfully reused → `expect(socket1.isSessionReused()).toBe(false)`
- `test/regression/issue/24575.test.ts:144` — socket._handle.fd should be accessible on TLS sockets → `expect(typeof client._handle.fd).toBe("number")`

## REGR6
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto.sign** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/11029-crypto-verify-null-algorithm.test.ts:24` — crypto.verify with null algorithm should work for RSA keys → `expect(signature).toBeInstanceOf(Buffer)`
- `test/regression/issue/11029-crypto-verify-null-algorithm.test.ts:74` — crypto.verify with null algorithm should work for Ed25519 keys → `expect(signature).toBeInstanceOf(Buffer)`

## REGR7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**tls.TLSSocket** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/25190.test.ts:111` — TLSSocket.isSessionReused > isSessionReused returns false when session not yet established → `expect(typeof socket.isSessionReused).toBe("function")`
- `test/regression/issue/25190.test.ts:113` — TLSSocket.isSessionReused > isSessionReused returns false when session not yet established → `expect(socket.isSessionReused()).toBe(false)`

## REGR8
type: specification
authority: derived
scope: module
status: active
depends-on: []

**events.find** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/29787.test.ts:152` — stdin stream stays open while concurrent fetch(file://) bodies finish (#29787) → `expect(err).toBeUndefined()`

## REGR9
type: specification
authority: derived
scope: module
status: active
depends-on: []

**net.connect** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24575.test.ts:32` — socket._handle.fd should be accessible on TCP sockets → `expect(client._handle.fd).toBeTypeOf("number")`

## REGR10
type: specification
authority: derived
scope: module
status: active
depends-on: []

**net.connect** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24575.test.ts:31` — socket._handle.fd should be accessible on TCP sockets → `expect(client._handle).toBeDefined()`

## REGR11
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**setInterval** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/25639.test.ts:27` — setInterval returns Timeout object with _idleStart property → `expect(typeof timer._idleStart).toBe("number")`

## REGR12
type: specification
authority: derived
scope: module
status: active
depends-on: []

**stream.getReader** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/29225.test.ts:131` — instanceof and prototype identity still work → `expect(reader).toBeInstanceOf(ReadableStreamBYOBReader)`

## REGR13
type: specification
authority: derived
scope: module
status: active
depends-on: []

**tls.connect** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24575.test.ts:142` — socket._handle.fd should be accessible on TLS sockets → `expect(client._handle.fd).toBeTypeOf("number")`

## REGR14
type: specification
authority: derived
scope: module
status: active
depends-on: []

**tls.connect** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24575.test.ts:141` — socket._handle.fd should be accessible on TLS sockets → `expect(client._handle).toBeDefined()`

## REGR15
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**JSON.parse** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 213)

Witnessed by 213 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/test-21049.test.ts:77` — fetch with Request object respects redirect: 'manual' option → `expect(bunResult).toEqual({ status: 302, url: '${server.url}/redirect', redirected: false, location: "/target", })`
- `test/regression/issue/28014.test.ts:94` — WebSocket.protocol should not mutate after receiving frames → `expect(result.openProtocol).toBe(PROTOCOL)`
- `test/regression/issue/26657.test.ts:48` — bun update -i select all with 'A' key > should update packages when 'A' is pressed to sele… → `expect(initialPackageJson.dependencies["is-even"]).toBe("0.1.0")`

## REGR16
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**stdout.trim** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 197)

Witnessed by 197 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/hashbang-still-works.test.ts:24` — hashbang-still-works > hashbang still works after bounds check fix → `expect(stdout.trim()).toBe("hello")`
- `test/regression/issue/comma-operator-this-binding.test.ts:41` — comma operator should strip 'this' binding in function calls → `expect(lines[0]).toBe("beans")`
- `test/regression/issue/5344.test.ts:51` — code splitting with re-exports between entry points should not produce duplicate exports → `expect(stdout.trim()).toBe("function function true")`

## REGR17
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Uint8Array** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 140)

Witnessed by 140 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/8254.test.ts:38` — Bun.write() should write past 2GB boundary without corruption → `expect(buf[0]).toBe(expected)`
- `test/regression/issue/27478.test.ts:31` — multipart formdata preserves null bytes in small binary files → `expect(parsed.byteLength).toBe(source.byteLength)`
- `test/regression/issue/23723.test.js:2` — doesn't crash → `expect(typeof Uint8Array !== undefined + "").toBe(true)`

## REGR18
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Promise.all** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 82)

Witnessed by 82 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/25869.test.ts:77` — user-event style wait pattern does not hang → `expect(result).toEqual(["timeout", "advanced"])`
- `test/regression/issue/20875.test.ts:284` — gRPC streaming calls > rapid successive streaming calls → `expect(results[i][2]).toEqual({ value: 'batch${i}', value2: i })`
- `test/regression/issue/14477/14477.test.ts:22` — JSXElement with mismatched closing tags produces a syntax error → `expect(exited).toEqual(Array.from({ length: fixtures.length }, () => 1))`

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
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**response.text** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 63)

Witnessed by 63 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/server-stop-with-pending-requests.test.ts:47` — server still works normally after jsref changes → `expect(text).toBe("Hello World")`
- `test/regression/issue/26387.test.ts:39` — Request.text() should work after many requests → `expect(responseText).toBe('ok:${body.length}')`
- `test/regression/issue/20053.test.ts:35` — issue #20053 - multi-frame zstd responses should be fully decompressed → `expect(text.length).toBe(part1.length + part2.length)`

## REGR21
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Promise** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 58)

Witnessed by 58 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/26143.test.ts:50` — issue #26143 - https GET request with body hangs > http.request GET with body should compl… → `expect(result.status).toBe(200)`
- `test/regression/issue/25589-write-end.test.ts:67` — http2 write() + end() pattern should only send two DATA frames (local server) → `expect(result).toBe("OK")`
- `test/regression/issue/25589-frame-size-grpc.test.ts:193` — HTTP/2 FRAME_SIZE_ERROR with @grpc/grpc-js > receives large response headers without FRAME… → `assert.strictEqual(response.value, "test")`

## REGR22
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**stderr.toString** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 50)

Witnessed by 50 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/invalid-escape-sequences.test.ts:20` — Invalid escape sequence \\x in identifier shows helpful error message → `expect(err).toContain("const \\x41 = 1;")`
- `test/regression/issue/15276.test.ts:15` — parsing npm aliases without package manager does not crash → `expect(stderr.toString()).toContain("error: bunbunbunbunbun@npm:another-bun@1.0.0 failed to resolve")`
- `test/regression/issue/026039.test.ts:52` — frozen lockfile should use scope-specific registry for scoped packages → `expect(stderrText).toContain("npm.pkg.github.com")`

## REGR23
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

## REGR24
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

## REGR25
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

## REGR26
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

## REGR27
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**parser.validatePE** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/pe-codesigning-integrity.test.ts:212` — should create valid PE executable with .bun section → `expect(validation.dos.signature).toBe(0x5a4d)`
- `test/regression/issue/pe-codesigning-integrity.test.ts:217` — should create valid PE executable with .bun section → `expect(validation.pe.signature).toBe(0x00004550)`
- `test/regression/issue/pe-codesigning-integrity.test.ts:218` — should create valid PE executable with .bun section → `expect(validation.pe.machine).toBe(isArm64 ? 0xaa64 : 0x8664)`

## REGR28
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

## REGR29
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**parser.validatePE** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/pe-codesigning-integrity.test.ts:213` — should create valid PE executable with .bun section → `expect(validation.dos.e_lfanew).toBeGreaterThan(0)`
- `test/regression/issue/pe-codesigning-integrity.test.ts:214` — should create valid PE executable with .bun section → `expect(validation.dos.e_lfanew).toBeLessThan(0x1000)`
- `test/regression/issue/pe-codesigning-integrity.test.ts:219` — should create valid PE executable with .bun section → `expect(validation.pe.numberOfSections).toBeGreaterThan(0)`

## REGR30
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

## REGR31
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

## REGR32
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

## REGR33
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

## REGR34
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

## REGR35
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

