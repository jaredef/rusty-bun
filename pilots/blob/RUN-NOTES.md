# Blob pilot — 2026-05-10

Fourth pilot. Composition class — Blob is the substrate that File extends in the W3C File API. Pilots 1–3 covered data-structure (TextEncoder), delegation-target (URLSearchParams), and algorithm (structuredClone). This pilot tests whether the apparatus handles **container-with-metadata + composition-substrate** surfaces.

## Pipeline

```
v0.13b enriched constraint corpus (Bun: 26 clauses across 9 properties on Blob)
       │
       ▼
AUDIT.md
       │
       ▼
simulated derivation v0   (CD + W3C File API §3 spec extract)
       │
       ▼
derived/src/lib.rs   (103 code-only LOC)
       │
       ▼
cargo test           (26 tests)
       │
       ▼
v0:  25 pass / 1 FAIL / 0 skip   ← verifier catches real derivation bug
       │
       ▼ one-line fix to slice() spec semantics
       │
       ▼
v1:  26 pass / 0 fail / 0 skip   ← apparatus closes the loop
```

## The verifier-caught bug — the load-bearing finding

The first verifier run failed exactly one test: `spec_slice_swapped_endpoints_yield_empty`. The pilot's v0 `slice()` had this code:

```rust
let (lo, hi) = (lo.min(hi), hi.max(lo));
```

This swaps endpoints when `start > end`, producing a slice over the absolute range. The W3C File API §3 spec says otherwise: *"Let span be max(relativeEnd − relativeStart, 0)."* When `relativeEnd < relativeStart`, the span is **0** — the slice is empty, not swapped.

The pilot's derivation had this wrong. The verifier surfaced it on the first run. The fix was one line: replace the swap with `let hi = if hi < lo { lo } else { hi };`. After the fix, 26/26 pass.

**This is the apparatus's value claim demonstrated in real time.** The TextEncoder pilot caught an apparatus bug (cluster-leakage). The URLSearchParams pilot caught a metric-framing finding (delegation targets). The structuredClone pilot caught nothing (the apparatus had hardened sufficiently). The Blob pilot caught a **derivation bug**: an LLM-derivation can plausibly mis-interpret spec wording in a way the constraint corpus alone could not have caught — but the verifier consuming the spec-derived antichain rep does catch it. The cybernetic loop's correctness depends on the verifier being non-trivial; this pilot is the first concrete demonstration that it is.

## Verifier results

```
running 26 tests

cd_blob1_size_from_string_part         ok    ◀ new Blob(["abcdef"]) → size 6
cd_blob1_class_exists                  ok    ◀ typeof Blob !== "undefined"
cd_blob1_byte_equality_roundtrip       ok    ◀ FormData multipart roundtrip
cd_blob2_no_name_property              ok    ◀ Blob has no .name (regression #10178)
cd_blob2_constructor_pattern           ok    ◀ [Exposed=*] global constructor
cd_blob3_empty_parts_constructs        ok    ◀ new Blob([]) instanceof Blob

spec_size_returns_byte_length          ok
spec_type_returns_empty_when_none      ok
spec_type_lowercases_ascii             ok    ◀ "Application/JSON" → "application/json"
spec_type_preserves_non_ascii          ok    ◀ "TEXT/Ω" → "text/Ω"
spec_slice_basic_range                 ok
spec_slice_default_end_is_size         ok
spec_slice_negative_offsets_clamp      ok    ◀ negative offsets resolve to size + offset
spec_slice_end_clamps_to_size          ok
spec_slice_content_type_override       ok
spec_slice_no_content_type_override_clears_to_empty ok
spec_slice_swapped_endpoints_yield_empty ok ◀ ★ verifier-caught-bug fix
spec_text_decodes_utf8                 ok
spec_text_lossy_on_invalid_utf8        ok
spec_array_buffer_returns_full_content ok
spec_bytes_alias_of_array_buffer       ok
spec_constructor_concatenates_string_parts   ok
spec_constructor_mixes_bytes_and_strings     ok
spec_constructor_includes_blob_parts         ok
spec_endings_transparent_preserves_input     ok
spec_endings_native_lf_on_unix         ok

result: 26 passed, 0 failed, 0 skipped
```

## LOC measurement

| Target | LOC |
|---|---:|
| Bun `Blob.rs` | 6,581 |
| Bun `Blob.zig` | 5,155 |
| Pilot derivation `lib.rs` (code-only) | **103** |

Naive ratio against Bun's Rust: **1.6%**. Naive ratio against Bun's Zig: **2.0%**.

Both ratios are misleading: Bun's Blob is *much* more than W3C File API §3. It implements file-backing, lazy I/O, multi-part backing stores, S3-backed Blobs, network-backed Blobs, large-file streaming, FormData integration, and a substantial Bun-specific extension surface. The pilot is a pure-bytes Blob with no backing-store machinery.

