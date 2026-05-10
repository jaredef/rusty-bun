# crypto — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: crypto-surface-property
  threshold: CRYP1
  interface: [crypto.subtle.exportKey, crypto.subtle.generateKey, crypto.verify, crypto.subtle.deriveBits, crypto.subtle.deriveKey, crypto.subtle.importKey, crypto.subtle.verify, crypto.subtle.exportKey, crypto.timingSafeEqual, crypto.generatePrimeSync, crypto.getRandomValues, crypto.randomInt, crypto.sign, crypto.subtle, crypto.subtle, crypto]

@imports: []

@pins: []

Surface drawn from 27 candidate properties across the Bun test corpus. Construction-style: 18; behavioral (high-cardinality): 9. Total witnessing constraint clauses: 128.

## CRYP1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.exportKey** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/crypto/web-crypto-sha3.test.ts:71` — HMAC with SHA-3 > HMAC-SHA3-384 generateKey default length → `expect(raw.byteLength).toBe(104)`
- `test/js/deno/crypto/webcrypto.test.ts:574` —  → `assertEquals(exportedKey2, jwk)`
- `test/js/web/crypto/web-crypto-sha3.test.ts:92` — RSA with SHA-3 hash > RSA-PSS with SHA3-256: generate, sign, verify, JWK export → `expect(jwk.kty).toBe("RSA")`

## CRYP2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.generateKey** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/deno/crypto/webcrypto.test.ts:584` —  → `assertEquals(key.type, "secret")`
- `test/js/deno/crypto/webcrypto.test.ts:585` —  → `assertEquals(key.extractable, true)`
- `test/js/deno/crypto/webcrypto.test.ts:586` —  → `assertEquals(key.usages, [ "sign" ])`

## CRYP3
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

## CRYP4
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.deriveBits** — exposes values of the expected type or class. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/crypto/x25519-derive-bits.test.ts:23` — X25519 deriveBits with known test vector → `expect(bits).toBeInstanceOf(ArrayBuffer)`
- `test/js/bun/crypto/x25519-derive-bits.test.ts:33` — X25519 deriveBits with null length returns full output → `expect(bits).toBeInstanceOf(ArrayBuffer)`
- `test/js/bun/crypto/x25519-derive-bits.test.ts:43` — X25519 deriveBits with zero length returns full output → `expect(bits).toBeInstanceOf(ArrayBuffer)`

## CRYP5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.deriveKey** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 2 test files. Antichain representatives:

- `test/js/deno/crypto/webcrypto.test.ts:828` —  → `assertEquals(derivedKey.type, "secret")`
- `test/js/bun/crypto/x25519-derive-bits.test.ts:114` — X25519 deriveKey produces an AES-GCM key from the shared secret → `expect(key.algorithm.name).toBe("AES-GCM")`
- `test/js/deno/crypto/webcrypto.test.ts:829` —  → `assertEquals(derivedKey.extractable, true)`

## CRYP6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.importKey** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/deno/crypto/webcrypto.test.ts:627` —  → `assertEquals(key.type, "private")`
- `test/js/deno/crypto/webcrypto.test.ts:628` —  → `assertEquals(key.extractable, true)`
- `test/js/deno/crypto/webcrypto.test.ts:629` —  → `assertEquals(key.usages, [ "sign" ])`

## CRYP7
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

## CRYP8
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.exportKey** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/24399.test.ts:16` — ECDSA exported JWK fields have correct length → `expect(jwk.d).toBeDefined()`
- `test/regression/issue/24399.test.ts:31` — ECDH exported JWK fields have correct length → `expect(jwk.d).toBeDefined()`
- `test/js/web/crypto/web-crypto-sha3.test.ts:93` — RSA with SHA-3 hash > RSA-PSS with SHA3-256: generate, sign, verify, JWK export → `expect(jwk.alg).toBeUndefined()`

## CRYP9
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.timingSafeEqual** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:156` — crypto.timingSafeEqual → `expect(crypto.timingSafeEqual(uuid, uuid)).toBe(true)`
- `test/js/web/web-globals.test.js:157` — crypto.timingSafeEqual → `expect(crypto.timingSafeEqual(uuid, uuid.slice())).toBe(true)`
- `test/js/web/web-globals.test.js:171` — crypto.timingSafeEqual → `expect(crypto.timingSafeEqual(uuid, crypto.randomUUID())).toBe(false)`

## CRYP10
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto.generatePrimeSync** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:817` — generatePrime(Sync) should return an ArrayBuffer → `expect(prime).toBeInstanceOf(ArrayBuffer)`
- `test/js/node/crypto/node-crypto.test.js:823` — generatePrime(Sync) should return an ArrayBuffer → `expect(prime).toBeInstanceOf(ArrayBuffer)`

## CRYP11
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.getRandomValues** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `crypto-random.spec.md:18` — crypto.getRandomValues method → `crypto.getRandomValues throws QuotaExceededError when typedArray byte length exceeds 65536`
- `crypto-random.spec.md:19` — crypto.getRandomValues method → `crypto.getRandomValues throws TypeMismatchError on non-integer typed arrays`

## CRYP12
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.randomInt** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:14` — crypto.randomInt should return a number → `expect(typeof result).toBe("number")`
- `test/js/node/crypto/node-crypto.test.js:25` — crypto.randomInt with one argument → `expect(typeof result).toBe("number")`

