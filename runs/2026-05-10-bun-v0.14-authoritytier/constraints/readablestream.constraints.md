# ReadableStream — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: readablestream-surface-property
  threshold: READ1
  interface: [ReadableStream, ReadableStream, ReadableStream.prototype.cancel, ReadableStream.prototype.getReader]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 4; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 5.

## READ1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ReadableStream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/streams/streams.test.js:472` — exists globally → `expect(typeof ReadableStream).toBe("function")`
- `streams.spec.md:8` — ReadableStream is exposed as a global constructor → `new ReadableStream() returns a default-constructed ReadableStream`

## READ2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**ReadableStream** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `streams.spec.md:7` — ReadableStream is exposed as a global constructor → `ReadableStream is defined as a global constructor in any execution context with [Exposed=*]`

## READ3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ReadableStream.prototype.cancel** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `streams.spec.md:19` — ReadableStream.prototype.cancel method → `ReadableStream.prototype.cancel rejects with TypeError when the stream is locked`

## READ4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ReadableStream.prototype.getReader** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `streams.spec.md:23` — ReadableStream.prototype.getReader method → `ReadableStream.prototype.getReader throws TypeError when the stream is already locked`

