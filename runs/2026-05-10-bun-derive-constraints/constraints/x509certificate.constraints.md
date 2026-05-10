# X509Certificate — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: x509certificate-surface-property
  threshold: XCER1
  interface: [X509Certificate, X509Certificate]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 24.

## XCER1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**X509Certificate** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 14)

Witnessed by 14 constraint clauses across 3 test files. Antichain representatives:

- `test/regression/issue/27025.test.ts:30` — X509Certificate properties should not crash on valid certificates → `expect(cert.ca).toBe(false)`
- `test/regression/issue/21274.test.ts:10` — #21274 → `expect(cert.subjectAltName).toEqual( "DNS:*.lifecycle-prober-prod-89308e4e-9927-4280-9e14-3330f6900396.asia-northeast1.managedkafka.gmk-lifecycle-prober-prod-1.cloud.goog", )`
- `test/js/node/crypto/x509-subclass.test.ts:31` — X509Certificate > instance uses correct prototype → `expect(cert instanceof X509Certificate).toBe(true)`

## XCER2
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

