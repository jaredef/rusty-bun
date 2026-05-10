# tls — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: tls-surface-property
  threshold: TLS1
  interface: [tls.connect, tls.TLSSocket, tls.createSecureContext, tls.TLSSocket, tls.checkServerIdentity, tls.connect, tls.connect, tls.rootCertificates, tls.rootCertificates, tls.rootCertificates]

@imports: []

@pins: []

Surface drawn from 10 candidate properties across the Bun test corpus. Construction-style: 10; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 14.

## TLS1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**tls.connect** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/25190.test.ts:42` — TLSSocket.isSessionReused > returns false for fresh connection without session reuse → `expect(socket.isSessionReused()).toBe(false)`
- `test/regression/issue/25190.test.ts:77` — TLSSocket.isSessionReused > returns true when session is successfully reused → `expect(socket1.isSessionReused()).toBe(false)`
- `test/regression/issue/24575.test.ts:144` — socket._handle.fd should be accessible on TLS sockets → `expect(typeof client._handle.fd).toBe("number")`

## TLS2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**tls.TLSSocket** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/25190.test.ts:111` — TLSSocket.isSessionReused > isSessionReused returns false when session not yet established → `expect(typeof socket.isSessionReused).toBe("function")`
- `test/regression/issue/25190.test.ts:113` — TLSSocket.isSessionReused > isSessionReused returns false when session not yet established → `expect(socket.isSessionReused()).toBe(false)`

## TLS3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**tls.createSecureContext** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/tls/tls-connect-socket-churn.test.ts:70` — createSecureContext memoises the native SSL_CTX (not the wrapper) by config → `expect(a.servername).toBeUndefined()`
- `test/js/node/tls/ssl-ctx-cache.test.ts:105` — SSL_CTX is freed once no owners remain (weak cache, not strong) → `expect(sc.context).toBeTruthy()`

## TLS4
type: specification
authority: derived
scope: module
status: active
depends-on: []

**tls.TLSSocket** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/tls/node-tls-connect.test.ts:144` — should be able to grab the JSStreamSocket constructor → `expect(socket._handle._parentWrap.constructor).toBeFunction()`

## TLS5
type: specification
authority: derived
scope: module
status: active
depends-on: []

**tls.checkServerIdentity** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/tls/node-tls-connect.test.ts:120` — should have checkServerIdentity → `expect(tls.checkServerIdentity).toBeFunction()`

## TLS6
type: specification
authority: derived
scope: module
status: active
depends-on: []

**tls.connect** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24575.test.ts:142` — socket._handle.fd should be accessible on TLS sockets → `expect(client._handle.fd).toBeTypeOf("number")`

## TLS7
type: specification
authority: derived
scope: module
status: active
depends-on: []

**tls.connect** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24575.test.ts:141` — socket._handle.fd should be accessible on TLS sockets → `expect(client._handle).toBeDefined()`

## TLS8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**tls.rootCertificates** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/tls/node-tls-server.test.ts:677` — tls.rootCertificates should exists → `expect(typeof tls.rootCertificates[0]).toBe("string")`

## TLS9
type: specification
authority: derived
scope: module
status: active
depends-on: []

**tls.rootCertificates** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/tls/node-tls-server.test.ts:675` — tls.rootCertificates should exists → `expect(tls.rootCertificates).toBeInstanceOf(Array)`

## TLS10
type: specification
authority: derived
scope: module
status: active
depends-on: []

**tls.rootCertificates** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/tls/node-tls-server.test.ts:674` — tls.rootCertificates should exists → `expect(tls.rootCertificates).toBeDefined()`

