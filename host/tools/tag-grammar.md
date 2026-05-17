# Tag-on-DAG Grammar (v1)

Companion: [`host/tools/dag-coordinates.json`](dag-coordinates.json) (manifest_version 1).

Supersedes the engagement's prior sequentially-accreting `Ω.5.{letter}` discipline going forward only. Existing tags are not renamed; the grammar applies from the instituting commit forward.

## §1. Form

```
Ω.5.<pipeline>.<layer>.<handle>[.<seq>]
```

- `<pipeline>` is a pipeline id from the manifest: `P01`–`P16`. The short handle may be used in informal prose; the canonical tag uses the id.
- `<layer>` is a layer id from the manifest, drawn from either `above_engine_layers` (`L0`–`L6`) or `engine_layers` (`E0`–`E5`). A single tag carries exactly one layer.
- `<handle>` is a short stable kebab-case name for the substrate node the move touches: `getown`, `mod-ns-default`, `math-imul`, `bigint-arith`, `cjs-prefix`, `super-ctor-slot-name`.
- `<seq>` is an optional integer disambiguator. Its appearance should be treated as a smell: a unique `(pipeline, layer, handle)` is the design intent. If `<seq>` is needed, re-evaluate handle granularity first.

Examples:

```
Ω.5.P04.L5.bigint-arith
Ω.5.P03.L4.fn-expr-early-return
Ω.5.P05.L2.dotjson-bare
Ω.5.P06.E3.module-ns-default
```

## §2. Constraint

Two distinct substrate moves MUST differ in at least one of (`pipeline`, `layer`, `handle`). Same triple equals same move equals same tag.

Corollary: two work-streams that resolve to the same triple are the same recognition by definition. If they look different, the handle is too coarse, the layer is mis-assigned, or the pipeline attribution is wrong. Refine one of the three coordinates and re-tag both.

## §3. Migration discipline

Existing `Ω.5.{letter}` tags are NOT renamed retroactively. Commit hashes remain the canonical identifier across the engagement. The mapping table in §5 is documentation-only, hand-curated for the last contiguous stretch (EXT 7) so the reader can see how the new grammar dissolves the collisions the letter scheme produced.

`manifest_version: 1` is from the instituting commit forward. Old commits keep their old tags.

## §4. Worked example — EXT 7 stretch re-tag

The letter sequence CCCCCCCC through NNNNNNNN spans two distinct accretion runs that re-used the same letters. The collisions:

- **CCCCCCCC** appears as both the 60-package long-tail probe (compiler emit, callee value-shape) and as real BigInt arithmetic substrate (runtime semantics). Two pipelines, two layers, two handles, one letter.
- **EEEEEEEE** appears as both `String.prototype.localeCompare` + `buffer.constants.MAX_STRING_LENGTH` and as `Uint8Array.of()` / `TypedArray.of()` static.
- **NNNNNNN** (7-letter) and **NNNNNNNN** (8-letter) sit one character apart visually and are routinely conflated in prose.

Re-tag (documentation only; commit hashes are canonical):

| old tag | commit | new tag | recognition |
| --- | --- | --- | --- |
| Ω.5.CCCCCCCC (first) | `f3f38deb` | `Ω.5.P03.L3.callee-shape-probe` | compiler probes Op::Call + Op::New callee value-shape for the 60-pkg long-tail |
| Ω.5.DDDDDDDD (first) | `c5fd6a13` | `Ω.5.P06.L3.bright-zone-installs` | host installs surface bright-zone APIs surveyed in the prior probe |
| Ω.5.EEEEEEEE (first) | `6e2fa3c4` | `Ω.5.P08.L5.string-localecompare` | proto-chain dispatch + intrinsic for `String.prototype.localeCompare`; also `buffer.constants.MAX_STRING_LENGTH` install (co-touched: P06.L2) |
| Ω.5.FFFFFFFF | `63ed4c5f` | `Ω.5.P05.L1.dotjson-bare` | resolver no longer rejects bare-specifier `.json` |
| Ω.5.GGGGGGGG | `26ff1f27` | `Ω.5.P02.L0.fn-expr-early-return` | parser early-return on function/class expression removed; IIFE-then-ternary parses |
| Ω.5.HHHHHHHH | `9cc66a37` | `Ω.5.P04.L4.meta-substrate-probes` | runtime meta-substrate: SetProp/SetIndex receiver-hint, Function-name, Array-preview |
| Ω.5.IIIIIIII | `f65a1c2a` | `Ω.5.P16.L5.side-effect-imports` | side-effect ImportDeclarations evaluate per ECMA §16.2.1.5 |
| Ω.5.JJJJJJJJ | `22721de6` | `Ω.5.P04.L5.math-imul` | `Math.imul` + ECMA ToInt32/ToUint32 for bitwise ops (Invalid-curve cluster) |
| Ω.5.KKKKKKKK | `a29227d9` | `Ω.5.P05.L4.module-exports-getter` | module loader dispatches getter for `module.exports` per ECMA §10.1.8 |
| Ω.5.LLLLLLLL | `0488b6cd` | `Ω.5.P06.L3.gops-hasown-pathparse` | host intrinsics: `Object.getOwnPropertySymbols` + `Object.hasOwn` + `path.parse/format/relative` |
| Ω.5.MMMMMMMM | `6ea3209a` | `Ω.5.P06.L3.events-async-resource` | host: `events.EventEmitterAsyncResource` + `stream.EventEmitter` legacy alias |
| Ω.5.NNNNNNNN (8) | `4a423808` | `Ω.5.P03.L4.super-ctor-slot-name` | compiler: source-ident suffix on super-ctor slot name |
| Ω.5.CCCCCCCC (second) | `e8f5aee1` | `Ω.5.P04.L5.bigint-arith` | runtime: real BigInt arithmetic substrate |
| Ω.5.CCCCCCCC follow-up | `12eb60a4` | `Ω.5.P01.L0.stringtobigint-prefix` | lexer / StringToBigInt accepts `0x`/`0o`/`0b` prefixes |
| Ω.5.DDDDDDDD (second) | `67ff87e9` | `Ω.5.P04.L5.bigint-closure` | runtime: BigInt closure — snowflake / decimal / large arithmetic |
| Ω.5.EEEEEEEE (second) | `f9ad502c` | `Ω.5.P08.L3.typedarray-of-static` | proto/intrinsic: `Uint8Array.of()` / `TypedArray.of()` static |

