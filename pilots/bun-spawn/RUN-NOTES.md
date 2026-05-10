# Bun.spawn pilot — 2026-05-10

**Thirteenth pilot. Tier-B #5 — completes the Tier-B Bun-namespace tier.** Pure-Rust derivation wraps `std::process::Command`. After Bun.spawn, all three flagship Bun-namespace surfaces (Bun.file + Bun.serve + Bun.spawn) are anchored.

## Pipeline

```
v0.13b enriched constraint corpus (Bun.spawn: 79 + 13 clauses)
       │
       ▼
AUDIT.md
       │
       ▼
simulated derivation v0   (CD + Bun docs at bun.sh/docs/api/spawn)
       │
       ▼
derived/src/lib.rs   (179 code-only LOC; std::process wrapper)
       │
       ▼
cargo test
   verifier:            19 tests (real subprocess invocation)
   consumer regression:  8 tests
       │
       ▼
27 pass / 0 fail / 0 skip (after 2 author-side test corrections)
```

## Verifier results: 19/19

```
spawn_sync (10 tests)
  echo captures stdout; nonzero exit code; stderr captured;
  stdin text passed through; stdin bytes passed through; cwd set;
  env set; env clear when set (no PATH inheritance via env command);
  stdout null discards; empty args errors

spawn (async-shaped, 5 tests)
  returns handle with pid; wait collects exit code; wait collects stderr;
  kill terminates a sleep child; stdin text via wait

Edge cases (4 tests)
  unknown program errors; multiline stdout preserved; binary stdout
  via printf octal; empty args errors
```

## Consumer regression: 8/8

```
Dev workflow check command exit code                    1
Test runner byte-equal stdout                           1
Build script env propagation                            1
CLI tool jq-style stdin pipe                            1
Dev server kills child on reload                        1
Bun docs canonical spawnSync example                    1
Lint tool runs in cwd                                   1
CI pipeline exit-code precision (0, 1, 2, 42, 127, 255) 1
```

## LOC measurement

```
Bun reference (subprocess machinery):
  src/spawn/process.rs                3,666 LOC
  src/spawn/static_pipe_writer.rs       366
  src/spawn/lib.rs                      379
  src/spawn_sys/spawn_process.rs        888
  src/spawn_sys/posix_spawn.rs          911
  src/spawn_sys/lib.rs                  179
  Total Bun spawn machinery           6,389 LOC

Pilot derivation (code-only):           179 LOC
Naive ratio:                           2.8%
Adjusted (data-layer scope, excluding
  syscall-wrapping + IPC + pipe-pool
  machinery; equivalent slice ~800-1200 LOC):    ~15-20%
```

The naive 2.8% ratio is a strong number but again unfair: Bun's spawn machinery includes syscall-level wrappers (posix_spawn integration), pipe-pool / static-buffer optimization, IPC channel support, and platform-specific code paths. The pilot is a thin std::process adapter at the data-layer.

## Findings

1. **AOT hypothesis #1 confirmed.** 179 code-only LOC, in the predicted 150-200 range.

2. **AOT hypothesis #2 confirmed.** First-run clean closure. Two failures on first run were **author-side test bugs**, not derivation bugs:
   - `spec_spawn_sync_env_clear_when_set` — used `sh -c "echo $PATH"` to test env-clear, but POSIX shells auto-initialize PATH from a built-in default if PATH is unset. Switched to invoking `env` directly which preserves the env_clear semantics. **The derivation correctly cleared the parent env**; my test misunderstood shell behavior.
   - `spec_spawn_sync_binary_stdout` — used `printf '\\x00...'` but `\x` escapes are non-portable in POSIX `sh`. Switched to octal `\000`. Derivation captured the bytes correctly; my test used wrong escape syntax.

   This is the **second pilot to surface only author-side test bugs** (after Pilot 10 / Buffer). The pattern: as the apparatus matures, the LLM-derivation gets the spec right; what fails is my own test discipline. Distinguishing author-bug from derivation-bug is itself an apparatus signal — both are caught by the verifier, but their fix locations differ.

3. **AOT hypothesis #3 partially confirmed.** Exit-callback timing is invoked synchronously inside `wait()`. Pilot doesn't expose `onExit` callback yet (deferred); the synchronous-wait pattern is equivalent for verification.

4. **First pilot to invoke real subprocesses.** Tests use `/bin/sh`, `cat`, `pwd`, `env`, `echo`, `printf`, `sleep`. Cross-platform note: tests assume POSIX-shell tools available; would need `cmd.exe` / `powershell.exe` adapters on Windows. Pilot scope is POSIX for the verifier; a Windows pilot is deferred.

## Updated 13-pilot table

| Pilot | Class | LOC | Adj. ratio |
|---|---|---:|---:|
| TextEncoder + TextDecoder | data structure | 147 | 17–25% |
| URLSearchParams | delegation target | 186 | 62% |
| structuredClone | algorithm | 297 | ~8.5% |
| Blob | composition substrate | 103 | 20–35% |
| File | inheritance/extension | 43 | 20–30% |
| AbortController + AbortSignal | event/observable | 126 | 25–35% |
| fetch-api (Headers + Request + Response) | system / multi-surface | 405 | 6.5% naive / ~20% adj |
| node-path | Tier-2 Node-compat pure-function | 303 | 8.3% naive / ~12–15% adj |
| streams (Readable + Writable + Transform) | substrate / async-state-machine | 453 | 11.2% naive / ~12–15% adj |
| buffer | Tier-2 Node-compat binary type | 261 | 11.1% naive / ~17% adj |
| Bun.file | Tier-2 Bun-namespace + first I/O | 95 | 3.0% naive / ~20–30% adj |
| Bun.serve | Tier-2 Bun-namespace flagship system | 175 | 0.5% naive / ~20–30% adj |
| **Bun.spawn** | **Tier-2 Bun-namespace subprocess** | **179** | **2.8% naive / ~15–20% adj** |

Thirteen-pilot aggregate: **2,773 LOC** of derived Rust against ~78,000+ LOC of upstream reference targets. **Aggregate naive ratio: ~3.6%.** Adjusted (equivalent-scope across all pilots): ~5-7%.

## Trajectory advance

Tier-B fully completed. All three flagship Bun-namespace surfaces (Bun.file, Bun.serve, Bun.spawn) anchored. Next queued: **Tier-C #6: Node fs (sync subset) pilot**.

## Files

```
pilots/bun-spawn/
├── AUDIT.md
├── RUN-NOTES.md              ← this file
└── derived/
    ├── Cargo.toml
    ├── src/
    │   └── lib.rs            (232 LOC, 179 code-only)
    └── tests/
        ├── verifier.rs            19 tests (real subprocess invocation)
        └── consumer_regression.rs  8 tests
```

## Provenance

- Tool: `derive-constraints` v0.13b.
- Constraint inputs: `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/bun.constraints.md` (Bun.spawn cluster).
- Spec input: none — Tier-2 ecosystem-only. Bun docs (https://bun.sh/docs/api/spawn) serve as authoritative reference.
- Reference target: Bun's `src/spawn/` + `src/spawn_sys/` directories (6,389 LOC total).
- Result: 27/27 across both verifier (19) and consumer regression (8). Zero derivation regressions. Two author-side test bugs surfaced + fixed before final.
