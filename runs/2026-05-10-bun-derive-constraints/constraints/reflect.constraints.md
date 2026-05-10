# Reflect — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: reflect-surface-property
  threshold: REFL1
  interface: [Reflect.getMetadata, Reflect.getMetadata]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 244.

## REFL1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Reflect.getMetadata** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 224)

Witnessed by 224 constraint clauses across 5 test files. Antichain representatives:

- `test/integration/typegraphql/src/unsolvable.test.ts:16` — basic metadata works → `expect(Reflect.getMetadata("design:type", M.prototype, "myval")).toBe(Number)`
- `test/integration/typegraphql/src/typegraphql.test.ts:50` — correct reflect.metadata types for getters → `expect(Reflect.getMetadata("design:type", User.prototype, "firstName")).toBe(String)`
- `test/integration/typegraphql/src/ts_example.test.ts:51` — ts_example → `expect(Reflect.getMetadata("design:type", line, "start")).toBe(Point)`

## REFL2
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

