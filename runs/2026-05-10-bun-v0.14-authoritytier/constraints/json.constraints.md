# JSON — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: json-surface-property
  threshold: JSON1
  interface: [JSON.parse, JSON.stringify, JSON.parse, JSON.parse, JSON.parse, JSON.stringify, JSON.parse]

@imports: []

@pins: []

Surface drawn from 7 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 7. Total witnessing constraint clauses: 357.

## JSON1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**JSON.parse** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 210)

Witnessed by 210 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/test-21049.test.ts:77` — fetch with Request object respects redirect: 'manual' option → `expect(bunResult).toEqual({ status: 302, url: '${server.url}/redirect', redirected: false, location: "/target", })`
- `test/regression/issue/28014.test.ts:94` — WebSocket.protocol should not mutate after receiving frames → `expect(result.openProtocol).toBe(PROTOCOL)`
- `test/regression/issue/26657.test.ts:48` — bun update -i select all with 'A' key > should update packages when 'A' is pressed to sele… → `expect(initialPackageJson.dependencies["is-even"]).toBe("0.1.0")`

## JSON2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**JSON.stringify** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 53)

Witnessed by 53 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/tui-app-tty-pattern.test.ts:104` — TUI app pattern: read piped stdin then reopen /dev/tty → `expect(jsonOutput).toBe(expected)`
- `test/js/web/streams/streams.test.js:238` — WritableStream > works → `expect(JSON.stringify(Array.from(Buffer.concat(chunks)))).toBe(JSON.stringify([1, 2, 3, 4, 5, 6]))`
- `test/js/web/html/URLSearchParams.test.ts:154` — URLSearchParams > non-standard extensions > should support .toJSON → `expect(JSON.stringify(params)).toBe("{}")`

## JSON3
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**JSON.parse** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 40)

Witnessed by 40 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/22157.test.ts:52` — issue 22157: compiled binary should not include executable name in process.argv → `expect(processArgv).toHaveLength(2)`
- `test/js/bun/resolve/require.test.ts:70` — require(specifier) > require.main > is a Module object when a file is run directly → `expect(main).toMatchObject({ id: ".", filename: file, path: expect.any(String), exports: {}, children: [], paths: expect.any(Array), })`
- `test/cli/watch/watcher-trace.test.ts:58` — BUN_WATCHER_TRACE creates trace file with watch events → `expect(event).toHaveProperty("timestamp")`

## JSON4
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**JSON.parse** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 27)

Witnessed by 27 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/streams/transform-stream-leak.test.ts:56` — dropped TransformStream is collectable → `expect(counts.WritableStream).toBeLessThan(50)`
- `test/js/web/html/FormData-file-error-leak.test.ts:50` — FormData serialization does not leak prior file buffers when a later file read fails → `expect(result.growthMB).toBeLessThan(10)`
- `test/js/web/fetch/fetch-proxy-tls-intern-race.test.ts:104` — SSLConfig intern/deref race does not cause use-after-free → `expect(result.driverOk).toBeGreaterThan(0)`

## JSON5
type: specification
authority: derived
scope: module
status: active
depends-on: []

**JSON.parse** — is defined and resolves to a non-nullish value at the documented call site. (behavioral; cardinality 12)

Witnessed by 12 constraint clauses across 4 test files. Antichain representatives:

- `test/js/bun/util/v8-heap-snapshot.test.ts:57` — v8 heap snapshot arraybuffer → `expect(parsed.snapshot).toBeDefined()`
- `test/js/bun/shell/bunshell.test.ts:751` — bunshell > variables > shell var → `expect(procEnv.FOO).toBeUndefined()`
- `test/cli/install/bun-pm-pkg.test.ts:376` — bun pm pkg > delete command > should delete nested properties → `expect(scripts.test).toBeUndefined()`

## JSON6
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**JSON.stringify** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 5 test files. Antichain representatives:

- `test/js/valkey/reliability/protocol-handling.test.ts:62` — RESP3 Data Type Handling > should handle RESP3 Set type → `expect(JSON.stringify(setResult)).toMatchInlineSnapshot( '"["member1","member2","42","","special \\r\\n character"]"', )`
- `test/js/bun/util/cookie.test.js:232` — Bun.CookieMap > toJSON → `expect(JSON.stringify(cookieMap, null, 2)).toMatchInlineSnapshot(' "{ "name": "value", "foo": "bar" }" ')`
- `test/js/bun/http/bun-serve-html.test.ts:201` — serve html → `expect(JSON.stringify(sourceMap, null, 2)).toMatchInlineSnapshot(' "{ "version": 3, "sources": [ "script.js", "dashboard.js" ], "sourcesContent": [ "let count = 0;\\n const button = document.getElemen…`

## JSON7
type: specification
authority: derived
scope: module
status: active
depends-on: []

**JSON.parse** — exposes values of the expected type or class. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/util/v8-heap-snapshot.test.ts:59` — v8 heap snapshot arraybuffer → `expect(parsed.nodes).toBeInstanceOf(Array)`
- `test/bundler/bun-build-compile-sourcemap.test.ts:124` — Bun.build compile with sourcemap > compile with sourcemap: external writes .map file to di… → `expect(mapContent.sources).toBeArray()`
- `test/js/bun/util/v8-heap-snapshot.test.ts:60` — v8 heap snapshot arraybuffer → `expect(parsed.edges).toBeInstanceOf(Array)`

