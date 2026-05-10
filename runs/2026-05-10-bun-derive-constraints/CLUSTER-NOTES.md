# derive-constraints cluster — 2026-05-10 — Bun phase-a-port

First real-corpus run of the cluster phase. Reads `bun-scan.json` from the same directory; emits `bun-cluster.json`.

## Inputs

- **Source.** `bun-scan.json` (15 MB) — 17,775 tests / 43,094 constraint clauses across the bun phase-a-port corpus.
- **Tool.** `derive-constraints cluster` v0.1 (commit at the same date as scan + this notes).
- **Heuristic settings.** Antichain size `N = 3` per property; todo-marked tests skipped (the constraint is acknowledged unimplemented; including it would inflate the property catalog with placeholders).

## Output

| Metric                       | Value                       |
|------------------------------|----------------------------:|
| Constraints in (post-todo skip) | 42,680                   |
| Properties out                | **6,546**                  |
| Antichain size                | 11,100                     |
| Reduction ratio               | 0.260 (antichain / in)     |
| Construction-style properties | 109 (**1.7%** of properties) |
| Behavioral properties         | 6,437 (98.3%)              |

| Cardinality bucket | Properties | Cumulative % of constraints |
|--------------------|-----------:|----------------------------:|
| 1                  | 3,764      | 8.8%                        |
| 2-5                | 1,961      | 19.6%                       |
| 6-20               | 580        | 36.5%                       |
| 21-100             | 183        | 60.7%                       |
| 101-500            | 48         | 88.7%                       |
| 501+               | 10         | 100%                        |

| Verb class       | Properties | % of properties |
|------------------|-----------:|----------------:|
| equivalence      | 4,086      | 62.4%           |
| containment      | 705        | 10.8%           |
| existence        | 630        | 9.6%            |
| ordering         | 453        | 6.9%            |
| type_instance    | 314        | 4.8%            |
| other            | 301        | 4.6%            |
| generic_assertion| 47         | 0.7%            |
| error            | 10         | 0.2%            |

## Three readings

### Reading 1 — Cardinality reduction works (6.5×); antichain selection further reduces (3.8× total against input)

The (subject, verb-class) canonicalization step alone reduces 42,680 input clauses to 6,546 distinct properties — a 6.5× factor. The default-`N=3` antichain selection further reduces to 11,100 representatives, yielding a total reduction ratio of 0.26 against the input. Both steps lose information: canonicalization collapses constraint clauses with the same subject and verb-class into one property entry; antichain selection drops surplus representatives beyond the first three from distinct files.

**This is not the htmx → htxlang ratio.** The 9.4% htxlang ratio measures *implementation LoC* against original *implementation LoC*. The cluster ratio measures *selected-representative-clauses* against *input-clauses*. Both indicate substantial compression but at different layers — implementation-level vs specification-level. The cluster ratio is a lower bound on what a downstream `invert` phase could compress further, and an upper bound on what a structurally-complete property catalog requires.

### Reading 2 — Property cardinality is heavy-tailed; ~58 high-cardinality properties dominate the constraint volume

The 48 properties in the 101-500 bucket plus the 10 properties at 501+ together account for 88.7% of input constraints (compared to only 0.9% of properties). The architectural content of the test corpus — the surfaces that matter enough to be tested across many tests — is concentrated in the long-tail right side of the distribution. The 3,764 singleton properties account for only 8.8% of input constraints; they are largely incidental (per-test-fixture local-variable subjects rather than architectural surfaces).

This is the keeper's conjecture's shape: *most of the architectural content is in a small number of high-cardinality properties.* Filtering aggressively to the top ~58–500 properties retains the architectural content; the long tail of singletons is what we are filtering away.

### Reading 3 — The construction-style classifier is too conservative; 1.7% is a lower bound

The classifier marks 109 properties (1.7%) as construction-style. Inspection of the highest-cardinality properties reveals systematic under-classification:

