# Bun.spawn pilot — coverage audit

**Thirteenth pilot. Tier-B #5 from the trajectory queue.** Bun's subprocess management — `Bun.spawn` and `Bun.spawnSync`. Pure-Rust derivation via `std::process`. Tier-2 ecosystem-only.

## Constraint inputs

From `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/bun.constraints.md`:
- `Bun.spawn` (production patterns): 79 cross-corroborated clauses
- `Bun.spawn` (existence / class): 13 clauses
- `Bun.spawnSync` (existence / class): smaller cluster
- Terminal-integration clauses (out of pilot scope)

Total: ~95+ cross-corroborated clauses on the spawn surface.

## Pilot scope

- `Bun::spawn_sync(args, options)` → `SyncSubprocess`  — synchronous; collects stdout/stderr/exit-code
- `Bun::spawn(args, options)` → `Subprocess` — async-shaped (returns handle; consumer calls `.wait()`)
- `SpawnOptions`: cwd, env, stdin (string/bytes/inherit/null), stdout (pipe/inherit/null), stderr (pipe/inherit/null)
- `Subprocess::pid()`, `.exited()`, `.kill(signal?)`, `.wait()` → `ExitStatus`
- `SyncSubprocess::exit_code`, `.stdout`, `.stderr`, `.success`
- `OnExit` callback support — invoked after `wait()` completes

Out of scope:
- Streaming stdio with ReadableStream / WritableStream wiring (would require streams pilot integration; deferred)
- Terminal mode (`options.terminal`) — pilot does not bind a PTY
- IPC (`stdin: "ipc"` channels)
- Resource limits / rlimit

## Approach

Wrap `std::process::Command` + `std::process::Child` + `std::process::Output`. Map Bun's options dictionary to Command builder calls. The pilot scope is a thin adapter over std::process — most of the LOC is option-translation rather than implementation.

## Ahead-of-time hypotheses

1. **Pilot is small in LOC** — std::process does the heavy lifting. Estimated 150-200 LOC.
2. **First-run clean closure expected.** Spawn semantics are concrete (Unix fork+exec, Windows CreateProcess); std::process documentation is good; Bun's API is a straightforward wrapper.
3. **The exit-callback timing semantics** is the most likely bug site. Bun's `onExit` fires after the child exits and after stdio is drained. Pilot will fire it inside `wait()`.

## Verifier strategy

~15-20 verifier tests that actually spawn child processes (using `/bin/echo`, `/bin/sh`, or platform-equivalent on Windows). Tests touch the OS but stay fast.

Consumer regression: ~6-8 tests citing real Bun-using consumer patterns (build scripts, test runners, dev-server child processes).
