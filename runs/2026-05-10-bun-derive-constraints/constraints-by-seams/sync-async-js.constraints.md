# sync+async/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: sync-async-js-surface-property
  threshold: SYNC1
  interface: [Bun.file]

@imports: []

@pins: []

Surface drawn from 13 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 12. Total witnessing constraint clauses: 131.

## SYNC1
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.file** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/workers/message-channel.test.ts:283` — cloneable and non-transferable equals (BunFile) → `expect(file).toBeInstanceOf(Blob)`
- `test/js/bun/image/image.test.ts:165` — Bun.Image > Bun.file() input chains the async file read into the pipeline → `expect(via).toBeInstanceOf(Bun.Image)`

## SYNC2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**e.message** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 34)

Witnessed by 34 constraint clauses across 5 test files. Antichain representatives:

- `test/js/workerd/html-rewriter.test.js:61` — HTMLRewriter > fast async error inside element handler → `expect(e.message).toBe("test")`
- `test/js/web/web-globals.test.js:167` — crypto.timingSafeEqual → `expect(e.message).toBe("Input buffers must have the same byte length")`
- `test/js/web/html/FormData.test.ts:253` — FormData > should throw on missing final boundary → `expect(typeof e.message).toBe("string")`

## SYNC3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**res.bytes** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 15)

Witnessed by 15 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/fetch/fetch-http3-client.test.ts:219` — fetch protocol: http3 > large response body (multi-packet) → `expect(buf.length).toBe(big.length)`
- `test/js/web/fetch/fetch-http3-adversarial.test.ts:137` — POST /echo (Uint8Array) → `expect(got.length).toBe(size)`
- `test/js/bun/image/image.test.ts:177` — Bun.Image > Bun.file() input chains the async file read into the pipeline → `expect((await res.bytes()).subarray(8, 12)).toEqual(Buffer.from("WEBP"))`

## SYNC4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**reader.read** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/streams/streams.test.js:541` — ReadableStream (direct) → `expect(chunk.value.join("")).toBe(Buffer.from("helloworld").join(""))`
- `test/js/node/fs/fs.test.ts:330` — FileHandle > FileHandle#readableWebStream → `expect(chunk.value instanceof Uint8Array).toBe(true)`
- `test/js/node/async_hooks/AsyncLocalStorage.test.ts:289` — async context passes through > readable stream .start → `expect(result.value).toBe("value")`

## SYNC5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**publicKey.kty** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 12)

Witnessed by 12 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/crypto.key-objects.test.ts:787` — crypto.KeyObjects > Test async elliptic curve key generation with 'jwk' encoding and named… → `expect(publicKey.kty).toEqual("EC")`
- `test/js/node/crypto/crypto.key-objects.test.ts:788` — crypto.KeyObjects > Test async elliptic curve key generation with 'jwk' encoding and named… → `expect(publicKey.kty).toEqual(privateKey.kty)`
- `test/js/node/crypto/crypto.key-objects.test.ts:821` — crypto.KeyObjects > Test async elliptic curve key generation with 'jwk' encoding and RSA. … → `expect(publicKey.kty).toEqual("RSA")`

## SYNC6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**HTMLRewriter** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 10)

Witnessed by 10 constraint clauses across 1 test files. Antichain representatives:

- `test/js/workerd/html-rewriter.test.js:99` — HTMLRewriter > HTMLRewriter: async replacement → `expect(await res.text()).toBe("<div><span>replace</span></div>")`
- `test/js/workerd/html-rewriter.test.js:271` — HTMLRewriter > handles element specific mutations → `expect(await res.text()).toBe( [ "<p>", "<span>prepend html</span>", "&lt;span&gt;prepend&lt;/span&gt;", "test", "&lt;span&gt;append&lt;/span&gt;", "<span>append html</span>", "</p>", ].join(""), )`
- `test/js/workerd/html-rewriter.test.js:291` — HTMLRewriter > handles element specific mutations → `expect(await res.text()).toBe("<p>&lt;span&gt;replace&lt;/span&gt;</p>")`

## SYNC7
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**caught.stack** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/fs/promises.test.js:226` — errors from fs.promises include async stack frames → `expect(caught.stack).toContain("at async level3")`
- `test/js/bun/util/bun-file.test.ts:66` — Bun.file() read errors include async stack frames → `expect(caught.stack).toContain("at async level2")`
- `test/js/node/fs/promises.test.js:227` — errors from fs.promises include async stack frames → `expect(caught.stack).toContain("at async level2")`

