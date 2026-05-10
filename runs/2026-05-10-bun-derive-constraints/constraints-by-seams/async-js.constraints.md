# async/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: async-js-surface-property
  threshold: ASYN1
  interface: [Bun.SHA1.hash, fetch, crypto.subtle.verify, ReadableStream, fs.access]

@imports: []

@pins: []

Surface drawn from 25 candidate properties across the Bun test corpus. Construction-style: 5; behavioral (high-cardinality): 20. Total witnessing constraint clauses: 436.

## ASYN1
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

## ASYN2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fetch** — exposes values of the expected type or class. (construction-style)

Witnessed by 5 constraint clauses across 4 test files. Antichain representatives:

- `test/js/web/fetch/fetch.upgrade.test.ts:40` — fetch upgrade > should upgrade to websocket → `expect(res.headers.get("sec-websocket-accept")).toBeString()`
- `test/js/node/http/node-fetch-primordials.test.ts:29` — fetch, Response, Request can be overriden → `expect(response).toBeInstanceOf(Response)`
- `test/js/node/http/node-fetch-cjs.test.js:12` — require('node-fetch') fetches → `expect(await fetch("http://" + server.hostname + ":" + server.port)).toBeInstanceOf(Response)`

## ASYN3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.verify** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/crypto/web-crypto-sha3.test.ts:53` — HMAC with SHA-3 > generateKey + sign + verify with SHA3-256 → `expect(await crypto.subtle.verify("HMAC", key, sig, data)).toBe(true)`
- `test/js/web/crypto/web-crypto-sha3.test.ts:57` — HMAC with SHA-3 > generateKey + sign + verify with SHA3-256 → `expect(await crypto.subtle.verify("HMAC", key, tampered, data)).toBe(false)`
- `test/js/web/crypto/web-crypto-sha3.test.ts:89` — RSA with SHA-3 hash > RSA-PSS with SHA3-256: generate, sign, verify, JWK export → `expect(await crypto.subtle.verify({ name: "RSA-PSS", saltLength: 32 }, publicKey, sig, data)).toBe(true)`

## ASYN4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ReadableStream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/streams/streams.test.js:472` — exists globally → `expect(typeof ReadableStream).toBe("function")`
- `test/js/web/fetch/utf8-bom.test.ts:131` — UTF-8 BOM should be ignored > readable stream > in ReadableStream.prototype.text() → `expect(await stream.text()).toBe("Hello, World!")`
- `test/js/web/fetch/utf8-bom.test.ts:141` — UTF-8 BOM should be ignored > readable stream > in ReadableStream.prototype.json() → `expect(await stream.json()).toEqual({ "hello": "World" })`

## ASYN5
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fs.access** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/bun-object/write.spec.ts:131` — Bun.write() on file paths > Given a path to a file in a non-existent directory > When no o… → `expect(await fs.access(rootdir, constants.F_OK)).toBeFalsy()`
- `test/js/bun/bun-object/write.spec.ts:132` — Bun.write() on file paths > Given a path to a file in a non-existent directory > When no o… → `expect(await fs.access(path.dirname(filepath), constants.F_OK)).toBeFalsy()`

## ASYN6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**SQL** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 220)

Witnessed by 220 constraint clauses across 4 test files. Antichain representatives:

- `test/js/sql/sqlite-sql.test.ts:13` — Connection & Initialization > common default connection strings > should parse common conn… → `expect(memory.options.adapter).toBe("sqlite")`
- `test/js/sql/sql-mysql.test.ts:813` — Connection ended error → `expect(await sql''.catch(x => x.code)).toBe("ERR_MYSQL_CONNECTION_CLOSED")`
- `test/js/sql/adapter-override.test.ts:11` — Adapter Override > postgres:// URL with adapter='sqlite' uses SQLite → `expect(sql.options.adapter).toBe("sqlite")`

## ASYN7
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**fetch** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 44)

Witnessed by 44 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:1973` — 304 not modified with missing content-length does not cause a request timeout → `expect(await response.arrayBuffer()).toHaveLength(0)`
- `test/js/web/fetch/fetch-http3-client.test.ts:207` — fetch protocol: http3 > JSON + query string → `expect(res.headers.get("content-type")).toContain("application/json")`
- `test/js/bun/http/serve-if-none-match.test.ts:45` — If-None-Match Support > ETag Generation > should automatically generate ETag for static re… → `expect(res.headers.get("ETag")).toMatch(/^"[a-f0-9]+"$/)`

## ASYN8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**res.text** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 17)

