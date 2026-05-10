# sync/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: sync-js-surface-property
  threshold: SYNC1
  interface: [crypto.generatePrimeSync, fs.lstatSync, fs.mkdirSync]

@imports: []

@pins: []

Surface drawn from 10 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 7. Total witnessing constraint clauses: 138.

## SYNC1
type: specification
authority: derived
scope: module
status: active
depends-on: []

**crypto.generatePrimeSync** — exposes values of the expected type or class. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/crypto/node-crypto.test.js:817` — generatePrime(Sync) should return an ArrayBuffer → `expect(prime).toBeInstanceOf(ArrayBuffer)`
- `test/js/node/crypto/node-crypto.test.js:823` — generatePrime(Sync) should return an ArrayBuffer → `expect(prime).toBeInstanceOf(ArrayBuffer)`

## SYNC2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.lstatSync** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/cp.test.ts:247` — symlinks - copied link target is the original target, not the source link path → `expect(fs.lstatSync(copiedLink).isSymbolicLink()).toBe(true)`

## SYNC3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**fs.mkdirSync** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs-mkdir.test.ts:291` — fs.mkdir - return values > mkdirSync returns undefined with recursive when no new folders … → `expect(result).toBeUndefined()`

## SYNC4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**buf.toString** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 43)

Witnessed by 43 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:3885` — numeric flags produce same result as string flags > numeric O_CREAT|O_RDWR|O_TRUNC is equi… → `expect(buf.toString("utf8", 0, bytesRead)).toBe("read-write")`
- `test/js/node/buffer.test.js:388` — ASCII slice → `expect(buf.toString("ascii", 0, str.length)).toBe(str)`
- `test/js/node/fs/fs.test.ts:3968` — synchronous I/O string flags > 'as+' creates and appends with read access → `expect(buf.toString("utf8", 0, bytesRead)).toBe("hello")`

## SYNC5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**sinon.spy** — satisfies the documented invariant. (behavioral; cardinality 33)

Witnessed by 33 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:1126` — FakeTimers > tickAsync > triggers even when some throw → `assert(catchSpy.calledOnce)`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:1142` — FakeTimers > tickAsync > calls function with global object or null (strict mode) as this → `assert(catchSpy.calledOnce)`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:1347` — FakeTimers > tickAsync > throws for invalid format → `assert(catchSpy.calledOnce)`

## SYNC6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**buf.slice** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 22)

Witnessed by 22 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:3900` — numeric flags produce same result as string flags > numeric O_RDONLY reads existing file → `expect(buf.slice(0, bytesRead).toString("utf8")).toBe("existing content")`
- `test/js/node/buffer.test.js:397` — ASCII slice → `expect(slice1[i]).toBe(slice2[i])`
- `test/js/node/fs/fs.test.ts:3930` — synchronous I/O string flags > 'rs' opens existing file for reading → `expect(buf.slice(0, bytesRead).toString("utf8")).toBe("sync content")`

## SYNC7
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

## SYNC8
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

## SYNC9
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

## SYNC10
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**spy10.getCall** — satisfies the documented invariant. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:739` — FakeTimers > tick > fires timers in correct order → `assert(spy10.getCall(0).calledBefore(spy13.getCall(0)))`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:740` — FakeTimers > tick > fires timers in correct order → `assert(spy10.getCall(4).calledBefore(spy13.getCall(3)))`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:1231` — FakeTimers > tickAsync > fires timers in correct order → `assert(spy10.getCall(0).calledBefore(spy13.getCall(0)))`

