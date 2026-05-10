# platform-cfg/@regression — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: platform-cfg-regression-surface-property
  threshold: PLAT1
  interface: [tty.ReadStream, Bun.RedisClient]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 2; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 23.

## PLAT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**tty.ReadStream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/tui-app-tty-pattern.test.ts:126` — tty.ReadStream handles non-TTY file descriptors correctly → `expect(stream.isTTY).toBe(false)`
- `test/regression/issue/tty-readstream-ref-unref.test.ts:24` — tty.ReadStream should have ref/unref methods when opened on /dev/tty → `expect(stream.isTTY).toBe(true)`
- `test/regression/issue/tui-app-tty-pattern.test.ts:129` — tty.ReadStream handles non-TTY file descriptors correctly → `expect(typeof stream.ref).toBe("function")`

## PLAT2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.RedisClient** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/29925.test.ts:67` — client.connect() recovers after the client enters the failed state → `expect(client.connected).toBe(true)`
- `test/regression/issue/29925.test.ts:86` — client.connect() recovers after the client enters the failed state → `expect(client.connected).toBe(true)`
- `test/regression/issue/29925.test.ts:111` — repeated close()/connect()/send() cycles do not lock up → `expect(client.connected).toBe(true)`

## PLAT3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**destStat.mode** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/25903.test.ts:28` — Bun.write() respects mode option when copying files via Bun.file() → `expect(destStat.mode & 0o777).toBe(0o600)`
- `test/regression/issue/25903.test.ts:48` — Bun.write() respects mode option with createPath when copying via Bun.file() → `expect(destStat.mode & 0o777).toBe(0o755)`
- `test/regression/issue/25903.test.ts:71` — Bun.write() uses default permissions when mode is not specified for Bun.file() copy → `expect(destStat.mode & 0o777).toBe(defaultMode)`

## PLAT4
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**events.filter** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/3657.test.ts:48` — fs.watch on directory emits 'change' events for files created after watch starts → `expect(testFileEvents.length).toBeGreaterThanOrEqual(2)`
- `packages/bun-vscode/src/features/tests/__tests__/socket-integration.test.ts:612` — Socket Integration - Real Bun Process > real Bun test runner with socket communication → `expect(foundEvents.length).toBeGreaterThanOrEqual(9)`
- `test/regression/issue/3657.test.ts:101` — fs.watch emits multiple 'change' events for repeated modifications → `expect(testFileEvents.length).toBeGreaterThanOrEqual(4)`

