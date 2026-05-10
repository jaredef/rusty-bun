# welch — Welch-bound packing diagnostic

A scanner that quantifies *how idiomatic-Rust-shaped* a Rust codebase is, by counting `unsafe` blocks, `unsafe fn` declarations, raw pointer types, `transmute` calls, and `extern` foreign-module blocks per file, then comparing against a baseline distribution from a corpus of mature idiomatic Rust crates.

Per [RESOLVE Doc 702 §3 Mapping 6 + §4 Addition 4](https://jaredfoy.com/resolve/doc/702-ai-assisted-cross-language-code-translation-as-a-pin-art-bilateral-under-sipe-t-threshold-conditions-reading-the-bun-zig-to-rust-port), dense unsafe / raw-pointer / transmute / FFI patterns in AI-translated Rust are the operational marker for a translation that has not re-settled onto Rust's natural ETF attractor — it is recreating the source language's semantics inside Rust syntax instead. This tool measures that.

## Pipeline

```
welch scan <dir>          → per-file metrics JSON
welch baseline <scan>     → per-metric distribution JSON
welch compare <baseline> <target> → per-file z-score anomaly report
```

A typical run:

```bash
# 1. Build a baseline from a curated set of mature idiomatic Rust crates.
welch scan ~/baselines/tokio    -o tokio.json
welch scan ~/baselines/hyper    -o hyper.json
welch scan ~/baselines/serde    -o serde.json
welch scan ~/baselines/ripgrep  -o ripgrep.json
# ... merge into a single baseline scan however you like;
# for now, the simplest approach is to run `welch baseline` on each
# and combine the resulting summaries by averaging, or scan a single
# parent directory containing all the baseline crates.

welch scan ~/baselines/all -o baseline-all.json
welch baseline baseline-all.json -o baseline-summary.json

# 2. Scan the target — for instance, the Bun phase-a-port branch.
welch scan ~/bun/phase-a-port -o bun-port.json

# 3. Compare. Files exceeding z >= 2.0 on any metric are flagged.
welch compare baseline-summary.json bun-port.json -z 2.0 --summary -o anomalies.json
```

## Metrics

All metrics are computed per file via syn 2 AST traversal. Comparisons are made on **density per kLOC** (count × 1000 / file LOC) so file size is not the comparison object.

| Metric          | Counts                                                                  |
|-----------------|-------------------------------------------------------------------------|
| `unsafe_blocks` | `unsafe { ... }` expression blocks                                      |
| `unsafe_loc`    | source lines spanned by unsafe blocks                                   |
| `unsafe_fns`    | `unsafe fn` declarations (top-level, in `impl`, in `trait`)             |
| `fns`           | total fn declarations (denominator for `unsafe_fns / fns` if you want it) |
| `raw_pointers`  | occurrences of `*const T` or `*mut T` types                             |
| `transmutes`    | calls whose path ends in `transmute`                                    |
| `extern_blocks` | `extern "..." { ... }` foreign-module blocks                            |

## Anomaly report format

The compare-step JSON contains:
- `aggregate` — corpus-wide z-scores: how the target's whole-codebase density compares to the baseline distribution of per-file densities
- `anomalous_files` — files exceeding the z-score threshold on at least one metric, sorted by maximum z-score descending

`Infinity` z-scores indicate a baseline std of zero on that metric (i.e., no baseline file had the construct). This is informative — a file with a transmute call when no baseline crate uses transmute is unambiguously anomalous — but should be read alongside the absolute target value, not the z alone.

## What the tool is not

- Not a correctness checker. High `unsafe` density is not by itself a bug. It is a *signal* that the file may need human review; mature unsafe-heavy Rust crates (e.g. low-level allocators, FFI shims) will register as anomalous against a general-purpose baseline.
- Not a substitute for differential testing (T1 of the Doc 699 / Doc 541 Fal-T5 simultaneity test). Welch-bound diagnostics flag the *shape* of the translation; differential testing flags the *behavior*. Both are needed.
- Not stable across drastically different baseline corpora. Pick a baseline that matches your target's domain (systems-level Rust for systems-level translations; web-framework Rust for web-framework translations).

## Build

```bash
cargo build --release
./target/release/welch --help
```

## Status

MVP. Validated end-to-end on a synthetic corpus where a hand-written vibe-port-shaped target file lights up all six metrics against a small idiomatic baseline. Ready to point at real codebases.

## Next

- Per-function (not just per-file) anomaly granularity
- Lifetime-annotation density and `Rc`/`Arc`/`Box` allocation density as additional idiomaticity markers
- Multi-baseline composition (merge several baseline scans into one summary)
- Optional output as a sortable HTML report rather than (or in addition to) JSON
