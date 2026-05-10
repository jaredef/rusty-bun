# platform-cfg/@regression — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: platform-cfg-regression-surface-property
  threshold: PLAT1
  interface: [tty.ReadStream]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 54.

## PLAT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**tty.ReadStream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/tui-app-tty-pattern.test.ts:126` — tty.ReadStream handles non-TTY file descriptors correctly → `expect(stream.isTTY).toBe(false)`
- `test/regression/issue/tty-readstream-ref-unref.test.ts:24` — tty.ReadStream should have ref/unref methods when opened on /dev/tty → `expect(stream.isTTY).toBe(true)`
- `test/regression/issue/tui-app-tty-pattern.test.ts:129` — tty.ReadStream handles non-TTY file descriptors correctly → `expect(typeof stream.ref).toBe("function")`

## PLAT2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Buffer.concat** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 40)

Witnessed by 40 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/isArray-proxy-crash.test.ts:22` — isArray + Proxy crash fixes > Buffer.concat accepts empty Proxy-wrapped array → `expect(result.length).toBe(0)`
- `test/regression/issue/29073.test.ts:145` — http2.createServer serves h2c response with well-formed frames (#29073) → `expect(bodyBytes.toString("utf8")).toBe("ok")`
- `test/regression/issue/27272.test.ts:35` — slice(0, N).stream() should only return N bytes → `expect(Buffer.concat(chunks).toString()).toBe("Hello")`

## PLAT3
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**events.filter** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/3657.test.ts:48` — fs.watch on directory emits 'change' events for files created after watch starts → `expect(testFileEvents.length).toBeGreaterThanOrEqual(2)`
- `packages/bun-vscode/src/features/tests/__tests__/socket-integration.test.ts:612` — Socket Integration - Real Bun Process > real Bun test runner with socket communication → `expect(foundEvents.length).toBeGreaterThanOrEqual(9)`
- `test/regression/issue/3657.test.ts:101` — fs.watch emits multiple 'change' events for repeated modifications → `expect(testFileEvents.length).toBeGreaterThanOrEqual(4)`