A scope-honest comparison: WebKit's `Blob.{h,cpp}` excluding backing-store runs ~300–500 LOC of C++ for the equivalent in-memory-bytes scope. Pilot ratio against that estimate: **20–35%**. This is still well below the htmx 9.4% prior, but for an honest reason — Blob is small enough that the constructor + slice + decode logic is most of what there is to derive.

The accumulating LOC ratio table across four pilots:

| Pilot | Pilot LOC | Reference target LOC | Naive ratio | Adjusted ratio |
|---|---:|---:|---:|---:|
| TextEncoder + TextDecoder | 147 | 1,116 (Bun RS) | 13.2% | 17–25% |
| URLSearchParams | 186 | 348 (Bun delegation glue) | 53% | 62% (vs WebKit) |
| structuredClone | 297 | 7,549 (WebKit C++) | 3.9% | ~8.5% (algorithm-only) |
| Blob (this pilot) | 103 | 6,581 (Bun RS) | 1.6% | 20–35% (in-memory scope) |

The naive numbers are useful to know but consistently mislead — they reflect target-project scope, not derivation scope. The adjusted numbers cluster in the 8.5%–35% range across pilot classes. This is wider than the htmx 9.4% prior suggested, but consistent: **derivation-from-constraints is a multiplier of dramatically less code, with the multiplier varying by what fraction of the target's complexity is the algorithm vs the binding/backing/integration layers.**

## Findings beyond the primary thesis

1. **The Heap-with-Ids architecture from structuredClone is not needed here.** Pilot's Blob is value-semantic — `Vec<u8>` byte storage, `String` mime type, no shared identity, no cycles. Different pilot class, different architecture. AOT hypothesis #4 confirmed: pilot architectures are pilot-class-specific, and the apparatus accommodates this without forcing a uniform shape on derivations.

2. **The structural property `expect(blob.name).toBeUndefined()` (Bun regression #10178) translates to "the type has no name field at all".** In Rust, the type system enforces the absence statically — there is no `.name` accessor to call. The pilot's `cd_blob2_no_name_property` test documents this structurally. JS's "undefined" is what Rust's type system catches at compile time. **This is a finding about pilot translation: some constraint reps map to compile-time invariants in Rust rather than runtime tests.**

3. **The first three AOT hypotheses confirmed:**
   - ✓ Negative-offset clamping for slice required spec material
   - ✓ Endings normalization is platform-specific; pilot defaults to LF
   - ✓ ASCII-lowercasing required spec material

4. **A fifth, unstated AOT hypothesis was implicitly tested and FAILED:** that the LLM-derivation would correctly transcribe spec semantics on first attempt. It did not. It mis-rendered slice's swapped-endpoints behavior. This is the most important finding of the run: **the verifier is what makes the simulation honest**. Without the spec-derived test case, the bug would have shipped silent.

## Implication for the apparatus' value claim

Four pilots in, the cybernetic loop has now closed in three different modes:

- Pilot 1: surfaced apparatus gaps; the loop closure was *forward* (apparatus → fixes → next pilot).
- Pilots 2–3: closed cleanly with no fixes needed; the loop closure was *demonstrated* (apparatus produced sufficient input).
- Pilot 4: closed after a verifier-caught derivation bug; the loop closure was *corrective* (apparatus's verifier caught the derivation engine's mistake).

These three modes are the three faces of a working cybernetic apparatus. Pilots 2–3 are "the easy case." Pilot 1 is "the apparatus learns from the pilot." Pilot 4 is "**the pilot learns from the apparatus**" — the verifier consumed a spec-derived antichain rep that the LLM-derivation hadn't paid attention to, surfaced the deviation, and forced the fix. **Without v0.13's spec ingestion, the rep would not have existed; without the verifier, the bug would have shipped.**

The third mode is the most important to demonstrate at scale. If the apparatus can only catch problems when the test corpus already had them, it's a pretty-printer for what tests already say. The Blob pilot proves that the **spec-source ingestion phase produces test material that surfaces real derivation bugs**. That's what closes the loop on the *correctness* of the derivation, not just the *adequacy* of the input.

## Files

```
pilots/blob/
├── AUDIT.md                  ← coverage audit
├── RUN-NOTES.md              ← this file
└── derived/
    ├── Cargo.toml
    ├── src/
    │   └── lib.rs            ← simulated derivation, 159 LOC (103 code-only)
    └── tests/
        └── verifier.rs       ← 26 tests (6 CD + 20 spec-derived), 100% pass after 1-line fix
```

## Provenance

- Tool: `derive-constraints` v0.13b.
- Constraint inputs: `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/blob.constraints.md` (3 properties / 20 clauses).
- Spec input: `specs/blob.spec.md` + W3C File API §3 (https://w3c.github.io/FileAPI/#blob-section).
- Reference targets: Bun `Blob.rs` (6,581 LOC), Bun `Blob.zig` (5,155 LOC).
- Result: 26/26 verifier closure after 1 derivation-bug fix surfaced by the verifier itself.