Witnessed by 17 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/html/FormData.test.ts:345` — FormData > file upload on HTTP server (receive) → `expect(body).toBe("baz")`
- `test/js/web/fetch/fetch.test.ts:251` — AbortSignal > AbortAfterFinish → `expect(await res.text()).toBe("Hello")`
- `test/js/web/fetch/fetch.stream.test.ts:788` — fetch() with streaming > chunked response works (single chunk) with ${compression} compres… → `expect(result).toBe(content)`

## ASYN9
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ctx.redis.publish** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 14)

Witnessed by 14 constraint clauses across 1 test files. Antichain representatives:

- `test/js/valkey/valkey.test.ts:6283` — PUB/SUB > publishing to a channel does not fail → `expect(await ctx.redis.publish(testChannel(), testMessage())).toBe(0)`
- `test/js/valkey/valkey.test.ts:6320` — PUB/SUB > subscribing to a channel receives messages → `expect(await ctx.redis.publish(channel, message)).toBe(1)`
- `test/js/valkey/valkey.test.ts:6346` — PUB/SUB > messages are received in order → `expect(await ctx.redis.publish(channel, message)).toBe(1)`

## ASYN10
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ctx.redis.exists** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 3 test files. Antichain representatives:

- `test/js/valkey/unit/basic-operations.test.ts:59` — String Commands > SET with expiry option → `expect(existsNow).toBe(true)`
- `test/js/valkey/reliability/protocol-handling.test.ts:73` — RESP3 Data Type Handling > should handle RESP3 Boolean type → `expect(typeof existsResult).toBe("boolean")`
- `test/js/valkey/reliability/error-handling.test.ts:203` — Protocol and Parser Edge Cases > should handle RESP protocol boundaries → `expect(await client.exists("key1")).toBe(true)`

## ASYN11
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ctx.redis.duplicate** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 12)

Witnessed by 12 constraint clauses across 1 test files. Antichain representatives:

- `test/js/valkey/valkey.test.ts:6780` — duplicate() > should create duplicate of connected client that gets connected → `expect(duplicate.connected).toBe(true)`
- `test/js/valkey/valkey.test.ts:6787` — duplicate() > should create duplicate of connected client that gets connected → `expect(await duplicate.get("test-original")).toBe("original-value")`
- `test/js/valkey/valkey.test.ts:6843` — duplicate() > should create multiple duplicates from same client → `expect(duplicate1.connected).toBe(true)`

## ASYN12
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response.json** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:1158` — Response > Response.json > works → `expect(await Response.json(input).text()).toBe(output)`
- `test/js/web/fetch/fetch.test.ts:1161` — Response > Response.json > works → `expect(await Response.json().text()).toBe("")`
- `test/js/web/fetch/fetch.test.ts:1163` — Response > Response.json > works → `expect(await Response.json("").text()).toBe('""')`

## ASYN13
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**util.promisify** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 3 test files. Antichain representatives:

