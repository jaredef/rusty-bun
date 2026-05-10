# derive-constraints invert — 2026-05-10 — Bun phase-a-port

First run of the invert phase. Reads `bun-cluster-v2.json`; emits 99 `.constraints.md` documents in [rederive grammar](https://github.com/jaredef/rederive) at `constraints/`.

## Inputs

- **Source.** `bun-cluster-v2.json` (4.5 MB) — 4,838 properties from the Bun test corpus.
- **Tool.** `derive-constraints invert` v0.1, sibling to `scan` and `cluster`. Implements [`docs/invert-phase-design.md §7 Phase 1`](../../docs/invert-phase-design.md): emit one `.constraints.md` per architectural surface plus a top-level `bun-runtime.constraints.md` index.
- **Settings.** `MAX_CONSTRAINTS_PER_SURFACE = 80`; `MIN_BEHAVIORAL_CARDINALITY = 5`. Surface emitted if it contains any construction-style property OR has total witnessing ≥ 20 clauses OR a single property at cardinality ≥ 10.

## Output

| Metric                   | Value     |
|--------------------------|----------:|
| Surface .constraints.md files | 98 + 1 index |
| Total constraints emitted | 487       |
| Properties skipped (sub-floor or noise) | 3,930 |
| Output dir size          | ~640 KB   |

The 4,838-property cluster catalog reduces to 487 emitted constraints across 98 surfaces — about a 10× compression at this layer. The skipped 3,930 properties are predominantly singletons + low-cardinality behavioral properties + local-variable subjects whose surface name failed the substantive-namespace allowlist.

## Top 20 surfaces by witnessing-clause count

```
2,609  bun           ← Bun.* runtime
1,308  glob          ← Bun.Glob construction
  886  fetch
  799  buffer        ← Buffer / Buffer.from / Buffer.isBuffer
  774  yaml          ← YAML.parse + Bun.YAML alias
  594  util
  375  path
  361  json
  ...
  process, crypto, dns, fs, response, tls, os, atomics, http2, performance,
  websocket, worker, promise, date, sql, redis, http, dgram, …
```

These are the publicly-named architectural surfaces of Bun's runtime contract. The bun-runtime.constraints.md index orders them by witnessing count so the substrate at rederive's derive step encounters the highest-leverage surfaces first.

## Sample output (head of bun.constraints.md)

```
# Bun — surface constraints derived from the Bun test corpus

*Auto-drafted ...*

@provides: bun-surface-property
  threshold: BUN1
  interface: [Bun.file, Bun.build, Bun.JSONL.parseChunk, Bun.inspect,
              Bun.spawn, Bun.Cookie, Bun.CookieMap, ...]

@imports: []
@pins: []

Surface drawn from 80 candidate properties across the Bun test corpus.
Construction-style: 69; behavioral (high-cardinality): 11. Total
witnessing constraint clauses: 2609.

## BUN1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.file** — produces values matching the documented patterns under
the documented inputs. (construction-style)

Witnessed by 247 constraint clauses across 5 test files. Antichain
representatives:

- `test/regression/issue/26851.test.ts:30` — --bail writes JUnit
  reporter outfile → `expect(await file.exists()).toBe(true)`
- `test/regression/issue/26647.test.ts:39` — Bun.file().stat() should
  handle UTF-8 paths with Japanese characters → `expect(bunStat.isFile()).toBe(true)`
- `test/regression/issue/14029.test.ts:41` — ...

## BUN2
type: predicate
...
**Bun.build** — ... (construction-style)
Witnessed by 182 constraint clauses ...
```

The grammar follows the rederive sample format exactly (per `/home/jaredef/rederive/samples/sign-module.constraints.md`): top-level `@provides` / `@imports` / `@pins` directives, then per-constraint H2 blocks with `type / authority / scope / status / depends-on` metadata, then prose body. The substrate at rederive's `derive` step would interpret the prose into target-language code; the antichain representatives carry the witnessing test invariants so the verification step can ground-truth the derivation.

## What the run shows

**The invert phase produces a tractable input for derivation.** 487 constraints across 98 surfaces is in the order-of-magnitude band [Doc 704 §3](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) predicts: small enough that a substrate can hold the per-surface constraint context with room for derivation, large enough to span Bun's actual API surface. The htmx case had 19 constraints / 3,937 words; Bun's case at 487 / ~80K words extrapolates the htxlang ratio to a larger API surface and suggests a derived-Rust LOC prediction in the 30K-150K range per [Doc 704 P1](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error).

**The `bun-runtime.constraints.md` index is the entry point.** It composes the surfaces via `@imports` and provides a cardinality-ordered top-twenty for orientation. The substrate consuming this index sees the runtime's contract as a composition of named surface properties, each with its own derivation budget and verification suite.

**Honest scope (per the design doc).** The MVP emits *draft prose stitched from antichain representatives*. The substrate at rederive's derive step ultimately interprets the prose into code, and the prose may need keeper-side editing before it derives well. Per [`docs/invert-phase-design.md §8`](../../docs/invert-phase-design.md): invert is a draft-author for the keeper, not a replacement for keeper authorship. The corpus's standing pattern is keeper-authored constraints + substrate-derived implementation; this MVP makes the keeper's authoring task massively easier (the corpus of test invariants is now structured prose ready for review and revision) without claiming to *be* keeper authorship.

## v0.2 refinements queued

- **Cross-surface `@imports` detection.** Currently each surface is emitted with empty `@imports`. Many surfaces have legitimate dependencies (e.g. `fetch` constraints reference `Response`, `Request`, `Headers`; `Bun.file` references `URL`). A v0.2 pass would scan each surface's antichain text for cross-surface references and emit explicit imports.
- **Property-level depends-on extraction.** Currently each constraint has empty `depends-on`. Identifying intra-surface dependencies (e.g. BUN5 depends on BUN2 because BUN5 is testing a method on a Bun.file returned by BUN2) would let rederive's resolver order the derivation correctly.
- **Better prose bodies.** The MVP's body is a templated narrative + raw witnessing-clause citations. A v0.2 could:
  - Group antichain representatives by behavior pattern (e.g., "with valid path", "with non-existent path", "with relative path")
  - Synthesize the verb-narrative into a contract-style claim ("Bun.file accepts a string path or URL and returns a BunFile reference; the BunFile exposes …")
  - Cite the test invariants as `@example` / `@counterexample` blocks so rederive's verification can run them directly
- **`@pins` extraction for behaviors that must be preserved verbatim.** Specific error messages tested by name; specific constants returned in API responses; etc.

## Files in this run

- `constraints/` — 99 `.constraints.md` files (98 surfaces + 1 index).
- `invert-summary.txt` — terminal summary captured from `--summary` stderr.

The output directory is checked into the rusty-bun repo as a real-corpus run artifact.

## Provenance

- Tool: `derive-constraints` v0.1 (cluster phase v0.2 + invert phase v0.1).
- Source: `bun-cluster-v2.json` from the cluster run on 2026-05-10.
- Tool runtime: well under a second on the dev machine.
