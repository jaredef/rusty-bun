# Array — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: array-surface-property
  threshold: ARRA1
  interface: [Array.isArray, Array.from, Array.from, Array.fromAsync, Array, Array.from]

@imports: []

@pins: []

Surface drawn from 6 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 6. Total witnessing constraint clauses: 225.

## ARRA1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Array.isArray** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 108)

Witnessed by 108 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/24850.test.ts:46` — CALL via prepared statement returns rows without leaking an error → `expect(Array.isArray(result)).toBe(true)`
- `test/regression/issue/22712.test.ts:9` — dns.resolve callback parameters match Node.js → `expect(Array.isArray(args[1])).toBe(true)`
- `test/js/valkey/valkey.test.ts:2263` — Set Operations > should pop random member with SPOP → `expect(Array.isArray(popped2)).toBe(true)`

## ARRA2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Array.from** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 68)

Witnessed by 68 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/s3-signature-performance.test.ts:47` — S3 presigned URL performance test with stack allocator → `expect(params).toEqual(sortedParams)`
- `test/regression/issue/s3-signature-order.test.ts:26` — S3 presigned URL should have correct query parameter order → `expect(params).toEqual(expected)`
- `test/regression/issue/27478.test.ts:30` — multipart formdata preserves null bytes in small binary files → `expect(Array.from(parsed)).toEqual(Array.from(source))`

## ARRA3
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Array.from** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 24)

Witnessed by 24 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/s3-signature-order.test.ts:29` — S3 presigned URL should have correct query parameter order → `expect(params).toContain("X-Amz-Algorithm")`
- `test/regression/issue/24007.test.ts:85` — issue #24007 - glob with recursive patterns > Bun.Glob recursive scan finds nested files → `expect(results).toContain(path.join("api", "health.get.ts"))`
- `test/js/web/html/URLSearchParams.test.ts:90` — URLSearchParams > does not crash when calling .toJSON() on a URLSearchParams object with a… → `expect(Array.from(params.keys())).toHaveLength(params.size)`

## ARRA4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Array.fromAsync** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 15)

Witnessed by 15 constraint clauses across 3 test files. Antichain representatives:

- `test/regression/issue/29298.test.ts:35` — standalone HTML inlines file-loader assets imported from JS as data URIs → `expect(distFiles).toEqual(["index.html"])`
- `test/js/bun/shell/bunshell-instance.test.ts:35` — $.lines → `expect(await Array.fromAsync(await $'echo hello'.lines())).toEqual(["hello", ""])`
- `test/js/bun/glob/scan.test.ts:570` — literal fast path > works → `expect(entries.sort()).toEqual( [ 'packages${path.sep}a${path.sep}package.json', 'packages${path.sep}b${path.sep}package.json', 'packages${path.sep}c${path.sep}package.json', ].sort(), )`

## ARRA5
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Array** — exposes values of the expected type or class. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/test/jest-extended.test.js:198` — jest-extended > toBeArray() → `expect(new Array()).toBeArray()`
- `test/js/bun/test/expect.test.js:3874` — expect() > toBeObject() → `expect(new Array(0)).toBeObject()`
- `test/js/bun/test/jest-extended.test.js:199` — jest-extended > toBeArray() → `expect(new Array(1, 2, 3)).toBeArray()`

## ARRA6
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Array.from** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/cli/run/glob-on-fuse.test.ts:66` — Bun.Glob.scanSync finds files on FUSE mount → `expect(results.length).toBeGreaterThanOrEqual(1)`
- `test/cli/heap-prof.test.ts:27` — --heap-prof generates V8 heap snapshot on exit → `expect(files.length).toBeGreaterThan(0)`
- `test/cli/heap-prof.test.ts:61` — --heap-prof-md generates markdown heap profile on exit → `expect(files.length).toBeGreaterThan(0)`

