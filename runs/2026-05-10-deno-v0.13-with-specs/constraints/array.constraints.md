# Array — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: array-surface-property
  threshold: ARRA1
  interface: [Array.isArray, Array.from]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 28.

## ARRA1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Array.isArray** — satisfies the documented invariant. (behavioral; cardinality 15)

Witnessed by 15 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit_node/tls_test.ts:581` — tls.getCACertificates returns bundled certificates → `assert(Array.isArray(certs))`
- `tests/unit_node/process_test.ts:1300` —  → `assert(Array.isArray(importedExecArgv))`
- `tests/unit_node/perf_hooks_test.ts:52` — [perf_hooks]: PerformanceObserver.supportedEntryTypes → `assert(Array.isArray(supported))`

## ARRA2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Array.from** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 3 test files. Antichain representatives:

- `tests/unit_node/sqlite_test.ts:495` — [node/sqlite] StatementSync iterate should not reuse previous state → `assertEquals(Array.from(stmt.iterate()), [])`
- `tests/unit/text_encoding_test.ts:116` —  → `assertEquals(Array.from(encoder.encode(fixture)), [ 0xf0, 0x9d, 0x93, 0xbd, 0xf0, 0x9d, 0x93, 0xae, 0xf0, 0x9d, 0x94, 0x81, 0xf0, 0x9d, 0x93, 0xbd ])`
- `tests/napi/object_test.js:84` — napi get_property_names → `assertEquals(Array.from(names).sort(), ["a", "b", "c"])`

