# CSRF — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: csrf-surface-property
  threshold: CSRF1
  interface: [CSRF.verify]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 18.

## CSRF1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**CSRF.verify** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 18)

Witnessed by 18 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/csrf.test.ts:10` — Bun.CSRF > CSRF exists → `expect(typeof CSRF.verify).toBe("function")`
- `test/js/bun/util/csrf.test.ts:37` — Bun.CSRF > verifies a valid token → `expect(isValid).toBe(true)`
- `test/js/bun/util/csrf.test.ts:47` — Bun.CSRF > rejects an invalid token → `expect(isValid).toBe(false)`

