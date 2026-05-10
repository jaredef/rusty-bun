# WritableStream — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: writablestream-surface-property
  threshold: WRIT1
  interface: [WritableStream, WritableStream, WritableStream.prototype.close, WritableStream.prototype.getWriter]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 4; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 4.

## WRIT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**WritableStream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/streams/streams.test.js:479` — exists globally → `expect(typeof WritableStream).toBe("function")`

## WRIT2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**WritableStream** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `streams.spec.md:57` — WritableStream is exposed as a global constructor → `WritableStream is defined as a global constructor in any execution context with [Exposed=*]`

## WRIT3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**WritableStream.prototype.close** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `streams.spec.md:71` — WritableStream.prototype.close method → `WritableStream.prototype.close throws TypeError when the stream is locked`

## WRIT4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**WritableStream.prototype.getWriter** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `streams.spec.md:75` — WritableStream.prototype.getWriter method → `WritableStream.prototype.getWriter throws TypeError when the stream is already locked`

