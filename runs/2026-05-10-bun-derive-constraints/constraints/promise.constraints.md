# Promise — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: promise-surface-property
  threshold: PROM1
  interface: [Promise.all, Promise, Promise.all, Promise, Promise.race, Promise, Promise]

@imports: []

@pins: []

Surface drawn from 7 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 7. Total witnessing constraint clauses: 189.

## PROM1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Promise.all** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 82)

Witnessed by 82 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/25869.test.ts:77` — user-event style wait pattern does not hang → `expect(result).toEqual(["timeout", "advanced"])`
- `test/regression/issue/20875.test.ts:284` — gRPC streaming calls > rapid successive streaming calls → `expect(results[i][2]).toEqual({ value: 'batch${i}', value2: i })`
- `test/regression/issue/14477/14477.test.ts:22` — JSXElement with mismatched closing tags produces a syntax error → `expect(exited).toEqual(Array.from({ length: fixtures.length }, () => 1))`

## PROM2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Promise** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 58)

Witnessed by 58 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/26143.test.ts:50` — issue #26143 - https GET request with body hangs > http.request GET with body should compl… → `expect(result.status).toBe(200)`
- `test/regression/issue/25589-write-end.test.ts:67` — http2 write() + end() pattern should only send two DATA frames (local server) → `expect(result).toBe("OK")`
- `test/regression/issue/25589-frame-size-grpc.test.ts:193` — HTTP/2 FRAME_SIZE_ERROR with @grpc/grpc-js > receives large response headers without FRAME… → `assert.strictEqual(response.value, "test")`

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
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Promise** — is defined and resolves to a non-nullish value at the documented call site. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/fetch/fetch.tls.wildcard.test.ts:132` — TLS wildcard hostname verification > should accept valid single-label wildcard match (foo.… → `expect(result.error).toBeUndefined()`
- `test/js/node/tls/node-tls-connect.test.ts:362` — getCipher, getProtocol, getEphemeralKeyInfo, getSharedSigalgs, getSession, exportKeyingMat… → `expect(socket.getFinished()).toBeUndefined()`
- `test/js/node/fs/fs-mkdir.test.ts:265` — fs.mkdir - return values > returns undefined with recursive when no new folders are create… → `expect(result).toBeUndefined()`

## PROM5
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

## PROM6
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Promise** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 4 test files. Antichain representatives:

- `test/regression/issue/26143.test.ts:51` — issue #26143 - https GET request with body hangs > http.request GET with body should compl… → `expect(result.data).toContain('"received":"{}"')`
- `test/js/web/fetch/fetch.tls.wildcard.test.ts:97` — TLS wildcard hostname verification > should reject multi-label wildcard match (sub.foo.exa… → `expect(result.error?.message).toContain("Hostname/IP does not match")`
- `test/js/bun/test/snapshot-tests/snapshots/snapshot.test.ts:51` — most types → `expect(new Promise(() => {})).toMatchSnapshot("Promise")`

## PROM7
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Promise** — exposes values of the expected type or class. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/workers/worker_blob.test.ts:83` — Revoking an object URL after a Worker is created before it loads should throw an error → `expect(result).toBeInstanceOf(ErrorEvent)`
- `test/js/web/workers/worker-postmessage-transfer.test.ts:89` — self.postMessage transfer list > postMessage(msg, [MessagePort]) transfers the port → `expect(first).toBeInstanceOf(MessagePort)`
- `test/js/node/tls/node-tls-connect.test.ts:356` — getCipher, getProtocol, getEphemeralKeyInfo, getSharedSigalgs, getSession, exportKeyingMat… → `expect(socket.getSharedSigalgs()).toBeInstanceOf(Array)`

