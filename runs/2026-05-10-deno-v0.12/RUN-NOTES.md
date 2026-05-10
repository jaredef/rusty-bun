# Deno v0.12 pipeline run — 2026-05-10

First v0.12 hardening pass. Fixes the cluster-phase subject-attribution leakage finding from the [TextEncoder pilot](../../pilots/textencoder/RUN-NOTES.md). This run validates that the fix actually closes the leak end-to-end on the Deno corpus.

## The fix

Two-part bug, two-part fix in `derive-constraints/src/extract/ts_js.rs`:

**Part 1 — `capture_bindings` was capturing too liberally.** Any `const X = ...` form became a candidate for binding substitution, regardless of what the RHS represented. Computed booleans like `const headerEnd = new TextDecoder().decode(buf).indexOf("\\r\\n\\r\\n") > 0` would record `headerEnd → <chain> > 0` as a binding, causing later references to `headerEnd` to splice the entire boolean expression in.

Fix: a new `is_simple_call_chain` predicate that accepts only RHS values matching `(await|new|typeof )* identifier ( [.member | (args) | [idx] ])*` — pure call/member chains, no top-level binary operators or comparisons.

**Part 2 — `resolve_binding` was substituting through method-call tails.** Even after Part 1, expressions like `assert(response.startsWith("HTTP/1.1"))` where `response` was bound to `new TextDecoder().decode(buf)` would substitute to `new TextDecoder().decode(buf).startsWith("HTTP/1.1")` and canonicalize to `TextDecoder` — but `.startsWith()` is a String method, not a TextDecoder property. The substitution shifted the assertion's architectural identity from String-result-of-decode to TextDecoder-itself.

Fix: in `resolve_binding`, refuse substitution when the tail after the bound head contains a `(` — i.e., a method call. Bare-identifier substitution (`expect(server)` → `expect(Bun.serve(...))`) and pure-getter-chain substitution (`expect(server.url)` → `expect(Bun.serve(...).url)`) remain valid; method-call tails are preserved as-is.

## Validation

The TEXT2 false-positive group on `TextDecoder` that the [TextEncoder pilot AUDIT](../../pilots/textencoder/AUDIT.md#what-the-auto-emitted-constraint-docs-say) named is **gone**:

```
v0.11 (broken):                          v0.12 (fixed):
─────────────────────                    ─────────────────────
TextDecoder properties: 2                TextDecoder properties: 1
  TEXT1 cardinality=69 (legit)             TEXT1 cardinality=61 (legit)
  TEXT2 cardinality=13 (false-positive)    [TEXT2 removed entirely]
```

The TEXT1 cardinality dropped 69→61 because 8 of the 69 reps were themselves false-positives substituted through the bug — they're now correctly attributed to their actual subjects (e.g., `response`, `body`, `data`).

## Aggregate impact on the Deno corpus

| Metric | v0.11 | v0.12 | Δ |
|---|---:|---:|---|
| total clauses | 11,399 | 11,399 | unchanged (extraction step is upstream of the fix) |
| properties | 1,852 | **2,130** | **+15.0%** — de-leaked subjects no longer collapse onto contaminated bindings |
| construction-style | 50 | 46 | −8% — fewer false architectural attributions |
| signal-vector clusters | 28 | 28 | ≈ |
| cross-namespace seams | 15 | 13 | −13% (slight rebalancing as some seam-pattern attributions shift) |
| welch surfaces | 110 | 97 | −12% (fewer subjects now match against welch's filename surfaces — some matches were spurious) |
| welch mismatches | 10 | 9 | unchanged structural class |
| pipeline runtime | 3.31s | 3.41s | +3% (extra `is_simple_call_chain` walk) |

The +15% properties number is the load-bearing finding. **278 properties had been silently merged onto contaminated subjects.** They are now distinguishable. This is the order-of-magnitude impact the keeper conjectured small fixes would have at scale.

## Honest accounting

- **The construction-style drop (50→46)** is a minor honesty cost: the four properties that lost construction-style status were all artifacts of the substitution bug — they were classified construction-style because the substituted subject named a public-API class (`TextDecoder`, `Headers`, etc.) but the assertion was actually about the *result* of a value-extraction call on that class. Removing them is a precision improvement.
- **The welch surface count drop (110→97)** is similar: 13 surfaces were matching only because a contaminated subject happened to share a name component with a welch filename. Those matches were spurious.

## Implication for future pilots

The `URLSearchParams` pilot — when it lands — will benefit because:
1. URLSearchParams is constructed via `new URLSearchParams(...)` and frequently bound (`const params = new URLSearchParams(...)`). Test patterns commonly assert on `.has("k")`, `.get("k")`, `.toString()`. Each of these is a method call, so under v0.11 the binding substitution would falsely attribute every `params.has("k")` assertion to URLSearchParams (which is technically correct but loses signal).
2. Under v0.12, `.has(...)` calls preserve `params` as the subject; constructor antichain reps show the binding chain explicitly. The constraint doc will distinguish "URLSearchParams as constructor surface" from "URLSearchParams.has as method surface" cleanly.

This is the compounding-at-scale property the keeper conjectured: a single static-analysis fix at the extractor level corrects every downstream constraint doc emitted by the pipeline. The TextEncoder pilot surfaced one instance; the v0.12 run shows the fix corrects 278 instances on the Deno corpus alone.

## Output summary

```
[1/8] scan          1,263 files, 6,228 tests, 11,399 clauses
[2/8] welch impl    940 files, 0 parse failures
[3/8] welch base    1,255 files, 0 parse failures
[4/8] welch summary 6 metrics over 329,409 LOC baseline
[5/8] welch compare implementation-source z-anomalies
[6/8] cluster       2,130 properties (46 construction-style)
[7/8] invert        namespace-grouped + by-seams
[8/8] seams+couple  28 signal clusters, 13 cross-namespace seams; 97 surfaces, 9 mismatches

runtime: 3.41s
```

## Provenance

- Tool: `derive-constraints` v0.12 (commit pending).
- Target: `denoland/deno` HEAD shallow clone.
- Baseline: `tokio` + `hyper` + `reqwest` + `serde` + `ripgrep` shallow clones.
- Companion run: `runs/2026-05-10-deno-v0.11/` — pre-fix counterpart for direct comparison.
- Pilot referencing this fix: `pilots/textencoder/AUDIT.md` (where the bug was first surfaced).
