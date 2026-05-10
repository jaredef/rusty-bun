# welch run — 2026-05-10 — Bun phase-a-port vs (tokio + ripgrep + serde) baseline

First real-corpus run of welch. Substantive, not just shape-validation.

## Inputs

- **Baseline.** `tokio` (HEAD `--depth=1`), `ripgrep` (HEAD `--depth=1`), `serde-rs/serde` (HEAD `--depth=1`). 1,085 .rs files / 268,935 LOC. Mature general-purpose idiomatic Rust.
- **Target.** `oven-sh/bun` branch `claude/phase-a-port` (HEAD `--depth=1`). 1,429 .rs files / 933,120 LOC. The publicly-visible Anthropic-driven Phase A port from Zig to Rust that Doc 702 reads.
- **Threshold.** `z ≥ 3.0` (or unbounded upward — see "JSON Infinity bug" below).
- **welch version.** v0.1 (commit `08ff7ba` plus the JSON-Infinity fix in this run's commit).

## Aggregate signal (corpus-wide density per kLOC, target vs baseline distribution)

| Metric         | Target | Baseline μ | Baseline σ |   z   |
|----------------|-------:|-----------:|-----------:|------:|
| `unsafe_blocks`|  13.89 |       2.91 |      10.11 |  +1.1 |
| `unsafe_loc`   |  13.89 |       2.91 |      10.11 |  +1.1 |
| `unsafe_fns`   |   1.42 |       0.74 |       3.99 |  +0.2 |
| `raw_pointers` |  14.64 |       1.21 |       7.84 |  +1.7 |
| `transmutes`   |   0.03 |       0.05 |       0.87 |  −0.0 |
| `extern_blocks`|   0.59 |       0.00 |       0.00 |  +inf |

Three readings worth holding in mind:

**Reading 1 — `transmutes ≈ 0`.** The construct most associated with cross-language reinterpret patterns (`u64↔f64`, `&[u8]↔&[T]`, etc.) is *not* elevated in the target. Aggregate target density (0.03 per kLOC) is essentially identical to the baseline mean (0.05). If the port were broadly recreating Zig's bitcasting via Rust transmutes — the failure mode the Lobsters discussion surfaces around `@fieldParentPtr` recreation — we would expect a clean upward signal here. We don't see one at the corpus-wide aggregate. This is mildly favorable.

**Reading 2 — `unsafe_blocks` and `raw_pointers` only mildly elevated.** Both are within 2σ. The phase-a-port has roughly 5× the unsafe-block density and 12× the raw-pointer density of the baseline at the corpus-wide aggregate, but the baseline's standard deviation is itself large (driven by tokio's lower-level syscall and runtime regions). The signal is real but mild.

**Reading 3 — `extern_blocks` at `+inf`.** The only sharply-anomalous aggregate metric. None of tokio + ripgrep + serde have `extern "C"` foreign-module blocks; the phase-a-port has 0.59 per kLOC. This is *expected* for a JS runtime port (Bun has to bind to JavaScriptCore, uWebSockets, zlib, zstd, BoringSSL, libuv, …). It is not by itself evidence of vibe-port quality; it is evidence that Bun is an FFI-heavy program. The baseline is wrong for this dimension.

## Per-file flagging

424 of 1,429 files (29.7%) flag at z ≥ 3 on at least one metric or are unbounded-upward anomalies. Distribution by top-level directory:

| Count | Directory                  | Comment                                                        |
|------:|----------------------------|----------------------------------------------------------------|
|  145  | `src/runtime`              | Mixed: napi, image codecs, FFI shims, runtime APIs             |
|   84  | `src/jsc`                  | JavaScriptCore C-API bindings — legitimate FFI                 |
|   24  | `src/uws_sys`              | uWebSockets C bindings — legitimate FFI                        |
|   10  | `src/http` / `src/bun_core`| Mixed                                                          |
|    9  | `src/io`                   | Mixed                                                          |
|    8  | `src/sql_jsc`, `src/event_loop`, `src/bundler` | Mixed                                  |
|    7  | `src/sys`, `src/string`, `src/http_jsc`, `src/bun_alloc` | Mostly legitimate FFI / low-level   |
|    6  | `src/install`              | Mixed                                                          |
|    5  | `src/ptr`                  | Likely raw-pointer utilities (legitimate)                      |

The top 25 highest-z non-`*_sys` files all turn out, on inspection, to be either FFI bindings (`analyze_jsc`, `javascript_core_c_api`, `napi_body`, `bindings/GeneratedBindings`), C-library wrappers (`backend_wic`, `codec_webp`, `codec_png`), or platform syscalls (`platform/darwin`). No file in the top tier flags as *non-FFI Rust written in a Zig-translated style*. This is the operational result, not a confirmation that no such files exist — the v0.1 baseline cannot distinguish them.

## What this run shows about welch's discriminative reach

**Honest read: welch v0.1 with a non-FFI baseline does not yet discriminate vibe-port quality from legitimate-FFI-port quality at the per-file level.** It correctly flags FFI shims as anomalous against tokio+ripgrep+serde — but FFI shims are *supposed* to be unsafe-heavy in any Rust binding to a C library. The aggregate-level signal (transmutes ≈ 0, unsafe_blocks/raw_pointers only ~1-2σ) is the more usable reading at this baseline configuration: it suggests the target is *not* doing wholesale type-punning across the codebase. That is itself information; it does not, however, certify that the port has succeeded.

To sharpen the discriminator, the next iteration needs:

1. **Multi-tier baseline.** Add FFI-binding crates to the baseline pool — `libc`, `openssl-sys`, `rusqlite`, `libloading`, `jemalloc-sys`, `zlib-rs`, `zstd-sys`. Then FFI-shaped target files compare against an FFI-shaped baseline distribution; non-FFI files compare against the general-purpose baseline.
2. **Per-file FFI classifier.** Tag files containing `extern "..."` blocks as FFI and route them to the FFI baseline; route remaining files to the general baseline. The classifier is already a metric (`extern_blocks > 0`) — wiring it into the comparison is the next step.
3. **Sub-file granularity.** Per-function unsafe density rather than per-file. A 1,000-LOC file with one unsafe FFI binding and 999 lines of safe Rust currently averages out; per-function metrics would surface the unsafe region without diluting it.

## Bug found and fixed in this run

`serde_json` silently writes `f64::INFINITY` as JSON `null` (the spec doesn't have an Infinity literal). Round-tripping the report through Python or jq lost the +inf signal entirely. Fixed in commit by splitting `MetricZScore.z` into `z: Option<f64>` (finite z, or `None` for zero-variance baselines) plus a separate `z_infinite: Option<i8>` carrying the deviation sign. The human-readable summary printer renders `+inf`/`−inf` correctly; structured JSON consumers can detect the unbounded case via `z == null && z_infinite == 1`.

## Files in this run

- `baseline-scan.json` — per-file metrics for 1,085 baseline files.
- `target-scan.json` — per-file metrics for 1,429 phase-a-port files.
- `baseline-summary.json` — per-metric distribution from baseline.
- `anomalies.json` — full anomaly report at z ≥ 3.0 (post-fix shape).
- `summary.txt` — human-readable summary captured from welch's stderr.

## Provenance

- Pipeline: shallow clones of HEAD on each repo, walked by `welch scan`, summarized by `welch baseline`, compared by `welch compare`.
- Reproducibility: pinning the baseline crates to specific commits (rather than HEAD `--depth=1`) is the next iteration's improvement; this run used latest-HEAD on each, which is sufficient for shape-validation but not byte-stable.
- Date of clones: 2026-05-10.
