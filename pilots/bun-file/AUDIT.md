# Bun.file pilot — coverage audit

**Eleventh pilot. First Tier-B Bun-namespace pilot.** Per the trajectory's Tier-B priority order. First pilot on a **fully Tier-2 ecosystem-only** surface — no WHATWG/W3C spec exists; Bun's tests ARE the spec.

## Constraint inputs

From `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/bun.constraints.md`:

| Property cluster | Cardinality | Notes |
|---|---:|---|
| `Bun.file` (containment / shape invariants) | 266 | behavioral |
| `Bun.file` (production patterns) | 198 | behavioral; mostly `.size` and `.text()` round-trips |
| `Bun.file` (class identity) | minor | `instanceof Blob` antichain rep |
| `Bun.file` (existence) | minor | construction-style |

**Total: ~470+ cross-corroborated clauses across `Bun.file` cluster groups.** This is the densest Bun-namespace surface in the corpus.

Antichain reps drawn from real Bun tests:
- `expect(file).toBeInstanceOf(Blob)` — BunFile extends Blob
- `expect(file.name).toEqual(import.meta.filename)` — `.name` returns the path
- `expect(bunStat.size).toBe(Buffer.byteLength(content))` — size reflects file byte length
- `expect(newSnapshot).toBe(await Bun.file(...).text())` — text() reads full UTF-8

## Pilot scope

In scope:
- `Bun::file(path)` → `BunFile`
- `BunFile::name()` — the path
- `BunFile::size()` — file byte length (queried lazily on first call)
- `BunFile::mime_type()` — inferred from extension or explicit
- `BunFile::last_modified()` — mtime in milliseconds since epoch
- `BunFile::exists()` — boolean predicate
- `BunFile::text()` — read full file as UTF-8 string
- `BunFile::bytes()` / `BunFile::array_buffer()` — read full file as bytes
- `BunFile::slice(start, end?, content_type?)` — returns a Blob (per spec, slicing returns Blob, not File)
- `BunFile::as_blob()` — coerce to Blob view (the `instanceof Blob` analog)

Out of pilot scope:
- `BunFile::stream()` returning ReadableStream — would compose with the Streams pilot but is deferred
- `BunFile::writer()` — write-side counterpart
- `BunFile::unlink()` — file deletion
- `BunFile::fd` — file descriptor
- Async/Promise semantics — pilot uses synchronous I/O analogs
- S3/network-backed files — Bun.file accepts `s3://` URLs; deferred

## Approach

`BunFile` composes with the rusty-blob substrate from Pilot 4: it owns a path + lazily-read bytes + metadata. Read methods touch the filesystem via `std::fs`. This is the **first pilot with real I/O** in the apparatus — all prior pilots were pure data-layer derivations.

The first I/O test of the pilot is what makes it a real Tier-B anchor: the apparatus must produce a derivation that reads files correctly, not just matches a spec.

## Ahead-of-time hypotheses

1. **Pilot is small in LOC, large in surface.** `Bun.file`'s API is mostly delegation to Blob (via composition) plus a few file-system-reading entry points. Estimated: 100-150 code-only LOC.

2. **The verifier suite needs a real-file fixture.** Pilot's tests will create a temp file, then derive a BunFile, then assert reads. AOT: tests use `std::env::temp_dir()` for isolation.

3. **The slice() returns Blob, not BunFile** invariant transcribes naturally into Rust's type system (returns `rusty_blob::Blob`, not `Self`). Pattern carried over from File pilot.

4. **First-run clean closure expected.** Bun.file's surface is concrete and the Bun tests + Bun docs define it well. AOT: 100% pass without verifier-caught derivation bugs.

## Verifier strategy

~15-20 verifier tests. Includes real I/O via temp files. Pilot succeeds if:
- Verifier closes with 0 fail
- I/O paths work cleanly across test invocations
- LOC ratio against Bun's `Bun.file`-bearing source is in apparatus' claimed range

Consumer regression: ~6-8 tests citing real consumer use of `Bun.file` (mostly Bun-internal at this point since Bun.file is Bun-specific; cite to Bun's own test corpus is acceptable for Tier-2 surfaces).
