# fs — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: fs-surface-property
  threshold: FS1
  interface: [fs.openSync]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 6.

## FS1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.openSync** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/_fs/_fs_raw_fd_test.ts:56` — [node/fs] openSync returns a real OS fd (not a small RID) → `assertEquals(typeof fd, "number")`

## FS2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fs.readSync** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/_fs/_fs_raw_fd_test.ts:80` — [node/fs] readSync with position reads at offset without moving cursor → `assertEquals(n1, 3)`
- `tests/unit_node/_fs/_fs_raw_fd_test.ts:87` — [node/fs] readSync with position reads at offset without moving cursor → `assertEquals(n2, 3)`
- `tests/unit_node/_fs/_fs_raw_fd_test.ts:129` — [node/fs] async read with position does not move cursor → `assertEquals(n2, 3)`

