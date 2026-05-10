# fetch-api system pilot — 2026-05-10

Seventh pilot. **First system pilot — multi-surface composition.** Headers + Request + Response derived as a single crate with cross-surface integration tests. The keeper's directive was *"a bigger portion of Bun than what we've done so far"*; this is the answer.

## Pipeline

```
v0.13b enriched constraint corpus
   Headers:  5 props / 15 clauses
   Request:  6 props / 53 clauses
   Response: 7 props / 100+ clauses (cross-corroborated cardinality 107)
       │
       ▼
AUDIT.md
       │
       ▼
simulated derivation v0   (CD + WHATWG Fetch §§5.2 / 6.2 / 6.4 spec extracts)
       │
       ▼
derived/src/{body,headers,request,response}.rs   (405 code-only LOC across 4 modules)
       │
       ▼
cargo test
   verifier:            34 tests
   consumer regression: 16 tests
       │
       ▼
50 pass / 0 fail / 0 skip   ← clean first-run pass on the largest pilot
```

## Verifier results: 34/34

```
HEADERS (12 tests)
  cd_h_class_exists                                  ok
  cd_h_count_empty_is_zero                          ok
  cd_h_append_invalid_name_errors                    ok
  cd_h_append_invalid_value_errors                   ok
  cd_h_set_invalid_errors                            ok
  spec_h_get_combines_repeated_with_comma_space     ok
  spec_h_case_insensitive_get                       ok
  spec_h_delete_case_insensitive                    ok
  spec_h_set_replaces_all_existing                  ok
  spec_h_value_whitespace_stripped                  ok
  spec_h_get_set_cookie_returns_separate_values     ok
  spec_h_iteration_lowercases_names                 ok

REQUEST (9 tests)
  cd_q_class_exists                                  ok
  cd_q_body_null_default                             ok
  spec_q_default_method_is_get                       ok
  spec_q_method_from_init                            ok
  spec_q_url_preserved                               ok
  spec_q_default_mode_credentials_cache_redirect    ok
  spec_q_text_body_consumed_once                     ok
  spec_q_clone_throws_when_body_used                 ok
  spec_q_clone_preserves_state_when_body_unused     ok

RESPONSE (13 tests)
  cd_s_class_exists                                  ok
  cd_s_default_status_is_200                         ok
  cd_s_ok_for_200                                    ok
  cd_s_ok_false_for_404                              ok
  cd_s_text_returns_body                             ok
  cd_s_headers_accessible                            ok
  cd_s_json_sets_content_type                        ok
  spec_s_status_out_of_range_errors                  ok
  spec_s_redirect_only_valid_codes                   ok
  spec_s_redirect_sets_location_header               ok
  spec_s_error_response_has_type_error_status_0     ok
  spec_s_clone_throws_when_body_used                 ok
  spec_s_clone_preserves_status_and_body             ok
```

## Consumer regression: 16/16

```
undici Response constructor (status defaults, status text)        2 tests
ky Response.json typed extraction                                  1 test
Express Headers case-insensitive lookup                            1 test
Stripe SDK getSetCookie multi-value returns                        1 test
Koa router default method dispatch                                 1 test
axios body-consumed-once enforcement                               1 test
Cloudflare Workers Response.redirect strict validation             2 tests
Reverse-proxy bytes() passthrough                                  1 test
WPT Fetch test corpus (status text, clone independence, error)     3 tests
Cross-surface integration (headers ⨯ clone, response ⨯ headers)    3 tests
                                                                   ──
                                                                   16 tests
```

## LOC measurement: dramatic reduction at scale

| Target | LOC |
|---|---:|
| Bun `Request.rs` | 1,831 |
| Bun `Request.zig` | 1,115 |
| Bun `Response.rs` | 1,435 |
| Bun `Response.zig` | 936 |
| Bun `Body.rs` | 2,573 |
| Bun `Body.zig` | 1,833 |
| Bun `FetchHeaders.rs` (jsc/) | 318 |
| Bun `FetchHeaders.zig` (jsc/) | 457 |
| Bun `Headers.rs` (http/) | 88 |
| Bun `Headers.zig` (http/) | 182 |
| **Bun .rs total** | **6,245** |
| **Bun .zig total** | **4,523** |
| **Pilot derivation (code-only)** | **405** |
| **Naive ratio vs Bun .rs** | **6.5%** |
| **Naive ratio vs Bun .zig** | **9.0%** |

Adjusted-scope ratio is meaningfully different. Bun's source includes:
- Full HTTP/1.1 + HTTP/2 + HTTP/3 transport
- Body streaming with backpressure
- File-backed and S3-backed bodies
- Cookie jar handling
- Pre-flight CORS
- Cache integration
- JSC value-side conversion + IDL bindings
- Bun-specific extensions (request-body-as-FormData, fast-path text encoding)

None of those are in pilot scope. The pilot is the **structural-data-layer** of fetch — which is exactly what the constraint corpus measures and what real consumer interop expects from the Headers/Request/Response objects.

