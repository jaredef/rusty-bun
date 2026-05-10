# Promise — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: promise-surface-property
  threshold: PROM1
  interface: [Promise, Promise.withResolvers, Promise]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 3. Total witnessing constraint clauses: 55.

## PROM1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Promise** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 34)

Witnessed by 34 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit_node/zlib_test.ts:43` — brotli compression async → `assertEquals(compressed instanceof Buffer, true)`
- `tests/unit_node/tls_test.ts:82` — tls over js-backed duplex pair does not panic → `assertEquals(received, "hello from server")`
- `tests/unit_node/http2_test.ts:755` — [node/http2 client] connect with pre-created socket → `assertEquals(body, "ok")`

## PROM2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Promise.withResolvers** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 16)

Witnessed by 16 constraint clauses across 4 test files. Antichain representatives:

- `tests/unit_node/async_hooks_test.ts:79` —  → `assertEquals(await deferred.promise, { x: 1 })`
- `tests/unit/worker_test.ts:49` — worker terminate → `assertEquals(await deferred1.promise, "Hello World")`
- `tests/unit/streams_test.ts:206` —  → `assertEquals(await cancel.promise, "resource closed")`

## PROM3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Promise** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 3 test files. Antichain representatives:

- `tests/unit_node/_fs/_fs_writeFile_test.ts:180` — Path can be an URL → `assert(res === null)`
- `tests/unit_node/_fs/_fs_readFile_test.ts:28` — readFileSuccess → `assert(data instanceof Uint8Array)`
- `tests/unit_node/_fs/_fs_mkdir_test.ts:19` — [node/fs] mkdir → `assert(result)`

