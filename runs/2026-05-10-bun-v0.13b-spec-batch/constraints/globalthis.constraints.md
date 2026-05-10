# globalThis — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: globalthis-surface-property
  threshold: GLOB1
  interface: [globalThis, globalThis.Buffer, globalThis.fetch.bind, globalThis.navigator]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 4; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 11.

## GLOB1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**globalThis** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:53` — self.${name} → `expect(globalThis[name]).toBe(callback)`
- `test/js/node/vm/vm.test.ts:465` — can modify global context → `expect(globalThis[props[1]]).toBe("baz")`
- `test/js/bun/globals.test.js:72` — writable → `expect(globalThis[name]).toBe(123)`

## GLOB2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**globalThis.Buffer** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/buffer.test.js:217` — Buffer global is settable → `expect(globalThis.Buffer).toBe(42)`
- `test/js/node/buffer.test.js:219` — Buffer global is settable → `expect(globalThis.Buffer).toBe(BufferModule.Buffer)`
- `test/js/node/buffer.test.js:220` — Buffer global is settable → `expect(globalThis.Buffer).toBe(prevBuffer)`

## GLOB3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**globalThis.fetch.bind** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:1605` — #2794 → `expect(typeof globalThis.fetch.bind).toBe("function")`

## GLOB4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**globalThis.navigator** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:233` — navigator → `expect(globalThis.navigator !== undefined).toBe(true)`

