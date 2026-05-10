# http2 — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: http2-surface-property
  threshold: HTTP1
  interface: [http2.connect, http2.connect]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 2; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 19.

## HTTP1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**http2.connect** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/http2/node-http2.test.js:599` — ${path.basename(nodeExecutable)} ${paddingStrategyName(paddingStrategy)} > Client Basics >… → `expect(client.destroyed).toBe(true)`
- `test/js/node/http2/node-http2.test.js:620` — ${path.basename(nodeExecutable)} ${paddingStrategyName(paddingStrategy)} > Client Basics >… → `expect(client.destroyed).toBe(true)`
- `test/js/node/http2/node-http2.test.js:690` — ${path.basename(nodeExecutable)} ${paddingStrategyName(paddingStrategy)} > Client Basics >… → `expect(client.destroyed).toBe(true)`

## HTTP2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**http2.connect** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/http2/node-http2.test.js:818` — ${path.basename(nodeExecutable)} ${paddingStrategyName(paddingStrategy)} > Client Basics >… → `expect(client.alpnProtocol).toBeUndefined()`
- `test/js/node/http2/node-http2.test.js:825` — ${path.basename(nodeExecutable)} ${paddingStrategyName(paddingStrategy)} > Client Basics >… → `expect(client.remoteSettings).toBeNull()`
- `test/js/node/http2/node-http2.test.js:1698` — http2 server handles multiple concurrent requests → `expect(client.encrypted).toBeFalsy()`

## HTTP3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**http2.connect** — exhibits the property captured in the witnessing test. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/http2/node-http2.test.js:817` — ${path.basename(nodeExecutable)} ${paddingStrategyName(paddingStrategy)} > Client Basics >… → `expect(client.connecting).toBeTrue()`
- `test/js/node/http2/node-http2.test.js:819` — ${path.basename(nodeExecutable)} ${paddingStrategyName(paddingStrategy)} > Client Basics >… → `expect(client.encrypted).toBeTrue()`
- `test/js/node/http2/node-http2.test.js:820` — ${path.basename(nodeExecutable)} ${paddingStrategyName(paddingStrategy)} > Client Basics >… → `expect(client.closed).toBeFalse()`