## SYNC8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**publicKey.crv** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/crypto.key-objects.test.ts:789` — crypto.KeyObjects > Test async elliptic curve key generation with 'jwk' encoding and named… → `expect(publicKey.crv).toEqual(curve)`
- `test/js/node/crypto/crypto.key-objects.test.ts:790` — crypto.KeyObjects > Test async elliptic curve key generation with 'jwk' encoding and named… → `expect(publicKey.crv).toEqual(privateKey.crv)`
- `test/js/node/crypto/crypto.key-objects.test.ts:869` — crypto.KeyObjects > Test async elliptic curve key generation with 'jwk' encoding > should … → `expect(publicKey.crv).toEqual(expectedCrv)`

## SYNC9
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Buffer.isBuffer** — exhibits the property captured in the witnessing test. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:343` — createHash > returns Buffer → `expect(Buffer.isBuffer(hash.digest())).toBeTrue()`
- `test/js/node/crypto/crypto.key-objects.test.ts:905` — crypto.KeyObjects > Test async RSA key generation with an encrypted private key, but encod… → `expect(Buffer.isBuffer(publicKeyDER)).toBeTrue()`
- `test/js/node/crypto/crypto.key-objects.test.ts:953` — crypto.KeyObjects > Test async RSA key generation with an encrypted private key → `expect(Buffer.isBuffer(publicKeyDER)).toBeTrue()`

## SYNC10
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**caught.code** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/fs/promises.test.js:225` — errors from fs.promises include async stack frames → `expect(caught.code).toBe("ENOENT")`
- `test/js/bun/util/bun-file.test.ts:65` — Bun.file() read errors include async stack frames → `expect(caught.code).toBe("ENOENT")`
- `test/js/node/fs/promises.test.js:246` — fs.promises async stack through Promise subclass → `expect(caught.code).toBe("ENOENT")`

## SYNC11
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**t.transformSync** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/resolve/lower-using-bun-target.test.ts:49` — using / await using is not lowered when targeting bun > single-use using declaration is no… → `expect(out).toContain("using server = open()")`
- `test/js/bun/resolve/lower-using-bun-target.test.ts:50` — using / await using is not lowered when targeting bun > single-use using declaration is no… → `expect(out).toContain("return server.url")`
- `test/js/bun/resolve/lower-using-bun-target.test.ts:51` — using / await using is not lowered when targeting bun > single-use using declaration is no… → `expect(out).toContain("await using conn = connect()")`

## SYNC12
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**catchSpy.firstCall.args** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:5762` — loop limit stack trace > setTimeout > provides a stack trace for running all async → `assert.equals(err.message, expectedMessage)`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:5801` — loop limit stack trace > requestIdleCallback > provides a stack trace for running all asyn… → `assert.equals(err.message, expectedMessage)`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:5843` — loop limit stack trace > setInterval > provides a stack trace for running all async → `assert.equals(err.message, expectedMessage)`

## SYNC13
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**e.message.length** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/plugin/plugins.test.ts:383` — errors > invalid loaders throw → `expect(e.message.length > 0).toBe(true)`
- `test/js/bun/plugin/plugins.test.ts:402` — errors > transpiler errors work → `expect(e.message.length > 0).toBe(true)`
- `test/js/bun/plugin/plugins.test.ts:417` — errors > invalid async return value → `expect(e.message.length > 0).toBe(true)`

