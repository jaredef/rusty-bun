# process — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: process-surface-property
  threshold: PROC1
  interface: [process.execPath, process.exitCode, process.getgid, process.getuid, process.getBuiltinModule, process.geteuid, process.kill]

@imports: []

@pins: []

Surface drawn from 8 candidate properties across the Bun test corpus. Construction-style: 7; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 17.

## PROC1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.execPath** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/process_test.ts:942` — process.execPath → `assertEquals(typeof process.execPath, "string")`
- `tests/unit_node/process_test.ts:943` — process.execPath → `assertEquals(process.execPath, process.argv[0])`
- `tests/unit_node/process_test.ts:952` — process.execPath is writable → `assertEquals(process.execPath, "/path/to/node")`

## PROC2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.exitCode** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/process_test.ts:883` — process.exitCode → `assertEquals(process.exitCode, undefined)`
- `tests/unit_node/process_test.ts:885` — process.exitCode → `assertEquals(process.exitCode, 127)`

## PROC3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.getgid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/process_test.ts:960` — process.getgid → `assertEquals(process.getgid, undefined)`
- `tests/unit_node/process_test.ts:962` — process.getgid → `assertEquals(process.getgid?.(), Deno.gid())`

## PROC4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.getuid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/process_test.ts:968` — process.getuid → `assertEquals(process.getuid, undefined)`
- `tests/unit_node/process_test.ts:970` — process.getuid → `assertEquals(process.getuid?.(), Deno.uid())`

## PROC5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.getBuiltinModule** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/process_test.ts:1369` — getBuiltinModule → `assertEquals(process.getBuiltinModule("something"), undefined)`

## PROC6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.geteuid** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/process_test.ts:976` — process.geteuid → `assertEquals(process.geteuid, undefined)`

## PROC7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.kill** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/process_test.ts:279` —  → `assertEquals(process.kill(p.pid, 0), true)`

## PROC8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**process.memoryUsage** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit_node/process_test.ts:869` — process.memoryUsage() → `assert(typeof mem.rss === "number")`
- `tests/unit_node/process_test.ts:870` — process.memoryUsage() → `assert(typeof mem.heapTotal === "number")`
- `tests/unit_node/process_test.ts:871` — process.memoryUsage() → `assert(typeof mem.heapUsed === "number")`

