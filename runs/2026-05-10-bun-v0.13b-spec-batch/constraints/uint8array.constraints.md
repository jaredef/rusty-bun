# Uint8Array — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: uint8array-surface-property
  threshold: UINT1
  interface: [Uint8Array]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 125.

## UINT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Uint8Array** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 125)

Witnessed by 125 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/8254.test.ts:38` — Bun.write() should write past 2GB boundary without corruption → `expect(buf[0]).toBe(expected)`
- `test/regression/issue/27478.test.ts:31` — multipart formdata preserves null bytes in small binary files → `expect(parsed.byteLength).toBe(source.byteLength)`
- `test/regression/issue/23723.test.js:2` — doesn't crash → `expect(typeof Uint8Array !== undefined + "").toBe(true)`

