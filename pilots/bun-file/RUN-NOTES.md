# Bun.file pilot — 2026-05-10

**Eleventh pilot. First Tier-B Bun-namespace pilot.** Per the trajectory's Tier-B priority order. First pilot on a fully Tier-2 ecosystem-only surface — no WHATWG/W3C spec exists; Bun's tests are the authoritative reference. Also the **first pilot with real filesystem I/O**.

## Pipeline

```
v0.13b enriched constraint corpus
  Bun.file: 470+ cross-corroborated clauses across cluster groups
  No spec extract — no spec exists. Bun's tests + Bun docs are the authority.
       │
       ▼
AUDIT.md
       │
       ▼
simulated derivation v0   (CD + Bun docs at bun.sh/docs/api/file-io)
       │
       ▼
derived/src/lib.rs   (95 code-only LOC — composes with rusty-blob from Pilot 4)
       │
       ▼
cargo test
   verifier:            24 tests (real I/O via std::env::temp_dir())
   consumer regression:  8 tests
       │
       ▼
32 pass / 0 fail / 0 skip   ← clean first-run pass
```

## Verifier results: 24/24

```
Construction (2 tests)
  construction does not read; open() alias

Name (1 test)
  CD: file.name returns the path

Exists (2 tests)
  true for real file; false for missing

Size (3 tests)
  CD: matches Buffer.byteLength; zero for empty; unicode byte-length not char-length

Text (3 tests)
  CD: text() reads UTF-8; unicode roundtrip; io::Error on missing file

Bytes / array_buffer (2 tests)
  raw bytes; array_buffer aliases bytes

MIME type (4 tests)
  inferred from .html/.json/.png/.svg; octet-stream for unknown; empty for no-ext;
  explicit overrides inferred

LastModified (1 test)
  ms since epoch, modern timestamp

Slice (3 tests)
  CD: returns Blob (not BunFile); negative offset clamps; content-type override

As-Blob (2 tests)
  CD: instanceof Blob analog preserves size + type; inferred type for HTML

Round-trip (1 test)
  utf-8 with newlines + unicode roundtrip
```

## Consumer regression: 8/8

```
Bun snapshot test pattern (regression #14029)             1
Bun.file size matches Buffer.byteLength (regression #26647) 1
Bun message-channel BunFile instanceof Blob              1
HTTP server static-file Response Content-Type            1
Exists-then-read guard pattern                            1
Chunked upload slice returns Blob                         1
Bundler path as cache key                                 1
Explicit type override for unknown extension              1
```

## LOC measurement: composition-compounding

```
Bun reference (BunFile is implemented within Bun's webcore Blob):
  runtime/webcore/Blob.rs      6,581 LOC   (BunFile + Blob + lazy I/O + S3 + network + path-buffer pool)
  runtime/webcore/Blob.zig     5,155 LOC

Pilot derivation:
  rusty-blob (Pilot 4 substrate)        103 LOC
  rusty-bun-file (this pilot)            95 LOC
  Combined Blob + BunFile pair          198 LOC

Naive ratio (combined pair vs Bun Blob.rs):     3.0%
Adjusted (data-layer scope only — file open +
  size + text + bytes + type + slice + lazy
  read; ~300-500 LOC equivalent slice of
  Blob.rs):                                   20-30%
```

The composition-compounding finding from Pilot 5 (File extends Blob) repeats here: BunFile = 95 LOC because most of the Blob-shape methods delegate to the rusty-blob substrate. The pair (Blob + BunFile) at 198 LOC against a ~6,581-LOC reference is a 3.0% naive ratio — among the strongest in the apparatus, on par with structuredClone's algorithm-only result.

## Findings

1. **AOT hypothesis #1 confirmed.** 95 code-only LOC, on the smaller end as predicted. Composition compounds.

2. **AOT hypothesis #2 confirmed (real-file fixtures).** Tests use `std::env::temp_dir()` + `std::process::id()` for isolation. Each test creates and cleans up its own fixture file. **First pilot with real I/O in the apparatus.** Pattern works; no I/O-specific apparatus refinements needed.

3. **AOT hypothesis #3 confirmed.** `slice()` returning `Blob` (not `Self`) transcribed naturally into Rust's type system, exactly as in the File pilot. The "slice strips File metadata" invariant is enforced at compile time.

4. **AOT hypothesis #4 confirmed.** First-run clean closure. No verifier-caught derivation bugs. The Bun docs + Bun's test corpus together provided a sufficient constraint surface even without a formal spec — this validates Doc 707's three-tier framing for the **Ecosystem tier**: where Bun's tests ARE the spec, the apparatus operates cleanly when the test corpus is dense enough.

5. **The 470+ cross-corroborated clauses on Bun.file** is the second-densest single-cluster cardinality in the corpus (after structuredClone's 166 + 39 + 5 + 3 + 14 = 227). Bun's Bun.file is among the most thoroughly self-tested surfaces in Bun.

## Updated 11-pilot table

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
| **Bun.file** | **Tier-2 Bun-namespace + first I/O** | **95** | **3.0% naive (paired with Blob) / ~20-30% adj** |

Eleven-pilot aggregate: **2,419 LOC** of derived Rust against ~40,000+ LOC of upstream reference targets. **Aggregate ratio: ~6.0%.**

## Trajectory advance

This completes Tier-B item #3 (Bun.file). The next queued items are Tier-B #4 (Bun.serve, the flagship Bun API; data-layer scope) and Tier-B #5 (Bun.spawn). Per the trajectory's resume protocol, Bun.serve is the natural next move because it composes with both fetch-api (already piloted) and Bun.file (just piloted) — the final unblocking for a system pilot of "Bun's HTTP serving layer at data-layer fidelity."

## Files

```
pilots/bun-file/
├── AUDIT.md
├── RUN-NOTES.md              ← this file
└── derived/
    ├── Cargo.toml            (path-dependency on rusty-blob)
    ├── src/
    │   └── lib.rs            (153 LOC, 95 code-only)
    └── tests/
        ├── verifier.rs            24 tests, all pass; uses real temp-file I/O
        └── consumer_regression.rs  8 tests, all pass
```

## Provenance

- Tool: `derive-constraints` v0.13b.
- Constraint inputs: `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/bun.constraints.md` (Bun.file cluster: 470+ clauses).
- Spec input: none — Tier-2 ecosystem-only. Bun's test corpus + Bun docs (https://bun.sh/docs/api/file-io) serve as authoritative reference.
- Substrate: `pilots/blob/derived/` (rusty-blob crate from Pilot 4).
- Reference target: Bun's `runtime/webcore/Blob.rs` (6,581 LOC) — BunFile implementation lives inside.
- Result: 32/32 across both verifier (24) and consumer regression (8). Zero regressions. Zero documented skips. Zero apparatus refinements queued.
