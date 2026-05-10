# X509Certificate — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: x509certificate-surface-property
  threshold: XCER1
  interface: [X509Certificate]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 13.

## XCER1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**X509Certificate** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/crypto/crypto_key_test.ts:771` — X509Certificate checkHost → `assertEquals(cert.checkHost("www.google.com"), undefined)`
- `tests/unit_node/crypto/crypto_key_test.ts:772` — X509Certificate checkHost → `assertEquals(cert.checkHost("agent1"), "agent1")`
- `tests/unit_node/crypto/crypto_key_test.ts:817` — X509Certificate validFromDate validToDate → `assertEquals( x509.validFromDate, new Date("2022-09-03T21:40:37.000Z"), )`

