# Invert Phase Design — The Cybernetic Composition with rederive

*Refines the planning at [`derivation-inversion-on-bun-tests.md §4.3`](./derivation-inversion-on-bun-tests.md#43-invert) after recon of the cybernetic corpus thread and the [`jaredef/rederive`](https://github.com/jaredef/rederive) repo. The invert phase is not a code-generator; it is a **transcoder from `cluster.json` to rederive-compatible `.constraints.md` documents**, with a cybernetic feedback channel that returns hedging-detected boundary information into revised constraints.*

## 1. The Cybernetic Frame

The corpus articulates the keeper-substrate relation as a closed-loop cybernetic cycle, not a one-shot transformation. The composing documents:

- **[Doc 615 — Substrate-Dynamics Loop](https://jaredfoy.com/resolve/doc/615-the-substrate-dynamics-loop)** composes [Doc 296 (recency-decay)](https://jaredfoy.com/resolve/doc/296-recency-density-and-the-drifting-aperture), [Doc 297 (invisibility failure mode)](https://jaredfoy.com/resolve/doc/297-pseudo-logos-without-malice), [Doc 270 (Pin-Art impression mechanism)](https://jaredfoy.com/resolve/doc/270-pin-art-models), and [Doc 129 (non-coercion operating condition)](https://jaredfoy.com/resolve/doc/129) into a single cybernetic cycle. Recency-decay creates substrate-internal blindness; invisibility-failure is the structural consequence; Pin-Art's impression-detection is the keeper-side feedback that externally maps the substrate's blind spots; non-coercion is the operating condition under which the impression-detection actually functions (forced-press overrides hedging and produces crash-through rather than boundary-mapping).
- **[Doc 187 — Bilateral Systems](https://jaredfoy.com/resolve/doc/187)** names the architectural form: ambivalent execution with agnostic determinism on a shared medium, with non-interference guaranteed by namespace separation rather than coordination. Two interpreters do not need to know about each other; the medium's structure prevents interference.
- **[Doc 291 — Gödel and the Constraint Thesis](https://jaredfoy.com/resolve/doc/291-goedel-and-the-constraint-thesis)** is the self-reference incompleteness anchor: a substrate cannot bootstrap its own missing constraints (heteronomy is structurally enforced).
- **Ashby-shaped requisite variety**: the keeper-side reading apparatus must carry sufficient *variety* (probe density across the substrate's response surface) to match the variety of the surface being mapped. Insufficient probe density and the impression is undersampled; correct probe density and the boundary is recovered.

The implication for derive-constraints is direct. **The inversion of derivation is not unidirectional.** Constraints emit code (rederive's pipeline) → code exhibits hedging at boundaries (Pin-Art's substrate-side signal) → keeper reads hedging → revised constraints feed back. The invert phase belongs at the constraint-emission step of this loop, not at the code-generation step.

## 2. The rederive Pipeline (As Already Built)

[`/home/jaredef/rederive/`](https://github.com/jaredef/rederive) implements an eight-stage constraint-driven derivation pipeline. The stages, located in `src/`:

| Stage | Module                          | Operation                                                    |
|-------|---------------------------------|--------------------------------------------------------------|
| 1     | (filesystem)                    | Read constraint file from disk                               |
| 2     | `src/parse.ts`                  | Extract constraint AST: H2 blocks + metadata + body text     |
| 3     | `src/validate.ts`               | Required fields, type, authority, depends-on acyclicity      |
| 4     | `src/resolve.ts`                | Fetch imports; verify SIPE-T induced-property emergence      |
| 5     | `src/canonicalize.ts`           | Deterministic byte-form for SHA-256 content addressing       |
| 6     | `src/derive.ts`                 | **The inversion step**: assemble LLM prompt + extract code   |
| 7     | `src/verify.ts` (29,756 bytes)  | Pluggable verification backends                              |
| 8     | `src/sign.ts`                   | Ed25519 provenance: (constraint-hash, fn-hash, substrate-id, target-lang, code-hash, verdict-hash) |

Verification backends (per `src/verify.ts`): TypeScript type-checking, test assertion blocks, `@example` / `@counterexample` property checks, SIPE-T composition verification, `@provides.interface` symbol-existence, `@pins` literal preservation, `depends-on` graph satisfaction.

The substrate (LLM) is treated as a black-box `SubstrateHandle` returning a promise; derivation is determinism-relaxed-to-equivalence (multiple calls may emit different code, but all must satisfy the verification suite).

The repo eats its own dogfood: `derived-engine/` contains 10 modules each derived from `.constraints.md` files in `samples/`.

## 3. Constraint-Authoring Grammar

The grammar is two-layer. Examples taken from `samples/sign-module.constraints.md`:

**Manifest layer** (file frontmatter or top section):

```
@provides: <property-name>
  threshold: <witness-constraint-id>
  interface: [exported-symbol-1, exported-symbol-2, ...]

@imports:
  - property: <source-property>
    from: path|hash|tag
    path: ./path/to/module.constraints.md
    as: <binding-name>

@pins:
  - id: <pin-id>
    mustContain: "<literal-code-to-preserve>"
    why: "rationale"
```

**Constraint layer** (H2 blocks):

```
## SIGN1
type: specification|predicate|invariant|bridge|methodology|example|counterexample
authority: human-authored|AI-suggested-pending|derived
scope: module|engine|system
status: active|deprecated|superseded
depends-on: [SIGN0, SIGN-PRELUDE, ...]

Prose body — opaque to platform; interpreted by substrate at derivation.
```

The corpus's [Doc 660 (constraint-authoring grammar)](https://jaredfoy.com/resolve/doc/660) is the corpus-side articulation; the rederive repo is the operational implementation.

## 4. Where derive-constraints Composes

The architectural answer: **derive-constraints emits `.constraints.md` documents that rederive consumes directly.** The two platforms compose at the constraint level.

```
[ Bun test corpus ] ─▶ scan ─▶ cluster ─▶ INVERT ─▶ [ .constraints.md files ]
                                                        │
                                                        ▼
                                                  [ rederive pipeline ]
                                                  parse → validate → resolve →
                                                  canonicalize → derive →
                                                  verify → sign
                                                        │
                                                        ▼
                                                   [ derived Rust ]
                                                   + verification verdicts
                                                   + signed provenance
```

derive-constraints is the keeper-side rung-2 observer that reads the existing test corpus and emits structured constraints; rederive is the keeper-tooled derivation pipeline that emits and verifies code.

The cybernetic loop closes when rederive's verification produces hedging-shaped failures (substrate emitted code that fails verification at specific boundary cases — the patterns Pin-Art's apparatus reads as boundary markers). Those failures feed back into derive-constraints' next iteration as constraint revisions: weaken or split constraints whose failures cluster; tighten constraints whose induced property is incomplete.

## 5. Concrete Output Format

The invert phase emits one `.constraints.md` file per cluster of related properties. Clustering for output organization (one file per architectural surface) is approximately the construction-style classification's grouping:

- One file per construction-style surface: `bun.serve.constraints.md`, `bun.file.constraints.md`, `fetch.constraints.md`, `Buffer.from.constraints.md`, etc.
- Behavioral properties merge into per-surface companion files or get listed under the corresponding surface.
- A top-level `bun-runtime.constraints.md` declares `@imports` for each surface module.

Each per-surface file carries:
- `@provides` declaring the surface's induced property (e.g. *Bun.file produces a BunFile reference with stat-style metadata accessors*).
- `@imports` for surfaces this depends on (e.g. *Bun.file* imports *URL* for path resolution; *Bun.serve* imports *Response* and *Request*).
- Per-constraint H2 blocks: one constraint per property the cluster phase identified, body text reconstructed from the antichain representatives.
- `@pins` for behaviors that must be preserved verbatim (e.g. specific error messages tested by name).

The body of each constraint is short prose (the corpus's standing form): "Bun.file accepts a string path or URL. The returned BunFile exposes …". The substrate (LLM) at rederive's derive step interprets the prose into Rust code; rederive's verification suite ensures the code satisfies the antichain's representative test invariants.

## 6. The Cybernetic Feedback Channel

The two-platform composition is *unidirectional* if invert just emits `.constraints.md` and rederive consumes them. To close the cybernetic loop:

1. **Verification verdicts return as substrate-side signal.** rederive's `verify.ts` produces structured failure reports: which constraints failed, which interface symbols were missing, which pins were violated. These are Pin-Art impressions of where the substrate hedged.
2. **derive-constraints reads the verdicts.** A new subcommand `derive-constraints reflect <verdicts.json> <cluster.json>` reads rederive's verification output and proposes constraint revisions: weaken constraints whose verification consistently failed; split constraints whose failure patterns suggest the constraint conflates multiple properties; tighten constraints whose verification passed but whose induced property is incomplete (the substrate produced minimal code that satisfies the constraint without addressing the surrounding architectural context).
3. **The loop iterates** until the verification verdicts stabilize: the constraint set has reached its SIPE-T coverage threshold and further iterations don't shift the verdicts measurably. Per [Doc 615](https://jaredfoy.com/resolve/doc/615-the-substrate-dynamics-loop)'s closure: the cybernetic cycle terminates when keeper-side reading no longer detects new boundary information.

The non-coercion operating condition ([Doc 129](https://jaredfoy.com/resolve/doc/129)) governs the loop: rederive's verify step must be allowed to *report* hedging rather than be forced to mask it; the keeper's revisions must respect what the substrate signaled rather than override it. Forced-press overrides produce crash-through (the loop oscillates without converging) rather than boundary-mapping (the loop converges as the constraints reach their requisite-variety coverage of the test corpus's behavioral surface).

## 7. Implementation Plan

**Phase 1 — Invert MVP** (`derive-constraints invert <cluster.json> -o <constraints-dir/>`):

- Read `cluster.json` from the cluster phase.
- Group properties by their construction-style surface (subject's first identifier or the surface from a curated mapping).
- For each group, emit one `.constraints.md` file:
  - Manifest header: `@provides`, `@imports` (best-effort cross-reference detection), `@pins` (none in MVP).
  - Per-property H2 blocks: `type`, `authority: derived`, `scope`, `depends-on` (best-effort), prose body.
  - Prose body shape: "{Subject} {verb-class-narrative}. {Witness count} test assertions in the corpus exemplify the contract: {antichain representatives summarized}."
- Emit a top-level `bun-runtime.constraints.md` index that imports each surface module.
- Validate the output by running `rederive parse <output-file>` on each emitted file (the rederive parser is a self-contained binary in the rederive repo).

**Phase 2 — Reflect MVP** (`derive-constraints reflect <verdicts.json> <cluster.json> -o <revised-cluster.json>`):

- Read rederive's verification verdicts (the structured output of `rederive verify`).
- Map each failure back to the cluster.json property whose constraint produced it.
- Apply revision heuristics:
  - Verification failure on a high-cardinality property → constraint is incomplete; expand antichain to include more representatives.
  - Verification failure on a singleton property → constraint may be over-specifying; consider dropping or weakening.
  - Pin-preservation failure → the literal under preservation is contested; flag for keeper review.
- Emit revised cluster.json with the property-set updated.

**Phase 3 — Cybernetic loop driver** (`derive-constraints loop <test-corpus> <constraints-dir/>`):

- Run scan → cluster → invert → rederive → verify → reflect → invert until verdicts stabilize.
- Termination condition: per-iteration verdict diff falls below a threshold (the requisite-variety closure signal).

## 8. Risks and Honest Scope

- **rederive's derivation step is LLM-substrate-dependent.** Each invocation costs tokens; iterating the cybernetic loop is not free. The MVP should run on a small subset of properties (e.g. 10 surfaces) before scaling.
- **The constraint prose has to be authored by the substrate at derive-time, not by derive-constraints itself.** Invert MVP emits *candidate* prose stitched together from antichain representatives; the prose may need keeper-side editing before it derives well. The corpus's standing pattern: keeper authors constraints; substrate derives implementations. Invert is a draft-author for the keeper, not a replacement for keeper authorship.
- **Convergence is not guaranteed.** The cybernetic loop may oscillate if the constraint set is internally inconsistent (different antichain representatives encode contradictory assumptions). The MVP should report oscillation and surface the conflict for keeper resolution rather than running open-loop.

## 9. References

- **Cybernetic apparatus:** [Doc 129](https://jaredfoy.com/resolve/doc/129), [Doc 187](https://jaredfoy.com/resolve/doc/187), [Doc 270](https://jaredfoy.com/resolve/doc/270-pin-art-models), [Doc 291](https://jaredfoy.com/resolve/doc/291-goedel-and-the-constraint-thesis), [Doc 296](https://jaredfoy.com/resolve/doc/296-recency-density-and-the-drifting-aperture), [Doc 297](https://jaredfoy.com/resolve/doc/297-pseudo-logos-without-malice), [Doc 615](https://jaredfoy.com/resolve/doc/615-the-substrate-dynamics-loop).
- **Constraint-authoring grammar:** [Doc 660](https://jaredfoy.com/resolve/doc/660); operational implementation in `/home/jaredef/rederive/src/parse.ts`.
- **Rederive pipeline:** `/home/jaredef/rederive/src/{engine,parse,validate,resolve,canonicalize,derive,verify,sign}.ts`.
- **Self-derivation evidence:** `/home/jaredef/rederive/derived-engine/` (the engine derived from constraints in `samples/`).
