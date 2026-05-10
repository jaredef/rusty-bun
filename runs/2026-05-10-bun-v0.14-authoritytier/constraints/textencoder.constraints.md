# TextEncoder — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: textencoder-surface-property
  threshold: TEXT1
  interface: [TextEncoder, TextEncoder]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 2; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 8.

## TEXT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**TextEncoder** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/web-globals.test.js:18` — exists → `expect(typeof TextEncoder !== "undefined").toBe(true)`
- `text-encoder.spec.md:8` — TextEncoder is exposed as a global constructor → `new TextEncoder() returns a TextEncoder instance`

## TEXT2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**TextEncoder** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `text-encoder.spec.md:7` — TextEncoder is exposed as a global constructor → `TextEncoder is defined as a global constructor in any execution context with [Exposed=*]`

## TEXT3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**TextEncoder.prototype.encode** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `text-encoder.spec.md:16` — TextEncoder.prototype.encode method → `TextEncoder.prototype.encode is a method that returns a Uint8Array`
- `text-encoder.spec.md:17` — TextEncoder.prototype.encode method → `TextEncoder.prototype.encode(input) returns the UTF-8 byte encoding of input`
- `text-encoder.spec.md:18` — TextEncoder.prototype.encode method → `TextEncoder.prototype.encode() with no argument returns Uint8Array of length 0`