- `test/js/node/zlib/zlib.test.js:159` — zlib.brotli > brotliCompress → `expect(compressed.toString()).toEqual(compressedBuffer.toString())`
- `test/js/node/dns/node-dns.test.js:545` — uses `dns.promises` implementations for `util.promisify` factory > util.promisify(dns.look… → `expect(await util.promisify(dns.lookup)("google.com")).toEqual(await dns.promises.lookup("google.com"))`
- `test/js/node/crypto/node-crypto.test.js:32` — crypto.randomInt with a callback → `expect(typeof result).toBe("number")`

## ASYN14
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Promise.race** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/timers/setImmediate.test.js:80` — setImmediate should not keep the process alive forever → `expect(await Promise.race([success(), fail()])).toBe(true)`
- `test/js/web/streams/streams.test.js:770` — ReadableStream errors the stream on pull rejection → `expect(await Promise.race([closed, read])).toBe("closed: pull rejected")`
- `test/js/sql/sql.test.ts:665` — PostgreSQL tests > Minimal reproduction of Bun.SQL PostgreSQL hang bug (#22395) → `expect(result[0].count).toBe("1")`

## ASYN15
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ctx.redis.getbit** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/valkey/valkey.test.ts:426` — Basic Operations > should handle multiple bit operations → `expect(await redis.getbit(bitKey, 0)).toBe(1)`
- `test/js/valkey/valkey.test.ts:427` — Basic Operations > should handle multiple bit operations → `expect(await redis.getbit(bitKey, 1)).toBe(0)`
- `test/js/valkey/valkey.test.ts:428` — Basic Operations > should handle multiple bit operations → `expect(await redis.getbit(bitKey, 2)).toBe(0)`

## ASYN16
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.file** — exhibits the property captured in the witnessing test. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/bun-file-exists.test.js:7` — bun-file-exists → `expect(await Bun.file(import.meta.path).exists()).toBeTrue()`
- `test/js/bun/util/bun-file-exists.test.js:8` — bun-file-exists → `expect(await Bun.file(import.meta.path + "boop").exists()).toBeFalse()`
- `test/js/bun/util/bun-file-exists.test.js:9` — bun-file-exists → `expect(await Bun.file(import.meta.dir).exists()).toBeFalse()`

## ASYN17
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**sql.file** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 3 test files. Antichain representatives:

- `test/js/sql/sqlite-sql.test.ts:1498` — SQL helpers > file execution → `expect(result[0].count).toBe(3)`
- `test/js/sql/sql.test.ts:1681` — PostgreSQL tests > sql file throws → `expect(await sql.file(rel("selectomondo.sql")).catch(x => x.code)).toBe("ENOENT")`
- `test/js/sql/sql-mysql.test.ts:787` — sql file throws → `expect(await sql.file(rel("selectomondo.sql")).catch(x => x.code)).toBe("ENOENT")`

## ASYN18
type: specification
authority: derived
scope: module
status: active
depends-on: []

**ctx.redis.hget** — is defined and resolves to a non-nullish value at the documented call site. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/js/valkey/valkey.test.ts:5456` — Hash Operations > should get and delete multiple hash fields using hgetdel → `expect(await redis.hget(key, "name")).toBeNull()`
- `test/js/valkey/unit/hash-operations.test.ts:52` — Basic Hash Commands > HGET native method → `expect(nonExistent).toBeNull()`
- `test/js/valkey/valkey.test.ts:5457` — Hash Operations > should get and delete multiple hash fields using hgetdel → `expect(await redis.hget(key, "city")).toBeNull()`

## ASYN19
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ctx.redis.lrange** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/valkey/valkey.test.ts:1868` — List Operations > should handle all LMOVE direction combinations → `expect(await redis.lrange("dst1", 0, -1)).toEqual(["a"])`
- `test/js/valkey/valkey.test.ts:1873` — List Operations > should handle all LMOVE direction combinations → `expect(await redis.lrange("dst2", 0, -1)).toEqual(["a"])`
- `test/js/valkey/valkey.test.ts:1878` — List Operations > should handle all LMOVE direction combinations → `expect(await redis.lrange("dst3", 0, -1)).toEqual(["b"])`

## ASYN20
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**ctx.redis.publish** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/valkey/valkey.test.ts:6657` — PUB/SUB > callback errors don't crash the client (without IPC) → `expect(await ctx.redis.publish(channel, "message1")).toBeGreaterThanOrEqual(1)`
- `test/js/valkey/valkey.test.ts:6661` — PUB/SUB > callback errors don't crash the client (without IPC) → `expect(await ctx.redis.publish(channel, "message2")).toBeGreaterThanOrEqual(1)`
- `test/js/valkey/valkey.test.ts:6664` — PUB/SUB > callback errors don't crash the client (without IPC) → `expect(await ctx.redis.publish(channel, "message1")).toBeGreaterThanOrEqual(1)`

## ASYN21
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**files.get** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/s3/s3.test.ts:1668` — writes archive with binary content to S3 → `expect(extractedBinary).toEqual(binaryData)`
- `test/js/bun/archive.test.ts:1174` — Bun.Archive > archive.files() > returns a Map of File objects → `expect(helloFile!.name).toBe("hello.txt")`
- `test/js/bun/archive.test.ts:1175` — Bun.Archive > archive.files() > returns a Map of File objects → `expect(await helloFile!.text()).toBe("Hello, World!")`

## ASYN22
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.promises.readFile** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:701` — promises.readFile → `expect(await fs.promises.readFile(import.meta.path, "utf-8")).toEqual(readFileSync(import.meta.path, "utf-8"))`
- `test/js/node/fs/fs.test.ts:702` — promises.readFile → `expect(await fs.promises.readFile(import.meta.path, { encoding: "latin1" })).toEqual( readFileSync(import.meta.path, { encoding: "latin1" }), )`
- `test/js/node/fs/fs.test.ts:967` — promises.readFile - UTF16 file path → `expect(await fs.promises.readFile(dest, "utf-8")).toEqual(expected)`

## ASYN23
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.readFile** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/patch/patch.test.ts:102` — apply > edgecase → `expect(await fs.readFile('${tempdir}/node_modules/is-even/index.js').then(b => b.toString())).toBe(newcontents)`
- `test/js/bun/bun-object/write.spec.ts:75` — Bun.write() on file paths > Given a path to a file in an existing directory > When the fil… → `expect(await fs.readFile(filepath, "utf-8")).toBe(content)`
- `test/js/bun/bun-object/write.spec.ts:81` — Bun.write() on file paths > Given a path to a file in an existing directory > When the fil… → `expect(await fs.readFile(filepath, "utf-8")).toBe("")`

## ASYN24
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**server.fetch** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/http/bun-server.test.ts:307` — Server > server.fetch should work with a string → `expect(await response.text()).toBe("Hello World!")`
- `test/js/bun/http/bun-server.test.ts:308` — Server > server.fetch should work with a string → `expect(response.status).toBe(200)`
- `test/js/bun/http/bun-server.test.ts:309` — Server > server.fetch should work with a string → `expect(response.url).toBe(url)`

## ASYN25
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**subprocess.exited** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/websocket/websocket.test.js:671` — websocket in subprocess > should exit → `expect(await subprocess.exited).toBe(0)`
- `test/js/web/websocket/websocket.test.js:713` — websocket in subprocess > should exit after killed → `expect(await subprocess.exited).toBe(143)`
- `test/js/web/websocket/websocket.test.js:727` — websocket in subprocess > should exit with invalid url → `expect(await subprocess.exited).toBe(1)`

