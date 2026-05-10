# structuredClone pilot — coverage audit

Third pilot in the rusty-bun apparatus. Most-thoroughly-witnessed property in the corpus: 227 clauses across 5 cluster groups, including 166 cardinality on STRU1 alone — the strongest single-surface coverage measured.

## Constraint inputs

From `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/structuredclone.constraints.md`:

| Property | Cardinality | Class | Witnessing source |
|---|---:|---|---|
| STRU1 | 166 | construction-style | Bun tests + spec extract (cross-corroborated) |
| STRU2 | 39 | construction-style | Bun tests + spec extract (cross-corroborated) |
| STRU3 | 5 | construction-style | Bun tests + spec extract |
| STRU4 | 3 | construction-style | spec only (DataCloneError throws) |
| STRU5 | 14 | behavioral | spec extract |
| **Total** | **227** | | |

The 166-cardinality STRU1 is the apparatus's strongest single-surface witness. Antichain representatives drawn from real Bun tests include:
- `expect(cloned).toStrictEqual({})` — empty object roundtrip
- `expect(cloned.size).toBe(0)` — Blob preservation
- `expect(cloned.file.name).toBe("example.txt")` — File preservation
- `expect(cloned).toBeInstanceOf(Array)` — class preservation across modified prototype
- `expect(cloned[0].a).toBeNull()` / `toBeUndefined()` — null/undefined property preservation

## Pilot scope

A pure-Rust simulated derivation of structured-clone is not 1:1 with the JS spec because:

- The spec is defined in terms of *JS values*; pure-Rust has no JS engine.
- The spec describes *transfer* via ArrayBuffer detachment; that requires JS-engine semantics.
- DOM nodes, MessagePort, etc. are platform types not present in pure-Rust.

The pilot models a **`Value` enum representing the JS structured-cloneable type universe** as a graph (because circular references must be supported per spec). The clone algorithm is then well-defined as a graph operation. This is the honest scope: the apparatus's claim is that the *algorithm* is derivable, not that the JS-engine integration is.

In-scope:
- Primitives: null, undefined, boolean, number, BigInt (i128 simplification), string
- Algebraic types: Date (epoch ms), RegExp (source + flags), Map (ordered), Set (ordered)
- Containers: plain object (string-keyed), array
- Buffers: ArrayBuffer (Vec<u8>); typed-array views (kind + buffer ref + offset + length)
- **Identity preservation**: shared references in input → shared references in output
- **Circular references**: handled via index-based serialization
- **Blob/File simplified**: pilot's Blob is `{size, type, bytes}`; File extends with `{name, lastModified}`. This is enough to satisfy the antichain reps that test Blob/File clone preservation.
- **Error cases**: function value → DataCloneError; explicit non-cloneable marker → DataCloneError.

Out of scope:
- ArrayBuffer transfer (requires JS-engine detachment semantics)
- DOM nodes (no analog in pure-Rust)
- MessagePort transfer (platform-specific)
- Symbols (would require a global symbol registry; orthogonal to clone)
- Error/Event/etc. platform types

## Approach

The spec defines structured-clone as a two-phase algorithm:
1. **StructuredSerialize**(value, memory) — produces a serialization record.
2. **StructuredDeserialize**(record, targetRealm) — produces a fresh value graph.

The pilot mirrors this exactly. Identity preservation falls out naturally: the serialization records reference shared values by index; deserialization rebuilds the graph using those indices. Circular references are handled by registering the target slot in `memory` *before* recursing, so a cycle that loops back through the same value finds the slot already filled.

This matches the rederive-style "two-phase derive" pattern: serialize is the formal-spec output; deserialize is the substrate's reconstruction. The pilot's pipeline is the spec's pipeline, transcribed to Rust.

## Verifier strategy

The verifier consumes:
1. **Bun antichain reps** transcribed to Rust assertions, mapped through the Value-type model
2. **Spec extract clauses** (STRU5 behavioral) as additional structural tests
3. **Targeted edge-case tests** for circular references, identity preservation, container types

Pilot succeeds if:
- 100% of in-scope antichain reps pass
- DataCloneError emerges correctly on Function values
- Circular references roundtrip without infinite recursion
- Shared-reference identity is preserved (graph topology preserved)

## Ahead-of-time hypotheses

1. **Identity preservation requires the Heap-with-Ids model.** A naive recursive clone would fail to preserve the topology where two different paths in the input graph terminate at the same node.
2. **The Value enum's natural Rust translation handles primitives by value automatically.** Rust's move/copy semantics naturally clone primitives; the algorithm's "primitives by value" clause is trivially satisfied.
3. **Circular references will work without special handling beyond the index-based serialization.** The two-phase approach handles cycles naturally because indices are assigned in pre-order.
4. **The pilot will not surface a v0.14 apparatus work item.** The constraint corpus + spec material is sufficient. (Pre-registered prediction.)

## LOC budget

Naive estimate: ~300-450 LOC for the algorithm + Value type + Heap, due to the breadth of supported types. Verifier: ~200-300 LOC across 30-50 tests.

Bun's own structured-clone code lives in WebKit's serialization machinery (`SerializedScriptValue.cpp`) which is several thousand LOC of C++ — but it handles full JS-engine integration. Pilot's competitive comparison is against **the algorithm itself**, not Bun's WebKit-glue contribution.

This is the [URLSearchParams pilot's Finding 3](../urlsearchparams/RUN-NOTES.md) re-applied: where Bun delegates to upstream, the apparatus's value claim is "competitive with upstream + eliminates binding layer."
