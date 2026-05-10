# sync/@bundler — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: sync-bundler-surface-property
  threshold: SYNC1
  interface: [bun.transformSync, nodeTranspiler.transformSync]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 19.

## SYNC1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**bun.transformSync** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 14)

Witnessed by 14 constraint clauses across 1 test files. Antichain representatives:

- `test/bundler/transpiler/transpiler.test.js:1338` — Bun.Transpiler > JSX keys → `expect(bun.transformSync("console.log(<div key={() => {}} points={() => {}}></div>);")).toBe( 'console.log(jsxDEV_7x81h0kn("div", { points: () => {} }, () => {}, false, undefined, this)); ', )`
- `test/bundler/transpiler/transpiler.test.js:1345` — Bun.Transpiler > JSX keys → `expect(bun.transformSync("console.log(<div points={() => {}} key={() => {}}></div>);")).toBe( 'console.log(jsxDEV_7x81h0kn("div", { points: () => {} }, () => {}, false, undefined, this)); ', )`
- `test/bundler/transpiler/transpiler.test.js:1352` — Bun.Transpiler > JSX keys → `expect(bun.transformSync("console.log(<div key={() => {}} key={() => {}}></div>);")).toBe( 'console.log(jsxDEV_7x81h0kn("div", {\n key: () => {}\n}, () => {}, false, undefined, this));\n', )`

## SYNC2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**nodeTranspiler.transformSync** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/bundler/transpiler/transpiler.test.js:1548` — Bun.Transpiler > require with a dynamic non-string expression → `expect(nodeTranspiler.transformSync("require('hi' + bar)")).toBe('require("hi" + bar);\n')`
- `test/bundler/transpiler/transpiler.test.js:1555` — Bun.Transpiler > CommonJS → `expect(nodeTranspiler.transformSync("module.require('hi' + 123)")).toBe('require("hi123");\n')`
- `test/bundler/transpiler/transpiler.test.js:1557` — Bun.Transpiler > CommonJS → `expect(nodeTranspiler.transformSync("module.require(1 ? 'foo' : 'bar')")).toBe('require("foo");\n')`

