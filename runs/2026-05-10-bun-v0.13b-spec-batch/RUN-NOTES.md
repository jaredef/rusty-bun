# Bun + specs pipeline run — 2026-05-10

Production-corpus comparative validation. The v0.13b apparatus (cluster-leakage-fix + spec ingestion + 15-surface spec corpus) runs against Bun's full test corpus alongside the same spec extracts, producing the densest cross-corroborated constraint corpus the apparatus has yet measured.

This validates that the apparatus's value at scale — Bun is the AI-translated corpus where pilots ultimately need to operate.

## Headline numbers

| Metric | Deno + specs (v0.13b) | Bun + specs (this run) | Ratio |
|---|---:|---:|---:|
| Test files scanned | 1,278 | 4,485 | 3.5× |
| Total clauses | 11,690 | 42,971 | 3.7× |
| Total properties | 2,310 | 6,163 | 2.7× |
| Construction-style | 79 | 337 | 4.3× |
| Spec-only properties | 180 | 168 | ≈ |
| **Cross-corroborated** | **11** | **23** | **2.1×** |
| Pipeline runtime | 3.74s | 9.31s | 2.5× |

The pipeline scales linearly-ish with corpus size. The cross-corroboration count grows roughly with property count, not just corpus size — Bun's tests are denser per file than Deno's, so the cross-corroboration ratio is higher in absolute terms but slightly compressed relative to property count.

## Tier-1 cross-corroborated surfaces (top 23)

```
card=  166  cs=construction  structuredClone        ← strongest single-surface coverage
card=   73  cs=construction  Response
card=   24  cs=behavioral    atob
card=   22  cs=construction  File
card=   19  cs=construction  AbortController
card=   18  cs=behavioral    crypto.randomUUID
card=   17  cs=construction  Blob
card=   14  cs=construction  URLSearchParams       ← upgraded to cs (was behavioral on Deno)
card=   10  cs=construction  Headers               ← upgraded to cs (was behavioral on Deno)
card=    9  cs=behavioral    Response.json
card=    8  cs=construction  Response.redirect
card=    7  cs=construction  FormData
card=    5  cs=construction  Request
card=    5  cs=construction  structuredClone       (a second cluster entry)
card=    5  cs=behavioral    Response
card=    4  cs=construction  AbortSignal.abort
card=    4  cs=construction  Response.error
card=    4  cs=behavioral    URL.canParse
card=    4  cs=behavioral    crypto.getRandomValues
card=    3  cs=construction  AbortSignal.any
card=    2  cs=construction  Blob
card=    2  cs=construction  TextEncoder           ← cross-corroborates for first time
card=    2  cs=behavioral    AbortSignal.timeout
```

## What Bun's corpus surfaces that Deno's didn't

**1. `structuredClone` exhibits 14× cardinality jump (12 → 166).** Bun's test suite exhaustively probes structuredClone semantics — circular references, supported types, transfer, identity preservation. This is a candidate for *the most-thoroughly-witnessed property in the apparatus*. Any future structuredClone derivation pilot has overwhelming constraint coverage to draw on.

**2. URLSearchParams's class upgrades behavioral → construction-style on Bun.** Deno's tests primarily exercised URLSearchParams.toString as behavioral. Bun's tests include construction-style assertions like `expect(typeof URLSearchParams).toBe("function")`, `expect(new URLSearchParams() instanceof URLSearchParams).toBe(true)`, etc. The construction-style class becomes the dominant attribution on Bun's corpus.

**3. Headers similarly upgrades behavioral → construction-style on Bun.** Same pattern.

**4. TextEncoder cross-corroborates for the first time.** Cardinality 2 — small, but real. The TextEncoder pilot's AUDIT.md documented 6 test-corpus clauses; running on Bun's denser corpus surfaces 2 of them as construction-style assertions that combine with the spec extract. The earlier pilot's 0-cross-corroboration had been a measure of Deno's test-coverage gap, not the apparatus's gap.

