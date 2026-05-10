# vm — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: vm-surface-property
  threshold: VM1
  interface: [vm.runInContext]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 15.

## VM1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**vm.runInContext** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/transpiler/repl-transform.test.ts:108` — Bun.Transpiler replMode > REPL session with node:vm > await with variable → `expect(result).toEqual({ value: 20 })`
- `test/js/bun/bun-object/deep-equals.spec.ts:76` — global object > main global object is not equal to vm global objects → `expect(areEqual).toBe(false)`
- `test/js/bun/transpiler/repl-transform.test.ts:191` — Bun.Transpiler replMode > object literal detection > {foo: await bar()} parsed as object l… → `expect(result.value).toEqual({ foo: 42 })`

## VM2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**vm.createContext** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/transpiler/repl-transform.test.ts:104` — Bun.Transpiler replMode > REPL session with node:vm > await with variable → `expect(ctx.x).toBe(10)`
- `test/js/bun/transpiler/repl-transform.test.ts:245` — Bun.Transpiler replMode > edge cases > destructuring assignment persists → `expect(ctx.a).toBe(1)`
- `test/js/bun/transpiler/repl-transform.test.ts:246` — Bun.Transpiler replMode > edge cases > destructuring assignment persists → `expect(ctx.b).toBe(2)`

## VM3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**vm.runInNewContext** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/util/node-inspect-tests/parallel/util-inspect.test.js:3013` — no assertion failures 3 → `assert.strictEqual(target.ctx, undefined)`
- `test/js/bun/jsc/domjit.test.ts:152` — DOMJIT > in NodeVM > vm.runInNewContext → `expect(vm.runInNewContext(code, { crypto, performance, TextEncoder, TextDecoder, dirStats })).toBe("success")`
- `test/js/node/util/node-inspect-tests/parallel/util-inspect.test.js:3018` — no assertion failures 3 → `assert.strictEqual(typeof target.ctx, "object")`

