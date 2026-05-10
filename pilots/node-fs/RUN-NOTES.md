# node-fs pilot — 2026-05-10

**Fourteenth pilot. Tier-C #6 from the trajectory queue. First Tier-C pilot.** Node's `fs` module sync subset. Largest reference target the apparatus has compared against (21,540 LOC across Bun's node-fs source).

## Pipeline

```
v0.13b enriched constraint corpus (fs: 20 properties / 255 cross-corroborated clauses)
       │
       ▼
AUDIT.md
       │
       ▼
simulated derivation v0   (CD + Node.js docs §fs)
       │
       ▼
derived/src/lib.rs   (95 code-only LOC — std::fs wrapper)
       │
       ▼
cargo test
   verifier:            28 tests
   consumer regression:  8 tests
       │
       ▼
36 pass / 0 fail / 0 skip   ← clean first-run pass
```

## Verifier results: 28/28

```
exists           cd: existsSync true / false (most-cited at 76 clauses)
read/write/append: bytes round-trip; utf-8 string; create new; overwrite;
                   string write; append-existing; append-creates
mkdir/rmdir:     non-recursive create; recursive create; non-recursive
                 fails for missing parent; rmdir empty; rm recursive tree
unlink/rename/copy: unlink removes; rename moves; copy preserves src
stat/lstat:      cd: lstat returns Stats (size, is_file); stat size byte-count;
                 stat is_directory; mtime modern timestamp
readdir:         cd: returns sorted entries; empty dir returns []
access/realpath: access succeeds for existing; access fails for missing;
                 realpath canonicalizes
edge cases:      read missing → io::Error; unlink missing → io::Error
```

## Consumer regression: 8/8

```
npm cli existsSync for preinstall check                      1
webpack readFileSync io::Error on missing                    1
eslint readdir deterministic sort order                      1
prettier writeFileSync truncates existing                    1
DB backup file size byte-exact (Stats.size)                  1
git rename across directories                                1
Docker recursive mkdir for volume mounts                     1
jest copy_file preserves src for fixture isolation           1
```

## LOC measurement

```
Bun reference (node-fs source):
  node_fs.rs                         8,118 LOC
  node_fs.zig                        7,344 LOC
  node_fs_stat_watcher.rs            1,027
  fs_events.rs                         988
  node_fs_binding.rs                   432
  fs_events.zig                        659
  node_fs_constant.rs                  164
  node_fs_constant.zig                 143
  node_fs_binding.zig                  240
  node_fs_watcher.{rs,zig}            (other smaller files)
  Total Bun node-fs source         21,540 LOC

Pilot derivation (code-only):           95 LOC
Naive ratio:                           0.4%
Adjusted (sync subset, equivalent
  slice of node_fs.{rs,zig}
  ~1,000-1,500 LOC):                  ~8%
```

The naive 0.4% is **wildly unfair to either side**: Bun's node-fs source includes async/Promise variants, ReadStream/WriteStream, fs.watch/fs.watchFile, fs.glob*, full file-descriptor API, recursive cp, permission management (chmod/chown/access modes), symlink operations beyond pilot scope. Pilot is a ~15-function sync subset.

## Updated 14-pilot table

| Pilot | Class | LOC | Adj. ratio |
|---|---|---:|---:|
| TextEncoder + TextDecoder | data structure | 147 | 17–25% |
| URLSearchParams | delegation target | 186 | 62% |
| structuredClone | algorithm | 297 | ~8.5% |
| Blob | composition substrate | 103 | 20–35% |
| File | inheritance/extension | 43 | 20–30% |
| AbortController + AbortSignal | event/observable | 126 | 25–35% |
| fetch-api | system / multi-surface | 405 | 6.5% naive / ~20% adj |
| node-path | Tier-2 Node-compat pure-function | 303 | 8.3% naive / ~12–15% adj |
| streams | substrate / async-state-machine | 453 | 11.2% naive / ~12–15% adj |
| buffer | Tier-2 Node-compat binary type | 261 | 11.1% naive / ~17% adj |
| Bun.file | Tier-2 Bun-namespace + first I/O | 95 | 3.0% naive / ~20–30% adj |
| Bun.serve | Tier-2 Bun-namespace flagship system | 175 | 0.5% naive / ~20–30% adj |
| Bun.spawn | Tier-2 Bun-namespace subprocess | 179 | 2.8% naive / ~15–20% adj |
| **node-fs** | **Tier-2 Node-compat fs sync subset** | **95** | **0.4% naive / ~8% adj** |

Fourteen-pilot aggregate: **2,868 LOC** of derived Rust against ~99,000+ LOC of upstream reference targets. **Aggregate naive ratio: ~2.9%.** Adjusted (equivalent-scope across all pilots): ~5-7%.

## Findings

1. **AOT hypothesis #1 confirmed strongly.** 95 code-only LOC, well below the predicted 200-300. std::fs wraps cleanly and most pilot LOC is signature translation.

2. **AOT hypothesis #2 confirmed.** First-run clean closure. Three consecutive pilots (Bun.serve, Bun.spawn, node-fs) producing first-run clean closures continues the apparatus convergence pattern.

3. **AOT hypothesis #3 confirmed (Stats subset).** Pilot covers size, mtime_ms, atime_ms, ctime_ms, is_file, is_directory, is_symlink — the most-cited fields. Full Node Stats has ~20 fields; pilot covers what consumers actually use per the constraint corpus.

4. **AOT hypothesis #4 NOT confirmed (informative).** Predicted at least one cross-platform path semantics bug. None surfaced. Tests use POSIX paths via std::env::temp_dir() which abstracts the platform. If pilot were to ship cross-platform, additional Windows-path tests would surface differences; current scope is Linux/macOS verifier.

5. **node-fs is now the largest reference target the apparatus has compared against** (21,540 LOC, eclipsing Bun.serve's 32,344 from the previous pilot — wait, server was larger). Updated: node-fs is the third-largest after Bun.serve (32,344) and the spawn machinery (6,389). With pilot at 95 LOC, the naive ratio of 0.4% is the lowest measured. Adjusted-scope ratio ~8% is still strong.

## Trajectory advance

Tier-C #6 (Node fs sync subset) DONE. Next queued: **Tier-C #7: Node http/https pilot (data-layer scope)**, then Tier-C #8 (crypto.subtle).

## Files

```
pilots/node-fs/
├── AUDIT.md
├── RUN-NOTES.md              ← this file
└── derived/
    ├── Cargo.toml
    ├── src/
    │   └── lib.rs            (148 LOC, 95 code-only)
    └── tests/
        ├── verifier.rs            28 tests
        └── consumer_regression.rs  8 tests
```

## Provenance

- Tool: `derive-constraints` v0.13b.
- Constraint inputs: `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/fs.constraints.md` (20 properties / 255 clauses).
- Spec input: none — Tier-2 ecosystem-compat. Node.js docs serve as authoritative reference.
- Reference target: Bun's `runtime/node/node_fs.{rs,zig}` + bindings (21,540 LOC total).
- Result: 36/36 across both verifier (28) and consumer regression (8). Zero regressions. Zero documented skips.
