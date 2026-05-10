# URLSearchParams pilot — 2026-05-10

Second pilot in the rusty-bun apparatus. The first ([TextEncoder](../textencoder/)) operated on the auto-emitted constraint doc *plus manual spec injection* and surfaced three v0.12/v0.13 work items. This pilot exercises the **post-v0.13b enriched constraint corpus** — the apparatus now provides the spec material as part of its normal output.

## Pipeline

```
auto-emitted constraint corpus (v0.13b: tests + 15 spec extracts)
       │
       ▼
AUDIT.md      ← richer than TextEncoder pilot's input — see §"What v0.13b enables"
       │
       ▼
simulated derivation (LLM)         (CD + WHATWG URL §5.2 + §5.2.5)
       │
       ▼
derived/src/lib.rs   (186 code-only LOC)
       │
       ▼
cargo test           (32 tests: 26 CD-derived + 6 spec-derived edges)
       │
       ▼
32 pass / 0 fail / 0 skip          ← apparatus's first 100% verifier closure
```

## Verifier results

```
running 32 tests

cd_cross_tostring_space_to_plus              ok   ◀ cross-corroborated rep
cd_cross_tostring_percent_encoding           ok   ◀ cross-corroborated rep

cd_construct_empty                           ok
cd_construct_from_query_with_leading_question ok
cd_construct_from_query_without_leading_question ok
cd_construct_decodes_plus_and_percent        ok
cd_construct_from_pairs                      ok
cd_append_never_replaces                     ok
cd_delete_by_name_removes_all                ok
cd_delete_with_value_removes_only_matching   ok
cd_get_returns_first_match                   ok
cd_get_returns_none_when_absent              ok
cd_getall_returns_all_matches                ok
cd_getall_empty_when_absent                  ok
cd_has_by_name                               ok
cd_has_with_value_matches_pair               ok
cd_set_replaces_all_existing                 ok
cd_set_preserves_position_of_first           ok
cd_set_appends_when_not_present              ok
cd_sort_orders_by_name                       ok
cd_sort_is_stable_within_equal_names         ok
cd_sort_uses_utf16_code_unit_order           ok   ◀ ahead-of-time hypothesis #1 confirmed
cd_size_counts_entries                       ok
cd_entries_preserves_insertion_order         ok
cd_foreach_invokes_for_each_entry            ok

spec_tostring_empty_is_empty                 ok
spec_tostring_joins_with_ampersand           ok
spec_tostring_round_trip_through_from_query  ok
spec_decode_preserves_invalid_percent_sequences ok
spec_encode_rfc3986_unreserved_punctuation_passthrough ok
spec_encode_other_ascii_is_percent_encoded   ok
spec_decode_lossy_invalid_utf8_falls_back    ok

result: 32 passed, 0 failed, 0 skipped
```

**100% pass, zero skips.** The TextEncoder pilot had a documented skip (TEXT2 cluster-noise) and a documented apparatus-finding (JS-undefined-vs-Rust-None). The URLSearchParams pilot has neither — the constraint corpus and the spec material together pin every behavior the verifier exercises. The cybernetic loop closes cleanly on this surface.

## LOC measurement — apparatus finding about the metric itself

A naive comparison against Bun's own source produces a confusing-looking number. Investigation of Bun's URLSearchParams shows **why**:

```
Bun's URLSearchParams Rust source:                        60 LOC  (binding stubs)
Bun's URLSearchParams C++ binding:                       204 LOC  (FFI glue)
Bun's URLSearchParams.h:                                  84 LOC
   (Total Bun-side):                                     348 LOC

WebKit/WebCore upstream URLSearchParams.cpp/h:        ~300 LOC  (the actual implementation)

Pilot derivation, lib.rs (code-only, no comments):       186 LOC
```

**Bun does not implement URLSearchParams.** It delegates to WebKit's `WebCore::URLSearchParams` C++ class via a thin FFI shim (URLSearchParams__create, URLSearchParams__fromJS, URLSearchParams__toString). The "implementation" lives upstream in WebKit, totally outside Bun's tree.

Three findings from this:

**Finding 1 — the LOC-ratio metric requires care when the target project imports its implementation.** Bun's URLSearchParams "footprint" of 348 LOC is ~all binding glue — the actual semantic logic is in WebKit. A naive `pilot LOC / target LOC` ratio gives **186/348 = 53%**, but the comparison is unfair: the pilot is doing the *whole* job (implementation + Rust API) and Bun is doing the *binding* job (delegating to upstream).

