# Blob pilot — coverage audit

Fourth pilot. Composition-class — Blob is the substrate that File extends. After pilot 3 (structuredClone, algorithm) showed the apparatus generalizes from data structures to algorithms, this pilot tests whether the apparatus handles **composition / has-a** relationships. Blob → File is the canonical web-platform inheritance pair.

## Constraint inputs

From `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/blob.constraints.md`:

| Property | Cardinality | Class | Source |
|---|---:|---|---|
| BLOB1 | 17 | construction-style | Bun tests + spec extract (cross-corroborated) |
| BLOB2 | 2 | construction-style | Bun tests + spec extract |
| BLOB3 | 1 | construction-style | Bun tests only |

Plus 6 spec-derived behavioral clauses on `Blob.prototype.{size,type,slice,text,arrayBuffer,bytes,stream}`. Total: 26 clauses across 9 properties.

Antichain representatives drawn from real Bun tests include:
- `expect(typeof Blob !== "undefined").toBe(true)` — global existence
- `expect(blob.size).toBe(6)` — basic byte-length probe (`new Blob(["abcdef"])`)
- `expect(new Blob([])).toBeInstanceOf(Blob)` — class preservation
- `expect(blob.name).toBeUndefined()` — Blob has no name property (regression #10178)
- FormData multipart roundtrip preserving Blob equality

## Pilot scope

Implement Blob per W3C File API §3:
- Constructor from parts (sequence of `BufferSource`, `USVString`, or another `Blob`)
- Constructor `BlobPropertyBag` with `type` and `endings`
- `size` getter
- `type` getter — lowercased ASCII per spec
- `slice(start, end, contentType)` — clamped offsets, optional content-type override
- `text()` — UTF-8 decode of full byte content
- `array_buffer()` / `bytes()` — raw byte access
- Endings normalization: `"transparent"` (default) vs `"native"` (CRLF on Windows-target, LF elsewhere)

Out of pilot scope:
- `stream()` returning `ReadableStream` — would require a streams pilot upstream
- File extension (separate pilot, will reuse Blob as substrate)
- Object URL registry (browser-only)
- Blob equality semantics in FormData multipart context (delegated to FormData impl)

## LOC budget

Bun's `Blob.zig` and `Blob.rs` likely exceed 1,000 LOC because Bun's Blob is a load-bearing IO primitive (file-backed Blobs, lazy I/O, multi-part backing stores). The pilot's Blob is a pure-bytes Blob without lazy I/O. Fair comparison: against a similarly-scoped Blob (bytes + type + slice + decode), the WebKit reference impl is ~200-400 LOC. Pilot target: ~100 LOC.

## Ahead-of-time hypotheses

1. Blob's slice semantics with negative offsets (clamped to size + offset) will be non-obvious from constraint corpus alone — spec extract carries this; AUDIT predicts the spec material is necessary.
2. Endings normalization is platform-specific; pilot defaults to LF (Unix native), matches `"transparent"` default.
3. ASCII-lowercasing of `type` will require explicit handling — not visible from constraint reps but in spec.
4. The pilot will demonstrate the **Heap-with-Ids architectural pattern from structuredClone is unnecessary here** — Blob doesn't need cycles or shared identity; pure value semantics suffice. Different pilot class, different architecture.

## Verifier strategy

CD-derived antichain reps + spec-extract clauses. Target ~25 tests. Pilot succeeds if 100% pass with 0 documented skips, matching the URLSearchParams and structuredClone closures. If any fail, the failure is data — surfaces what the apparatus's constraint document was missing for this composition class.