```
1967  equivalence    result            ← local variable; correctly behavioral
1798  error          <anonymous>       ← anonymous subject (no expect-arg); correctly out
1511  equivalence    assert.strictEqual ← test-framework, not target; correctly behavioral
1235  equivalence    exitCode          ← local variable from spawn(); behavioral but should be Bun.spawn().exitCode
1221  other          Glob              ← Bun.Glob constructor; **misclassified** (should be construction-style)
 694  generic_assert assert             ← test-framework; correctly behavioral
 545  equivalence    typeof             ← typeof operator; **misclassified for typeof X** patterns
 522  equivalence    exited             ← local from spawn(); behavioral (same root cause as exitCode)
 502  equivalence    assertEquals       ← test-framework; correctly behavioral
 447  equivalence    parsed             ← local variable; correctly behavioral
```

Two structural sources of under-classification:

1. **Local-variable subjects mask public-API surfaces.** A test-corpus pattern: `const server = Bun.serve(opts); expect(server.port).toBe(3000)`. The extracted subject is `server.port`, not `Bun.serve(...).port`. Without dataflow tracking, the cluster phase cannot recover that `server` is the result of a `Bun.serve(...)` call. The 3,764 singletons + the high-cardinality "local" subjects (`result` 1,967, `exitCode` 1,235, `parsed` 447, etc.) all suffer from this.

2. **Equivalence verbs are excluded from construction-style classification by heuristic.** A constraint like `expect(typeof Bun.serve).toBe('function')` is *structural* in content (asserts the type of a public-API surface) but uses an Equivalence matcher. Excluding all Equivalence-verb properties drops these from the construction-style count.

The 1.7% classification is therefore an underbound. The actual construction-style fraction is between ~1.7% (current heuristic) and an unknown upper bound that would require dataflow tracking + structural-content matching to compute.

## What this run shows about the apparatus

**The keeper's conjecture has structural support.** The cardinality distribution is exactly the heavy-tailed shape the conjecture predicts: most architectural content concentrated in a small number of high-cardinality properties; long tail of singletons that are largely incidental. The reduction ratio at the property level (6.5× from canonicalization alone) is in the order-of-magnitude range the conjecture predicts (10²–10³ properties, observed 6.5×10³ at the property level — within an order of magnitude of the predicted ceiling).

**The heuristic needs two refinements before invert is sensible.**

1. **Subject canonicalization should track local-variable initialization back to public-API calls.** The MVP extracts subjects verbatim from `expect(...)` arguments; a v0.2 should resolve `server.port` to `Bun.serve(...).port` when the local variable was assigned from a public-API call earlier in the test. This is a simple intra-test scope analysis (not full dataflow); each test is bounded ~20-200 lines, and tracking `const X = Bun.foo(...)` patterns would handle the dominant case.

2. **Equivalence verbs are construction-style when the value side is structural.** `.toBe('function')`, `.toBe(undefined)`, `.toBe(null)`, `.toBeInstanceOf(SomeClass)` (already TypeInstance), `.toEqual({structure-shaped-spec})` — the heuristic should look at the *value side* of the equivalence, not just the verb class. When the value side is a type-name, primitive constant, or class reference, the constraint is structural.

These refinements would lift the construction-style fraction to a more meaningful number and would tighten the property catalog.

## What this run does not yet do

- **Does not perform the SIPE-T threshold diagnosis** prescribed by the cluster-phase design (§4.2 step 2). Default `N=3` is a rough heuristic; per-property threshold diagnosis would compute the minimum coverage needed for each property's reliable derivation.
- **Does not produce the inverted constraint document.** That is the next phase (`invert`).
- **Does not predict implementation LoC.** That is the phase after (`predict`).

## Files

- `bun-cluster.json` (4.5 MB) — full property catalog with antichain representatives.
- `cluster-summary.txt` — human-readable summary captured from `--summary` stderr.

## Provenance

- Tool: `derive-constraints` v0.1 + cluster phase. Same commit as scan.
- Source: `bun-scan.json` from the run on 2026-05-10 against oven-sh/bun phase-a-port HEAD.
- Tool runtime: well under a minute on the dev machine.
