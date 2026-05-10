# TextEncoder + TextDecoder — constraint-doc coverage audit

**Pilot purpose:** the [Doc 705](https://jaredfoy.com/resolve/doc/705-pin-art-operationalized-for-intra-architectural-seam-detection) apparatus emits constraint documents intended as *input to derivation*. This pilot tests whether the auto-emitted constraint doc — by itself — is sufficient input for an LLM-simulated derivation engine to produce a working Rust implementation. The audit phase characterizes what the doc says, what the doc doesn't say, and where the gap is.

The audit is the *rung-1 substrate* per the [substrate-plus-injection memory](../../runs/2026-05-10-deno-v0.11/RUN-NOTES.md): the discipline (`derive-constraints invert`) produces what it produces, and the gap-vs-spec is what the speech act (rung-2) has to fill. Naming the gap is the first deliverable.

## What the auto-emitted constraint docs say

### TextEncoder (from `runs/2026-05-10-bun-derive-constraints/constraints/textencoder.constraints.md`)

```
@provides: textencoder-surface-property
  threshold: TEXT1
  interface: [TextEncoder]

Surface drawn from 1 candidate property. Construction-style: 1; behavioral: 0.
Total witnessing constraint clauses: 6.

TEXT1: TextEncoder produces values matching the documented patterns
       under the documented inputs. (construction-style)

Antichain representatives:
  - `expect(typeof TextEncoder !== "undefined").toBe(true)`           (existence)
  - `expect(encoder.encode(undefined).length).toBe(0)`                (.encode argument-coercion)
  - `assertEquals(encoder.toString(), "[object TextEncoder]")`        (.toString tag)
```

**6 constraint clauses, 1 property.** The auto-emitted constraint document for TextEncoder is **radically insufficient** as a derivation input. It tells a derivation engine that:

1. `TextEncoder` exists as a global identifier.
2. `encoder.encode(undefined)` returns a value of `.length === 0`.
3. `encoder.toString() === "[object TextEncoder]"`.

It does not tell the derivation engine what `.encode(string)` does, what its return type is (Uint8Array? Buffer? something else?), what encoding is used (UTF-8? UTF-16?), what `.encoding` reads, what `.encodeInto` does, that it has any constructor parameters, or anything about its UTF-8 byte-emission semantics. The test corpus contains exactly the *negative* boundary case (`undefined → length 0`) and the *typeof* tag — no positive operational test of what the encoder produces.

### TextDecoder (from `runs/2026-05-10-deno-v0.11/constraints/textdecoder.constraints.md`)

```
TEXT1 (cardinality 69): TextDecoder produces values matching the documented patterns.
  Antichain representatives all of the form `new TextDecoder().decode(stdout)`
  asserting the decoded result equals an expected string literal.

TEXT2 (cardinality 13): TextDecoder satisfies the documented invariant.
  Antichain representatives are unrelated `assert(...)` calls in test files
  that happen to mention TextDecoder elsewhere — false-attribution noise.
```

**82 constraint clauses, 2 properties.** Slightly better than TextEncoder, but still insufficient. The 69-cardinality property witnesses that `decode(bytes) → string` round-trips correctly under specific known inputs (mostly `\"connected\\n\"`-style network-protocol decoding). It does not say anything about:
- what encodings other than UTF-8 are supported
- BOM handling (`ignoreBOM` constructor option)
- fatal mode (`fatal: true` raises `TypeError` on invalid sequences)
- streaming decode (`{stream: true}` option)
- the `encoding` getter
- `decode(undefined)` behavior
- behavior on partial UTF-8 sequences

The 13-cardinality "TEXT2" property is **classifier noise**: the antichain representatives are `assert(headerEnd > 0)`, `assert(response.startsWith(...))`, etc. — these clauses are in test files that happen to also use TextDecoder elsewhere, but they don't witness any TextDecoder property. They surfaced because subject canonicalization fell back to `TextDecoder` for unrelated clauses in those test files. **This is a separate apparatus finding** — the cluster phase's subject-attribution is leaking across clause boundaries within the same test, producing spurious property attributions. Worth a separate fix outside this pilot's scope.

## What the WHATWG Encoding spec says (rung-2 input)

The [WHATWG Encoding Standard](https://encoding.spec.whatwg.org/) specifies these constructs concretely:

### TextEncoder (§9 https://encoding.spec.whatwg.org/#textencoder)

```
[Exposed=*] interface TextEncoder {
  constructor();
  [[unrestricted]] readonly attribute DOMString encoding;        // always "utf-8"
  [NewObject] Uint8Array encode(optional USVString input = "");
  TextEncoderEncodeIntoResult encodeInto(USVString source, Uint8Array destination);
};
```

Operationally:
- `encoding` getter always returns the literal string `"utf-8"`.
- `encode(input)`: convert `input` (a USVString — invalid UTF-16 surrogates already replaced with U+FFFD) to a UTF-8 byte sequence; return as a new `Uint8Array`.
- `encode()` with no argument or `encode(undefined)`: input coerces to `"undefined"` (the string), which encodes to 9 UTF-8 bytes — **but the auto-emitted constraint asserts length 0**. Reading the spec precisely, `optional USVString input = ""` means absent argument defaults to `""` (empty string), encoding to 0 bytes. `undefined` passed explicitly coerces via JavaScript's USVString conversion: `undefined` → string `"undefined"` → 9 bytes. **The Bun test asserts encode(undefined).length === 0 — this is non-spec-compliant behavior; Bun is matching V8/web-platform-tests where `undefined` short-circuits to empty.** Cross-reference required; this is a real platform-implementation detail not derivable from spec alone.
- `encodeInto(source, destination)`: write UTF-8 bytes of `source` into `destination` (a `Uint8Array`); never write past the destination's length; return `{read: number_of_USV_chars_consumed, written: number_of_bytes_written}`.

### TextDecoder (§10 https://encoding.spec.whatwg.org/#textdecoder)

```
dictionary TextDecoderOptions { boolean fatal = false; boolean ignoreBOM = false; };
dictionary TextDecodeOptions    { boolean stream = false; };

[Exposed=*] interface TextDecoder {
  constructor(optional DOMString label = "utf-8", optional TextDecoderOptions options = {});
  readonly attribute DOMString encoding;
  readonly attribute boolean fatal;
  readonly attribute boolean ignoreBOM;
  USVString decode(optional BufferSource input, optional TextDecodeOptions options = {});
};
```

Operationally:
- Constructor's `label` is normalized via the [encoding label table](https://encoding.spec.whatwg.org/#concept-encoding-get) (e.g., `"UTF-8"`, `"utf8"`, `"unicode-1-1-utf-8"` all → `"utf-8"`); unknown labels throw `RangeError`.
- `encoding` returns the canonical name of the resolved encoding.
- `fatal` and `ignoreBOM` reflect constructor options.
- `decode(buffer, options)`: decode the byte sequence in `buffer` per the resolved encoding's decoding algorithm. If `fatal === true`, raise `TypeError` on invalid sequences; otherwise emit U+FFFD replacement characters. If `ignoreBOM === false` AND the encoding is one of UTF-8/UTF-16BE/UTF-16LE AND a BOM is present at the start, consume it. If `options.stream === true`, retain partial-sequence state for the next `decode` call; otherwise flush state and end-of-input handling.

## Pilot scope

The full WHATWG Encoding decoder specification is large (50+ encodings, label-resolution table, complete decoder state machines for each). The pilot deliberately scopes to:

1. **TextEncoder: full spec.** UTF-8 encoder is small and well-determined. ~30 LOC achievable.
2. **TextDecoder: UTF-8 only, fatal/ignoreBOM/stream options supported.** Other encodings throw `RangeError` from the constructor's label-resolution. ~120 LOC achievable.

This scope keeps the pilot at the minimal-viable-existence-proof tier. A real Bun-port-target TextDecoder would need the full encoding registry; that's deferred.

## The coverage gap, named

| Surface element | Constraint doc says | WHATWG says | Gap source |
|---|---|---|---|
| `TextEncoder` exists | yes | yes | — |
| `.encode(string) → Uint8Array, UTF-8` | no | yes | A (test-corpus, no positive test) |
| `.encode(undefined).length === 0` | yes | "undefined" → 9 bytes (?) | B (impl-vs-spec divergence; needs cross-reference) |
| `.encode().length === 0` (no arg) | no | yes (default `""`) | A |
| `.encoding === "utf-8"` | no | yes | A |
| `.encodeInto(s, dest) → {read, written}` | no | yes | A |
| `.toString() === "[object TextEncoder]"` | yes | yes (Object.prototype.toString @@toStringTag) | — |
| `TextDecoder()` (label: "utf-8" default) | no | yes | A |
| `TextDecoder(label)` label-resolution | no | yes | A |
| `TextDecoder(label, {fatal, ignoreBOM})` | no | yes | A |
| `.decode(bytes) → string` | yes (UTF-8 only) | yes (full encoding registry) | A (partial — UTF-8 only) |
| `.decode(bytes, {stream: true})` | no | yes | A |
| `.encoding`, `.fatal`, `.ignoreBOM` getters | no | yes | A |

**A = test-corpus coverage gap.** Most surface elements are not exercised by any test in the Bun or Deno corpora at a level that lets `derive-constraints invert` extract them as properties. They exist as tests *somewhere* (Web Platform Tests has thousands), but those tests aren't in the corpus we scanned.

**B = impl-vs-spec divergence.** The single non-A row is interesting on its own: the constraint says `encode(undefined).length === 0` but the spec says undefined-as-USVString coerces to `"undefined"` (9 bytes). This is a real platform implementation detail (V8's TextEncoder short-circuits undefined; the WHATWG IDL would say 9 bytes; web-platform-tests assert 0 bytes; Bun matches V8/WPT). The auto-emitted constraint captured a real implementation invariant the spec does not. **This is the first apparatus win:** the auto-emitted constraint surfaces an implementation invariant that an LLM derivation from spec alone would get wrong.

## Implication for the simulation pipeline

The simulated derivation must consume **constraint doc + WHATWG spec + Web Platform Tests sample** as input. The constraint doc alone is insufficient. The spec alone misses the `encode(undefined) → length 0` invariant.

Both inputs are required. The synthesis is exactly the dyad's substrate-plus-injection: the corpus discipline produces constraint doc; the speech act injects spec semantics; the result is a derivable specification.

## What the pilot proves or disproves

- **Proves**: the apparatus's constraint output is *necessary but not sufficient* for derivation. (Already proved by the audit above.)
- **Tests**: whether constraint-doc + spec + WPT-sample is *sufficient* — that's the simulated derivation step.
- **Proves or disproves**: whether the verifier (constraint antichain → cargo test) catches the divergence the audit just named (the `encode(undefined) → 0 bytes` invariant). This is the apparatus's first end-to-end loop closure: the discipline emits a constraint, the simulated derivation reads spec instead and gets it wrong, the verifier catches the gap, the iteration writes spec back into the inputs. That's the cybernetic closure.
