# structuredClone — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: structuredclone-surface-property
  threshold: STRU1
  interface: [structuredClone, structuredClone, structuredClone, structuredClone]

@imports: []

@pins: []

Surface drawn from 5 candidate properties across the Bun test corpus. Construction-style: 4; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 227.

## STRU1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**structuredClone** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 166 constraint clauses across 4 test files. Antichain representatives:

- `test/js/web/structured-clone-fastpath.test.ts:7` — Structured Clone Fast Path > structuredClone should work with empty object → `expect(cloned).toStrictEqual({})`
- `test/js/web/structured-clone-blob-file.test.ts:12` — structuredClone with Blob and File > Blob structured clone > empty Blob → `expect(cloned.size).toBe(0)`
- `test/js/bun/util/inspect.test.js:637` — empty Blob and File inspect as zero-byte, not detached → `expect(cloned.file.name).toBe("example.txt")`

## STRU2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**structuredClone** — exposes values of the expected type or class. (construction-style)

Witnessed by 39 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/structured-clone-fastpath.test.ts:305` — Structured Clone Fast Path > structuredClone of array with modified prototype → `expect(cloned).toBeInstanceOf(Array)`
- `test/js/web/structured-clone-blob-file.test.ts:11` — structuredClone with Blob and File > Blob structured clone > empty Blob → `expect(cloned).toBeInstanceOf(Blob)`
- `test/js/bun/util/inspect.test.js:636` — empty Blob and File inspect as zero-byte, not detached → `expect(cloned.file).toBeInstanceOf(File)`

## STRU3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**structuredClone** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/structured-clone-fastpath.test.ts:454` — Structured Clone Fast Path > objects with null and undefined property values → `expect(cloned[0].a).toBeNull()`
- `structured-clone.spec.md:7` — structuredClone is exposed as a global function → `structuredClone is defined as a global function in any execution context with [Exposed=*]`
- `test/js/web/structured-clone-fastpath.test.ts:455` — Structured Clone Fast Path > objects with null and undefined property values → `expect(cloned[0].b).toBeUndefined()`

## STRU4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**structuredClone** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `structured-clone.spec.md:23` — structuredClone unsupported types → `structuredClone throws DataCloneError on functions`
- `structured-clone.spec.md:24` — structuredClone unsupported types → `structuredClone throws DataCloneError on DOM nodes outside the supported list`
- `structured-clone.spec.md:25` — structuredClone unsupported types → `structuredClone throws DataCloneError on values containing non-cloneable references`

## STRU5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**structuredClone** — satisfies the documented invariant. (behavioral; cardinality 14)

Witnessed by 14 constraint clauses across 1 test files. Antichain representatives:

- `structured-clone.spec.md:9` — structuredClone is exposed as a global function → `structuredClone(value, options) accepts a StructuredSerializeOptions with transfer`
- `structured-clone.spec.md:12` — structuredClone supported types → `structuredClone clones primitives by value`
- `structured-clone.spec.md:13` — structuredClone supported types → `structuredClone clones Date with the same time value`

