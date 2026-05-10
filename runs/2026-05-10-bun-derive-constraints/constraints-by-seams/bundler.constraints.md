# @bundler — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: bundler-surface-property
  threshold: BUND1
  interface: [S, Reflect.getMetadata, result.metafile, metaFile.text, result3.outputs, code.includes, napiModule.getCompilationCtxFreedCount, transpiler.transform]

@imports: []

@pins: []

Surface drawn from 11 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 11. Total witnessing constraint clauses: 108.

## BUND1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**S** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 22)

Witnessed by 22 constraint clauses across 1 test files. Antichain representatives:

- `test/bundler/transpiler/decorators.test.ts:454` — decorators random → `expect(S[h]).toBe(30)`
- `test/bundler/transpiler/decorators.test.ts:457` — decorators random → `expect(S[q]).toBe(202)`
- `test/bundler/transpiler/decorators.test.ts:466` — decorators random → `expect(S[u3]).toBe(undefined)`

## BUND2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Reflect.getMetadata** — is defined and resolves to a non-nullish value at the documented call site. (behavioral; cardinality 20)

Witnessed by 20 constraint clauses across 1 test files. Antichain representatives:

- `test/bundler/transpiler/decorator-metadata.test.ts:401` — decorator metadata > design: type, paramtypes, returntype → `expect(Reflect.getMetadata("design:type", A)).toBeUndefined()`
- `test/bundler/transpiler/decorator-metadata.test.ts:403` — decorator metadata > design: type, paramtypes, returntype → `expect(Reflect.getMetadata("design:returntype", A)).toBeUndefined()`
- `test/bundler/transpiler/decorator-metadata.test.ts:405` — decorator metadata > design: type, paramtypes, returntype → `expect(Reflect.getMetadata("design:type", A.prototype)).toBeUndefined()`

## BUND3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**result.metafile** — is defined and resolves to a non-nullish value at the documented call site. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 1 test files. Antichain representatives:

- `test/bundler/metafile.test.ts:51` — bundler metafile > metafile option returns metafile object → `expect(metafile.inputs).toBeDefined()`
- `test/bundler/metafile.test.ts:53` — bundler metafile > metafile option returns metafile object → `expect(metafile.outputs).toBeDefined()`
- `test/bundler/metafile.test.ts:557` — Bun.build metafile option variants > metafile: string writes JSON to file path → `expect(metafile.inputs).toBeDefined()`

## BUND4
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**metaFile.text** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/bundler/metafile.test.ts:742` — bun build --metafile-md > generates markdown metafile with default name → `expect(content).toContain("# Bundle Analysis Report")`
- `test/bundler/metafile.test.ts:743` — bun build --metafile-md > generates markdown metafile with default name → `expect(content).toContain("## Quick Summary")`
- `test/bundler/metafile.test.ts:744` — bun build --metafile-md > generates markdown metafile with default name → `expect(content).toContain("## Entry Point Analysis")`

## BUND5
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**result3.outputs** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/bundler/resolver/cache-invalidation.test.ts:58` — resolver cache invalidation > directory with index.js deleted then recreated → `expect(text3).toContain("99")`
- `test/bundler/resolver/cache-invalidation.test.ts:104` — resolver cache invalidation > directory with index.ts deleted then recreated → `expect(text3).toContain("a * b")`
- `test/bundler/resolver/cache-invalidation.test.ts:151` — resolver cache invalidation > direct file deleted then recreated → `expect(text3).toContain("2")`

## BUND6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**code.includes** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/bundler/transpiler/transpiler.test.js:3349` — Bun.Transpiler > transform > removes types → `expect(code.includes("mod")).toBe(true)`
- `test/bundler/transpiler/transpiler.test.js:3350` — Bun.Transpiler > transform > removes types → `expect(code.includes("xx")).toBe(true)`
- `test/bundler/transpiler/transpiler.test.js:3351` — Bun.Transpiler > transform > removes types → `expect(code.includes("ActionFunction")).toBe(true)`

## BUND7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**napiModule.getCompilationCtxFreedCount** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/bundler/native-plugin.test.ts:122` — native-plugins > works in a basic case → `expect(compilationCtxFreedCount).toBe(2)`
- `test/bundler/native-plugin.test.ts:180` — native-plugins > doesn't explode when there are a lot of concurrent files → `expect(compilationCtxFreedCount).toBe(2)`
- `test/bundler/native-plugin.test.ts:252` — native-plugins > doesn't explode when there are a lot of concurrent files AND the filter r… → `expect(compilationCtxFreedCount).toBe(2)`

## BUND8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**transpiler.transform** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/bundler/transpiler/transpiler.test.js:3274` — Bun.Transpiler > transform > supports macros → `expect(out.includes("Test failed")).toBe(false)`
- `test/bundler/transpiler/transpiler.test.js:3275` — Bun.Transpiler > transform > supports macros → `expect(out.includes("Test passed")).toBe(true)`
- `test/bundler/transpiler/transpiler.test.js:3278` — Bun.Transpiler > transform > supports macros → `expect(out.includes("keepSecondArgument")).toBe(false)`

## BUND9
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Object.keys** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/bundler/metafile.test.ts:75` — bundler metafile > metafile inputs contain file metadata → `expect(inputKeys.length).toBeGreaterThanOrEqual(2)`
- `packages/bun-vscode/src/features/tests/__tests__/bun-test-controller.test.ts:1042` — BunTestController - Integration and Coverage > _internal getter > should provide consisten… → `expect(methodNames.length).toBeGreaterThanOrEqual(16)`
- `test/bundler/metafile.test.ts:103` — bundler metafile > metafile outputs contain chunk metadata → `expect(outputKeys.length).toBeGreaterThanOrEqual(1)`

## BUND10
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**result.metafile** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/bundler/metafile.test.ts:52` — bundler metafile > metafile option returns metafile object → `expect(typeof metafile.inputs).toBe("object")`
- `test/bundler/metafile.test.ts:54` — bundler metafile > metafile option returns metafile object → `expect(typeof metafile.outputs).toBe("object")`
- `test/bundler/metafile.test.ts:556` — Bun.build metafile option variants > metafile: string writes JSON to file path → `expect(typeof metafile).toBe("object")`

## BUND11
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**result1.outputs** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/bundler/resolver/cache-invalidation.test.ts:29` — resolver cache invalidation > directory with index.js deleted then recreated → `expect(text1).toContain("42")`
- `test/bundler/resolver/cache-invalidation.test.ts:123` — resolver cache invalidation > direct file deleted then recreated → `expect(text1).toContain("1")`
- `test/bundler/resolver/cache-invalidation.test.ts:217` — resolver cache invalidation > extension resolution after file deletion → `expect(text1).toContain("js version")`

