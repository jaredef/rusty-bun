# net — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: net-surface-property
  threshold: NET1
  interface: [net.Server, net.Server.__proto__, net.Server.prototype.__proto__, net.connect, net.connect]

@imports: []

@pins: []

Surface drawn from 5 candidate properties across the Bun test corpus. Construction-style: 5; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 5.

## NET1
type: specification
authority: derived
scope: module
status: active
depends-on: []

**net.Server** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/net/server.spec.ts:46` — net.Server > is callable → `expect(net.Server()).toBeInstanceOf(net.Server)`

## NET2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**net.Server.__proto__** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/net/server.spec.ts:42` — net.Server > extends EventEmitter → `expect(net.Server.__proto__).toBe(EventEmitter)`

## NET3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**net.Server.prototype.__proto__** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/net/server.spec.ts:82` — net.Server.prototype > has EventEmitter methods → `expect(net.Server.prototype.__proto__).toBe(EventEmitter.prototype)`

## NET4
type: specification
authority: derived
scope: module
status: active
depends-on: []

**net.connect** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24575.test.ts:32` — socket._handle.fd should be accessible on TCP sockets → `expect(client._handle.fd).toBeTypeOf("number")`

## NET5
type: specification
authority: derived
scope: module
status: active
depends-on: []

**net.connect** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24575.test.ts:31` — socket._handle.fd should be accessible on TCP sockets → `expect(client._handle).toBeDefined()`

