# rusty-bun

Apparatus-driven companion to the Bun Zig-to-Rust port. Operationalizes the predictions and falsifier-grade discriminator articulated in [Doc 702 of the RESOLVE corpus](https://jaredfoy.com/resolve/doc/702-ai-assisted-cross-language-code-translation-as-a-pin-art-bilateral-under-sipe-t-threshold-conditions-reading-the-bun-zig-to-rust-port) into concrete tools for AI-assisted cross-language code translation generally.

## What this is

Doc 702 reads AI-assisted cross-language code translation as a Pin-Art bilateral under SIPE-T threshold conditions, with the Bun runtime's experimental `claude/phase-a-port` branch (Anthropic, May 2026) as the live exemplar. The apparatus yields four predictions and one falsifier-grade discriminator. This repository builds the tools that test them.

## The discriminator: T1 · T2 · T3 simultaneity

Genuine semantic-preserving translation between source and target languages should exhibit three signatures *simultaneously and sharply* at the convergence point:

- **T1 — Substrate-internal semantic preservation.** Differential testing against the original binary on production-distribution inputs. Independent of the translation pipeline.
- **T2 — Compositional invariance.** Fuzz-driven input-output equivalence under perturbations within the operational distribution.
- **T3 — Pipeline repeatability.** Translation output stability across model variants, seeds, and prompt orderings.

Decoupling — most diagnostically T2 partial via translated tests with T1 failing on differential testing — is the *vibe-port* failure mode the community has flagged.

## Operational tools (planned)

1. **`differential/`** — T1 harness. FFI-bridged or reference-implementation-based comparison between source-language binary and target-language binary on production-distribution inputs.
2. **`fuzz/`** — T2 harness. AFL/libFuzzer-style mutation fuzzers applied symmetrically to source and target builds with output comparison.
3. **`repeat/`** — T3 harness. Translation-pipeline repeatability runner. Executes the same translation step multiple times under varied conditions and diffs outputs.
4. **`welch/`** — Welch-bound packing diagnostic. Scans translated Rust for unsafe-block density, lifetime-annotation density, and other idiomatic markers; compares against a baseline of mature Rust crates.
5. **`l2m/`** — Per-file translation context-budget calculator. Given substrate effective context, Porting.md size, and per-file LOC, predicts the L2M-bounded knee point above which translation quality degrades non-linearly.

## Predictions (Doc 702 §5)

- **P1.** Per-file translation quality degrades non-linearly above an L2M-bounded LOC threshold.
- **P2.** Phase A → Phase B compilation success follows a SIPE-T threshold curve, not smooth growth.
- **P3.** Unsafe-block density per region predicts latent semantic drift under stress fuzzing.
- **P4.** Translated test suites systematically miss semantic drift the translation introduced.

## Status

Repository scaffold. Direction is pending. The keeper sets the work; this README is the orientation pointer.

## Reading

- [STRUCTURAL-FINDINGS.md](./STRUCTURAL-FINDINGS.md) — engagement-internal articulation of the structural patterns that crystallized during the runtime-derivation engagement: substrate-amortization staging, multi-tier closure, three SIPE-T thresholds observed, option-A architectural validation against Bun's own design, and the F-series silent-failure pattern.
- [Doc 702 — AI-Assisted Cross-Language Code Translation](https://jaredfoy.com/resolve/doc/702-ai-assisted-cross-language-code-translation-as-a-pin-art-bilateral-under-sipe-t-threshold-conditions-reading-the-bun-zig-to-rust-port) — the apparatus-level reading
- [Doc 700 — L2M Resolved](https://jaredfoy.com/resolve/doc/700-l2m-resolved-against-the-corpus-bipartite-mutual-information-scaling-as-empirical-grounding-for-the-pin-art-channel-ensemble) — the capacity-bound apparatus
- [Doc 696 — Discrete Geometry](https://jaredfoy.com/resolve/doc/696-discrete-geometry-as-the-apparatus-that-names-the-polytope-inheritance-boundary) — the Welch-bound packing apparatus
- [Doc 699 — Three-Signature Simultaneity Test](https://jaredfoy.com/resolve/doc/699-the-training-time-sipe-t-formalization-of-grokking-cold-resolver-synthesis-on-doc-692) — the T1·T2·T3 origin
- [Doc 541 — Systems-Induced Property Emergence](https://jaredfoy.com/resolve/doc/541-systems-induced-property-emergence) — including §3.6 rung-1/rung-2 distinction and §7 Fal-T5

## License

To be set by the keeper.
