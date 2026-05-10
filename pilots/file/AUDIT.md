# File pilot — coverage audit

Fifth pilot. Inheritance/extension class — File extends Blob in the W3C File API. Pilot 4 (Blob) established the substrate; this pilot tests whether the apparatus handles **inheritance/extension** as a derivation class. The pair (Blob → File) is the canonical web-platform composition example.

## Constraint inputs

From `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/file.constraints.md`:

| Property | Cardinality | Class | Source |
|---|---:|---|---|
| FILE1 | 22 | construction-style | Bun tests + spec extract (cross-corroborated) |
| FILE2 | 1 | construction-style | spec extract |

Plus 4 spec-derived clauses on `File.prototype.{name, lastModified, webkitRelativePath}` and the "File extends Blob" relationship. Total: 27 clauses across ~5 properties.

Antichain reps drawn from real Bun tests include:
- `expect(typeof File !== "undefined").toBe(true)` — global existence
- `expect(file.name).toBe("example.txt")` — name preservation through structuredClone
- `expect(blob.name).toBe("file.txt")` — File-from-Uint8Array constructor with name
- `expect(new File([], "x")).toBeInstanceOf(File)` — class identity

## Pilot scope

Implement File per W3C File API §4:
- `new File(parts, name)` constructor
- `new File(parts, name, options)` with `FilePropertyBag` (`type`, `endings`, `lastModified`)
- Inherits Blob's `size`, `mime_type`, `slice`, `text`, `array_buffer`, `bytes` via delegation
- Own getters: `name`, `last_modified`, `webkit_relative_path`
- File extends Blob — instances should pass both `is_blob` and `is_file` predicates

## Approach: composition over inheritance

Rust has no class inheritance. The pilot models File-extends-Blob as **composition**: File holds an inner Blob plus File-specific fields. Delegation methods proxy to the inner Blob. This matches the spec's "inherits from Blob" idiomatically — the IDL's `interface File : Blob` corresponds to having a Blob as a member.

The structural-equivalence test `instanceof Blob` becomes "File can be coerced to Blob view" — pilot exposes `as_blob()` for explicit access. This is the type-system analog of JS's prototype-chain check.

## LOC budget

Bun's `File.rs`/`File.zig` is likely small (File is mostly metadata over a Blob). WebKit's `File.{h,cpp}` is ~100-200 LOC. Pilot target: ~50-70 LOC since most of the surface is Blob delegation.

## Ahead-of-time hypotheses

1. **The pilot will be the smallest derivation in the apparatus.** File adds 3 fields + 1 constructor over Blob. Most behavior is delegation.
2. **Composition-as-inheritance pattern will be free of borrow-checker friction.** Owned Blob inside File — no shared identity, no cycles. Different from structuredClone's Heap-with-Ids requirement.
3. **No verifier-caught-derivation-bug expected this pilot.** File's surface is mostly metadata-preservation tests; the spec is unambiguous; the constraints are concrete. Pilot 4's bug-catch was on slice's swapped-endpoints subtlety — File has no analogous semantic ambiguity.
4. **Cross-corroboration of FILE1's 22 reps suggests Bun's File implementation is well-tested at the metadata-preservation layer.** Pilot exercises the same metadata invariants and should pass cleanly.

## Verifier strategy

CD antichain reps + spec extracts. Target ~15 tests. Pilot succeeds if:
- 100% pass with 0 documented skips
- Composition-as-inheritance pattern lands without borrow-checker complications
- LOC ratio against scope-honest target (in-memory File without backing-store) is below the URLSearchParams 62% delegation-target ratio
