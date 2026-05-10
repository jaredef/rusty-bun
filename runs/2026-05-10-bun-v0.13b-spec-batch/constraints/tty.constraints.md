# tty — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: tty-surface-property
  threshold: TTY1
  interface: [tty.ReadStream, tty.WriteStream]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 2; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 7.

## TTY1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**tty.ReadStream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/tui-app-tty-pattern.test.ts:126` — tty.ReadStream handles non-TTY file descriptors correctly → `expect(stream.isTTY).toBe(false)`
- `test/regression/issue/tty-readstream-ref-unref.test.ts:24` — tty.ReadStream should have ref/unref methods when opened on /dev/tty → `expect(stream.isTTY).toBe(true)`
- `test/regression/issue/tui-app-tty-pattern.test.ts:129` — tty.ReadStream handles non-TTY file descriptors correctly → `expect(typeof stream.ref).toBe("function")`

## TTY2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**tty.WriteStream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/test-process-stdout-async-iterator.test.ts:28` — tty.WriteStream has Symbol.asyncIterator → `expect(typeof stream[Symbol.asyncIterator]).toBe("function")`

