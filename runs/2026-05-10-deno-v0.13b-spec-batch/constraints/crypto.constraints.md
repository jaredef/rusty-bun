# crypto — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: crypto-surface-property
  threshold: CRYP1
  interface: [crypto.subtle.exportKey, crypto.subtle.generateKey, crypto.subtle.importKey, crypto.subtle.deriveKey, crypto.getRandomValues, crypto, crypto.timingSafeEqual]

@imports: []

@pins: []

Surface drawn from 9 candidate properties across the Bun test corpus. Construction-style: 7; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 44.

## CRYP1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.exportKey** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 13 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/webcrypto_test.ts:358` —  → `assertEquals(exportedKey2, jwk)`
- `tests/unit/webcrypto_test.ts:377` —  → `assertEquals(exportedKey.kty, "oct")`
- `tests/unit/webcrypto_test.ts:378` —  → `assertEquals(exportedKey.alg, "HS512")`

## CRYP2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.generateKey** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/webcrypto_test.ts:372` —  → `assertEquals(key.type, "secret")`
- `tests/unit/webcrypto_test.ts:373` —  → `assertEquals(key.extractable, true)`
- `tests/unit/webcrypto_test.ts:374` —  → `assertEquals(key.usages, ["sign"])`

## CRYP3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.importKey** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/webcrypto_test.ts:415` —  → `assertEquals(key.type, "private")`
- `tests/unit/webcrypto_test.ts:416` —  → `assertEquals(key.extractable, true)`
- `tests/unit/webcrypto_test.ts:417` —  → `assertEquals(key.usages, ["sign"])`

## CRYP4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.deriveKey** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/webcrypto_test.ts:674` —  → `assertEquals(derivedKey.type, "secret")`
- `tests/unit/webcrypto_test.ts:675` —  → `assertEquals(derivedKey.extractable, true)`
- `tests/unit/webcrypto_test.ts:676` —  → `assertEquals(derivedKey.usages, ["sign"])`

## CRYP5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.getRandomValues** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `crypto-random.spec.md:18` — crypto.getRandomValues method → `crypto.getRandomValues throws QuotaExceededError when typedArray byte length exceeds 65536`
- `crypto-random.spec.md:19` — crypto.getRandomValues method → `crypto.getRandomValues throws TypeMismatchError on non-integer typed arrays`

## CRYP6
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `crypto-random.spec.md:7` — crypto is exposed as a global object → `crypto is defined as a global object in any execution context with [Exposed=*]`

## CRYP7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.timingSafeEqual** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/crypto/crypto_timing_safe_equal_test.ts:14` — timingSafeEqual ArrayBuffer and TypedArray → `assertEquals(eq, true)`

## CRYP8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.exportKey** — satisfies the documented invariant. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/webcrypto_test.ts:508` —  → `assert(exportedPrivateKey)`
- `tests/unit/webcrypto_test.ts:2161` —  → `assert(jwk.x)`
- `tests/unit/webcrypto_test.ts:2162` —  → `assert(jwk.y)`

## CRYP9
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**crypto.subtle.generateKey** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/webcrypto_test.ts:371` —  → `assert(key)`
- `tests/unit/webcrypto_test.ts:499` —  → `assert(keyPair.privateKey)`
- `tests/unit/webcrypto_test.ts:500` —  → `assert(keyPair.publicKey)`

