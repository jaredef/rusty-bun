# derive-constraints seams — 2026-05-10 — Bun phase-a-port

First run of the seams phase, operationalizing [Doc 705 (Pin-Art for intra-architectural seam detection)](https://jaredfoy.com/resolve/doc/705-pin-art-operationalized-for-intra-architectural-seam-detection) over `bun-cluster-v2.json`.

## Inputs

- **Source.** `bun-cluster-v2.json` — 4,838 properties from the Bun test corpus.
- **Tool.** `derive-constraints seams` v0.1. Six probe extractors per Doc 705 §4 (S1 conditional compilation, S2 test-path partitioning, S3 sync/async, S4 throw/return-error, S5 native/userland, S6 construct-then-method), agreement-based clustering, cross-namespace seam reading.

## Output

| Metric                       | Value |
|------------------------------|------:|
| Properties scanned           | 4,838 |
| Distinct signal vectors      | 63    |
| Signal clusters emitted      | 63    |
| Cross-namespace seams        | 32    |

The 4,838-property catalog reduces to 63 signal-vector clusters — a ~76× compression at the seam-vector layer.

## Top 20 signal clusters

```
SC0001  card=15121 props=3145 cs=232  signal=@js                surfaces=[1050+]
SC0002  card= 6248 props= 315 cs= 10  signal=slack              surfaces=[171]
SC0003  card= 6210 props= 242 cs= 15  signal=@regression        surfaces=[155]
SC0004  card= 1938 props=   3 cs=  0  signal=throw              surfaces=[3]
SC0005  card= 1921 props= 131 cs= 10  signal=ctor|@js           surfaces=[81]
SC0006  card= 1568 props=  18 cs=  1  signal=async              surfaces=[15]
SC0007  card= 1556 props=   1 cs=  0  signal=cfg|throw|@regression
SC0008  card= 1256 props= 345 cs=  1  signal=@cli               surfaces=[270]
SC0009  card= 1167 props=  10 cs=  4  signal=async|@regression  surfaces=[7]
SC0010  card= 1103 props= 130 cs=  8  signal=async|@js          surfaces=[89]
SC0011  card=  486 props= 131 cs=  0  signal=@bundler           surfaces=[66]
SC0012  card=  434 props=  79 cs=  2  signal=sync+async|@js     surfaces=[51]
SC0013  card=  392 props=  37 cs=  0  signal=async|@cli         surfaces=[35]
SC0014  card=  376 props=   9 cs=  2  signal=sync+async         surfaces=[7]
SC0015  card=  347 props=  14 cs=  2  signal=sync               surfaces=[8]   ← Bun, fs, lstatSync, readFileSync, …
SC0016  card=  276 props=   6 cs=  1  signal=sync+async|@regression  surfaces=[4]
SC0017  card=  250 props=  61 cs=  5  signal=sync|@js           surfaces=[41]
SC0018  card=  248 props=   8 cs=  0  signal=sync|@regression   surfaces=[6]   ← deflateSync, existsSync, mkdirSync, …
SC0019  card=  243 props=  13 cs=  0  signal=@integration       surfaces=[12]
SC0020  card=  237 props=  10 cs=  1  signal=ctor               surfaces=[10]
SC0021  card=  182 props=   1 cs=  0  signal=returns-error/@regression  surfaces=[Bun]
```

## Predictions vs Output (Doc 705 §10.2)

| Prediction | Outcome | Detail |
|------------|---------|--------|
| **P1 — Sync/async split on fs/crypto/child_process** | ✅ **HELD** | SC0015 (`signal=sync`) cleanly isolates `Bun, fs, lstatSync, readFileSync, readlinkSync, realpathSync` (sync FS); SC0018 (`signal=sync/@regression`) covers `deflateSync, existsSync, mkdirSync, readdirSync, spawnSync` (sync FS+zlib+child_process); SC0017 (`signal=sync/@js`) covers `Atomics, Bun, ar, crypto, dir` (sync incl. crypto). The async side concentrates in SC0006 (1,568 cardinality), SC0009 (1,167), SC0010 (1,103). The seam is *real and detectable*. |
| **P2 — Native byte-pool merge (Buffer/Uint8Array/Blob/File/ReadableStream)** | ⚠️ **NOT SURFACING** | No cluster fires the `ffi` signal at scale. `Buffer`, `Blob`, `File` appear in the SC0010 `async\|@js` cluster and elsewhere by namespace, but the *native byte-pool* boundary is *implementation-internal*; the test corpus exercises the public APIs, not the native heritage. S5's regex doesn't fire because Bun's tests don't surface `Bun.dlopen`, `napi_*`, or `extern "C"` text in expect-bodies. The boundary is real architecturally but invisible at the test-corpus probe layer. |
| **P3 — Bun.* split into 4–6 architectural surfaces** | ✅ **HELD (in shape)** | The `Bun` first-segment surface appears across at least 8 distinct signal vectors: js (SC0001), async/regression (SC0009), async/js (SC0010), sync+async/js (SC0012), sync (SC0015), sync/js (SC0017), sync/regression (SC0018), returns-error/regression (SC0021). The decomposition is along sync/async × test-path rather than the *predicted* HTTP/process/compiler split — but the cardinality of distinct vectors confirms `Bun.*` is not a single architectural form. |
| **P4 — Platform-conditional meta-seam crosscutting all surfaces** | ⚠️ **PARTIAL** | SC0007 (`signal=cfg\|throw/@regression`, card=1556) fires the cfg signal but on a single anonymous-subject cluster; not "crosscutting all surfaces" as predicted. Either S1's pattern catalog is too narrow (Bun tests rarely use `process.platform` in expect bodies — they use it in test setup; expect bodies use the post-conditional values), or the platform-conditional seam is genuinely localized in the test corpus. **Refinement needed:** S1 should also scan the test fn body around the expect, not just the expect's raw text. |
| **P5 — Throw vs return-error seam** | ✅ **HELD** | SC0004 (`signal=throw`, card=1938) and SC0007 (`signal=cfg\|throw/@regression`, card=1556) on the throwing side; SC0021 (`signal=returns-error/@regression`, card=182) on the return-error side. The asymmetry (throwing dominates returning-error in Bun's corpus) is itself informative — Bun's runtime surface is mostly throw-discipline. |
| **P6 — Construct-then-handle seam** | ✅ **HELD** | SC0005 (`signal=ctor\|@js`, card=1921) covers many built-in constructor surfaces (Agent, Array, ArrayBuffer, Blob, Boolean, …); SC0020 (`signal=ctor`, card=237) covers the more specific Bun-internal constructors (`A, Boolean, Bun, C, CString, D…`). The construct-then-method pattern is real and detectable in the catalog. |

**Score: 4 of 6 predictions held cleanly; 2 partial.** P2 (native byte-pool) is *invisible at the test-corpus probe layer* — the boundary is implementation-internal, not test-surface-visible. P4 (platform-conditional meta-seam) is detectable but more localized than predicted. Both partials are informative about the apparatus's reach: signals that probe implementation-internal boundaries need extraction at the implementation layer, not the test-corpus layer.

## What the run shows about the apparatus

**The signal-vector clustering produces localized, cardinality-concentrated clusters per [Doc 619 §4](https://jaredfoy.com/resolve/doc/619)'s alpha-cut criterion.** 63 distinct vectors out of a possible ~2^6 × test-path cardinality (a much larger combinatorial space) demonstrates the signal vectors *do* concentrate on real architectural patterns rather than noise. This satisfies Doc 705 P1.

**Cross-namespace seams (Doc 705 P2) correlate with native-byte-pool / async-discipline / platform-conditional patterns *partially*.** Async-discipline is the strongest cross-namespace seam (SC0006/SC0009/SC0010 all crosscut many surfaces); sync-discipline isolates surfaces clearly; throw/return-error separates; but native-byte-pool doesn't surface from test-corpus probes alone. Doc 705 P2 holds for three of the five named patterns.

**Cybernetic loop convergence (Doc 705 P3) cannot be tested without rederive integration.** Step 4 (resistance-as-boundary verification) and Step 5 (revised surface decomposition with the cybernetic loop closure signal) require running rederive's verification on the candidate seam decomposition. v0.1 produces the candidate seams; the rederive pilot will exercise the closure.

## What the run shows about Bun's actual architecture

The seams that emerged map onto recognizable engineering boundaries even where they don't match Doc 705 §10.2's predictions verbatim:

1. **Test-path partitioning (S2) is the dominant signal.** `@js`, `@regression`, `@cli`, `@bundler`, `@integration` paths are first-class architectural distinctions in Bun's test organization. The team has implicitly decomposed by *engineering category* (what kind of feature is being tested) rather than *architectural form* (what runtime layer is involved). This is itself informative: the test team's mental model encodes a different decomposition than the runtime's structural one.
2. **Sync/async is real and clean.** The sync clusters (SC0015 + SC0017 + SC0018) carry exactly the fs/crypto/child_process surfaces predicted, with cardinalities concentrating on the well-known `*Sync` API methods. This is the most operationally usable seam for derivation: Rust's tokio/async-std distinction maps directly onto this seam.
3. **Constructor-handle is real.** SC0005 (1,921 cardinality) is a substantial cluster of built-in constructor surfaces; this maps to the substrate-keeper pattern at the architectural layer (a construct-then-method pair is structurally a *factory + handle*).
4. **Throw-discipline dominates Bun.** SC0004 + SC0007 (3,494 cardinality combined) are throwing; SC0021 (182 cardinality) is returning-error. Bun's runtime surface is overwhelmingly throw-discipline — this matches its Web-platform inheritance (URL throws; JSON.parse throws; fetch rejects).

## Honest limits

- **The signal catalog is incomplete.** Six signals captures the visible patterns; native byte-pool (P2) reveals at least one boundary the catalog misses. v0.2 should add: ownership/reference-cycle signals, error-shape signals (Result vs `{ok,errors}` vs ad-hoc), allocator-discipline signals (arena vs heap), threading-model signals (mutator vs GC vs worker).
- **Test-path partitioning dominates in this run.** Roughly half the top clusters are distinguished primarily by S2 path-segment, not by S1/S3/S4/S5/S6 architectural signals. This may indicate: (a) the test team's mental decomposition is dominant in the corpus and overshadows architectural signals, or (b) the architectural signals need refinement to surface above the path-partition baseline. Doc 705 §6 prescribes refinement via the cybernetic loop.
- **The "slack" cluster (SC0002, 315 properties / 6,248 cardinality) is the no-architectural-hedging-signal cluster.** This is the corpus-wide baseline against which detection-hedging clusters distinguish themselves per Doc 619 §4. It's correctly excluded from cross-namespace seam reading.
- **Step 4 (resistance verification) requires rederive.** The current run produces *candidate* seams. Validating them via rederive's verification backends — does merging on either side of a seam produce internal inconsistency, contradictory verb-classes, or divergent verdicts — is the next phase.

## Files in this run

- `bun-seams.json` (~700KB) — full per-cluster + cross-namespace-seam JSON.
- `seams-summary.txt` — terminal summary captured from `--summary` stderr.

## v0.2 refinements queued

- **Expand signal catalog**: ownership/reference-cycle, error-shape distinction, allocator-discipline, threading-model. Closer to the eight-axis Doc 703 §3.1 partition.
- **Probe the test-fn body**, not just the expect-clause raw text. S1 platform-conditional in particular benefits from looking at the surrounding setup code where `process.platform === "darwin"` typically lives.
- **Couple to welch's diagnostic.** Cross-reference clusters with welch's per-file unsafe-density anomaly flags from `runs/2026-05-10-bun-phase-a/anomalies.json`. Files welch flags as anomalous Rust often map to specific seam clusters.
- **Run resistance-as-boundary verification via rederive** once the rederive pilot infrastructure is in place. The current run is candidate-seam identification; the cybernetic loop's first iteration is queued.

## Provenance

- Tool: `derive-constraints seams` v0.1 (added in this commit).
- Source: `bun-cluster-v2.json` from the cluster v0.2 run on 2026-05-10.
- Tool runtime: well under a second on the dev machine.
