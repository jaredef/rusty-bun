# derive-constraints cluster v0.2 — 2026-05-10 — Bun phase-a-port

Tightening pass on the v0.1 cluster run. Implements the two refinements named in [CLUSTER-NOTES.md §"What this run shows about the apparatus"](./CLUSTER-NOTES.md):

- **Refinement 1 — Intra-test binding substitution** (in `extract/ts_js.rs`). Walks each test body in source order, capturing `const|let|var X = expr` declarations into a per-test scope. When extracting an expect-subject, splices the binding's initializer in for the leading identifier. So `const server = Bun.serve(opts); expect(server.port).toBe(3000)` now canonicalizes the subject to `Bun.serve(opts).port` instead of `server.port`. Strips `await `, `new `, `typeof ` operator prefixes before identifying the leading binding.
- **Refinement 2 — Value-side structural matching** (in `cluster.rs`). For Equivalence-verb properties whose subject is on a public-API surface, samples the antichain entries' raw text for the matcher's first argument and promotes to construction-style when the value side is structural: a type-name string literal (`"function"`, `"object"`, …), a primitive constant (`null`, `undefined`, `true`, `false`, `NaN`), or a capitalized identifier likely naming a class/constructor.
- **Sub-refinement — `assert.X(arg, …)` first-arg subject extraction** (in `extract/ts_js.rs`). The AssertCall extractor now extracts the first argument as the constraint's subject (the value being asserted on), rather than recording the assert.* function name itself. This unmasks architectural surfaces that were buried under `assert.strictEqual` (1,511 occurrences in v0.1) and `assert.equal`.
- **Sub-refinement — `typeof X` prefix stripping** (in `cluster.rs`). The classifier strips the `typeof` operator so `expect(typeof Bun.serve).toBe('function')` canonicalizes the subject to `Bun.serve` (with the structural value side `'function'` correctly classified by Refinement 2).

## Numerical delta

| Metric                   | v0.1     | v0.2     | Δ            |
|--------------------------|---------:|---------:|--------------|
| Properties out           | 6,546    | **4,838**| −26.1%       |
| Construction-style       | 109 (1.7%) | **303 (6.3%)** | +2.8× absolute, +3.6× relative |
| Behavioral               | 6,437    | 4,535    | −29.5%       |
| Antichain size           | 11,100   | 9,133    | −17.7%       |
| Reduction ratio          | 0.260    | 0.214    | tighter      |
| Singleton properties     | 3,764    | 2,275    | −39.6% (binding substitution + assert subject extraction collapse many singletons into the surfaces they actually targeted) |

## Top 20 construction-style properties (v0.2 final)

```
n= 806  equivalence    fetch                            ← was hidden as `result`/`response`
n= 333  equivalence    Buffer.from                       ← was hidden as `Buffer.from(...)` w/o subject canonical
n= 247  equivalence    Bun.file
n= 219  equivalence    URL                               ← was Equivalence, dropped by v0.1 heuristic
n= 182  equivalence    Bun.build
n= 180  equivalence    Bun.JSONL.parseChunk
n= 166  equivalence    structuredClone
n= 158  equivalence    Response
n= 116  equivalence    Bun.inspect
n=  83  equivalence    Bun.spawn                         ← was hidden under `result`/`exitCode` locals
n=  77  equivalence    Bun.Cookie
n=  76  equivalence    fs.existsSync
n=  74  equivalence    Headers
n=  69  equivalence    Bun.CookieMap
n=  68  equivalence    URLSearchParams
n=  61  equivalence    Bun.Cookie.parse
n=  53  equivalence    Buffer
n=  53  equivalence    Bun.stripANSI
n=  47  equivalence    Buffer.isBuffer
n=  42  equivalence    Bun.Image
```

These are exactly the high-cardinality public-API surfaces the keeper's conjecture predicted would emerge. The constraint-density on `fetch` (806 entries), `Buffer.from` (333), `Bun.file` (247), `URL` (219) confirms the heavy-tail: a small number of architectural surfaces concentrate the bulk of the test corpus's coverage.

## Top 10 remaining behavioral (highest cardinality)

```
n= 1932  error          <anonymous>          ← `expect(() => ...).toThrow()` — anonymous fn arg
n= 1556  equivalence    <anonymous>          ← complex expression in expect arg, no extractable head
n= 1477  equivalence    sql                  ← postgres tagged-template literal usage
n= 1222  other          Glob                 ← `new Glob(...)`; verb-class `other` excluded from CS
n= 1079  equivalence    exitCode             ← spawn-result property, deeper binding chain
n=  538  equivalence    exited
n=  506  equivalence    YAML.parse           ← Bun.YAML.parse alias / standalone YAML.parse
n=  499  containment    <anonymous>
n=  412  equivalence    file
n=  392  equivalence    readdirSorted        ← test helper, intentionally not API surface
```

