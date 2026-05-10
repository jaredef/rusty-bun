# sync/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: sync-js-surface-property
  threshold: SYNC1
  interface: [Bun.spawnSync, fs.lstatSync, crypto.generatePrimeSync, fs.mkdirSync]

@imports: []

@pins: []

Surface drawn from 8 candidate properties across the Bun test corpus. Construction-style: 4; behavioral (high-cardinality): 4. Total witnessing constraint clauses: 64.

## SYNC1
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Bun.spawnSync** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 8 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/console/console-log.test.ts:47` — long arrays get cutoff → `expect(proc.stderr.toString("utf8")).toBeEmpty()`
- `test/js/sql/sql.test.ts:11874` — PostgreSQL tests > should proper handle connection errors > should not crash if connection… → `expect(result.stderr?.toString()).toBeFalsy()`
- `test/js/bun/spawn/spawnsync-no-microtask-drain.test.ts:87` — spawnSync with timeout still works → `expect(result.exitCode).toBeNull()`

## SYNC2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.lstatSync** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/cp.test.ts:156` — symlinks - single file → `expect(stats.isSymbolicLink()).toBe(true)`
- `test/js/node/fs/cp.test.ts:160` — symlinks - single file → `expect(stats2.isSymbolicLink()).toBe(true)`
- `test/js/node/fs/cp.test.ts:174` — symlinks - single file recursive → `expect(stats.isSymbolicLink()).toBe(true)`

## SYNC3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto.generatePrimeSync** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:817` — generatePrime(Sync) should return an ArrayBuffer → `expect(prime).toBeInstanceOf(ArrayBuffer)`
- `test/js/node/crypto/node-crypto.test.js:823` — generatePrime(Sync) should return an ArrayBuffer → `expect(prime).toBeInstanceOf(ArrayBuffer)`

## SYNC4
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fs.mkdirSync** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs-mkdir.test.ts:291` — fs.mkdir - return values > mkdirSync returns undefined with recursive when no new folders … → `expect(result).toBeUndefined()`

## SYNC5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.spawnSync** — exhibits the property captured in the witnessing test. (behavioral; cardinality 15)

Witnessed by 15 constraint clauses across 4 test files. Antichain representatives:

- `test/js/node/child_process/child_process_ipc_large_disconnect.test.js:9` — child_process_ipc_large_disconnect → `expect(actual.stdout.toString()).toStartWith('2: a\n2: b\n2: c\n2: d\n')`
- `test/js/bun/typescript/type-export.test.ts:185` — js file type export → `expect(result.stderr?.toString().trim()).toInclude('error: "not_found" is not declared in this file')`
- `test/js/bun/spawn/spawn-signal.test.ts:66` — spawnSync AbortSignal works as timeout → `expect(subprocess.success).toBeFalse()`

## SYNC6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**sinon.fake** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/test/fake-timers/sinonjs/issue-59.test.ts:16` — issue #59 > should install and uninstall the clock on a custom target → `assert.equals(setTimeoutFake.callCount, 1)`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:2499` — FakeTimers > runAllAsync > should run micro-tasks scheduled between timers → `assert.equals(fake.args[0][0], 1)`
- `test/js/bun/test/fake-timers/sinonjs/issue-59.test.ts:18` — issue #59 > should install and uninstall the clock on a custom target → `assert.equals(setTimeoutFake.callCount, 1)`

## SYNC7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**this.clock.now** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:837` — FakeTimers > tick > treats missing argument as 0 → `assert.equals(this.clock.now, 0)`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:873` — FakeTimers > tick > returns the current now value → `assert.equals(clock.now, value)`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:1385` — FakeTimers > tickAsync > treats missing argument as 0 → `assert.equals(clock.now, 0)`

## SYNC8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.resolveSync** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/resolve/resolve.test.ts:508` — wildcard exports with @ in matched subpath > resolves a subpath whose wildcard match start… → `expect(Bun.resolveSync("test-pkg/plain/index.js", root)).toBe( join(root, "node_modules/test-pkg/dist/packages/plain/index.js"), )`
- `test/js/bun/resolve/resolve.test.ts:511` — wildcard exports with @ in matched subpath > resolves a subpath whose wildcard match start… → `expect(Bun.resolveSync("test-pkg/@scope/sub/index.js", root)).toBe( join(root, "node_modules/test-pkg/dist/packages/@scope/sub/index.js"), )`
- `test/js/bun/resolve/resolve.test.ts:528` — wildcard exports with @ in matched subpath > resolves a subpath that contains `@` mid-segm… → `expect(Bun.resolveSync("test-pkg/with@sign/sub/index.js", root)).toBe( join(root, "node_modules/test-pkg/dist/packages/with@sign/sub/index.js"), )`