All four prior collisions resolve: the two CCCCCCCCs differ on `(pipeline, layer, handle)`; the two DDDDDDDDs differ on `(layer, handle)`; the two EEEEEEEEs differ on `(layer, handle)`; the 7-vs-8-letter N collisions become non-issues because letter count is no longer load-bearing.

## §5. Open issues

**5.1 Multi-pipeline moves.** Some moves touch multiple pipelines simultaneously (per Doc 720 §II, the Ω.5.f classes round and Ω.5.dd Map/Set round touched compiler emit plus runtime dispatch plus intrinsic install). Convention: tag at the **primary** pipeline (where the load-bearing semantic change lives), and list co-touched pipelines parenthetically in the commit body as `co-touched: P0X.LY, P0Z.LY`. The EXT 7 re-tag above uses this convention for the `localeCompare` move. The composite-tag form (`Ω.5.P03+P04.L5.handle`) is rejected as a default because it complicates parsing and search.

**5.2 Cross-boundary substrate nodes.** A node may have above-engine surface (consumer touches L2 or L3) and below-engine cause (the engine cuts at E2 or E5). Example: Module Namespace default-synthesis is an above-engine surface concern but the cut sits at E5 (Bun's host realm augments construction). Convention: tag at the layer where the **change** lands, not where the **symptom** surfaces. If the fix is installed in the host's realm bootstrap, the layer is `E5`. If the fix is a surface install above the engine, the layer is `L2` or `L3`.

**5.3 `manifest_version` bumps.** v2 starts only when the pipeline enumeration or the layer enumeration changes (Doc 720 §VIII anticipates additions, e.g., a real Promise-resolution pipeline once async dispatch lands). Adding new handles does not bump the manifest. New alphabet members (a fifth K-class, a fifth stub-class) do bump the manifest, by §6 below.

**5.4 Handle stability.** Two handles for the same node across multiple moves should be the same string. The handle vocabulary is implicit; if a recurring node accretes synonyms (`bigint-arith` vs `bigint`, `module-ns-default` vs `mod-ns-default`), the catalogue tool should canonicalize. No catalogue tool exists yet; this is a deferred follow-up.

## §6. Relation to existing apparatus

Per **Doc 716** §II–V (three-projection tracker): `pipeline-id` is the DAG-projection, `layer` is the lattice-projection, and `handle` together with the manifest's alphabet sections (`stub_alphabet`, `engine_alphabet`) is the alphabet-projection. Each tag is a triple-projection coordinate over the substrate node touched.

Per **Doc 715** (the consumer-substrate DAG as load-bearing object): the DAG was already the load-bearing object beneath the joint MI lattice. The tag-on-DAG grammar names that object in the tag itself, rather than reading it off the commit body.

Per **Doc 727** §V Form 3 (the second articulation chain): the grammar produces a second articulation chain (positional, over the manifest's coordinate system) over the same substrate the prior `Ω.5.{letter}` chain (sequential, over time of accretion) ran over. Convergence between the two chains on individual moves corroborates substrate-tracking; divergence localizes basin self-reinforcement. Honest note: this is one form of self-corroboration, not a sufficient external read on its own.

Per **Doc 716** §V's alphabet-stability conjecture and **Doc 717** §VI's engine-cut stability conjecture: any handle that resists clean attribution to `(pipeline, layer)` is a candidate signal that the alphabet at that node is wider than the manifest knows. Such cases should be logged for review at the next manifest-version bump.
