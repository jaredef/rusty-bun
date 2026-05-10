# platform-cfg/threaded/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: platform-cfg-threaded-js-surface-property
  threshold: PLAT1
  interface: [util.inspect, Promise]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 360.

## PLAT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**util.inspect** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 355)

Witnessed by 355 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/broadcastchannel/broadcast-channel.test.ts:221` — user options are forwarded through custom inspect → `expect(util.inspect(bc, { compact: true, breakLength: 2 })).toBe( "BroadcastChannel { name:\n 'hello',\n active:\n true }", )`
- `test/js/node/util/node-inspect-tests/parallel/util-inspect.test.js:31` — no assertion failures → `assert.strictEqual(util.inspect(1), "1")`
- `test/js/node/util/node-inspect-tests/parallel/util-inspect-proxy.test.js:71` — no assertion failures → `assert.strictEqual(util.inspect(r.proxy), "<Revoked Proxy>")`

## PLAT2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Promise** — exposes values of the expected type or class. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/workers/worker_blob.test.ts:83` — Revoking an object URL after a Worker is created before it loads should throw an error → `expect(result).toBeInstanceOf(ErrorEvent)`
- `test/js/web/workers/worker-postmessage-transfer.test.ts:89` — self.postMessage transfer list > postMessage(msg, [MessagePort]) transfers the port → `expect(first).toBeInstanceOf(MessagePort)`
- `test/js/node/tls/node-tls-connect.test.ts:356` — getCipher, getProtocol, getEphemeralKeyInfo, getSharedSigalgs, getSession, exportKeyingMat… → `expect(socket.getSharedSigalgs()).toBeInstanceOf(Array)`

