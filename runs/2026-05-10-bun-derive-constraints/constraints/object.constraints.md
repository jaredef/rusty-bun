# Object — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: object-surface-property
  threshold: OBJE1
  interface: [Object.keys, Object.getOwnPropertyDescriptor, Object.keys, Object.is, Object.getPrototypeOf, Object.prototype.hasOwnProperty.call, Object.keys, Object.assign]

@imports: []

@pins: []

Surface drawn from 10 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 10. Total witnessing constraint clauses: 122.

## OBJE1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Object.keys** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 39)

Witnessed by 39 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/structured-clone-fastpath.test.ts:471` — Structured Clone Fast Path > objects with many properties exceeding maxInlineCapacity → `expect(Object.keys(cloned[0]).length).toBe(100)`
- `test/js/valkey/valkey.test.ts:5824` — Hash Operations > should get random fields with values using hrandfield WITHVALUES → `expect(Object.keys(obj).length).toBe(2)`
- `test/js/third_party/jsonwebtoken/verify.test.js:96` — verify > should not mutate options → `expect(Object.keys(options).length).toEqual(1)`

## OBJE2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Object.getOwnPropertyDescriptor** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 19)

Witnessed by 19 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/29169.test.ts:35` — process.ppid is a live accessor (#29169) → `expect(typeof before!.get).toBe("function")`
- `test/regression/issue/246-child_process_object_assign_compatibility.test.ts:19` — child process stdio properties should be enumerable for Object.assign() → `expect(Object.getOwnPropertyDescriptor(child, key)?.enumerable).toBe(true)`
- `test/js/web/html/URLSearchParams.test.ts:218` — size property should be configurable (issue #9251) → `expect(descriptor!.configurable).toBe(true)`

## OBJE3
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Object.keys** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/246-child_process_object_assign_compatibility.test.ts:12` — child process stdio properties should be enumerable for Object.assign() → `expect(Object.keys(child)).toContain("stdin")`
- `test/regression/issue/09469.test.ts:23` — 09469 → `expect(Object.keys(ret)).toHaveLength(2)`
- `test/js/sql/sqlite-sql.test.ts:1912` — Performance & Edge Cases > handles many columns → `expect(Object.keys(result[0])).toHaveLength(columnCount)`

## OBJE4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Object.is** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 12)

Witnessed by 12 constraint clauses across 4 test files. Antichain representatives:

- `test/js/web/structured-clone-fastpath.test.ts:117` — Structured Clone Fast Path > structuredClone should work with array of special numbers → `expect(Object.is(cloned[0], -0)).toBe(true)`
- `test/js/bun/yaml/yaml.test.ts:1619` — Bun.YAML > stringify > edge cases > handles special number formats → `expect(Object.is(YAML.parse(YAML.stringify(-0)), -0)).toBe(true)`
- `test/js/bun/jsonl/jsonl-parse.test.ts:1127` — Bun.JSONL > fuzz-like stress tests > number edge cases > negative zero → `expect(Object.is(result[0], -0)).toBe(true)`

## OBJE5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Object.getPrototypeOf** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/29225.test.ts:132` — instanceof and prototype identity still work → `expect(Object.getPrototypeOf(reader)).toBe(ReadableStreamBYOBReader.prototype)`
- `test/js/node/process/call-constructor.test.js:6` — the constructor of process can be called → `expect(Object.getPrototypeOf(obj)).toEqual(Object.getPrototypeOf(process))`
- `test/js/node/fs/fs-stats-constructor.test.ts:49` — Stats instances share Stats.prototype → `expect(Object.getPrototypeOf(fromSync)).toBe(Stats.prototype)`

## OBJE6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Object.prototype.hasOwnProperty.call** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/26284.test.ts:24` — hasOwnProperty('clock') returns false before useFakeTimers → `expect(Object.prototype.hasOwnProperty.call(globalThis.setTimeout, "clock")).toBe(false)`
- `test/regression/issue/25869.test.ts:22` — setTimeout.clock is not set before useFakeTimers → `expect(Object.prototype.hasOwnProperty.call(globalThis.setTimeout, "clock")).toBe(false)`
- `test/regression/issue/26284.test.ts:30` — hasOwnProperty('clock') returns true after useFakeTimers → `expect(Object.prototype.hasOwnProperty.call(globalThis.setTimeout, "clock")).toBe(true)`

## OBJE7
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

## OBJE8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Object.assign** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/246-child_process_object_assign_compatibility.test.ts:58` — tinyspawn-like library usage should work → `expect(subprocess instanceof Promise).toBe(true)`
- `test/js/web/html/FormData.test.ts:178` — FormData > should parse multipart/form-data (${name}) with ${C.name} → `expect(expected[key] instanceof Blob).toBe(true)`
- `test/regression/issue/246-child_process_object_assign_compatibility.test.ts:61` — tinyspawn-like library usage should work → `expect(typeof subprocess.stdout.pipe).toBe("function")`

## OBJE9
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Object.isFrozen** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 4 test files. Antichain representatives:

- `test/js/web/structured-clone-fastpath.test.ts:227` — Structured Clone Fast Path > structuredClone of frozen array should produce a non-frozen c… → `expect(Object.isFrozen(cloned)).toBe(false)`
- `test/js/node/tls/node-tls-rootcertificates-immutable.test.ts:65` — tls > rootCertificates should be immutable → `expect(Object.isFrozen(tls.rootCertificates)).toBe(true)`
- `test/js/bun/jsonl/jsonl-parse.test.ts:1952` — Bun.JSONL > fuzz-like stress tests > adversarial input > parse result objects are not froz… → `expect(Object.isFrozen(result)).toBe(false)`

## OBJE10
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Object.keys** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/util/node-inspect-tests/parallel/util-inspect.test.js:2157` — no assertion failures 3 → `assert(colors.includes("gray"), colors)`
- `test/js/node/http2/node-http2-continuation.test.ts:389` — HTTP/2 CONTINUATION frames - Server Side > server sends 120 response headers via CONTINUAT… → `assert.ok( responseHeaderCount >= 100, 'Should receive at least 100 response headers (got ${responseHeaderCount})', )`
- `test/js/node/util/node-inspect-tests/parallel/util-inspect.test.js:2961` — no assertion failures 3 → `assert.deepStrictEqual(colors.gray, [0, 0])`

