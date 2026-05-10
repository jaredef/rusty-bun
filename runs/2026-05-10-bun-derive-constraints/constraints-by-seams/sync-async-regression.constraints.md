# sync+async/@regression — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: sync-async-regression-surface-property
  threshold: SYNC1
  interface: [tty.WriteStream]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 272.

## SYNC1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**tty.WriteStream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/test-process-stdout-async-iterator.test.ts:28` — tty.WriteStream has Symbol.asyncIterator → `expect(typeof stream[Symbol.asyncIterator]).toBe("function")`

## SYNC2
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Bun.file** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 271)

Witnessed by 271 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/patch-bounds-check.test.ts:140` — patch application should work correctly with valid patches → `expect(patchedFile).toMatchInlineSnapshot(' "// Valid patch comment module.exports = require('./lodash');" ')`
- `test/regression/issue/cyclic-imports-async-bundler.test.js:93` — cyclic imports with async dependencies should generate async wrappers → `expect(bundled).toMatchInlineSnapshot(' "var __defProp = Object.defineProperty; var __returnValue = (v) => v; function __exportSetter(name, newValue) { this[name] = __returnValue.bind(null, newValue);…`
- `test/regression/issue/3192.test.ts:43` — issue #3192 > yarn lockfile quotes workspace:* versions correctly → `expect(yarnLock).toContain('"package-b@workspace:*"')`

