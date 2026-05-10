# Promise — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: promise-surface-property
  threshold: PROM1
  interface: [Promise.all, Promise, Promise.all, Promise.race]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 4. Total witnessing constraint clauses: 119.

## PROM1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Promise.all** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 75)

Witnessed by 75 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/25869.test.ts:77` — user-event style wait pattern does not hang → `expect(result).toEqual(["timeout", "advanced"])`
- `test/regression/issue/20875.test.ts:284` — gRPC streaming calls > rapid successive streaming calls → `expect(results[i][2]).toEqual({ value: 'batch${i}', value2: i })`
- `test/regression/issue/14477/14477.test.ts:22` — JSXElement with mismatched closing tags produces a syntax error → `expect(exited).toEqual(Array.from({ length: fixtures.length }, () => 1))`

## PROM2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Promise** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 18)

Witnessed by 18 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/workers/worker_blob.test.ts:25` — Worker from a Blob → `expect(result).toBe("hello")`
- `test/js/web/timers/setTimeout.test.js:34` — setTimeout → `expect(result[j]).toBe(j)`
- `test/js/web/timers/setInterval.test.js:30` — setInterval → `expect(result).toBe(10)`

## PROM3
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Promise.all** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 18)

Witnessed by 18 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/20875.test.ts:281` — gRPC streaming calls > rapid successive streaming calls → `expect(results).toHaveLength(10)`
- `test/cli/install/migration/pnpm-migration.test.ts:139` — folder dependencies > links to the root package are resolved correctly → `expect( await Promise.all([ file(join(packageDir, "node_modules", "two-range-deps", "package.json")).json(), file(join(packageDir, "node_modules", "no-deps", "package.json")).json(), ]), ).toMatchInli…`
- `test/cli/install/bun-workspaces.test.ts:430` — workspace aliases > combination → `expect(files).toMatchObject([ { name: "@org/a" }, { name: "@org/b" }, { name: "c" }, { name: "c" }, { name: "@org/a" }, { name: "@org/a" }, ])`

## PROM4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Promise.race** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/timers/setImmediate.test.js:80` — setImmediate should not keep the process alive forever → `expect(await Promise.race([success(), fail()])).toBe(true)`
- `test/js/web/streams/streams.test.js:770` — ReadableStream errors the stream on pull rejection → `expect(await Promise.race([closed, read])).toBe("closed: pull rejected")`
- `test/js/sql/sql.test.ts:665` — PostgreSQL tests > Minimal reproduction of Bun.SQL PostgreSQL hang bug (#22395) → `expect(result[0].count).toBe("1")`

