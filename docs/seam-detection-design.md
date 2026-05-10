# Seam Detection — Pin-Art Applied to Intra-Architectural Boundaries

*The current `derive-constraints invert` output groups properties by first-identifier-segment (`Bun`, `fs`, `fetch`, `Buffer`). These are namespace boundaries, not architectural seams. The real architectural boundaries — where one form meets another inside the runtime — are interior to those namespaces and crosscut several of them. This document designs the seam-detection layer that finds them, operationalizing Pin-Art per Doc 270, Doc 619, Doc 678, Doc 685, Doc 693, and Doc 658 over the existing 4,838-property cluster catalog.*

## 1. The Problem with First-Segment Grouping

The invert MVP's surface decomposition takes `Bun.serve`, `Bun.file`, `Bun.spawn` → surface `bun`; `fs.readFileSync`, `fs.existsSync` → surface `fs`. This produces clean namespace partitions but conflates *what the surface does* with *what namespace exposes it*. Three concrete frictions:

- **`fs` collapses sync/async.** `fs.readFileSync` and `fs.promises.readFile` are different architectural forms (synchronous syscall path vs Promise-wrapped async I/O); the seam between them is a real architectural boundary the runtime must implement carefully. First-segment grouping puts them in the same surface module.
- **`Bun` straddles event-loop, FFI, parser, and runtime concerns.** Bun.serve is HTTP-server architecture; Bun.spawn is child-process architecture; Bun.file is FS-abstraction; Bun.build is compiler-pipeline; Bun.JSONL.parseChunk is parser-state-machine architecture. They share a namespace but no architectural form.
- **Architectural seams crosscut namespaces.** The native ↔ userland seam runs through `Buffer`, `Uint8Array`, `Blob`, `File`, `ReadableStream` — they are different namespaces but they all sit on one side of one architectural seam (the byte-pool boundary between JS-side typed arrays and native C++ byte buffers). First-segment grouping makes that invisible.

Per [Doc 704 §3](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error)'s formalization-then-derivation frame, the constraint set the substrate derives from must reflect the *architectural* form, not the *naming* convention. Otherwise the derivation produces target-language code that recreates the source-language's namespace organization rather than the runtime's actual structure.

## 2. The Corpus's Apparatus

