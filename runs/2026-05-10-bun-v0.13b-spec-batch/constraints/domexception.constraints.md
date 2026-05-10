# DOMException — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: domexception-surface-property
  threshold: DOME1
  interface: [DOMException]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 11.

## DOME1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**DOMException** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:8` — exists → `expect(typeof DOMException !== "undefined").toBe(true)`
- `test/js/node/domexception-node.test.js:5` — DOMException in Node.js environment > exists globally → `expect(typeof DOMException).toBe("function")`
- `test/js/node/domexception-node.test.js:11` — DOMException in Node.js environment > creates instance with message and name → `expect(error.message).toBe("Error message")`

