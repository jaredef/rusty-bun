# Deno v0.11 pipeline run — 2026-05-10

Clean end-to-end pipeline run on Deno after the v0.11 cluster fix landed (commit 57b25aa). This is the corrected counterpart to `runs/2026-05-10-deno-comparative/`, which captured the pre-fix output for diagnostic purposes. Use this run for downstream comparison; use the comparative run when illustrating the apparatus's failure-mode-then-resolution arc.

## Pipeline runtime

3.31 seconds end-to-end (8 phases). Identical scan/welch numbers to the comparative run since those phases are upstream of the cluster fix.

## Output summary

| Phase | Output | Result |
|---|---|---|
| 1/8 scan | scan.json | 1,263 files, 6,228 tests, 11,399 clauses |
| 2/8 welch impl-scan | welch-target-scan.json | 940 files, 0 parse failures |
| 3/8 welch baseline-scan | welch-baseline-scan.json | 1,255 files (tokio + hyper + reqwest + serde + ripgrep), 0 parse failures |
| 4/8 welch baseline-summary | welch-baseline-summary.json | 1,255 files, 329,409 LOC, 6 metrics |
| 5/8 welch compare | welch-anomalies.json | impl-source z-anomalies vs baseline |
| 6/8 cluster | cluster.json | **1,852 properties / 50 construction-style** (was 0 pre-fix) |
| 7/8 invert | constraints/ + constraints-by-seams/ | **37 namespace docs** (was 28 pre-fix); 1 by-seams doc |
| 8/8 seams + couple | seams.json + coupled.json | 28 signal clusters, 15 cross-namespace seams; 110 surfaces, 10 mismatches |

## What changed vs the comparative run

- **Cluster construction-style:** 0 → 50. See `runs/2026-05-10-deno-comparative/COMPARATIVE-NOTES.md` §"The construction-style finding (zero on Deno → 50 after v0.11 fix)" for the full diagnosis.
- **Constraint document count:** 28 → 37 namespace-grouped documents under `constraints/`. New entries cover `Event`, `EventTarget`, `CustomEvent`, `Cluster`, `Process`, `Fs`, `Http`, `GlobalThis`, `QueueMicrotask`, `AbortSignal`, `Assert`. These are surfaces that exist on Deno's runtime but were invisible to the original namespace-allowlist.
- **Coupling surfaces and mismatches:** 110 surfaces total (unchanged); 10 mismatches (unchanged count). Mismatch composition shifts slightly: `tty` now appears as a `seams_hot_welch_cold` candidate where the original run had it absent. The 9 welch-hot-seams-cold candidates (http, Buffer, buffer, util, process, crypto, Stream, url, os) reproduce.

## Mismatch candidates (rederive-pilot-target candidates)

```
http     welch_hot_seams_cold
Buffer   welch_hot_seams_cold
buffer   welch_hot_seams_cold
util     welch_hot_seams_cold
process  welch_hot_seams_cold
crypto   welch_hot_seams_cold
Stream   welch_hot_seams_cold
url      welch_hot_seams_cold
os       welch_hot_seams_cold
tty      seams_hot_welch_cold
```

The 9 welch-hot-seams-cold surfaces are JS-runtime universals — same architectural class identified on Bun. The `tty` seams-hot-welch-cold candidate is novel: tests probe TTY behavior architecturally (signal patterns surface) but the implementation crate doesn't show outsized native-FFI density (welch baseline is matched well there).

## Provenance

- Tool: `derive-constraints` v0.11 (commit 57b25aa).
- Target: `denoland/deno` HEAD shallow clone, 2026-05-10 (re-cloned for this run).
- Baseline: `tokio` + `hyper` + `reqwest` + `serde` + `ripgrep` shallow clones at `/tmp/welch-corpus/baseline/`.
- Pipeline runtime: 3.31 seconds end-to-end.
- Run directory size: see `du -sh runs/2026-05-10-deno-v0.11/` for current footprint.
