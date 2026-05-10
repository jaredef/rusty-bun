# derive-constraints scan — 2026-05-10 — oven-sh/bun phase-a-port

First real run of the new `derive-constraints` extractor against the entire publicly-visible Bun phase-a-port branch.

## Inputs

- **Target.** `oven-sh/bun` branch `claude/phase-a-port`, HEAD `--depth=1` shallow clone, the same target as the welch run at `runs/2026-05-10-bun-phase-a/`.
- **Scope.** All files matching `*.test.ts`, `*.test.tsx`, `*.spec.ts`, `*.test.js`, `*.test.jsx`, `*.test.mjs`, `*.test.cjs`, `*.spec.js`, `*.zig`, `*.rs`. The walker skips `.git/` and `node_modules/`.
- **Tool version.** `derive-constraints` v0.1 (this commit).

## Output

| Metric                | Value   |
|-----------------------|--------:|
| Files scanned         | 4,470   |
| Parse failures        | 0       |
| Tests extracted       | 17,775  |
| Constraint clauses    | 43,094  |

| Language     | Files | Tests   | Constraints |
|--------------|------:|--------:|------------:|
| TypeScript   | 1,548 | 15,093  | 34,604      |
| JavaScript   |   195 |  2,417  |  7,583      |
| Rust         | 1,429 |    133  |    427      |
| Zig          | 1,298 |    132  |    480      |

## Three readings

**Reading 1 — Zero parse failures across 4,470 files.** Tree-sitter-typescript handles every `.test.ts` in Bun's corpus including the ones using template literals, decorators, and TypeScript-only syntax. syn 2 handles every `.rs` in phase-a-port. The hand-rolled Zig regex extractor handles all `.zig` files cleanly. The MVP's parser routing is robust enough to take to the next phase.

**Reading 2 — TS/JS dominates the constraint pool, ~98% of total.** ~42K of 43K extracted constraints come from JS-runtime tests; only ~900 from source-internal Rust+Zig tests. This matches the planning-doc reading at [`docs/derivation-inversion-on-bun-tests.md §3`](../../docs/derivation-inversion-on-bun-tests.md): the JS-runtime tests are where Bun's external contract lives, and the source-internal tests are implementation invariants. The bulk of the latent formal architecture the keeper's conjecture invokes is in the JS-runtime layer.

**Reading 3 — Constraint cardinality is in the planning-doc's predicted band.** The planning doc predicted total clause count ≈ 10⁴–10⁵; observed is 4.3 × 10⁴. Average constraints per test ≈ 2.4 (43,094 / 17,775); this is consistent with the test-as-precondition-postcondition shape (one expect for setup verification, one for the property under test, occasionally a third for cleanup invariants). The smaller-double-digits-per-test target named in the planning doc holds for individual tests; the total is dominated by the long tail of high-test-count files (sql.test.ts: 807 tests / 1,529 constraints; valkey.test.ts: 439 / 1,205; fake-timers.test.ts: 436 / 754).

## Top 10 TypeScript test files by test count

```
   807 tests,  1529 constraints,  12401 loc — test/js/sql/sql.test.ts
   439 tests,  1205 constraints,   6903 loc — test/js/valkey/valkey.test.ts
   436 tests,   754 constraints,   6163 loc — test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts
   402 tests,   407 constraints,   6424 loc — test/js/bun/yaml/yaml-test-suite.test.ts
   318 tests,   444 constraints,   1573 loc — test/js/bun/json5/json5.test.ts
   ... (1,548 TS files total)
```

## Sample extraction (TS)

`test/js/sql/sql-prepare-false.test.ts`:
```
TEST "PostgreSQL prepare: false > basic parameterized query" (L35-39, kind=test)
   L 38  expect_chain  subj="x"   raw=expect(x).toBe(42)
TEST "PostgreSQL prepare: false > multiple parameterized queries sequentially" (L41-52)
   L 45  expect_chain  subj="a"   raw=expect(a).toBe(1)
   L 48  expect_chain  subj="b"   raw=expect(b).toBe("hello")
   L 51  expect_chain  subj="c"   raw=expect(c).toBeCloseTo(3.14)
```

The describe-chain reconstruction is working ("PostgreSQL prepare: false > basic parameterized query" reflects nesting); subject-extraction picks the first argument of `expect(...)`; raw form is whitespace-collapsed verbatim.

## Sample extraction (Zig)

`src/collections/linear_fifo.zig`:
```
TEST "LinearFifo(u8, .Dynamic)" (L452-536, 29 constraints)
   L 457  zig_testing_expect  subj="testing.expectEqual/@as(usize, 5)"
          raw=try testing.expectEqual(@as(usize, 5), fifo.readableLength());
   L 458  zig_testing_expect  subj="testing.expectEqualSlices/u8"
          raw=try testing.expectEqualSlices(u8, "HELLO", fifo.readableSlice(0));
```

Brace-tracking correctly bounds the test block (L452–L536, a 85-line test); regex-based clause detection captures all `try testing.expect*(...)` calls; subject contains both the verb and the first argument, useful for the cluster phase's axis identification.

## What this run shows

The scan phase works. It produces structured JSON over the entire Bun test corpus in well under a minute, with no parse failures and no observable extraction bugs in the samples inspected. The output is in the right shape for the cluster phase to consume.

The next phase (`cluster`) needs an axis catalog. The 8 axes from [`docs/porting-md-analysis.md §3.1`](../../docs/porting-md-analysis.md) cover the Zig→Rust translation lattice but not the JS-runtime API surface. New axes for the JS-runtime layer (per-namespace API surface; Node.js compatibility surface; web-platform API surface; HTTP/FS/syscall semantic; async/streaming semantics) need to be defined before the JS-runtime constraints can be hierarchically clustered.

The Pin-Art LOC prediction (planning-doc §6) waits on cluster + invert. With 4.3 × 10⁴ constraints in hand, if the htxlang precedent (~70 LOC per constraint at greenfield scale) extrapolates, predicted Rust LOC for a derivation-inverted Bun would be in the ~3 × 10⁶ range — substantially larger than the observed phase-a-port's ~9.3 × 10⁵ LOC. That gap is informative: either Bun's actual constraints are denser than htxlang's (per-constraint LOC ratio is lower at scale), or the test corpus over-specifies (extracts incidental implementation details as constraints), or the phase-a-port under-implements (gaps relative to the test contract). Distinguishing among these is what the predict phase tests.

## Files in this run

- `bun-scan.json` — 15 MB; full per-file extraction. Each TestFile entry contains all extracted tests with constraint clauses.
- `summary.txt` — terminal summary captured from `--summary` stderr.

## Provenance

- Tool: `derive-constraints` v0.1, commit hash visible in git log.
- Target clone: same shallow clone of oven-sh/bun claude/phase-a-port from the welch run on the same date.
- Tool runtime: well under a minute on the dev machine; rayon-parallelized over the 4,470-file corpus.
