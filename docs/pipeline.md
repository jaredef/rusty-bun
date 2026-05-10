# Pipeline — running the full rusty-bun analysis on a corpus

The `derive-constraints pipeline` subcommand runs the full eight-phase analysis end-to-end. It is a thin orchestrator over the existing per-phase subcommands; intermediate JSON outputs land in the output directory so any phase's result can be inspected or fed into a downstream tool.

## Inputs

| Flag | What it is |
|---|---|
| `--test-corpus <DIR>` | Test source root. Walked by `derive-constraints scan` for `.test.ts` / `.test.js` / `.zig` / `.rs` test sources. |
| `--impl-source <DIR>` | Implementation source root (target-language `.rs` files welch operates on). For Bun and similar projects this is often the same path as `--test-corpus`. |
| `--baseline <DIR>` | Directory of idiomatic-Rust crates that welch's anomaly detection uses as the comparison distribution. |
| `--out <DIR>` | Output directory; created if missing. |
| `--welch-binary <PATH>` | Optional. Defaults to `<workspace>/welch/target/release/welch`. |

## Phases

```
[1/8] scan          — derive-constraints AST scan over test corpus
[2/8] welch scan    — welch idiomaticity scan over impl-source
[3/8] welch scan    — welch idiomaticity scan over baseline
[4/8] welch baseline — distribution summary from baseline scan
[5/8] welch compare  — z-score anomalies of impl-source vs baseline
[6/8] cluster        — property canonicalization + antichain selection
[7/8] invert         — emit .constraints.md (both namespace + --by-seams)
[8/8] seams + couple — architectural-seam detection + welch coupling
```

## Outputs

```
out/scan.json                       — per-file extracted constraints
out/welch-target-scan.json          — welch impl-source per-file metrics
out/welch-baseline-scan.json        — welch baseline per-file metrics
out/welch-baseline-summary.json     — per-metric distribution from baseline
out/welch-anomalies.json            — z-scored anomalies of impl-source
out/cluster.json                    — property catalog with antichain
out/constraints/                    — namespace-grouped .constraints.md
out/constraints-by-seams/           — seam-grouped .constraints.md (Doc 705)
out/seams.json                      — signal-vector clusters
out/coupled.json                    — seams ↔ welch cross-reference
```

All outputs are JSON (or markdown for the constraint documents) and rederive-compatible where applicable.

## Example: Bun phase-a-port

```bash
$ derive-constraints pipeline \
    --test-corpus /tmp/welch-corpus/target/bun \
    --impl-source /tmp/welch-corpus/target/bun \
    --baseline   /tmp/welch-corpus/baseline \
    --out        ./runs/2026-05-10-bun
```

Runs end-to-end in ~5 seconds on a workstation. The `runs/2026-05-10-bun-derive-constraints/` directory in this repository contains the equivalent output produced phase-by-phase during the apparatus's iterative development.

## Re-running

The pipeline writes intermediate JSON to stable paths, so phases downstream of an unchanged input can be re-run independently with the per-phase subcommands. For example, after editing `cluster.rs` and rebuilding:

```bash
$ derive-constraints cluster ./out/scan.json -o ./out/cluster.json
$ derive-constraints seams ./out/cluster.json -o ./out/seams.json --corpus-root /tmp/welch-corpus/target/bun
$ derive-constraints couple ./out/seams.json ./out/welch-anomalies.json -o ./out/coupled.json
```

The pipeline subcommand is the convenient shorthand; the per-phase subcommands give finer control.
