# buffer pilot — 2026-05-10

**Tenth pilot. Tier-A #2 from the trajectory queue.** Buffer is Node's binary-data type, used by 70%+ of npm packages. After the Streams pilot anchored streams (Tier-A #1), this anchors Buffer — the second major Tier-A substrate.

## Pipeline

```
v0.13b enriched constraint corpus
  buffer: 5 properties / 26 cross-corroborated clauses
       │
       ▼
AUDIT.md
       │
       ▼
simulated derivation v0   (CD + Node.js docs §Buffer)
       │
       ▼
derived/src/lib.rs   (261 code-only LOC)
       │
       ▼
cargo test
   verifier:            44 tests (2 corrected from author-error after first run)
   consumer regression: 11 tests
       │
       ▼
55 pass / 0 fail / 0 skip   ← derivation clean on first run; 2 verifier
                              tests had author-side bugs (own typos), fixed
                              independently
```

## LOC measurement

```
Bun reference (Node-compat Buffer):
  node-fallbacks/buffer.js                    2,035 LOC   (JS-side polyfill)
  runtime/node/buffer.rs                        184
  runtime/node/buffer.zig                        90
  js/internal/buffer.ts                          50
  Total Bun-side                              2,359 LOC

Pilot derivation (code-only):                   261 LOC
Naive ratio vs full Bun-side:                  11.1%
Naive ratio vs polyfill subset only:           12.8%
Adjusted (excluding numeric readers/writers
  which are out of pilot scope; equivalent-
  scope subset of polyfill is ~1,500 LOC):    ~17.4%
```

The pilot omits numeric-reader/writer methods (`readUInt8`, `writeUInt32BE`, etc.), which constitute ~500 LOC of the polyfill. Equivalent-scope adjusted ratio: ~17%.

## Verifier results: 44/44

```
Static factories (10 tests)
  alloc zeros, alloc_filled pattern repeat, from_string utf-8 roundtrip,
  to_string default utf-8, concat with alloc (CD), concat total_length
  truncates / pads with zeros, byte_length utf-8 / utf-16le, is_encoding
  known / unknown names

Encodings (8 tests)
  utf-8 unicode roundtrip, utf-16le roundtrip, latin1 one-byte-per-char,
  ascii high-bit strip, base64 encode / decode / arbitrary-bytes
  roundtrip, hex lowercase encode / case-insensitive decode

Compare / equals (4 tests)
  equals byte-match, unequal, compare static -1/0/1, compare with ranges

Slice / subarray (3 tests)
  subarray range extraction, clamps out-of-range, slice == subarray

Index of / includes (3 tests)
  index_of finds substring + occurrences, last_index_of, includes

Fill / write / copy (10 tests)
  fill_byte full / range, fill_bytes pattern, write into existing buffer,
  write with offset, write truncates at buffer end, copy basic / target
  offset / source range

To-string ranges + edge cases (6 tests)
  partial range, end-clamps-to-length, alloc(0), concat([]), index_of("")
  returns offset, more
```

## Consumer regression: 11/11

```
Node fs.readFile utf-8 roundtrip                                     1
Node http body assembly via Buffer.concat                            1
crypto compare deterministic ordering                                1
Express body-parser byteLength matches encoded size                  1
JOSE base64 arbitrary bytes roundtrip                                1
pino logger pre-allocated write stops at buffer end                  1
ws library mask fill + equals pattern                                1
busboy multipart boundary scanning via indexOf                       1
node-postgres bytea hex decode                                       1
multer filename utf-16le decode                                      1
Bun-specific alloc(16381) padding + concat                           1
```

## Findings

1. **AOT hypothesis #1 confirmed.** Pilot at 261 LOC, on the smaller end despite 6 codecs. Encoding implementations are dense but each is small (base64 is largest at ~50 LOC).

