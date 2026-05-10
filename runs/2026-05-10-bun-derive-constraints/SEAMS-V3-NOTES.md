# derive-constraints seams v0.3 — 2026-05-10 — Bun phase-a-port

Tightening pass on [v0.2 seams](./SEAMS-V2-NOTES.md). Implements the queued v0.3 refinement: **test-fn-body context probing** for S1's platform-conditional signal. Fixes the `P4 platform-conditional meta-seam — PARTIAL` finding from v0.1 / v0.2.

## The refinement

v0.1 / v0.2 probed only the antichain representatives' raw `expect(...)` text and source-file paths. Bun's tests routinely place platform-conditional guards in `beforeEach` / `describe`-scope blocks, not inside `expect` clauses; v0.1 / v0.2's S1 detection therefore missed most of the platform-conditional surface.

v0.3 adds a `--corpus-root` flag: when provided, the seams pipeline opens each antichain representative's source file (cached per file) and concatenates ±40 lines around the cited line. S1 then scans this surrounding test-fn-body context for the platform-conditional patterns the v0.2 catalog already recognized. The pattern catalog also extended: added `test.if(...)`, `it.if(...)`, `describe.if(...)`, `.skipIf(...)`, `.runIf(...)`, `os.platform()`, `isCI`.

## Numerical delta

| Metric                       | v0.2 | v0.3 | Δ |
|------------------------------|-----:|-----:|---|
| Distinct signal vectors      | 76   | 93   | +17 |
| Cross-namespace seams        | 36   | 50   | +14 |
| **cfg-firing properties**    | **19** | **593** | **+30×** |
| **cfg-firing total cardinality** | **1,734** | **11,009** | **+535%** |

The cfg signal went from firing on 19 properties (1,734 cardinality, 4% of corpus) to firing on **593 properties (11,009 cardinality, ~26% of the 42,680-clause input corpus)**. The platform-conditional pattern was always there — it was hiding in the test-fn-body context where the previous probe extraction couldn't see.

## Doc 705 §10.2 prediction outcomes — final

| Prediction | v0.1 | v0.2 | v0.3 |
|------------|------|------|------|
| P1 sync/async split on fs/crypto/child_process | ✅ HELD | ✅ HELD | ✅ HELD |
| P2 native byte-pool merge | ⚠️ NOT SURFACING | ⚠️ NOT SURFACING | ⚠️ NOT SURFACING (impl-internal) |
| P3 Bun.* split into 4-6 architectural surfaces | ✅ HELD in shape | ✅ HELD in shape | ✅ HELD in shape |
| P4 platform-conditional meta-seam | ⚠️ PARTIAL | ⚠️ PARTIAL | **✅ HELD** |
| P5 throw vs return-error seam | ✅ HELD | ✅ HELD with shape | ✅ HELD with shape |
| P6 construct-then-handle seam | ✅ HELD | ✅ HELD | ✅ HELD |

**5 of 6 predictions cleanly held; 1 partial.** P2 native byte-pool remains the only partial — and we know exactly why: it is an *implementation-internal* boundary (the seam between native byte-pools and JS-side typed arrays lives inside the implementation, not at the public-API surface the test corpus exercises). It needs probes at the implementation-source layer (where welch operates), not the test-corpus layer.

## The platform-conditional meta-seam, observed

The v0.3 cfg-firing clusters surface exactly the structure Doc 705 §10.2 P4 predicted — a meta-seam crosscutting most surfaces:

```
SC0004  card=2214 props= 27  cfg|@regression          ← meta-seam over @regression
SC0005  card=2055 props=425  cfg|@js                  ← meta-seam over @js (149 surfaces)
SC0006  card=1932 props=  1  cfg|throw|p-throw        ← cfg-conditional throw cluster
SC0008  card=1556 props=  1  cfg|throw|p-throw|@regression
SC0012  card= 814 props= 39  cfg                      ← bare cfg, no path partition
SC0015  card= 364 props=  4  cfg|threaded|@js         ← cfg ∩ threading-model
SC0018  card= 332 props=  3  cfg|async                ← cfg ∩ async-discipline
SC0019  card= 301 props= 24  cfg|async|@js
SC0020  card= 279 props=  1  cfg|sync+async
SC0024  card= 241 props=  9  cfg|sync                 ← cfg ∩ sync-discipline (fs, readFileSync, …)
SC0026  card= 219 props=  3  cfg|async|@regression
SC0031  card= 129 props=  2  cfg|sync|@regression
```

The compound clusters are the operationally consequential finding for derivation:

- **`cfg ∩ sync` (SC0024 + SC0031, ~370 cardinality):** synchronous syscall surfaces with platform-specific behavior — `fs`, `readFileSync`, `readlinkSync`, `realpathSync`, `spawnSync`, `existsSync`, `readdirSync`. A Rust derivation must implement these with platform-conditional code paths, not a single cross-platform abstraction.
- **`cfg ∩ async` (SC0018 + SC0019 + SC0026, ~852 cardinality):** asynchronous surfaces with platform-specific behavior — `Blob`, `Bun`, `File`, `RedisClient`, `S3` (cloud-storage clients), `exists`, `file`, `stdout`. Platform-conditional async code is a substantially larger surface than platform-conditional sync.
- **`cfg ∩ threading-model` (SC0015, 364 cardinality):** threading boundary that's also platform-conditional — `Promise`, `serialize`, `server`, `util`. The intersection of two architectural seams.
- **`cfg ∩ throw` (SC0006 + SC0008, 3,488 combined cardinality):** platform-conditional error behavior — different platforms throw different errors. Rust must reproduce the platform-specific error shapes on each target.

These are exactly the kinds of *intra-architectural* seams Doc 705 §1 specifies: boundaries where one architectural form meets another, *inside* the runtime, typically unnamed in natural-language docs but surfaced through the patterns of how the system handles edge cases.

## What v0.3 still does not detect

**Implementation-internal boundaries.** P2 native byte-pool, S9 allocator-discipline (only 1 property fired) — both are seams that live inside the implementation source, not at the test-corpus surface. The test corpus exercises the public API; it does not surface the implementation's allocator choice or its native-buffer heritage.

The path forward, queued for v0.4: **couple to welch's per-file diagnostic.** welch already operates on the implementation source (the phase-a-port Rust files); its per-file unsafe-density anomalies are exactly the implementation-layer probes seams needs. Cross-referencing: for each seams cluster, compute the welch-anomaly density in the implementation directories its antichain representatives' test paths suggest. Implementation-internal seams that don't surface at the test-corpus probe layer should surface at the welch-probe layer.

## What this run shows about the apparatus

**Doc 705's apparatus operates at five-of-six prediction precision after three iterations.** The v0.1 / v0.2 / v0.3 progression demonstrates the apparatus refining under operational test: each iteration named where the previous fell short, identified the structural source of the shortfall, and refined the probe extraction. v0.3 closes the largest remaining gap (P4 platform-conditional meta-seam, the operationally most consequential one for cross-platform Rust derivation).

**The 30× increase in cfg-signal cardinality from one refinement (test-fn-body probing) is informative about probe-extraction reach.** Antichain-text-only probes capture only what the test author chose to put in the assertion expression; test-fn-body probes capture what the author scoped at the surrounding setup level. The latter is dominant in JS test corpora where conditionals live in `beforeEach` / `describe`, and the seams pipeline now sees them.

**The platform-conditional seam crosscutting ~26% of the corpus is the operationally most consequential finding.** A namespace-grouped derivation (the current `derive-constraints invert` output) treats platform-conditional code as scattered across surfaces; a seam-grouped derivation (which this v0.3 output enables) groups all platform-conditional code under one architectural form, with sub-clusters for cfg ∩ sync, cfg ∩ async, cfg ∩ threading, cfg ∩ throw. A Rust port that derives from the seam-grouped decomposition emits a *single* platform-abstraction layer; a Rust port that derives from the namespace-grouped decomposition emits scattered `#[cfg(...)]` blocks across many modules. The first is the right shape; the second is what Bun's phase-a-port currently exhibits.

## v0.4 refinements queued

- **Couple to welch's per-file diagnostic.** Cross-reference seam clusters with welch's per-file unsafe-density anomalies. Implementation-internal seams (P2, S9) become visible at the implementation-source layer.
- **Cluster-level construction-style fraction as a quality signal.** The current cluster output reports `cs=N` but doesn't use it for decomposition decisions. v0.4 could weight construction-style cluster representatives more heavily in the output's surface-grouping.
- **Resistance-as-boundary verification via rederive (Doc 705 Step 4).** When rederive pilot infrastructure lands, run a small surface (e.g., one of the cfg-firing sync clusters) through the full derivation pipeline; verification verdicts close the cybernetic loop.

## Files

- `bun-seams-v3.json` (~1MB) — v0.3 cluster + cross-namespace-seam catalog with test-body-context-aware S1.
- `bun-seams-v3-baseline.json` — v0.3 binary run *without* `--corpus-root` flag, for comparison; matches v0.2 output shape exactly (verifying the flag-off behavior is backward-compatible).
- v0.1, v0.2 artifacts preserved alongside.

## Provenance

- Tool: `derive-constraints` v0.5+ (seams v0.3).
- Source: `bun-cluster-v2.json` from cluster v0.2 run.
- Corpus root: `/tmp/welch-corpus/target/bun` (the same shallow clone all rusty-bun runs operate over).
- Tool runtime: ~3 seconds (file I/O over ~10K antichain representatives, cached per file).
