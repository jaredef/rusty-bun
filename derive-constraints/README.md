# derive-constraints — Phase 1 (scan)

Extracts behavioural constraints from a test corpus by AST-walking each test file. The first phase of the four-phase pipeline articulated in [`../docs/derivation-inversion-on-bun-tests.md`](../docs/derivation-inversion-on-bun-tests.md).

> **Apparatus.** Per the corpus's tests-as-constraints frame ([Doc 159 / Doc 247](https://jaredfoy.com/resolve/doc/247-the-derivation-inversion)), an executable test suite is the most precise statement of behavioural constraints available. This tool extracts those constraints into a structured form. Cluster / invert / predict are the subsequent phases that consume the JSON this tool emits.

## Pipeline position

```
[ test corpus on disk ]
        │
        ▼
   ┌─────────┐         ┌────────┐         ┌────────┐         ┌─────────┐
   │  scan   │ ──json─▶│cluster │ ──json─▶│ invert │ ──md──▶│ predict │
   │ (this)  │         │ (TBD)  │         │ (TBD)  │         │ (TBD)   │
   └─────────┘         └────────┘         └────────┘         └─────────┘
```

## What scan does

Walks a directory tree, finds test source files, parses each with the appropriate front-end, and emits per-file JSON containing every test it can identify and every assertion clause inside each test.

| Language     | File pattern                              | Parser                  | Test detection                                  |
|--------------|-------------------------------------------|-------------------------|--------------------------------------------------|
| TypeScript   | `*.test.ts`, `*.test.tsx`, `*.spec.ts`    | tree-sitter             | `test()`, `it()`, `describe()` call sites       |
| JavaScript   | `*.test.js`, `*.test.jsx`, `*.test.mjs`, `*.test.cjs`, `*.spec.js` | tree-sitter | same                                  |
| Rust         | `*.rs`                                    | syn 2                   | functions / impl methods / trait fns with `#[test]` |
| Zig          | `*.zig`                                   | regex + brace tracking  | `test "name" { ... }` blocks                    |

Test names reconstruct any enclosing `describe(...)` chain, so a TS test name reads as `"Bun.serve > 404 fallback > with custom handler"`. Test markers (`test.skip`, `test.todo`, `test.failing`, Rust `#[ignore]`) are recorded; the test is included even when disabled, with the marker carrying the information that the constraint is acknowledged but un-validated.

Constraint detection identifies:
- `expect(x).toY(z)` chains (TS/JS) — recorded as `ExpectChain`
- `assert(...)`, `assert.equal(x, y)`, `assertEquals(...)` (TS/JS) — `AssertCall`
- `assert!`, `assert_eq!`, `assert_ne!`, `debug_assert!`, `panic!`, `unreachable!`, `todo!` macros (Rust) — `AssertMacro`
- `try testing.expect*(...)` and `try aliased.expect*(...)` patterns (Zig) — `ZigTestingExpect`

Each clause carries the source line, a verbatim raw form (whitespace-collapsed), and a best-effort subject identifier (e.g. for `expect(Bun.serve).toBeFunction()` the subject is `Bun.serve`).

## Output shape

```json
{
  "root": "/abs/path",
  "files": [
    {
      "path": "test/js/foo.test.ts",
      "language": "typescript",
      "loc": 135,
      "tests": [
        {
          "name": "Foo > does the thing",
          "kind": "test",
          "line_start": 35, "line_end": 39,
          "constraints": [
            { "line": 38, "raw": "expect(x).toBe(42)",
              "kind": "expect_chain", "subject": "x" }
          ],
          "skip": false, "todo": false, "failing": false
        }
      ],
      "parse_failure": null
    }
  ],
  "stats": {
    "files_scanned": 1, "parse_failures": 0,
    "tests_total": 1, "constraints_total": 1,
    "by_language": { ... }
  }
}
```

## Build & run

```bash
cargo build --release
./target/release/derive-constraints scan <directory> -o scan.json --summary
```

## First real run

`runs/2026-05-10-bun-derive-constraints/` records a scan over the entire `oven-sh/bun` `claude/phase-a-port` branch. 4,470 files / 17,775 tests / 43,094 constraints / 0 parse failures. See `runs/2026-05-10-bun-derive-constraints/RUN-NOTES.md` for the breakdown.

## What scan does not do

- Does not resolve identifier aliases. `import { test as t } from "bun:test"; t("foo", ...)` is not detected as a test (the universe of name-shapes seen in Bun's test corpus is narrow enough that the simple match suffices for the MVP, but this is a stated limitation).
- Does not resolve cross-file references. Constraint clauses contain raw call-site text; tracking what `Bun.serve` actually means at the implementation layer is left to the cluster/invert phases.
- Does not classify constraints by axis. That is the cluster phase's job.

## Next phases

- **cluster** — classify each constraint by axis (API namespace, syscall surface, allocation context, threading, FFI, …); emit a hierarchical partition lattice ([Doc 701](https://jaredfoy.com/resolve/doc/701-ill-resolved-against-the-corpus-information-lattice-learning-as-the-mature-prior-art-framework-for-the-pin-art-bilateral-and-the-joint-mi-lattice)).
- **invert** — derivation-invert the partition lattice into a hierarchical constraint document in prose ([Doc 247](https://jaredfoy.com/resolve/doc/247-the-derivation-inversion)).
- **predict** — emit the Pin-Art LOC prediction for the derived implementation ([Doc 270](https://jaredfoy.com/resolve/doc/270-pin-art-models)).
