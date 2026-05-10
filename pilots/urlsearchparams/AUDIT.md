# URLSearchParams pilot — coverage audit

Second pilot in the rusty-bun apparatus, exercising the v0.13/v0.13b enriched constraint corpus. Counterpart to [pilots/textencoder](../textencoder/), now with spec-source ingestion landed.

## Constraint inputs

From `runs/2026-05-10-deno-v0.13b-spec-batch/cluster.json` after the v0.13b spec corpus extension:

| Property subject | Cardinality | Class | Source |
|---|---:|---|---|
| URLSearchParams | 7 | behavioral | **BOTH** (spec ∩ test) |
| URLSearchParams | 5 | behavioral | spec |
| URLSearchParams | 1 | construction-style | spec |
| URLSearchParams.prototype.append | 2 | behavioral | spec |
| URLSearchParams.prototype.delete | 2 | behavioral | spec |
| URLSearchParams.prototype.get | 2 | behavioral | spec |
| URLSearchParams.prototype.getAll | 2 | behavioral | spec |
| URLSearchParams.prototype.has | 2 | behavioral | spec |
| URLSearchParams.prototype.set | 2 | behavioral | spec |
| URLSearchParams.prototype.sort | 2 | behavioral | spec |
| URLSearchParams.prototype.toString | 2+1 | behavioral | spec |
| URLSearchParams.prototype.size | 1 | behavioral | spec |
| URLSearchParams.prototype.entries | 1 | behavioral | spec |
| URLSearchParams.prototype.keys | 1 | behavioral | spec |
| URLSearchParams.prototype.values | 1 | behavioral | spec |
| URLSearchParams.prototype.forEach | 1 | behavioral | spec |
| **17 properties total** | **35 clauses** | | |

This is the **dramatically richer** constraint surface the v0.13b spec ingestion produced. By contrast, the TextEncoder pilot's auto-emitted constraint doc had **6 clauses across 1 property**. Here the apparatus emits **35 clauses across 17 properties**, with 8 distinct method-level subjects — exactly the per-method granularity a derivation step needs.

The 1 cross-corroborated property (`URLSearchParams` itself, cardinality 7) is the strongest constraint: both Deno's tests and the WHATWG URL §5.2 spec witness it. The Deno test reps include real-world toString cases:
- `assertEquals(searchParams, "str=this+string+has+spaces+in+it")`
- `assertEquals(searchParams, "str=hello%2C+world%21")`

These two reps alone pin down the form-urlencoded space-as-plus and percent-encoding semantics. Combined with the spec extracts that say `URLSearchParams.prototype.toString joins entries with "&" and pairs with "="` and `URLSearchParams.prototype.toString percent-encodes per the form-urlencoded character set`, the toString contract is fully constrained.

## What the v0.13b apparatus enables that the TextEncoder pilot's input did not

| Surface element | TextEncoder pilot | URLSearchParams pilot |
|---|---|---|
| Constructor existence | ✓ | ✓ |
| Method-level subjects | ✗ (only top-level subject) | ✓ (8 distinct method subjects) |
| Method semantics | ✗ (had to inject from spec manually) | ✓ (spec-derived clauses present) |
| Test-corpus witness for behavior | ✗ (negative-case only) | ✓ (cross-corroborated toString cases) |
| Coverage gap | gap-A on every method | gap-A is closed by spec ingestion |

This is the v0.13b payoff. The pilot can now operate from constraint inputs alone without the manual spec-injection overhead the TextEncoder pilot required.

## Pilot scope

Implement `URLSearchParams` as a Rust library:
- Internal storage: ordered `Vec<(String, String)>` (preserves insertion order; spec mandates ordered list)
- Constructor: from string (form-urlencoded), from `&[(K, V)]` pairs, from `&[(K, V)]` records
- Methods: `append`, `delete` (with optional value), `get`, `get_all`, `has` (with optional value), `set`, `sort` (stable, by name in code-unit order), `to_string`, `size`
- Iteration: `entries`, `keys`, `values`, `for_each` callback adapter
- form-urlencoded encode/decode per WHATWG URL §5.2.5

Out-of-scope for this pilot:
- HTMLFormElement constructor input (browser-only)
- IDL-level argument validation (Rust's type system does this)
- URL backreference (the pilot is standalone; URL.searchParams association is a separate pilot)

## LOC budget

Bun's hand-written URLSearchParams Rust port lives at:
```
/tmp/welch-corpus/target/bun/src/runtime/webcore/URLSearchParams.rs (TBD: measure on disk)
```

Naive ratio target: 13-25% (per the TextEncoder pilot's measured range).

## Verifier strategy

The verifier consumes:
1. **Cross-corroborated antichain reps** as ground-truth tests (toString cases from Deno).
2. **Spec-derived antichain reps** as additional tests, transcribed as Rust assertions.
3. **WHATWG URL §5.2.5 form-urlencoded test vectors** as the encoder/decoder oracle.

Pass/fail/skip per constraint, same as the TextEncoder pilot. Pilot succeeds if:
- 100% of cross-corroborated reps pass.
- ≥ 90% of spec-derived reps pass.
- Documented skips are explicitly noted as out-of-pilot-scope (browser-only forms, IDL-level coercions).

## Ahead-of-time hypotheses

1. The **stable-sort-by-code-unit** spec wording for `sort()` will require a careful test (UTF-16 ordering on identifiers — different from byte ordering for non-ASCII).
2. The **form-urlencoded encoder set** is *narrower* than `application/x-www-form-urlencoded` from RFC 1738; the Deno test rep `"hello%2C+world%21"` (encoding `, ` and `!`) confirms this.
3. The **constructor-from-string optional leading "?"** is non-obvious; spec extract clause `URLSearchParams constructor accepts a query-string starting with optional "?"` makes it explicit. Pilot must handle this.
4. The pilot will likely surface a **second classifier-noise finding** in the cluster phase, similar to v0.12's TEXT2 fix — bound `params` variables in real test corpora propagate widely. This is a pre-registered prediction; if no such finding emerges, that itself is data.