## CRYP13
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto.sign** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/11029-crypto-verify-null-algorithm.test.ts:24` — crypto.verify with null algorithm should work for RSA keys → `expect(signature).toBeInstanceOf(Buffer)`
- `test/regression/issue/11029-crypto-verify-null-algorithm.test.ts:74` — crypto.verify with null algorithm should work for Ed25519 keys → `expect(signature).toBeInstanceOf(Buffer)`

## CRYP14
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/crypto/web-crypto.test.ts:38` — Web Crypto > has globals → `expect(crypto.subtle !== undefined).toBe(true)`
- `test/js/web/crypto/web-crypto.test.ts:134` — Web Crypto > unwrapKey JWK error handling > rejects when wrapped bytes are not valid JSON → `expect(err.name).toBe("DataError")`

## CRYP15
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/crypto/web-crypto.test.ts:133` — Web Crypto > unwrapKey JWK error handling > rejects when wrapped bytes are not valid JSON → `expect(err).toBeInstanceOf(DOMException)`
- `test/js/web/crypto/web-crypto.test.ts:148` — Web Crypto > unwrapKey JWK error handling > rejects when wrapped bytes are valid JSON but … → `expect(err).toBeInstanceOf(TypeError)`

## CRYP16
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `crypto-random.spec.md:7` — crypto is exposed as a global object → `crypto is defined as a global object in any execution context with [Exposed=*]`

## CRYP17
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.getCurves** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/crypto.test.ts:159` — crypto.getCurves > should return an array of strings → `expect(typeof crypto.getCurves()[0]).toBe("string")`

## CRYP18
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.randomBytes** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:8` — crypto.randomBytes should return a Buffer → `expect(crypto.randomBytes(1) instanceof Buffer).toBe(true)`

## CRYP19
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.randomUUID** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 18)

Witnessed by 18 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:149` — crypto.timingSafeEqual → `expect(uuidStr.length).toBe(36)`
- `crypto-random.spec.md:10` — crypto.randomUUID method → `crypto.randomUUID returns a v4 UUID as a USVString`
- `test/js/web/web-globals.test.js:150` — crypto.timingSafeEqual → `expect(uuidStr[8]).toBe("-")`

## CRYP20
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.createDiffieHellman** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:569` — DiffieHellman > should have correct method names → `expect(dh.generateKeys.name).toBe("generateKeys")`
- `test/js/node/crypto/node-crypto.test.js:570` — DiffieHellman > should have correct method names → `expect(dh.computeSecret.name).toBe("computeSecret")`
- `test/js/node/crypto/node-crypto.test.js:571` — DiffieHellman > should have correct method names → `expect(dh.getPrime.name).toBe("getPrime")`

## CRYP21
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.createCipheriv** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:543` — Cipheriv > should have correct method names → `expect(cipher.update.name).toBe("update")`
- `test/js/node/crypto/node-crypto.test.js:544` — Cipheriv > should have correct method names → `expect(cipher.final.name).toBe("final")`
- `test/js/node/crypto/node-crypto.test.js:545` — Cipheriv > should have correct method names → `expect(cipher.setAutoPadding.name).toBe("setAutoPadding")`

## CRYP22
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.createDecipheriv** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:556` — Decipheriv > should have correct method names → `expect(decipher.update.name).toBe("update")`
- `test/js/node/crypto/node-crypto.test.js:557` — Decipheriv > should have correct method names → `expect(decipher.final.name).toBe("final")`
- `test/js/node/crypto/node-crypto.test.js:558` — Decipheriv > should have correct method names → `expect(decipher.setAutoPadding.name).toBe("setAutoPadding")`

## CRYP23
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.createHash** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:504` — Hash > should have correct method names → `expect(hash.update.name).toBe("update")`
- `test/js/node/buffer.test.js:1293` — truncation after decode → `expect(crypto.createHash("sha1").update(Buffer.from("YW55=======", "base64")).digest("hex")).toBe( crypto.createHash("sha1").update(Buffer.from("YW55", "base64")).digest("hex"), )`
- `test/js/node/crypto/node-crypto.test.js:505` — Hash > should have correct method names → `expect(hash.digest.name).toBe("digest")`

## CRYP24
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.createECDH** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:583` — ECDH > should have correct method names → `expect(ecdh.generateKeys.name).toBe("generateKeys")`
- `test/js/node/crypto/node-crypto.test.js:584` — ECDH > should have correct method names → `expect(ecdh.computeSecret.name).toBe("computeSecret")`
- `test/js/node/crypto/node-crypto.test.js:585` — ECDH > should have correct method names → `expect(ecdh.getPublicKey.name).toBe("getPublicKey")`

## CRYP25
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.createHmac** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:515` — Hmac > should have correct method names → `expect(hmac.update.name).toBe("update")`
- `test/js/node/crypto/node-crypto.test.js:516` — Hmac > should have correct method names → `expect(hmac.digest.name).toBe("digest")`
- `test/js/node/crypto/node-crypto.test.js:517` — Hmac > should have correct method names → `expect(hmac._transform.name).toBe("_transform")`

## CRYP26
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.deriveBits** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/deno/crypto/webcrypto.test.ts:753` —  → `assertEquals(result.byteLength, 128 / 8)`
- `test/js/bun/crypto/x25519-derive-bits.test.ts:34` — X25519 deriveBits with null length returns full output → `expect(bits.byteLength).toBe(32)`
- `test/js/deno/crypto/webcrypto.test.ts:964` —  → `assertEquals(derivedKey.byteLength, keySize / 8)`

## CRYP27
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.generateKey** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/deno/crypto/webcrypto.test.ts:583` —  → `assert(key)`
- `test/js/deno/crypto/webcrypto.test.ts:722` —  → `assert(keyPair.privateKey)`
- `test/js/deno/crypto/webcrypto.test.ts:723` —  → `assert(keyPair.publicKey)`