**Finding 2 — against the actual implementation target (WebKit's C++ URLSearchParams), the pilot is competitive: ~186 LOC of standalone Rust against ~300 LOC of WebKit C++.** Ratio is **62%**. This is a *much* worse number than the TextEncoder pilot's 13-25%, but for an honest reason: URLSearchParams is intrinsically more code (12 methods + form-urlencoded encoder/decoder + UTF-16 sort + multi-form constructor) than TextEncoder (3 methods + UTF-8 encoder).

**Finding 3 — the apparatus's value claim shifts when the target project delegates to upstream.** "Derivation-from-constraints is dramatically smaller than hand-written port" doesn't quite apply when Bun didn't hand-write the port — it imported it. The right framing for these surfaces: "derivation-from-constraints is *competitive* with the upstream implementation, plus *eliminates the need for a binding layer*." That's a different kind of value claim, but real.

This is a finding about the apparatus's narrative scope. The htmx 9.4% prior was for a single attribute set with no upstream-import option; URLSearchParams' 62% is for a class Bun chose to import. Both are honest data points; the apparatus needs to characterize the target before claiming a ratio.

## The cybernetic loop that closed

Three pilots-worth of work converged on this run:

1. **TextEncoder pilot** named the constraint-doc-alone-is-insufficient finding (AUDIT.md gap-A).
2. **v0.12** fixed cluster-phase subject-attribution leakage so URLSearchParams.prototype.* would surface as distinct method-level subjects (the v0.11 substitution bug would have collapsed `params.has("k")` style assertions back onto a single URLSearchParams subject).
3. **v0.13/v0.13b** added spec-source ingestion so URLSearchParams' 13 method-level invariants are present in the constraint corpus alongside the test reps.

The URLSearchParams pilot's 100%-pass, 0-skip verifier result is the closure of that loop. Each of the prior three commits paid off concretely: without v0.12's leakage fix, the constraint doc would have had 1 catch-all URLSearchParams property instead of 17 method-level ones; without v0.13b's spec ingestion, half the methods would have had no constraint witness at all; without the TextEncoder pilot framing the simulation, the verifier wouldn't have closed.

## Findings beyond the pilot's primary thesis

- **AOT hypothesis #1 confirmed (UTF-16 code-unit sort).** Sort comparison must operate on UTF-16 code units, not Rust `char` (USV) order. The test `cd_sort_uses_utf16_code_unit_order` distinguishes: a supplementary-plane character (0xD83D 0xDE00 — emoji) sorts BEFORE a BMP full-width character (0xFF21 — Ａ) per UTF-16 ordering, even though the USV codepoint is higher.
- **AOT hypothesis #2 confirmed (form-urlencoded set narrower than RFC 1738).** Tested via `spec_encode_other_ascii_is_percent_encoded`: `!@#$^()` all percent-encode despite being unreserved in RFC 3986.
- **AOT hypothesis #3 confirmed (optional leading "?").** `cd_construct_from_query_with_leading_question` and `cd_construct_from_query_without_leading_question` both pass with the same effect.
- **AOT hypothesis #4 (cluster-phase second classifier-noise finding) — NOT confirmed.** Inspection of the v0.13b URLSearchParams cluster output shows clean per-method subject extraction. v0.12's leakage fix appears comprehensive for this surface.

## Files

```
pilots/urlsearchparams/
├── AUDIT.md                  ← coverage audit (rung-1 substrate analysis)
├── RUN-NOTES.md              ← this file
└── derived/
    ├── Cargo.toml
    ├── src/
    │   └── lib.rs            ← simulated derivation, 280 LOC (186 code-only)
    └── tests/
        └── verifier.rs       ← 32 tests (26 CD + 6 spec-edge), 100% pass
```

## Cycling back

This pilot's findings feed back into the apparatus differently than the TextEncoder pilot's did:

1. **No v0.14 work items surfaced** — the constraint corpus and spec material together produced sufficient input for a clean derivation. The hardening floor (v0.12 + v0.13 + v0.13b) is firm enough for a production pilot.
2. **The LOC-ratio metric needs nuance** — pilot results should record whether the target project implements or imports the derived surface, and present competitive-vs-upstream-impl numbers when delegation is in play.
3. **Cross-corroboration is the right tier-1 signal** — the 2 cross-corroborated reps (Deno toString assertions) caught the form-urlencoded encoder set correctness immediately. Without those, pure-spec derivation might have produced an RFC-3986-compliant encoder (which is wrong for form-urlencoded).

## Provenance

- Tool: `derive-constraints` v0.13b (commit e1939b1).
- Constraint inputs: `runs/2026-05-10-deno-v0.13b-spec-batch/cluster.json` + `specs/url-search-params.spec.md`.
- Spec input: WHATWG URL §5.2 + §5.2.5 (https://url.spec.whatwg.org/#interface-urlsearchparams).
- Derivation engine: LLM (this session), recording the input bundle in source-code comments per the simulation framing keeper directed.
- Verifier: `cargo test --release` against `pilots/urlsearchparams/derived/`.
- Result: 32 pass / 0 fail / 0 skip — first 100% verifier closure in this apparatus.
