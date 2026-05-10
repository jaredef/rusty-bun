# Error — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: error-surface-property
  threshold: ERRO1
  interface: [Error, Error.captureStackTrace, Error.prepareStackTrace, Error]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 4. Total witnessing constraint clauses: 56.

## ERRO1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Error** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 32)

Witnessed by 32 constraint clauses across 5 test files. Antichain representatives:

- `test/js/third_party/socket.io/socket.io-middleware.test.ts:105` — middleware > should pass an object → `expect(err.message).toBe("Authentication error")`
- `test/js/node/v8/capture-stack-trace.test.js:579` — Error.prepareStackTrace returns a CallSite object → `expect(error.stack[0][Symbol.toStringTag]).toBe("CallSite")`
- `test/js/node/util/util-promisify.test.js:280` — util.promisify > callback cases > should also throw error inside Promise.all → `assert.strictEqual(err, e)`

## ERRO2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Error.captureStackTrace** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/v8/capture-stack-trace.test.js:288` — capture stack trace edge cases → `expect(Error.captureStackTrace({})).toBe(undefined)`
- `test/js/node/v8/capture-stack-trace.test.js:289` — capture stack trace edge cases → `expect(Error.captureStackTrace({}, () => {})).toBe(undefined)`
- `test/js/node/v8/capture-stack-trace.test.js:290` — capture stack trace edge cases → `expect(Error.captureStackTrace({}, undefined)).toBe(undefined)`

## ERRO3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Error.prepareStackTrace** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/prepare-stack-trace-crash.test.ts:16` — Error.prepareStackTrace should not crash when stacktrace parameter is not an array → `expect(typeof result).toBe("string")`
- `test/js/node/v8/capture-stack-trace.test.js:669` — Error.prepareStackTrace on an array with non-CallSite objects doesn't crash → `expect(result).toBe("Error: ok\n at [object Object]\n at [object Object]\n at [object Object]")`
- `test/regression/issue/prepare-stack-trace-crash.test.ts:21` — Error.prepareStackTrace should not crash when stacktrace parameter is not an array → `expect(typeof result).toBe("string")`

## ERRO4
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Error** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/v8/capture-stack-trace.test.js:809` — Error.captureStackTrace includes async frames from the await chain → `expect(err.stack).toContain("at innerAsync")`
- `test/js/bun/test/snapshot-tests/snapshots/snapshot.test.ts:38` — most types → `expect(new Error("hello")).toMatchSnapshot("Error")`
- `test/js/node/v8/capture-stack-trace.test.js:810` — Error.captureStackTrace includes async frames from the await chain → `expect(err.stack).toContain("at async outerAsync")`

