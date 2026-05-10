# node-path pilot — coverage audit

**Eighth pilot. Largest single-module pilot to date.** First Node-compat (Tier 2 ecosystem) surface. Per the keeper's "something a bit bigger in bun" directive, this scales the apparatus to a 3,656-LOC reference target with 375 cross-corroborated clauses.

## Constraint inputs

From `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/path.constraints.md`:

- 21 candidate properties
- 375 cross-corroborated witnessing clauses
- 6 construction-style + 15 behavioral

Top-cardinality properties:
```
PATH1   path.win32.isAbsolute        22 cs   construction-style
PATH2   path.toNamespacedPath        12 cs   construction-style
PATH3-21  various behavioral path.* methods (basename, dirname, parse,
          format, join, resolve, normalize, isAbsolute, relative, ...)
```

Antichain reps drawn from real Bun tests:
- `assert.strictEqual(path.win32.isAbsolute(""), false)` — empty-input edge case
- `assert.strictEqual(path.posix.basename("/foo/bar/baz.html", ".html"), "baz")` — extension stripping
- `assert.strictEqual(path.posix.normalize("/foo/bar//baz/asdf/quux/.."), "/foo/bar/baz/asdf")`
- `assert.strictEqual(path.posix.join("/foo", "bar", "baz/asdf"), "/foo/bar/baz/asdf")`
- `assert.strictEqual(path.posix.resolve("/foo/bar", "./baz"), "/foo/bar/baz")`
- `assert.deepStrictEqual(path.posix.parse("/home/user/dir/file.txt"), {root: "/", dir: "/home/user/dir", base: "file.txt", ext: ".txt", name: "file"})`
- `assert.strictEqual(path.posix.relative("/foo/bar", "/foo/bar/baz"), "baz")`

## Pilot scope

POSIX `path.*` utilities. Win32 namespace optional within scope.

In scope:
- `basename(path, ext?)` — last segment, optionally without extension
- `dirname(path)` — all but last segment
- `extname(path)` — file extension including dot
- `parse(path)` → `{root, dir, base, name, ext}`
- `format(parsed)` → path string
- `isAbsolute(path)` — leading-`/` test for POSIX
- `join(...paths)` — concat with `/` then normalize
- `normalize(path)` — collapse `..`, `.`, redundant `/`
- `relative(from, to)` — path from absolute `from` to absolute `to`
- `resolve(...paths)` — resolve to absolute, prepending CWD if needed
- `sep` constant `"/"`
- `delimiter` constant `":"`
- `posix.*` namespace — pilot's primary path semantics
- `win32.*` namespace — secondary; backslash separator + drive-letter handling

Out of pilot scope:
- `toNamespacedPath` (Win32-specific UNC/NT-namespace prefixes; deferred)
- Volume-root case nuances on Win32
- CWD resolution beyond a fixed pilot CWD (real apparatus would query std::env)

## LOC budget

Bun reference: `runtime/node/path.rs` is **3,656 LOC**, `runtime/node/path.zig` is **2,986 LOC**. By far the largest reference target the apparatus has compared against in a single pilot.

Pilot target: ~250-350 LOC for POSIX + minimal Win32. Adjusted ratio: <10% expected because path is *all* algorithm; no I/O / FFI / runtime integration in either reference.

## Ahead-of-time hypotheses

1. **Path is the apparatus' easiest pilot per LOC.** Pure functions, no shared state, no async, no I/O, no allocator concerns. The derivation should be smaller than fetch-api despite higher clause count.

2. **The verifier will catch at least one derivation bug** because path semantics have subtle edge cases (empty inputs, trailing slashes, dot-segments, drive letters). AOT prediction: at least one bug surfaces. Most likely candidates: `normalize` with multi-`..` patterns, or `relative` when paths share no common prefix.

3. **Win32 mode will be skipped or partial.** The constraint corpus has Win32 reps but pilot scope keeps POSIX primary. AOT: documented skips for Win32-specific tests.

4. **The aggregate pilot LOC ratio will improve** with this pilot's addition. Path is small relative to its reference; ratio against 3,656 LOC of Bun should be <10%.

## Verifier strategy

Tests transcribed from Bun's actual antichain representatives + spec-derived edge cases. Target ~30-40 tests across the path surface. Pilot succeeds if:
- Verifier closes with 0 fail across POSIX
- Win32 tests either pass or are documented skips
- LOC ratio against Bun's path.rs is in the apparatus' claimed range

Consumer regression: ~10 tests citing real Node-ecosystem dependencies on path semantics.
