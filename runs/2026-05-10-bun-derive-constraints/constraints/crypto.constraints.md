# crypto — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: crypto-surface-property
  threshold: CRYP1
  interface: [crypto.verify, crypto.getRandomValues, crypto.subtle.deriveBits, crypto.subtle.verify, crypto.subtle.exportKey, crypto.generatePrimeSync, crypto.randomInt, crypto.sign, crypto.subtle, crypto.subtle, crypto.getCurves, crypto.randomBytes]

@imports: []

@pins: []

Surface drawn from 24 candidate properties across the Bun test corpus. Construction-style: 12; behavioral (high-cardinality): 12. Total witnessing constraint clauses: 147.

## CRYP1
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

## CRYP2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.getRandomValues** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:121` — crypto.getRandomValues → `expect(array).toBe(foo)`
- `test/js/web/web-globals.test.js:122` — crypto.getRandomValues → `expect(array.reduce((sum, a) => (sum += a === 0), 0) != foo.length).toBe(true)`
- `test/js/web/web-globals.test.js:130` — crypto.getRandomValues → `expect(array).toBe(foo)`

## CRYP3
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

## CRYP4
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

## CRYP5
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

## CRYP6
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto.generatePrimeSync** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:817` — generatePrime(Sync) should return an ArrayBuffer → `expect(prime).toBeInstanceOf(ArrayBuffer)`
- `test/js/node/crypto/node-crypto.test.js:823` — generatePrime(Sync) should return an ArrayBuffer → `expect(prime).toBeInstanceOf(ArrayBuffer)`

## CRYP7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.randomInt** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:14` — crypto.randomInt should return a number → `expect(typeof result).toBe("number")`
- `test/js/node/crypto/node-crypto.test.js:25` — crypto.randomInt with one argument → `expect(typeof result).toBe("number")`

## CRYP8
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto.sign** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/11029-crypto-verify-null-algorithm.test.ts:24` — crypto.verify with null algorithm should work for RSA keys → `expect(signature).toBeInstanceOf(Buffer)`
- `test/regression/issue/11029-crypto-verify-null-algorithm.test.ts:74` — crypto.verify with null algorithm should work for Ed25519 keys → `expect(signature).toBeInstanceOf(Buffer)`

## CRYP9
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/crypto/web-crypto.test.ts:38` — Web Crypto > has globals → `expect(crypto.subtle !== undefined).toBe(true)`
- `test/js/web/crypto/web-crypto.test.ts:134` — Web Crypto > unwrapKey JWK error handling > rejects when wrapped bytes are not valid JSON → `expect(err.name).toBe("DataError")`

## CRYP10
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/crypto/web-crypto.test.ts:133` — Web Crypto > unwrapKey JWK error handling > rejects when wrapped bytes are not valid JSON → `expect(err).toBeInstanceOf(DOMException)`
- `test/js/web/crypto/web-crypto.test.ts:148` — Web Crypto > unwrapKey JWK error handling > rejects when wrapped bytes are valid JSON but … → `expect(err).toBeInstanceOf(TypeError)`

## CRYP11
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.getCurves** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/crypto.test.ts:159` — crypto.getCurves > should return an array of strings → `expect(typeof crypto.getCurves()[0]).toBe("string")`

## CRYP12
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.randomBytes** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:8` — crypto.randomBytes should return a Buffer → `expect(crypto.randomBytes(1) instanceof Buffer).toBe(true)`

## CRYP13
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.randomUUID** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 15)

Witnessed by 15 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:149` — crypto.timingSafeEqual → `expect(uuidStr.length).toBe(36)`
- `test/js/web/web-globals.test.js:150` — crypto.timingSafeEqual → `expect(uuidStr[8]).toBe("-")`
- `test/js/web/web-globals.test.js:151` — crypto.timingSafeEqual → `expect(uuidStr[13]).toBe("-")`

## CRYP14
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.exportKey** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 15)

Witnessed by 15 constraint clauses across 3 test files. Antichain representatives:

- `test/regression/issue/24399.test.ts:17` — ECDSA exported JWK fields have correct length → `expect(jwk.d!.length).toBe(expectedLength)`
- `test/js/web/crypto/web-crypto-sha3.test.ts:71` — HMAC with SHA-3 > HMAC-SHA3-384 generateKey default length → `expect(raw.byteLength).toBe(104)`
- `test/js/deno/crypto/webcrypto.test.ts:574` —  → `assertEquals(exportedKey2, jwk)`

## CRYP15
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.createHash** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 14)

Witnessed by 14 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:298` — createHash > ${name} - "Hello World" → `expect(hash.digest("hex")).toBe(v.value)`
- `test/js/node/crypto/node-crypto.test.js:307` — createHash > ${name} - "Hello World" → `expect(hash.digest("hex")).toBe(v.value)`
- `test/js/node/crypto/node-crypto.test.js:324` — createHash > ${name} - "Hello World" -> binary → `expect(hash.digest()).toEqual(Buffer.from(v.value, "hex"))`

## CRYP16
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.privateDecrypt** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 10)

Witnessed by 10 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/crypto-rsa.test.js:75` — RSA encryption/decryption > privateDecrypt with rsaKeyPem → `expect(decryptedBuffer.toString()).toBe(input)`
- `test/js/node/crypto/crypto-rsa.test.js:80` — RSA encryption/decryption > privateDecrypt with otherEncrypted → `expect(otherDecrypted.toString()).toBe(input)`
- `test/js/node/crypto/crypto-rsa.test.js:85` — RSA encryption/decryption > privateDecrypt with rsaPkcs8KeyPem → `expect(decryptedBuffer.toString()).toBe(input)`

## CRYP17
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

## CRYP18
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.createHmac** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:515` — Hmac > should have correct method names → `expect(hmac.update.name).toBe("update")`
- `test/js/node/crypto/crypto-hmac-algorithm.test.ts:47` — createHmac works with various algorithm names → `expect(hmac1.digest("hex")).toBe(hmac2.digest("hex"))`
- `test/js/node/crypto/node-crypto.test.js:516` — Hmac > should have correct method names → `expect(hmac.digest.name).toBe("digest")`

## CRYP19
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.generateKey** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/deno/crypto/webcrypto.test.ts:584` —  → `assertEquals(key.type, "secret")`
- `test/js/deno/crypto/webcrypto.test.ts:585` —  → `assertEquals(key.extractable, true)`
- `test/js/deno/crypto/webcrypto.test.ts:586` —  → `assertEquals(key.usages, [ "sign" ])`

## CRYP20
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

## CRYP21
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

## CRYP22
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.deriveBits** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `test/js/deno/crypto/webcrypto.test.ts:753` —  → `assertEquals(result.byteLength, 128 / 8)`
- `test/js/bun/crypto/x25519-derive-bits.test.ts:34` — X25519 deriveBits with null length returns full output → `expect(bits.byteLength).toBe(32)`
- `test/js/deno/crypto/webcrypto.test.ts:780` —  → `assertEquals(result.byteLength * 8, 256)`

## CRYP23
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.generateKey** — satisfies the documented invariant. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/js/deno/crypto/webcrypto.test.ts:583` —  → `assert(key)`
- `test/js/deno/crypto/webcrypto.test.ts:722` —  → `assert(keyPair.privateKey)`
- `test/js/deno/crypto/webcrypto.test.ts:723` —  → `assert(keyPair.publicKey)`

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