**5. Request appears at cardinality 5 (cs=construction).** Did not cross-corroborate at all on Deno. Bun's Request tests include construction-style assertions matching the spec extract.

**6. Response.error and AbortSignal.any appear cross-corroborated.** These are static methods on web-platform classes; Bun's corpus exercises them while Deno's didn't. The spec extracts captured them; Bun's tests independently witnessed them.

## Construction-style ratio

```
                    Bun + specs   Deno + specs
construction-style:     337 / 6163 = 5.5%       79 / 2310 = 3.4%
```

Bun's corpus has roughly 1.6× the construction-style ratio of Deno's. Hand-written-vs-AI-translated does NOT explain this — Deno is hand-written and has the lower ratio. The likely explanation is **test-author style**: Bun's tests use Jest-style `expect(...).toBeFunction()` / `.toBeInstanceOf()` / etc. assertions that match the construction-style classifier's structural-equivalence-value heuristic; Deno's tests use `assertEquals(value, expected)` style that requires the value-side to be a structural literal to qualify, and most Deno assertions compare runtime values to runtime values.

This is an apparatus finding about test-style sensitivity in the construction-style classifier. **It's not a bug** — both styles are legitimate; both produce useful signal — but cross-corpus comparison should normalize for test-style before drawing conclusions about architectural classifications.

## Implication for pilot scaling

Three pilot targets now have *Bun-corpus cross-corroborated* coverage at >10 cardinality:

```
structuredClone   166  ← extremely thorough; could be next pilot
Response           73  ← richest fetch-API surface; medium-large pilot
atob               24  ← small pilot, fully spec'd, simple
File               22
AbortController    19
crypto.randomUUID  18
Blob               17
URLSearchParams    14  ← already piloted (62% LOC ratio against WebKit)
Headers            10
```

The atob/btoa pair is a candidate for an *even smaller* MVE than URLSearchParams — ~20-30 LOC of derivation against a fully Base64-spec'd surface, with 24 cross-corroborated cardinality on Bun. Could be a 1-2 hour pilot.

The structuredClone surface is the most-thoroughly-witnessed in the corpus. A successful structuredClone derivation pilot would be the apparatus's strongest empirical anchor — it's an algorithm, not a data structure, and its test coverage in Bun is exhaustive.

## What this run does NOT prove

- **It does not prove the apparatus produces correct constraint docs at scale.** Cross-corroboration is a *coverage* signal, not a *correctness* signal. The constraint docs may still contain false-attribution noise that v0.12 missed; only piloted derivation can verify.
- **It does not prove the apparatus scales linearly beyond Bun.** Bun is the largest corpus we've measured. Scaling to (e.g.) Node.js's full test corpus would test the linearity claim.
- **It does not prove the construction-style ratio is meaningful as a cross-project signal.** As noted, test-style asymmetry confounds the comparison.

## Output summary

```
[1/8] scan          4,485 files, 19,237 tests, 42,971 clauses
      + specs (15 files): 291 clauses
[2/8] welch impl    Bun's full Rust source
[3/8] welch base    1,255 files (tokio + hyper + reqwest + serde + ripgrep)
[4/8] welch summary 6 metrics over 329,409 LOC baseline
[5/8] welch compare implementation-source z-anomalies
[6/8] cluster       6,163 properties (337 construction-style)
[7/8] invert        namespace-grouped + by-seams
[8/8] seams+couple  100 signal clusters, 46 cross-namespace seams; 289 surfaces, 17 mismatches

runtime: 9.31s
```

## Provenance

- Tool: `derive-constraints` v0.13b (commit 6a018b6 + apparatus state through URLSearchParams pilot).
- Target: `oven-sh/bun` HEAD shallow clone at `/tmp/welch-corpus/target/bun`.
- Specs: `/home/jaredef/rusty-bun/specs/` — 15 files, 291 clauses.
- Baseline: `tokio` + `hyper` + `reqwest` + `serde` + `ripgrep`.
- Companion runs: `runs/2026-05-10-deno-v0.13b-spec-batch/` for direct comparison.
