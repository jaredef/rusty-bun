# buffer pilot — coverage audit

**Tenth pilot. Tier-A #2 from the trajectory queue.** Buffer is Node's binary-data type, used by 70%+ of npm packages. After the Streams pilot anchored streams, this pilot anchors Node-compat Buffer — the second major substrate Tier-A queued item.

## Constraint inputs

From `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/buffer.constraints.md`:

- 5 candidate properties / 26 cross-corroborated clauses
- 3 construction-style + 2 behavioral

Bun's reference targets:

```
runtime/node/buffer.rs                184 LOC
runtime/node/buffer.zig                90
js/internal/buffer.ts                  50
node-fallbacks/buffer.js            2,035   ← JS-side Buffer polyfill
                                    ─────
Total Bun-side Buffer source        2,359
```

The JS-side polyfill (`node-fallbacks/buffer.js`) is the load-bearing reference because it implements the Node Buffer API surface in JS (which the runtime wires up to ArrayBuffer-backed storage).

Pilot reference comparison: 2,035 LOC of polyfill vs derivation.

## Pilot scope

In scope (Buffer's data-layer surface):

**Static factories:**
- `Buffer::alloc(size, fill?)` — zeroed by default; optional fill byte/string
- `Buffer::alloc_unsafe(size)` — uninitialized in Node; pilot zeroes (Rust-safe)
- `Buffer::from_string(s, encoding?)` — utf-8 default
- `Buffer::from_bytes(bytes)` — copy from byte slice
- `Buffer::from_array(values)` — accept iterable of u8
- `Buffer::byte_length(s, encoding?)` — encoded byte length without allocation
- `Buffer::compare(a, b)` — lexicographic over bytes
- `Buffer::concat(list, total_length?)` — concatenate buffers
- `Buffer::is_buffer(obj)` — pure type-check
- `Buffer::is_encoding(name)` — known-encoding check

**Instance methods:**
- `len()` (Node's `.length`)
- `to_string(encoding?, start?, end?)` — to-string with range + encoding
- `write(string, offset?, length?, encoding?)` — write into existing buffer
- `fill(value, start?, end?, encoding?)` — fill range
- `equals(other)` — byte equality
- `compare(other, target_start?, target_end?, source_start?, source_end?)` — three-way comparison
- `slice(start?, end?)` / `subarray(start?, end?)` — view (subarray) or copy (Node's slice was deprecated to subarray)
- `index_of(value, byte_offset?, encoding?)` — find first
- `last_index_of(value, byte_offset?, encoding?)` — find last
- `includes(value, byte_offset?, encoding?)` — contains
- `copy(target, target_start?, source_start?, source_end?)` — bytes from self into target

**Encodings:**
- `utf-8` (default)
- `utf-16le`
- `latin1` / `binary`
- `ascii`
- `base64` (RFC 4648, with padding)
- `hex` (lowercase output, case-insensitive input)

Out of pilot scope:
- `base64url`, `ucs-2`, `utf16` aliases beyond the canonical names
- `readUInt8/readUInt16BE/...` numeric readers (large surface; defer)
- `writeUInt8/...` numeric writers
- TypedArray inheritance plumbing
- ArrayBuffer-backed shared-memory semantics
- INSPECT_MAX_BYTES integration with `util.inspect`

## LOC budget

Pilot target: ~300-450 code-only LOC. Adjusted ratio against equivalent-scope Bun polyfill subset (~1,500 LOC excluding numeric readers/writers): 20-25%.

## Ahead-of-time hypotheses

1. **Encoding handling is the largest LOC contributor.** Six encodings × encode + decode = 12 codecs. Base64 is the most expensive (~50 LOC); hex and utf-16le are smaller; utf-8 is mostly delegate-to-std.

2. **At least one verifier-caught derivation bug expected.** Buffer's slice-vs-subarray distinction is subtle: Node's `slice` was historically a copy (pre-v8) then changed to a view (v8+). Pilot will follow current Node spec (subarray-equivalent view). AOT prediction: a Bun-test antichain rep that depends on the older copy-semantics surfaces this divergence.

3. **`Buffer.compare` returning -1/0/1 vs the Rust `Ordering` analog** is a translation quirk. Node specifies -1/0/1 integers exactly; pilot will return i32 to match.

4. **The cite-source consumer regression suite will be the densest yet** — Buffer is used by virtually every Node package that touches binary data: HTTP servers, file readers, crypto, image/audio processing, database drivers, network protocols.

## Verifier strategy

~40-50 verifier tests across the buffer surface + ~10-15 consumer regression tests. Pilot succeeds if:
- Verifier closes with documented skips at most for numeric-reader/writer surface
- Consumer regression cites real Node-ecosystem dependencies
- LOC ratio is in apparatus' claimed range