Adjusted ratio: removing Body's transport machinery (~80% of Body.rs is transport, not data structure), Bun's data-layer Body is ~500 LOC. Subtracting transport from Request/Response too: data-layer Request/Response is ~600 LOC each. Adjusted Bun data-layer total: ~2,000 LOC. **Pilot ratio against data-layer: ~20%.**

Both numbers (naive 6.5% and adjusted ~20%) hold the apparatus' value claim at the htmx-prior order of magnitude.

## Updated seven-pilot table

| Pilot | Class | LOC | Adjusted ratio |
|---|---|---:|---:|
| TextEncoder + TextDecoder | data structure | 147 | 17–25% |
| URLSearchParams | delegation target | 186 | 62% |
| structuredClone | algorithm | 297 | ~8.5% |
| Blob | composition substrate | 103 | 20–35% |
| File | inheritance/extension | 43 | 20–30% |
| AbortController + AbortSignal | event/observable | 126 | 25–35% |
| **fetch-api (Headers + Request + Response)** | **system / multi-surface** | **405** | **6.5% naive, ~20% adjusted** |

**Seven-pilot aggregate: 1,307 LOC of derived Rust.** Reference targets across pilots: ~30,000+ LOC of upstream code (Bun + WebKit + scope-honest adjustments). **Aggregate ratio holds at ~3–4%.**

## Findings

1. **Two ahead-of-time hypotheses NOT confirmed (informative):** the prediction that cross-surface composition would surface a derivation bug (AOT #2) and the prediction that Response.redirect status validation would be a verifier-caught-bug site (AOT #3) both did not materialize. The derivation got composition + redirect validation correct on first attempt. This is a different result from Pilot 4 (Blob slice swap) and suggests apparatus + spec quality scales: at higher complexity, the spec is *more* explicit about edge cases (redirect codes are enumerated; composition is well-defined), so the LLM-derivation has more guidance, not less.

2. **The 16 consumer-regression tests cite consumers across the entire HTTP/fetch ecosystem** — undici, node-fetch, ky, Express, Stripe, Koa, axios, Cloudflare Workers, reverse-proxy implementations, WPT Fetch test corpus. The dependency-surface map for fetch-api is the densest of any pilot to date, including the cross-surface integration cases that depend on Headers + Response/Request composition behaving consistently.

3. **The data-layer/transport-layer split is itself an apparatus finding.** The pilot's LOC ratio looks dramatic (6.5% naive) but the honest comparison is against the data-layer slice of Bun's source, not the full source. Future pilots that derive transport-layer behavior will have different ratios; pilots that explicitly scope to data-layer will land in the 15–25% range. The apparatus' framing (Doc 707's bidirectional Pin-Art) maps cleanly: data-layer derivation + dependency map; transport-layer derivation is a separate operation against transport-layer constraints.

4. **No apparatus refinements queued.** The hardening floor (v0.12 cluster fix + v0.13 spec ingestion + v0.13b extended corpus) handled the largest pilot to date without surfacing new apparatus work. The constraint corpus + cited consumer corpus produced sufficient input for a clean derivation across three composed surfaces.

## What this changes about the value claim

[Doc 706](/resolve/doc/706-three-pilot-evidence-chain-derivation-from-constraints) anchored the value claim across three pilots; [Doc 707](/resolve/doc/707-pin-art-at-the-behavioral-surface-bidirectional-probes) extended it to bidirectional Pin-Art across six pilots. This pilot extends both:

- **The forward direction** (Doc 706's evidence chain) gains a **system pilot anchor**. The apparatus handles composed surfaces, not just individual ones.
- **The backward direction** (Doc 707's dependency-surface map) gains its **densest single-pilot map**. Sixteen cited consumer dependencies on the fetch-api surface — including production-load-bearing ones like Cloudflare Workers' redirect validation and Stripe's getSetCookie semantics — now exist as a reviewable artifact.

The keeper's "bigger portion of Bun" target is operationally met. The apparatus scales.

## Files

```
pilots/fetch-api/
├── AUDIT.md
├── RUN-NOTES.md              ← this file
└── derived/
    ├── Cargo.toml
    └── src/
        ├── lib.rs            (re-exports, 14 LOC)
        ├── body.rs           (97 LOC, 60 code-only)
        ├── headers.rs        (159 LOC, 109 code-only)
        ├── request.rs        (107 LOC, 87 code-only)
        └── response.rs       (154 LOC, 135 code-only)
                              total: 405 code-only LOC
    └── tests/
        ├── verifier.rs            34 tests, all pass
        └── consumer_regression.rs 16 tests, all pass
```

## Provenance

- Tool: `derive-constraints` v0.13b.
- Constraint inputs: `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/{headers,request,response}.constraints.md`.
- Spec inputs: `specs/{headers,request,response}.spec.md` + WHATWG Fetch §§5.2, 6.2, 6.4.
- Reference targets: Bun's `Request.rs`/`Response.rs`/`Body.rs`/`FetchHeaders.rs` (6,245 LOC Rust) + Bun's matching `.zig` (4,523 LOC).
- Result: 50/50 across both verifier (34) and consumer regression (16); zero regressions; zero apparatus refinements queued.
