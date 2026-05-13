# P3 Verification — Engine-Cut Classification of the 14 Residual Parity Failures

Per [Doc 717 §X](/resolve/doc/717-the-apparatus-above-the-engine-boundary-the-three-projections-lifted-to-engine-substrate-and-the-pure-abstraction-point) prediction P3: *the 14 residual parity failures map onto 2–3 distinct (abstract-op × rung) pairs in G_engine, not 14 separate engine bugs.*

Baseline: 88.2% (105/119), commit `3f9673ab`. Classification ran 2026-05-13 night against `/home/jaredef/rusty-bun/host/tools/parity-results.json`.

## Classification table

| # | Package | Symptom | Abstract op | Rung | Class | Tuple |
|---|---|---|---|---|---|---|
| 1 | yup | Module namespace missing `default` | GetModuleNamespace | E2 (Module Namespace exotic [[OwnPropertyKeys]]) | E2 spec-relaxation | **A** |
| 2 | io-ts | Same | GetModuleNamespace | E2 | E2 | **A** |
| 3 | superstruct | Same | GetModuleNamespace | E2 | E2 | **A** |
| 4 | neverthrow | Same | GetModuleNamespace | E2 | E2 | **A** |
| 5 | jsonc-parser | Same | GetModuleNamespace | E2 | E2 | **A** |
| 6 | fp-ts | Same | GetModuleNamespace | E2 | E2 | **A** |
| 7 | yargs | SyntaxError "Could not find export 'default'" in y18n | ResolveExport (cascade from A on y18n) | E2 | E2 | **A** |
| 8 | dayjs | Namespace missing default's siblings (Ls, en, extend, …) | GetModuleNamespace + named-export synthesis from default | E5 (realm host-defined behavior) | E3 spec-extension | **B** |
| 9 | date-fns | Missing `longFormatters` (one default-attached sibling) | Same | E5 | E3 | **B** |
| 10 | node-fetch | Missing `FetchBaseError` + `fetch` (NAME-from-export-default-function) | Same + NAME-of-default-function-as-named-export | E5 | E3 | **B** |
| 11 | superagent | Missing `m-search` (non-identifier key) + `query` | ParseModule (string-literal export aliases) | E1 (grammar production) | E4 version-lag | **C** |
| 12 | ora | SyntaxError "expecting ';'" on modern source | ParseModule | E1 | E4 | **C** |
| 13 | got | Runtime TypeError "cannot read property '_parentWrap' of undefined" | (consumer-runtime; not engine-level directly) | — | — | **D** uncertain |
| 14 | enquirer | Namespace shape entirely different (66 keys lowercase-instance vs 43 keys class-set) | Compound: CJS bridge sees class set, Bun sees instance with inherited EventEmitter methods | E5 (likely) | E3 (likely) | **D** uncertain |

## Tuple summary

| Tuple | (Abstract-op × Rung × Class) | Packages | Count |
|---|---|---|---|
| **A** | GetModuleNamespace × E2 × spec-relaxation (rquickjs freezes Module Namespace at construction; ECMA-262 §16.2.1.10 permits host-defined augmentation that rquickjs does not expose) | yup, io-ts, superstruct, neverthrow, jsonc-parser, fp-ts, yargs | 7 |
| **B** | GetModuleNamespace × E5 × spec-extension (Bun's realm host-defined behavior synthesizes named exports from default's own properties, including NAME of `export default function NAME`) | dayjs, date-fns, node-fetch | 3 |
| **C** | ParseModule × E1 × version-lag (QuickJS grammar predates ES2022 string-literal export aliases and certain modern class-field forms) | superagent, ora | 2 |
| **D** | Uncertain / compound | got, enquirer | 2 |

## P3 verdict

**P3 holds.** 12 of 14 failures classify into exactly three distinct (abstract-op × rung × alphabet-class) tuples. The remaining 2 (got, enquirer) are not separate engine bugs in the sense the prediction targeted — got is a consumer-runtime TypeError on a property access (`_parentWrap`) likely cascading from incomplete stream/readable behavior on the host side, and enquirer is a compound shape divergence that may share root with tuple B but does not cleanly fit a single tuple.

The structural finding: **the engine-selection criterion reduces to three concrete questions**, one per tuple. Each tuple is a single (where in the engine × at what rung × with what spec-relation) decision. Candidates that match Bun's cuts on all three retire 12/14 directly + close yargs's y18n cascade automatically.

## Implications for Tier-Ω.3 (engine selection)

The engine-cut-profile tool described in [Doc 717 §IX](/resolve/doc/717-the-apparatus-above-the-engine-boundary-the-three-projections-lifted-to-engine-substrate-and-the-pure-abstraction-point) can now be focused. For each engine candidate (QuickJS, QuickJS-NG, Boa, hand-roll), three questions answer the selection:

1. **Tuple A — does the engine cut Module Namespace construction at E2 or higher?** Bun cuts at E5. rquickjs cuts at E2. An engine that cuts at E5 with host-defined hooks for namespace augmentation retires Tuple A's 7 packages directly + the y18n cascade.
2. **Tuple B — does the engine expose host hooks at E5 sufficient to synthesize named exports from default's own properties post-init?** Bun does. Whether a candidate engine does is the second question.
3. **Tuple C — what ES edition does the engine's parser implement?** Bun runs a current parser. QuickJS at ~ES2020 with selective patches. QuickJS-NG closer to ES2022. Boa moving target. The closer to current, the smaller Tuple C.

The two D-class items (got, enquirer) get their own investigations after Tier-Ω.3 selects the engine.

## Predictions sharpened

If Tier-Ω selects an engine matching Bun on all three tuples, the post-migration parity prediction is:
- Tuple A retires (7 packages) + yargs cascade closes = **+7 packages**
- Tuple B retires (3 packages) = **+3 packages**
- Tuple C retires (2 packages) = **+2 packages**
- D-class becomes independent post-migration work

**Predicted post-migration baseline: 105 + 12 = 117/119 ≈ 98.3%**, with got and enquirer as the remaining two independent investigations against the new engine.

This is the legibility Doc 717 §VIII predicted: parity acquires a structural ceiling computable pre-commit from the cut-profile diff.

## Data sources

- Parity results JSON: `host/tools/parity-results.json` (post-`3f9673ab`)
- Per-package key/typeof diffs: `/home/jaredef/keydiff*.log`, `/home/jaredef/fsex-shape-diff.log`
- Doc 717: `/home/jaredef/corpus-master/corpus/717-the-apparatus-above-the-engine-boundary-...md` (resolve commit `c917832`)