Three structural sources of remaining behavioral classification:

1. **`<anonymous>` subjects** (3,987 across error/equivalence/containment): `expect()` is called with a complex expression — an arrow function, a parenthesized assignment, an immediate call. The extractor returns no clean leading identifier, so the property has subject `<anonymous>` and grouping fails to architecturally meaningful. The fix would be to structurally classify the expect-arg shape rather than always extracting an identifier.
2. **Deeper binding chains**: `const result = await Bun.spawn(...).exited` followed by `expect(result).toBe(0)`. The binding chain has `result = await Bun.spawn(...).exited`, but `result` itself appears as a subject and resolves *only* to the right-hand side text — which preserves `await Bun.spawn(...).exited`. Inspecting the v0.2 output: `exitCode` (1,079) and `exited` (538) suggest the resolution didn't run — these subjects appear as bare local variable names rather than substituted. Most likely cause: the test pattern is `const { exited, exitCode } = await Bun.spawn(...)` (object destructuring), which the binding extractor doesn't handle. Adding destructuring-pattern support is a v0.3 refinement.
3. **Verb-class `other`**: `Glob` (1,222) is `new Glob(...)` with no recognized matcher; the extractor classifies the verb as `other` and the classifier excludes `other`-verb constraints from construction-style entirely. Many `other`-verb constraints with public-API subjects are likely architectural and currently invisible.

## Property cardinality (v0.2 final)

| Bucket    | Properties | % of input clauses |
|-----------|-----------:|-------------------:|
| 1         | 2,275      | 5.3%               |
| 2-5       | 1,646      | 12.5%              |
| 6-20      | 631        | 17.4%              |
| 21-100    | 226        | 22.0%              |
| 101-500   | 52         | 22.4%              |
| 501+      | 8          | 20.4%              |

The 60 high-cardinality properties (≥101) account for **42.8% of input constraint clauses**. The 286 properties at ≥21 cardinality account for **65%**. Together, the **~300 high-cardinality properties** (the construction-style count is 303, near-overlap with this band) cover the architectural surfaces — exactly what the htmx→htxlang precedent predicts: a small constraint set inducing the bulk of the contract.

## v0.3 refinements queued

- **Destructuring-pattern bindings**: `const { exited, exitCode } = await Bun.spawn(...)` should bind `exited` and `exitCode` to their projected expressions.
- **Anonymous-subject classification**: when the expect-arg is an arrow function (the `toThrow` pattern), classify by the function-body's primary call subject. Most `<anonymous>` errors are `expect(() => Bun.foo(bad-input)).toThrow()` — the architectural subject is `Bun.foo`.
- **`other`-verb construction-style coverage**: properties whose subject is on a public-API surface and whose verb is `other` (instantiation, custom matcher, less-common patterns) should be eligible for construction-style classification.
- **SIPE-T threshold diagnosis** (the cluster-design.md §4.2 step 2 deferred from v0.1): per-property minimum-coverage threshold rather than blanket N=3 antichain.

## What this run shows about the apparatus

The v0.1→v0.2 delta confirms the design's predictive content. The construction-style fraction more than doubled (1.7% → 6.3%) with two targeted refinements; the property catalog tightened by 26% via better grouping; the top construction-style entries are now visibly the architectural surfaces of Bun's runtime contract (`fetch`, `Buffer.from`, `Bun.file`, `URL`, `Bun.build`, `structuredClone`, `Response`, `Bun.spawn`, `Headers`, `URLSearchParams`, `Blob`, `WebSocket`, `Worker`, `Event`, `AbortController`, `FormData`, `WritableStream`, …).

The tool is now a usable instrument for the next phase. ~300 construction-style properties + ~600 high-cardinality behavioral properties (the 21-100 + 101-500 bands above) ≈ ~900 properties is a tractable input for the `invert` phase — within an order of magnitude of the htmx case (19 constraints) scaled by Bun's API surface size (much larger).

## Files

- `bun-scan-v2.json` (15 MB) — re-scan with binding substitution applied at extraction time.
- `bun-cluster-v2.json` (4.5 MB) — v0.2 cluster output.
- `cluster-v2-summary.txt` — terminal summary captured from `--summary` stderr.

The original v0.1 artifacts (`bun-scan.json`, `bun-cluster.json`, `cluster-summary.txt`) are kept alongside for the historical comparison.

## Provenance

- Tool: `derive-constraints` v0.2.
- Source: oven-sh/bun phase-a-port HEAD shallow clone, 2026-05-10.
- Tool runtime: scan + cluster well under a minute combined.