2. **AOT hypothesis #2 NOT confirmed.** Predicted at least one verifier-caught derivation bug. The two verifier failures on first run were **author-side test-typo bugs**, not derivation bugs:
   - `cd_buffer_alloc_filled_repeats_pattern` — I wrote a redundant assertion that fired before the corrected assertion. The derivation produced `"abababa"` correctly; my test asserted it twice with one wrong expected value.
   - `spec_encoding_ascii_strips_high_bit` — I asserted on a character index (`b'l'`) that wasn't in the test string. The derivation correctly stripped the high bit; my test verified the wrong character.
   
   Distinguishing apparatus-bugs from author-bugs is itself an apparatus finding: **the verifier surfaces both, but the cite-source discipline + spec material in the comments lets us differentiate quickly**. Both author-side bugs were spotted by reading the assertion against the spec text; both were one-line fixes in the test, not the derivation.

3. **AOT hypothesis #3 confirmed (compare returns i32).** Node specifies -1/0/1 integers; pilot returns `i32` to match exactly. Three tests verify the precise return values (`-1`, `0`, `1`) rather than any negative/zero/positive.

4. **AOT hypothesis #4 confirmed strongly.** Consumer regression cites 11 distinct production codepaths across the Node ecosystem: fs, http, crypto, express, JOSE, pino, ws, busboy, node-postgres, multer, plus a Bun-specific. Buffer's dependency-surface is among the densest the apparatus has measured.

## Updated 10-pilot table

| Pilot | Class | LOC | Adj. ratio |
|---|---|---:|---:|
| TextEncoder + TextDecoder | data structure | 147 | 17–25% |
| URLSearchParams | delegation target | 186 | 62% |
| structuredClone | algorithm | 297 | ~8.5% |
| Blob | composition substrate | 103 | 20–35% |
| File | inheritance/extension | 43 | 20–30% |
| AbortController + AbortSignal | event/observable | 126 | 25–35% |
| fetch-api (Headers + Request + Response) | system / multi-surface | 405 | 6.5% naive / ~20% adj |
| node-path | Tier-2 Node-compat pure-function | 303 | 8.3% naive / ~12–15% adj |
| streams (Readable + Writable + Transform) | substrate / async-state-machine | 453 | 11.2% naive / ~12–15% adj |
| **buffer** | **Tier-2 Node-compat binary type** | **261** | **11.1% naive / ~17% adj** |

Ten-pilot aggregate: **2,324 LOC** of derived Rust against ~39,000+ LOC of upstream reference targets. **Aggregate ratio: ~6.0%.**

## Trajectory advance

This completes Tier-A item #2 (Buffer pilot). Both Tier-A substrate items are now anchored. The trajectory's next priority is **Tier-B #3: Bun.file pilot** — the first Tier-2 Bun-namespace anchor.

Per the trajectory's resume protocol, four pilots remaining in Tier-B (Bun.file, Bun.serve, Bun.spawn) and Tier-C (Node fs, http/https, crypto.subtle).

## Files

```
pilots/buffer/
├── AUDIT.md
├── RUN-NOTES.md              ← this file
└── derived/
    ├── Cargo.toml
    ├── src/
    │   └── lib.rs            (345 LOC, 261 code-only)
    └── tests/
        ├── verifier.rs            44 tests, all pass
        └── consumer_regression.rs 11 tests, all pass
```

## Provenance

- Tool: `derive-constraints` v0.13b.
- Constraint inputs: `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/buffer.constraints.md` (5 properties / 26 clauses).
- Spec input: Node.js docs §Buffer (no formal IDL; documentation serves as authoritative reference).
- Reference target: Bun's `node-fallbacks/buffer.js` (2,035 LOC) + `runtime/node/buffer.{rs,zig}` (274 LOC) + `js/internal/buffer.ts` (50 LOC).
- Result: 55/55 across both verifier (44) and consumer regression (11). Zero derivation regressions. Two author-side test typos surfaced + fixed before final.
