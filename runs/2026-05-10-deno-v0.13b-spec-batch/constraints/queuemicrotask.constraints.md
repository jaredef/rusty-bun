# queueMicrotask — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: queuemicrotask-surface-property
  threshold: QUEU1
  interface: [queueMicrotask, queueMicrotask, queueMicrotask]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 9.

## QUEU1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**queueMicrotask** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 2 test files. Antichain representatives:

- `tests/unit/timers_test.ts:531` —  → `assertEquals(typeof queueMicrotask, "function")`
- `queue-microtask.spec.md:9` — queueMicrotask is exposed as a global function → `queueMicrotask returns undefined`

## QUEU2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**queueMicrotask** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `queue-microtask.spec.md:17` — queueMicrotask error handling → `queueMicrotask throws TypeError when callback is not a function`

## QUEU3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**queueMicrotask** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `queue-microtask.spec.md:7` — queueMicrotask is exposed as a global function → `queueMicrotask is defined as a global function in any execution context with [Exposed=*]`

## QUEU4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**queueMicrotask** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `queue-microtask.spec.md:8` — queueMicrotask is exposed as a global function → `queueMicrotask(callback) schedules callback for invocation in the microtask queue`
- `queue-microtask.spec.md:12` — queueMicrotask scheduling semantics → `queueMicrotask runs callback before the next macrotask`
- `queue-microtask.spec.md:13` — queueMicrotask scheduling semantics → `queueMicrotask runs callbacks in FIFO order within the same microtask checkpoint`

