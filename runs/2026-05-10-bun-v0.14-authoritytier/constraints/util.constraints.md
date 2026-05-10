# util — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: util-surface-property
  threshold: UTIL1
  interface: [util.promisify, util.TextDecoder, util.TextEncoder, util.inherits]

@imports: []

@pins: []

Surface drawn from 8 candidate properties across the Bun test corpus. Construction-style: 4; behavioral (high-cardinality): 4. Total witnessing constraint clauses: 576.

## UTIL1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**util.promisify** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/dns/node-dns.test.js:545` — uses `dns.promises` implementations for `util.promisify` factory > util.promisify(dns.look… → `expect(await util.promisify(dns.lookup)("google.com")).toEqual(await dns.promises.lookup("google.com"))`
- `test/js/node/crypto/node-crypto.test.js:32` — crypto.randomInt with a callback → `expect(typeof result).toBe("number")`

## UTIL2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**util.TextDecoder** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/util/util.test.js:330` — util > TextDecoder > is same as global TextDecoder → `expect(util.TextDecoder === globalThis.TextDecoder).toBe(true)`

## UTIL3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**util.TextEncoder** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/util/util.test.js:322` — util > TextEncoder > is same as global TextEncoder → `expect(util.TextEncoder === globalThis.TextEncoder).toBe(true)`

## UTIL4
type: specification
authority: derived
scope: module
status: active
depends-on: []

**util.inherits** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/util/util.test.js:67` — util > inherits → `expect(util.inherits(Wat, Bar)).toBeUndefined()`

## UTIL5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**util.inspect** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 351)

Witnessed by 351 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/broadcastchannel/broadcast-channel.test.ts:221` — user options are forwarded through custom inspect → `expect(util.inspect(bc, { compact: true, breakLength: 2 })).toBe( "BroadcastChannel { name:\n 'hello',\n active:\n true }", )`
- `test/js/node/util/node-inspect-tests/parallel/util-inspect.test.js:31` — no assertion failures → `assert.strictEqual(util.inspect(1), "1")`
- `test/js/node/util/node-inspect-tests/parallel/util-inspect-proxy.test.js:71` — no assertion failures → `assert.strictEqual(util.inspect(r.proxy), "<Revoked Proxy>")`

## UTIL6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**util.format** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 181)

Witnessed by 181 constraint clauses across 4 test files. Antichain representatives:

- `test/js/node/util/util.test.js:335` — util > format → `expect(util.format("%s:%s", "foo")).toBe("foo:%s")`
- `test/js/node/util/node-inspect-tests/parallel/util-inspect-proxy.test.js:77` — no assertion failures → `assert.strictEqual(util.format("%s", r.proxy), "<Revoked Proxy>")`
- `test/js/node/util/node-inspect-tests/parallel/util-format.test.js:27` — no assertion failures → `assert.strictEqual(util.format(), "")`

## UTIL7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**util.inspect** — satisfies the documented invariant. (behavioral; cardinality 33)

Witnessed by 33 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/util/node-inspect-tests/parallel/util-inspect.test.js:136` — no assertion failures → `assert.match(util.inspect({ a: { a: { a: { a: {} } } } }, undefined, undefined, true), /Object/)`
- `test/js/node/util/node-inspect-tests/internal-inspect.test.js:25` — no assertion failures → `assert.match(util.inspect(e2), /\[cause\]: Error: cause\n/)`
- `test/js/node/util/node-inspect-tests/parallel/util-inspect.test.js:137` — no assertion failures → `assert.doesNotMatch(util.inspect({ a: { a: { a: { a: {} } } } }, undefined, null, true), /Object/)`

## UTIL8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**util.formatWithOptions** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 3 test files. Antichain representatives:

- `test/js/node/util/util.test.js:338` — util > formatWithOptions → `expect(util.formatWithOptions({ colors: true }, "%s:%s", "foo")).toBe("foo:%s")`
- `test/js/node/util/node-inspect-tests/parallel/util-format.test.js:123` — no assertion failures → `assert.strictEqual( util.formatWithOptions({ numericSeparator: true }, "%i %d", 1180591620717411303424n, 12345678901234567890123n), "1_180_591_620_717_411_303_424n 12_345_678_901_234_567_890_123n", )`
- `test/js/node/util/node-inspect-tests/internal-inspect.test.js:16` — no assertion failures → `assert.strictEqual(util.formatWithOptions({ numericSeparator: true }, "%d", 4000), "4_000")`

