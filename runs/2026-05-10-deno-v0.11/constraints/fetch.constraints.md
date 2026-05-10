# fetch — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: fetch-surface-property
  threshold: FETC1
  interface: [fetch, fetch]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 174.

## FETC1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fetch** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 146)

Witnessed by 146 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit_node/http_test.ts:189` — [node/http] .writeHead() → `assertEquals(res.status, 404)`
- `tests/unit_node/fetch_test.ts:14` — fetch node stream → `assertEquals( await response.text(), await Deno.readTextFile("tests/testdata/assets/fixture.json"), )`
- `tests/unit_node/async_hooks_test.ts:61` —  → `assertEquals(await res.text(), "Hello World")`

## FETC2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fetch** — satisfies the documented invariant. (behavioral; cardinality 28)

Witnessed by 28 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit_node/http_test.ts:148` — [node/http] chunked response → `assert(res.ok)`
- `tests/unit/worker_test.ts:734` — Worker with native HTTP → `assert(await response.bytes())`
- `tests/unit/serve_test.ts:1850` —  → `assert(resp.body)`