[Doc 270 (Pin-Art Models)](https://jaredfoy.com/resolve/doc/270-pin-art-models) and [Doc 619 (Pin-Art Canonical Formalization)](https://jaredfoy.com/resolve/doc/619) establish boundary-detection: hedging under constraint-density functions as a population of independent probes pressing against a structural surface. **Detection-hedging clusters at propositional joints where the system detects a seam**; slack-hedging distributes uniformly. The joint pattern of probe-positions records the surface's shape. Doc 619 §4 supplies the discriminator: detection-hedging produces *localized convex cluster-shaped boundaries* under alpha-cut separation, distinguishing it from slack-hedging's diffuse noise.

[Doc 678 (Coherence Amplification and Decoherence as Inverse Pin-Art Operations)](https://jaredfoy.com/resolve/doc/678-coherence-amplification-and-decoherence-as-inverse-pin-art-operations) promotes Pin-Art to bidirectional information-transfer: information-out (decoherence detects boundaries) and information-in (coherence amplification names them under constraint-accumulation). Same probe-surface-reading structure.

[Doc 685 (Self-Reinforcing Boundary)](https://jaredfoy.com/resolve/doc/685-the-self-reinforcing-boundary): once a boundary is stated, the substrate's output reinforces it through three modes (explicit restatement, hedging around the boundary, implicit respect without statement). The positive-feedback loop means *named* boundaries stabilize while *unnamed* ones leak.

[Doc 693 (Resistance as Boundary-Indication)](https://jaredfoy.com/resolve/doc/693-resistance-as-boundary-indication): "Resistance to resolution against the standing apparatus is a surface marker of an unnamed boundary." Doc 693 §6 queues cross-discipline traces; the abstract pattern at §1 applies to *any* unnamed boundary, including intra-architectural.

[Doc 658 (Hierarchical Pin-Art Constraint Specs)](https://jaredfoy.com/resolve/doc/658) is the closest existing document: ring-stratified constraint specification where edge-case bugs indicate Ring-1 constraint misses at lifecycle boundaries. The methodology: identify lifecycle boundaries → state Ring-1 constraints → verify ring coherence → read against the edge-case surface for constraint gaps.

[PRESTO bilateral boundary](https://jaredfoy.com/resolve/doc/420) (C1 of Doc 420 / Doc 185) is the worked instance of an architectural seam: namespace separation enforced as a structural invariant, with operational properties (totality of consumption, ordering determinism, boundary integrity, non-modification) preserving the seam.

**The gap.** No corpus document operationalizes Pin-Art for *intra-architectural seam detection over a constraint-clustering catalog*. The apparatus is sound; the instantiation is specific to this codebase. The present design fills that gap for the Bun case.

## 3. Architectural-Hedging Signals

The corpus's Pin-Art apparatus reads hedging in the substrate's natural-language output (probabilistic markers, scope-narrowing, conditional clauses). Applied to a *codebase*, the analogous signals are *architectural hedging* — the patterns by which the test corpus and the implementation acknowledge a boundary without naming it. Six signal types:

1. **Conditional compilation.** `#[cfg(target_os = "...")]`, `if (process.platform === "darwin")`, `if (Environment.isWindows)`. The test or implementation hedges across a platform seam.
2. **Test-file path partitioning.** Tests under `test/js/node/fs/` vs `test/js/web/streams/` vs `test/js/bun/sql/` — the directory structure encodes a partial taxonomy of architectural surfaces. Cross-directory test density patterns reveal which surfaces the team has implicitly identified as separable.
3. **Sync/async partitioning.** Methods on the same surface that have separate sync and async forms (`readFileSync` vs `readFile`; `existsSync` vs `access`). The seam is the synchronous-syscall boundary.
4. **Throw vs return-error partitioning.** `JSON.parse` throws; `JSON.parseSafe` returns Result. The seam is the error-discipline boundary.
5. **Native vs userland partitioning.** Antichain representatives whose raw text references `Bun.dlopen`, `napi_*`, `extern "C"`, or whose test files sit in `*_sys` directories signal a native-binding boundary.
6. **Construct-then-method partitioning.** Subjects that pair a constructor with a method-bag (`Bun.Glob` constructor + `glob.scan()` / `glob.scanSync()` methods). The seam is between the constructor's allocation contract and the method's stateful invocation contract.

Each signal is a probe. A property *carries* one or more signals (or none). The joint pattern of which properties carry which signals — read across the existing 4,838-property cluster catalog — is the Pin-Art impression of the runtime's architectural surface.

## 4. The Operational Procedure

`derive-constraints seams <cluster.json> -o seams.json`. Pipeline:

1. **Probe extraction.** For each property, scan its antichain representatives' raw text and file paths for the six signal types above. Emit a per-property signal vector: `{ cfg: bool, path_components: [...], sync_sibling: bool, async_sibling: bool, throws: bool, native: bool, constructed_then_methoded: bool, ... }`.

2. **Signal-cluster identification.** Group properties whose signal vectors agree (within tolerance). Each cluster is a candidate seam. The seam's name comes from the dominant signal: "sync I/O", "platform-darwin", "throwing-parser", "native-byte-pool", etc.

3. **Cross-namespace seam reading.** For each candidate seam, list which existing first-segment surfaces it crosses. A seam that stays within one surface (e.g., `fs.readFileSync` separating from `fs.readFile` — both within `fs`) tells you to *split* the surface. A seam that crosses many surfaces (e.g., `Buffer`, `Uint8Array`, `Blob`, `File`, `ReadableStream` all referencing a native byte-pool) tells you to *merge* them under a new architectural surface module.

4. **Resistance-as-boundary verification.** Per [Doc 693](https://jaredfoy.com/resolve/doc/693-resistance-as-boundary-indication): a candidate seam is genuine if attempting to merge the properties on either side produces *resistance* — internal inconsistency, contradictory verb-classes, divergent verification verdicts. False seams merge cleanly. Real seams resist merging.

5. **Output.** A revised surface decomposition: `seams.json` listing the renamed/split/merged surfaces, with each property re-assigned to its real architectural home rather than its first-identifier-segment home.

## 5. The Cybernetic Loop

The seam-detection layer feeds the rest of the pipeline:

- **Re-invert with revised surfaces.** Run `derive-constraints invert` again, taking the revised surface decomposition from `seams.json` rather than the first-segment default. The resulting `.constraints.md` files cluster properties by architectural form.
- **Derive via rederive.** rederive's pipeline now operates over architectural surfaces rather than namespaces; the substrate's derivation produces target-language code organized by architectural form rather than recreating the source's namespace organization.
- **Verification verdicts return as seam-revisions.** Per [Doc 615 (substrate-dynamics-loop)](https://jaredfoy.com/resolve/doc/615-the-substrate-dynamics-loop): if rederive's verification fails on cross-seam properties (substrate hedged because the constraints from two different seams are conflated), the failure pattern reveals the seam was misidentified. Feed back into the next `seams` iteration.

This is Pin-Art operating at the codebase scale: probes are constraint-cluster signals; surface is the runtime's actual architectural form; reading is the seam decomposition; non-coercion is allowing the verification step to *report* misclassification rather than masking it. The loop closes when the seam decomposition stabilizes (Doc 615 closure signal) — verification verdicts match the seams the previous iteration named.

## 6. Predicted Output for the Bun Case

A first pass with the apparatus should reveal:

- **Sync/async split.** `fs.readFileSync` vs `fs.readFile`; `fs.existsSync` vs `fs.access`; etc. The `fs` surface decomposes into `fs-sync` (synchronous syscalls; threading model: blocking) and `fs-async` (Promise-wrapped; threading model: thread-pool dispatch). Same architectural pattern likely on `child_process`, `crypto`.
- **Native byte-pool merge.** `Buffer`, `Uint8Array`, `ReadableStream`, `Blob`, `File`, `FormData` — the byte-content side of these unifies into a single architectural surface (native byte-pool with type-stamped views) regardless of namespace.
- **Bun.* split into 4–6 architectural surfaces.** `Bun.serve` + `Bun.WebSocket` + `Bun.connect` → HTTP/networking; `Bun.spawn` + `Bun.file` + `Bun.write` → process+filesystem; `Bun.build` + `Bun.Transpiler` → compiler-pipeline; `Bun.SQL` + `Bun.Cookie` → datastore-bindings; `Bun.JSONL` + `Bun.YAML` → parser-state-machines; `Bun.inspect` + `Bun.stripANSI` → formatting.
- **Platform-conditional seam.** All `#[cfg(target_os)]` constraints across all surfaces compose into a *platform-conditional* meta-seam that crosscuts everything. The architectural form of platform-conditional code is the same regardless of which surface uses it.
- **Throw vs return-error seam.** A small number of surfaces have throw-discipline (Web platform: `URL` constructor throws on invalid URLs; `JSON.parse` throws); most have return-Result-discipline (Node-compat: `fs.access` returns; many Bun.* methods return `{ok, errors}` shapes). The seam is the error-discipline boundary.

If these predictions land empirically (the `seams` tool's first run produces approximately this decomposition), the apparatus is operational and the next concrete move is to plug the revised surfaces into `invert` for a second pass.

## 7. Honest Scope and v0.1 Limits

- **The signal set is not exhaustive.** Six signal types are MVP scope; more architectural-hedging signals exist (allocator boundary, ownership boundary, lifetime-annotation density patterns, error-propagation patterns). v0.2 expands the signal catalog; v0.1 demonstrates the apparatus.
- **Signal-cluster identification is heuristic.** The MVP uses simple agreement-within-tolerance grouping. A more rigorous approach would compute a similarity metric over signal vectors and run a hierarchical clustering with stability tests. v0.1 ships the tractable simpler form first.
- **Resistance-as-boundary verification (step 4) requires running rederive.** The MVP can identify *candidate* seams without running full rederive verification; verification confirms or rejects. The cybernetic loop's first iteration is informative even at the candidate-seam level.
- **The corpus's Doc 270 §1 caveat applies**: Pin-Art is restricted to its documented applications unless new applications demonstrate the same four-component structure (probes, surface, non-coercion, reading). The present design articulates the four components explicitly: probes = signal vectors per property; surface = architectural form; non-coercion = verification reports rather than masks; reading = seam decomposition. The instantiation extends Pin-Art's reach only if these components hold operationally on real data.

## 8. Concrete Next Step

Build `derive-constraints seams` MVP per the procedure at §4. Scope:

- New module `src/seams.rs` in derive-constraints.
- Reads `cluster.json`; emits `seams.json`.
- Six signal extractors over property antichain text + file paths (regex-based for the MVP).
- Signal-vector clustering via simple agreement counts.
- Cross-namespace cross-reference (which existing first-segment surfaces does each seam touch).
- No rederive integration in v0.1 — that's the verification step queued for v0.2.

Validation: run on `bun-cluster-v2.json`; inspect candidate seams against the §6 predictions; report.

## 9. References

- [Doc 270 — Pin-Art Models](https://jaredfoy.com/resolve/doc/270-pin-art-models). The boundary-detection apparatus.
- [Doc 619 — Pin-Art Canonical Formalization](https://jaredfoy.com/resolve/doc/619). Detection-hedging cluster discriminator (alpha-cut separation, convex cluster-shaped boundaries).
- [Doc 658 — Hierarchical Pin-Art Constraint Specs](https://jaredfoy.com/resolve/doc/658). Ring-stratified constraint specification — the closest existing apparatus to this design.
- [Doc 678 — Coherence Amplification and Decoherence as Inverse Pin-Art Operations](https://jaredfoy.com/resolve/doc/678-coherence-amplification-and-decoherence-as-inverse-pin-art-operations). Pin-Art's bidirectional information-transfer form.
- [Doc 685 — The Self-Reinforcing Boundary](https://jaredfoy.com/resolve/doc/685-the-self-reinforcing-boundary). Once-stated boundaries stabilize via substrate output reinforcement.
- [Doc 693 — Resistance as Boundary-Indication](https://jaredfoy.com/resolve/doc/693-resistance-as-boundary-indication). Resistance is the surface marker of an unnamed boundary; the trace methodology applies to intra-architectural seams as well as cross-discipline traces.
- [Doc 615 — The Substrate-Dynamics Loop](https://jaredfoy.com/resolve/doc/615-the-substrate-dynamics-loop). The cybernetic closure signal: verdicts feed back as constraint revisions.
- [Doc 187 — Bilateral Systems](https://jaredfoy.com/resolve/doc/187) + [Doc 420 — PRESTO Dissertation](https://jaredfoy.com/resolve/doc/420). The worked architectural-seam instance: namespace separation enforced as structural invariant.
- [Doc 704 — The "Port" as Translation Is a Category Error](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error). The frame this design serves: constraint set must reflect architectural form, not naming convention.
- Companion in this repo: [`docs/invert-phase-design.md`](./invert-phase-design.md), [`docs/cluster-phase-design.md`](./cluster-phase-design.md), [`runs/2026-05-10-bun-derive-constraints/INVERT-NOTES.md`](../runs/2026-05-10-bun-derive-constraints/INVERT-NOTES.md).
