# async/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: async-js-surface-property
  threshold: ASYN1
  interface: [Bun.SHA1.hash, crypto.subtle.verify, stream.text, fs.access, util.promisify]

@imports: []

@pins: []

Surface drawn from 22 candidate properties across the Bun test corpus. Construction-style: 5; behavioral (high-cardinality): 17. Total witnessing constraint clauses: 167.

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

## ASYN3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**stream.text** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/fetch/utf8-bom.test.ts:131` — UTF-8 BOM should be ignored > readable stream > in ReadableStream.prototype.text() → `expect(await stream.text()).toBe("Hello, World!")`
- `test/js/web/fetch/body.test.ts:528` — body → `expect(await stream.text()).toBe("bun")`
- `test/js/web/fetch/body-clone.test.ts:399` — ReadableStream with mixed content (starting with string) can be converted to text → `expect(typeof text).toBe("string")`

## ASYN4
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fs.access** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/bun-object/write.spec.ts:131` — Bun.write() on file paths > Given a path to a file in a non-existent directory > When no o… → `expect(await fs.access(rootdir, constants.F_OK)).toBeFalsy()`
- `test/js/bun/bun-object/write.spec.ts:132` — Bun.write() on file paths > Given a path to a file in a non-existent directory > When no o… → `expect(await fs.access(path.dirname(filepath), constants.F_OK)).toBeFalsy()`

## ASYN5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**util.promisify** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:545` — uses `dns.promises` implementations for `util.promisify` factory > util.promisify(dns.look… → `expect(await util.promisify(dns.lookup)("google.com")).toEqual(await dns.promises.lookup("google.com"))`
- `test/js/node/crypto/node-crypto.test.js:32` — crypto.randomInt with a callback → `expect(typeof result).toBe("number")`

## ASYN6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**blob.text** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 24)

Witnessed by 24 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/structured-clone-blob-file.test.ts:404` — structuredClone with Blob and File > deserialize of crafted payloads > in-process: offset … → `expect(await blob.text()).toBe("")`
- `test/js/web/streams/streams.test.js:670` — ReadableStream for Blob → `expect(await blob.text()).toBe("abdefghijklmnop")`
- `test/js/web/fetch/utf8-bom.test.ts:22` — UTF-8 BOM should be ignored > Blob > with emoji > in text() → `expect(await blob.text()).toBe("Hello, World! 🌎")`

## ASYN7
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**proc.stdout.text** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 18)

Witnessed by 18 constraint clauses across 5 test files. Antichain representatives:

- `test/js/node/process/process-stdin.test.ts:69` — stdin with 'readable' event handler should receive data when paused → `expect(await proc.stdout.text()).toMatchInlineSnapshot(' "got chunk {"type":"Buffer","data":[97,98,99,10,100,101,102,10]} " ')`
- `test/js/node/child_process/child_process.test.ts:392` — should call close and exit before process exits → `expect(data).toContain("closeHandler called")`
- `test/js/bun/util/sleep.test.ts:74` — sleep should keep the event loop alive → `expect(await proc.stdout.text()).toContain("event loop was not killed")`

## ASYN8
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

## ASYN9
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response.json** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:1158` — Response > Response.json > works → `expect(await Response.json(input).text()).toBe(output)`
- `response.spec.md:21` — Response.json static method → `Response.json(data) returns a Response containing the JSON serialization of data`
- `test/js/web/fetch/fetch.test.ts:1161` — Response > Response.json > works → `expect(await Response.json().text()).toBe("")`

## ASYN10
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

## ASYN11
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**s3file.text** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/s3/s3.test.ts:597` — ${credentials.service} > Bun.file > should download file via Bun.file().text() → `expect(text).toBe("Hello Bun!")`
- `test/js/bun/s3/s3.test.ts:659` — ${credentials.service} > Bun.file > should be able to upload large files using writer() #1… → `expect(await s3file.text()).toBe(mediumPayload.repeat(2))`
- `test/js/bun/s3/s3.test.ts:680` — ${credentials.service} > Bun.file > should be able to upload large files using flush and p… → `expect(await s3file.text()).toBe(mediumPayload.repeat(2))`

## ASYN12
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

## ASYN13
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**body.get** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/html/FormData.test.ts:368` — FormData > file send on HTTP server (receive) → `expect(await (body.get("foo") as Blob).text()).toBe("baz")`
- `test/js/web/html/FormData.test.ts:418` — FormData > send on HTTP server with FormData & Blob (roundtrip) → `expect(await (body.get("foo") as Blob).text()).toBe("baz")`
- `test/js/web/html/FormData.test.ts:419` — FormData > send on HTTP server with FormData & Blob (roundtrip) → `expect(body.get("bar")).toBe("baz")`

## ASYN14
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**fetch** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/http/bun-serve-html.test.ts:261` — serve html → `expect(await (await fetch('http://${hostname}:${port}/a-different-url')).text()).toMatchInlineSnapshot( '"Hello World"', )`
- `test/js/bun/http/bun-serve-html-entry.test.ts:474` — bun *.html → `expect(homeJs).toContain('document.getElementById("counter")')`
- `test/js/bun/http/bun-serve-html-entry.test.ts:475` — bun *.html → `expect(homeJs).toContain("Click me:")`

## ASYN15
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**response.json** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/fetch/utf8-bom.test.ts:64` — UTF-8 BOM should be ignored > Response > in json() → `expect(await response.json()).toEqual({ "hello": "World" } as any)`
- `test/js/web/fetch/fetch.test.ts:1315` — Response > should consume body correctly > with json first → `expect(await promise).toEqual({ "hello": "world" })`
- `test/js/bun/util/zstd.test.ts:369` — Zstandard HTTP compression > can fetch and automatically decompress zstd-encoded JSON → `expect(json).toEqual(testData.json)`

## ASYN16
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

## ASYN17
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

## ASYN18
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

## ASYN19
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

## ASYN20
type: specification
authority: derived
scope: module
status: active
depends-on: []

**redis.hget** — is defined and resolves to a non-nullish value at the documented call site. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/valkey/valkey.test.ts:5441` — Hash Operations > should get and delete hash field using hgetdel → `expect(check).toBeNull()`
- `test/js/valkey/valkey.test.ts:5456` — Hash Operations > should get and delete multiple hash fields using hgetdel → `expect(await redis.hget(key, "name")).toBeNull()`
- `test/js/valkey/valkey.test.ts:5457` — Hash Operations > should get and delete multiple hash fields using hgetdel → `expect(await redis.hget(key, "city")).toBeNull()`

## ASYN21
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**result.text** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/fetch/body-stream.test.ts:138` — Request.prototoype.${RequestPrototypeMixin.name}() ${
            useRequestObject
       … → `expect(await result.text()).toBe(name)`
- `test/js/node/http/node-fetch.test.js:154` — node-fetch request body streams properly → `expect(await result.text()).toBe("response sent")`
- `test/js/bun/shell/exec.test.ts:95` — bun exec > works with latin1 paths → `expect(result.text()).toBe("hi\n")`

## ASYN22
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

