# rusty-bun → Bun: Bug Catcher

A living record of bugs, divergences, suspect behaviors, and behavioral invariants surfaced during the rusty-bun engagement against Bun. Formatted for the Bun team to act on, ignore, or investigate as fits their priorities.

Per [Doc 707 (Pin-Art at the Behavioral Surface)](https://jaredfoy.com/resolve/doc/707-pin-art-at-the-behavioral-surface-bidirectional-probes), the apparatus produces two outputs: a derivation, AND a dependency-surface map of the original. This document is the **dependency-surface map plus apparatus-found edge cases**, structured for Bun maintainers' use.

---

## Conventions

Each entry has:
- **Surface** — the Bun API or runtime area
- **Observation** — what the apparatus saw
- **Source** — citation (consumer file:fn / Bun source / spec section / pilot RUN-NOTES)
- **Severity** — low / medium / high
- **Suggested action** — file issue / contribute test / no action / investigate / FYI

The categories below sort entries by what the Bun team can do with them, not by which surface they touch. Severity is the apparatus' best-effort classification, not Bun's. Suggested actions are non-binding.

---

## A — Behavioral invariants Bun is implicitly committed to

These are behaviors of Bun that real downstream consumers depend on. The consumer cite is the *evidence* that the behavior is load-bearing. If Bun changes any of these, the cited consumer breaks. None of these are bugs in Bun; they are constraints on future change.

### A1. URLSearchParams.sort is UTF-16 code-unit ordered, case-sensitive
- **Source.** AWS SDK v3 — `signature-v4/src/SignatureV4.ts`, function `getCanonicalQuery`. AWS SigV4 canonical-query-string protocol depends on this exact ordering. Cited at `pilots/urlsearchparams/derived/tests/consumer_regression.rs:consumer_aws_sdk_sort_is_stable_for_canonical_request`.
- **Severity.** High. Changing this breaks all S3/CloudFront/etc. signing for AWS SDK v3 users on Bun.
- **Suggested action.** No action; document as a public stability commitment. Add a Bun test that asserts case-sensitive uppercase-before-lowercase sort on a multi-key URLSearchParams.

### A2. URLSearchParams percent-encodes `[` and `]`, not literal-passthrough
- **Source.** Stripe SDK Node — `src/utils.ts`, function `queryStringifyRequestData`. Stripe expects metadata keys like `metadata[name]` to encode as `metadata%5Bname%5D`. Cited at `pilots/urlsearchparams/derived/tests/consumer_regression.rs:consumer_stripe_brackets_are_percent_encoded_per_spec`.
- **Severity.** Medium. Changing this would break the wire format for Stripe API calls.
- **Suggested action.** No action; this is per WHATWG URL §5.2.5 form-urlencoded character set already.

### A3. URLSearchParams parses `?debug` (no `=`) as `debug=""`, not absent
- **Source.** Express's URLSearchParams fallback path — `lib/utils.js compileQueryParser`. Many Express-using apps treat `?debug` as truthy and rely on `searchParams.has("debug")` being `true` after this parse. Cited at `pilots/urlsearchparams/derived/tests/consumer_regression.rs:consumer_express_query_parse_empty_value`.
- **Severity.** Medium. Per WHATWG URL §5.2.5 the empty-value case is correct, but the apparatus surfaces it explicitly because some HTTP libraries assume otherwise.
- **Suggested action.** FYI; documented at WHATWG URL §5.2.5.

### A4. structuredClone preserves shared-reference identity within a single call
- **Source.** immer 10+ — `src/utils/plugins.ts`, the `current()` implementation depends on this for draft-finalize. If two draft properties point to the same source object, the cloned drafts must also point to a single (cloned) target object.
- **Severity.** High. immer is the most-depended-on state-management library in the React/Redux ecosystem (~10M weekly downloads). Breaking shared-ref identity breaks immer entirely.
- **Suggested action.** No action; this is per HTML §2.10. Worth a Bun test that asserts the shared-ref invariant on a small object graph.

### A5. structuredClone preserves circular references without infinite recursion
- **Source.** Worker postMessage / comlink — `comlink/src/comlink.ts`, the proxy machinery may build cycles. HTML §10.5 mandates structuredClone for cross-realm message passing.
- **Severity.** High.
- **Suggested action.** No action; per HTML §2.10.

### A6. structuredClone preserves TypedArray-view-shares-buffer semantics
- **Source.** Many ML/numerics libraries (numjs, ml-matrix, gl-matrix). Two views over the same ArrayBuffer must, after clone, point to the same (cloned) ArrayBuffer.
- **Severity.** Medium-high.
- **Suggested action.** Add a Bun test that constructs two views over a single ArrayBuffer, structured-clones the parent, and asserts the cloned views share the cloned buffer. (Reference test: `pilots/structured-clone/derived/tests/consumer_regression.rs:consumer_typed_array_views_share_cloned_buffer`.)

### A7. AbortController.abort is idempotent and listeners fire at most once
- **Source.** node-fetch — `src/index.js` request cancellation path registers a one-shot abort listener. If the listener fires twice (because abort is called twice), node-fetch double-cleans and crashes.
- **Severity.** High. Confirmed real-world bug class for any consumer that does cleanup-once-on-abort.
- **Suggested action.** Add a Bun test that calls `controller.abort()` repeatedly and asserts the listener fires exactly once.

### A8. AbortSignal listener registered AFTER abort fires immediately
- **Source.** p-event (sindresorhus/p-event) and similar Promise-cancellation idioms. Code may attach an abort listener to a signal that's already aborted, expecting synchronous (microtask) firing.
- **Severity.** Medium. Per DOM §3.3 spec; but easy to get wrong in any implementation.
- **Suggested action.** FYI; spec-mandated. Add a Bun test if not already present.

### A9. AbortSignal default reason is DOMException with name `AbortError` and code 20
- **Source.** Many libraries branch on `err.name === "AbortError"` or `err.code === 20` to distinguish abort from other errors. Per DOM §3.3 spec.
- **Severity.** High. Changing the default reason class breaks every consumer that error-discriminates on it.
- **Suggested action.** No action; spec-mandated.

### A10. Blob slice() returns a new Blob; slicing a File does NOT preserve File identity
- **Source.** uppy chunked upload — `@uppy/utils/src/getFileType.ts`. Uppy slices a File for chunked uploads and **explicitly re-wraps each chunk in a fresh File** because it knows the slice returns a Blob. Per W3C File API §4 spec.
- **Severity.** Medium.
- **Suggested action.** FYI. Bun likely has this right.

### A11. Blob.text() does NOT normalize line endings
- **Source.** Azure Storage SDK — `@azure/storage-blob`. Azure uploads sometimes use Windows CRLF; the SDK calls `.text()` and expects raw bytes back, NOT normalized to LF.
- **Severity.** Medium.
- **Suggested action.** Add a Bun test that creates a Blob from `b"line1\r\nline2"`, calls `.text()`, asserts the result still contains `\r\n`.

### A12. Blob constructor preserves byte-equality across part-list construction styles
- **Source.** formdata-polyfill — `lib/formdata.mjs`. The polyfill builds multipart bodies by concatenating Blobs of header bytes, separators, and entry bodies. `new Blob([blob1, blob2, ...])` and `new Blob([combinedBytes])` must produce byte-equal results.
- **Severity.** Medium-high.
- **Suggested action.** No action; per W3C File API §3.

### A13. TextDecoder fatal-mode error propagation distinguishes protocol corruption
- **Source.** node-mysql2 — `lib/parsers/`. MySQL2 uses TextDecoder with `fatal:true` to detect protocol corruption immediately rather than emit garbled strings.
- **Severity.** Medium.
- **Suggested action.** Add a Bun test that decodes invalid UTF-8 with `fatal:true` and asserts a TypeError fires.

### A14. TextDecoder default `ignoreBOM:false` consumes UTF-8 BOM
- **Source.** PapaParse and other CSV parsers — handle Windows-exported CSVs with leading BOM. Many such parsers strip BOM client-side BEFORE TextDecoder; relying on TextDecoder NOT double-stripping.
- **Severity.** Low.
- **Suggested action.** FYI; spec-mandated.

### A15. Headers iteration produces lowercased names regardless of insertion case
- **Source.** Stripe SDK reads multiple Set-Cookie headers via `headers.getSetCookie()`. Express `req.get(name)` is case-insensitive on the lookup; downstream code expects iteration to be deterministic-case (lowercased).
- **Severity.** Medium.
- **Suggested action.** No action; per WHATWG Fetch §5.2.

### A16. Response.redirect rejects non-redirect status codes
- **Source.** Cloudflare Workers documentation explicitly notes Response.redirect rejects `200`, `304`, `400`, etc. Production Workers code relies on this rejection for safety (preventing accidental 200-with-Location-header).
- **Severity.** High.
- **Suggested action.** No action; per WHATWG Fetch §6.4.

### A17. Buffer.compare returns -1/0/1 exactly (not arbitrary negative/positive)
- **Source.** Node's `crypto.timingSafeEqual` and consumer code that branches on `Buffer.compare === 0` for equality. The returned integer must be exactly `-1`, `0`, or `1`, not any negative/zero/positive value (some Rust ports return arbitrary `Ordering` values cast to int).
- **Severity.** Medium.
- **Suggested action.** Add a Bun test that asserts the precise return values.

### A18. Buffer.byteLength matches actual encoded byte count
- **Source.** Express body-parser — `lib/types/json.js`. Sets Content-Length from `Buffer.byteLength(body, "utf-8")`. If byteLength returns a different value than `from(s).length`, body-parser writes a wrong Content-Length and the HTTP server sees truncation.
- **Severity.** High.
- **Suggested action.** Add a Bun test asserting the equality on Unicode strings (where multi-byte chars exercise the codec).

### A19. ReadableStream cancel propagates to the underlying source
- **Source.** undici — `lib/web/fetch/body.js`. When the consumer cancels mid-stream, undici expects the underlying source to be notified (so HTTP request can be aborted upstream).
- **Severity.** High. Without propagation, the upstream request continues consuming bandwidth/CPU after cancellation.
- **Suggested action.** No action; per WHATWG Streams §4. Verify Bun has a test asserting source.cancel is called when reader.cancel fires.

### A20. ReadableStream.tee branches share cancellation propagation policy
- **Source.** WHATWG Streams §4.tee. Both branches must cancel for the source to be notified; cancelling one branch does NOT propagate. comlink and other Worker libraries rely on this for fan-out semantics.
- **Severity.** Medium-high.
- **Suggested action.** Add a Bun test that asserts cancelling branch A while branch B is still active leaves the source uncancelled.

---

## B — Spec-vs-implementation divergences worth documenting

Places where Bun's behavior diverges from the literal WHATWG/W3C/ECMA spec text, where Bun matches V8/WPT or other implementations rather than the spec wording.

### B1. TextEncoder.encode(undefined) returns Uint8Array of length 0, not 9
- **Source.** Bun follows V8/WPT — confirmed by the antichain rep `expect(encoder.encode(undefined).length).toBe(0)` from Bun's own test corpus. Spec wording (WHATWG Encoding §9.1.encode) says `optional USVString input = ""` with `undefined → coerce to "undefined"`, which would give 9 bytes.
- **Severity.** Low. WPT enforces the 0-bytes behavior; Bun is correct against the operational test suite, divergent against the literal IDL.
- **Suggested action.** Document this divergence somewhere (release notes, or a comment in `runtime/webcore/TextEncoder.zig`). It's the kind of behavioral invariant a future contributor might "fix" without realizing it would break existing consumers. (See pilots/textencoder/AUDIT.md "What the WHATWG Encoding spec says" §.)

### B2. Win32 path.isAbsolute("/foo") returns true (forward-slash root)
- **Source.** Bun test `is-absolute.test.js` — `path.win32.isAbsolute("/")` returns `true`. Node's docs are ambiguous on whether forward-slash counts as absolute on Win32; Bun follows Node's actual behavior, not the doc-strict reading.
- **Severity.** Low.
- **Suggested action.** FYI; aligned with Node.

---

## C — Subtle spec edge cases any implementer might get wrong

Test cases the apparatus's verifier surfaced where the LLM-derivation initially got wrong — useful as regression-test contributions to Bun's own test corpus, even if Bun has these right today, because they protect against future regressions.

### C1. Blob.slice with start > end yields an empty Blob, NOT a swapped-range Blob
- **Source.** W3C File API §3.slice: *"Let span be max(relativeEnd − relativeStart, 0)."* When `relativeEnd < relativeStart`, span is 0.
- **Pilot evidence.** The rusty-bun Blob pilot's first-run derivation got this wrong (wrote `(lo.min(hi), hi.max(lo))` which swapped). The verifier test `spec_slice_swapped_endpoints_yield_empty` caught it. See `pilots/blob/RUN-NOTES.md` §"The verifier-caught bug".
- **Severity.** Low. Bun probably has this right (Bun is mature) but the test-case contribution is cheap and protective.
- **Suggested action.** Contribute a Bun test asserting `blob.slice(4, 2).size === 0` for a blob of length 6.

### C2. URLSearchParams.sort uses UTF-16 code-unit order, NOT USV order
- **Source.** WHATWG URL §5.2.sort. Subtle: a supplementary-plane character (e.g., U+1F600 = surrogate pair 0xD83D 0xDE00) has UTF-16 code unit 0xD83D, which sorts BEFORE a BMP full-width character (e.g., U+FF21 = code unit 0xFF21). USV (codepoint) order would put U+FF21 first.
- **Pilot evidence.** The rusty-bun URLSearchParams pilot explicitly tested this. See `pilots/urlsearchparams/derived/tests/verifier.rs:cd_sort_uses_utf16_code_unit_order`.
- **Severity.** Low. Bun likely has this right.
- **Suggested action.** Contribute a Bun test for the surrogate-pair-vs-BMP case.

### C3. Form-urlencoded character set is narrower than RFC 3986 unreserved
- **Source.** WHATWG URL §5.2.5. The form-urlencoded set passes through only `[A-Za-z0-9*-._]`; characters like `!`, `@`, `#`, `$`, `^`, `(`, `)` percent-encode despite being unreserved in RFC 3986.
- **Pilot evidence.** Multiple consumer regression tests rely on this. See `pilots/urlsearchparams/derived/tests/consumer_regression.rs:consumer_undici_fetch_body_roundtrip`.
- **Severity.** Low.
- **Suggested action.** FYI.

### C4. Blob ASCII type is lowercased, non-ASCII is preserved
- **Source.** W3C File API §3 — only ASCII `A-Z` → `a-z` is required. `Image/SVG+XML` becomes `image/svg+xml`; `TEXT/Ω` becomes `text/Ω` (Ω passes through).
- **Pilot evidence.** `pilots/blob/derived/tests/verifier.rs:spec_type_lowercases_ascii` and `spec_type_preserves_non_ascii`.
- **Severity.** Low.

### C5. extname(".bashrc") returns "" (leading-dot semantics)
- **Source.** Node `path` module spec. Files starting with a dot have no extension per Node convention.
- **Pilot evidence.** `pilots/node-path/derived/tests/verifier.rs:cd_posix_extname_dotfile_no_extension`.
- **Severity.** Low.

### C6. Buffer.indexOf("") returns the byteOffset, NOT 0
- **Source.** Node `Buffer.indexOf` docs. `buf.indexOf("", 5)` returns `5`, not `0`.
- **Pilot evidence.** `pilots/buffer/derived/tests/verifier.rs:spec_buffer_index_of_empty_needle_returns_offset`.
- **Severity.** Low.

### C7. ReadableStream.tee snapshot semantics: branches start with already-enqueued chunks
- **Source.** WHATWG Streams §4.tee. Chunks already in the source's queue at tee-time appear in BOTH branches, in order. Pull-only chunks pulled after tee also distribute to both.
- **Pilot evidence.** `pilots/streams/derived/tests/verifier.rs:spec_tee_returns_two_streams`.
- **Severity.** Medium. Easy to get the snapshot timing wrong in any implementation.
- **Suggested action.** Contribute a Bun test that pre-enqueues chunks, calls tee, and asserts both branches see the pre-enqueued chunks.

---

## D — Open questions / suspect behaviors to verify

Items where the apparatus saw evidence but didn't dig further. These are FYI to Bun maintainers; the apparatus does not claim any of these are bugs, only that they're worth a quick check.

### D1. The `expect(blob.name).toBeUndefined()` regression test (#10178)
- **Source.** Bun test `test/js/web/fetch/blob.test.ts:203`. The test name says "blob: can set name property #10178" and asserts `blob.name === undefined`. The combination "can set name property" + "name is undefined" is grammatically odd; suggests a regression-test for an issue that was fixed by REMOVING a previously-existing `.name` field.
- **Severity.** Unknown.
- **Suggested action.** Verify the test name still describes the intended behavior. Possibly rename to "blob does not expose a name property (#10178)".

### D2. `path.toNamespacedPath` accepts non-string inputs without coercion
- **Source.** Bun test `to-namespaced-path.test.js:11-13` asserts `path.toNamespacedPath(null) === null` and `path.toNamespacedPath(100) === 100` — pass-through for non-string inputs. Per Node docs, this is a Win32-specific function whose behavior on non-Win32 should be identity. The Bun tests verify that.
- **Severity.** Low.
- **Suggested action.** FYI; aligned with Node.

### D3. Bun.serve constraint corpus is sparse on the surface itself
- **Source.** `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/server.constraints.md` has only 14 cross-corroborated clauses despite Bun.serve being the flagship Bun API. Indicates Bun's tests construct via `const server = Bun.serve(...)` and then operate on `server.fetch(...)`, which the cluster phase correctly attributes to `server` (a local binding).
- **Severity.** Low (apparatus-side observation, not a Bun bug).
- **Suggested action.** Bun could add a few direct-attribution tests like `expect(typeof Bun.serve).toBe("function")` to make the surface more discoverable to apparatuses that scan tests.

---

## E — Apparatus-found derivation gotchas (also useful to Bun)

Cases where the LLM-simulated derivation initially failed. The same failure pattern may apply to any implementer reading the spec; a Bun-test capturing each is a contribution.

### E1. URLSearchParams.set must preserve the position of the first existing entry
- **Spec.** WHATWG URL §5.2.set: replace all existing values for the name with one, AT THE FIRST EXISTING POSITION.
- **Failure pattern.** A naive `delete(name); append(name, value)` implementation appends at the end, breaking position-preservation.
- **Suggested action.** Contribute a Bun test like:
  ```js
  const p = new URLSearchParams("a=1&k=old&b=2&k=old2");
  p.set("k", "new");
  expect([...p.entries()]).toEqual([["a","1"], ["k","new"], ["b","2"]]);
  ```

### E2. WritableStream.close with sink-error transitions to errored, not closed
- **Spec.** WHATWG Streams §5. If `sink.close()` returns/rejects with an error, the stream's state must become Errored (with the error stored), not Closed.
- **Suggested action.** Contribute a Bun test for this transition.

### E3. ReadableStream.cancel propagates only the FIRST cancel in tee branches
- **Spec.** WHATWG Streams §4.tee.cancel. The source's cancel callback fires only once, and only when both tee branches have cancelled.
- **Suggested action.** Contribute a Bun test that cancels branch A, then branch B, and asserts the source's cancel callback fired exactly once.

### E4. JS-host stateful types: Rust closures capturing Rc<RefCell> stored on JS objects break QuickJS' GC
- **Source.** `host/RUN-NOTES.md` § "Findings" #1 — surfaced during the rusty-bun-host JS integration iteration on 2026-05-10. QuickJS asserts `list_empty(&rt->gc_obj_list)` at runtime drop because its GC does not track Rust-side references that JS objects hold transitively.
- **Triggering pattern.** `Function::new(ctx.clone(), move |...| { state.borrow_mut().method() })` where `state` is `Rc<RefCell<...>>`, the Function is stored on a JS Object, and Rust's drop happens after JS' drop.
- **Severity.** Apparatus-side; relevant to any project binding Rust state to a JS host (Bun's own JSC bindings; Deno's V8 bindings; future runtime-derivation projects). Not a Bun bug; an integration discipline.
- **Suggested action.** Document the alternative pattern (stateless Rust helpers + JS-side class holding pure-JS state) anywhere a team is binding a Rust API into a JS host. The rusty-bun apparatus' formalization is at `host/HOST-INTEGRATION-PATTERN.md`. Bun's own JSC bindings already use a similar pattern (state on the JS-side via JSC class infrastructure); the finding is more relevant to greenfield Rust-to-JS integrations.

### E5. rquickjs Opt<T> requires JS-side arity omission, not undefined-as-value
- **Source.** `host/RUN-NOTES.md` § "Findings" #2.
- **Triggering pattern.** A JS class wrapper passes `this._optionalField` to a Rust function expecting `Opt<T>`. When the field is undefined, rquickjs converts undefined → T directly and errors with "Error converting from js 'undefined' into type 'X'".
- **Severity.** Apparatus-side; rquickjs-specific.
- **Suggested action.** Apply the JS-side branching pattern documented at `host/HOST-INTEGRATION-PATTERN.md` whenever a JS class delegates to a Rust function with optional arguments. Not directly relevant to Bun's JSC bindings (different binding crate with different optional-arg semantics) but useful to other projects evaluating rquickjs for Rust-to-JS bindings.

### E6. Wrapping sync user callbacks in `async () => await fn()` breaks microtask resumption under rquickjs
- **Source.** Streams pilot wiring on 2026-05-10. ReadableStream's pull-driven test deadlocked on the second `await reader.read()` when the host scheduled `Promise.resolve().then(async () => { await source.pull(controller); ... })`. Observed: 7 microtasks ran, then the pump ran dry with the result-promise unresolved. Switching to a sync invocation with explicit thenable detection (`const r = source.pull(c); if (r && typeof r.then === "function") r.then(...) else _pulling = false`) fixed it.
- **Triggering pattern.** A user-supplied callback that is synchronous gets wrapped in an async IIFE that awaits its return. The outer `await` introduces a microtask boundary that, in combination with QuickJS' microtask scheduling, drops the resumption of an awaiter that was resolved synchronously inside the user callback (here: pendingRead resolved by enqueue, called inside pull).
- **Severity.** Apparatus-side; rquickjs/QuickJS-specific.
- **Suggested action.** When invoking user-supplied callbacks that may be sync OR async, do not blanket-wrap with `await`. Detect thenable explicitly: invoke synchronously, branch on `result && typeof result.then === "function"` to handle the async path, otherwise treat as resolved. Pattern documented in `host/HOST-INTEGRATION-PATTERN.md` § "Sync-or-async user callbacks".

---

### E7. WeakRef and FinalizationRegistry are absent from rusty-bun-host's embedded QuickJS (basin boundary)
- **Source.** Direct probe 2026-05-10 — Doc 709 §6 P1 falsifier-direction test. `typeof WeakRef` returns `"undefined"` in rusty-bun-host; `"function"` in Bun 1.3.11. Both globals (WeakRef + FinalizationRegistry) are absent.
- **Severity.** Apparatus-side scope-limit; rquickjs/QuickJS-build-specific.
- **What it means.** The rusty-bun-host basin (per Doc 709) has a real boundary here. Consumer code using WeakRef-based caches, FinalizationRegistry-based resource cleanup, or any GC-coupled pattern will not run on rusty-bun-host as currently built. Real Bun runs such code via JavaScriptCore which has both.
- **Re-open condition.** Either (i) the embedded engine is upgraded to a QuickJS build with WeakRef support (mainline QuickJS-NG has discussed it; rquickjs would need to expose it), OR (ii) rquickjs is replaced with a different engine binding that exposes WeakRef + FinalizationRegistry.
- **Per M8(b):** scope-limit recorded; no Tier-J fixture depending on WeakRef has been built, so no fixture removal needed. Future fixture-author attempts on this axis must check this entry first.

### E8. crypto.subtle.importKey / sign / verify absent from rusty-bun-host — PARTIALLY CLOSED 2026-05-11
- **PARTIAL CLOSURE 2026-05-11 (HMAC-SHA-256)**: pilot gains `hmac_sha256` (RFC 2104, verified against RFC 4231 Test Cases 1 + 4); host gains `hmacSha256Bytes` + `timingSafeEqualBytes` bindings + JS-side WebCrypto wrappers. consumer-hmac-signer Tier-J fixture verified 11/11 byte-identical, including the RFC 4231 wire-level vector.
- **EXTENDED 2026-05-11 (SHA-1 + HMAC-SHA-1)**: pilot gains `digest_sha1` (FIPS 180-4) + `hmac_sha1` (RFC 2104 over SHA-1, verified against RFC 2202 Test Cases 1 + 2); host's JS-side wrappers refactored to dispatch on hash name via HASHES table. consumer-sha1-suite Tier-J fixture verified 12/12 byte-identical, including FIPS 180-1 digest vectors + RFC 2202 HMAC vectors.
- **EXTENDED 2026-05-11 (SHA-384 + SHA-512 + HMAC variants)**: pilot gains `digest_sha512` (FIPS 180-4, 80-round 64-bit-word 128-byte-block) + `digest_sha384` (SHA-512 with SHA-384 IV, truncated to 48 bytes) + `hmac_sha512` + `hmac_sha384` (RFC 2104 with 128-byte block per RFC 4231). Verified against FIPS 180-4 digest vectors + RFC 4231 Test Cases 1 + 2 for HMAC-SHA-512 + Test 1 for HMAC-SHA-384. HASHES dispatch table extended with SHA384 + SHA512 entries. consumer-sha512-suite Tier-J fixture verified 12/12 byte-identical including wire-level vectors and hash-isolation between 384 and 512.
- **EXTENDED 2026-05-11 (compositional verification: vendored JWKS-verifier library)**: a real-shape `jwks-verifier` library vendored at host/tests/fixtures/consumer-jwks-verifier-suite/node_modules/jwks-verifier/ implementing the production JWKS-based JWT verification pattern used by Auth0 / Okta / AWS Cognito / Google. Real-world security disciplines: kid match between token header and JWKS entry, alg match between header and JWKS entry's alg claim, use === "sig" check, signing-input reconstruction from raw segments, key-cache by kid. Multi-algorithm: RS256 / ES256 / PS256 dispatched through a single ALGS registry. consumer-jwks-verifier-suite Tier-J fixture verified 10/10 byte-identical against Bun 1.3.11. Tests: RS256 verify+claims, ES256 verify, wrong-iss rejection, expired rejection, alg-confusion attack rejection, unknown-kid rejection, tampered-payload rejection, key-cache works, multi-alg via one verifier, PS256-against-RS256-key rejection. **This fixture is the strongest telos sub-criterion 5 evidence the engagement has produced** — it exercises the entire E.8 closure (RSASSA-PKCS1-v1_5 + RSA-PSS + ECDSA + JWK parsing) end-to-end through a vendored library shape, with production security disciplines folded in.
- **EXTENDED 2026-05-11 (RSASSA-PKCS1-v1_5 over SHA-1/256/384/512)**: pilot gains EMSA-PKCS1-v1_5 encoding (RFC 8017 §9.2: DigestInfo prefix per hash + 0xff padding + 0x00 0x01 framing) + rsa_pkcs1_v15_sign + rsa_pkcs1_v15_verify. Closes the legacy/canonical RSA signature scheme used by JWS JWT RS256/RS384/RS512, X.509 CA signatures, code-signing. Deterministic (no nonce — distinguishing trait from PSS); cross-engine byte-equal signatures are the strong differential signal. ~70 LOC of pilot, M10-staged on the existing bigint substrate (compositionally vacuous M7 fold-back). consumer-jwt-rs256-suite Tier-J fixture verified 10/10 byte-identical against Bun 1.3.11 — real JWS JWT construction (header.payload.signature in JWS Compact Serialization with base64url-encoded segments), signature verification, tampered-payload + tampered-signature rejection, determinism (byte-equal across signs), hash-binding (key imported for SHA-384 cannot verify SHA-256 signature), RS384 + RS512 round-trips.
- **EXTENDED 2026-05-11 (ECDSA + ECDH over P-384 and P-521)**: pilot's EC code refactored to be curve-parameterized. New `Curve` struct carrying (p, n, b, G, coord_bytes) — a = -3 hardcoded since all NIST short-Weierstrass curves use this. New `ec_double` / `ec_add` / `ec_scalar_mul` / `ecdsa_sign` / `ecdsa_verify` / `ecdh` take a `&Curve` reference. `curve_p256()` / `curve_p384()` / `curve_p521()` constructor functions. P-256-specific functions retained for backward compatibility; they thinly wrap the generic API via `curve_p256()`. Author-side F6 caught in-round: my P-521 prime hex was missing 2 'f' digits (130 chars instead of 132 → bit_length 513 vs 521); diagnosed by Python sanity-check of the curve equation on a Bun-generated public key. Fixed. Pilot tests passed for ECDSA-P-384-SHA-384, ECDH-P-384, ECDSA-P-521-SHA-512, ECDH-P-521. Host bindings refactored from per-(curve,hash) functions to generic `ecdsaSignBytes` / `ecdsaVerifyBytes` / `ecdhBytes` taking curve name + hash name as parameters. JS-side importKey accepts crv ∈ {P-256, P-384, P-521} (previously rejected non-P-256); sign/verify dispatch reads hash from algorithm.hash and curve from the imported key's _curve; deriveBits requires both keys to share a curve. consumer-ec-curves-suite Tier-J fixture verified 4/4 byte-identical against Bun 1.3.11 across all four (curve, op) combinations.
- **EXTENDED 2026-05-11 (ECDH P-256 deriveBits)**: pilot gains `ecdh_p256(d, qx, qy)` — pure ECDH per SEC 1 §3.3.1: validate Q is on-curve, compute shared point d·Q via existing `p256_scalar_mul`, return the x-coordinate as 32 big-endian bytes. ~30 LOC reusing the EC substrate from the ECDSA round. Host exposes `ecdhP256Bytes` Rust binding; JS-side importKey path for EC keys extended to accept `{name:"ECDH"}` (specName branching mirrors RSA-OAEP/PSS pattern); `crypto.subtle.deriveBits` extended to dispatch ECDH alongside HKDF and PBKDF2, validating that `algorithm.public` is an ECDH public key and truncating output to the requested bit length. consumer-ecdh-p256-suite Tier-J fixture verified 5/5 byte-identical against Bun 1.3.11 — and because ECDH is deterministic given the two keys, byte-equal shared-secret across engines IS the differential signal (stronger than randomized-OAEP/PSS/ECDSA surfaces). Tests: import, Diffie-Hellman property (d_A·Q_B = d_B·Q_A), truncation prefix property, peer-key-flows-through (self-DH ≠ peer-DH), determinism. The DH property byte-equal across engines confirms both implementations compute the same EC scalar multiplication.
- **EXTENDED 2026-05-11 (ECDSA P-256 / SHA-256 sign/verify)**: pilot gains elliptic-curve substrate over NIST P-256 — finite-field mod_add/sub/mul/inv (Fermat inverse via mod_pow) + P-256 point doubling and addition in affine coordinates + scalar multiplication via binary double-and-add + ECDSA sign/verify per FIPS 186-4 §6.4 with WebCrypto P1363 r‖s signature format. Hand-typo bug caught during build: my initial G_y was wrong in the lower half (`2bbe33738df27a1480a8c1d335adcc9b` instead of canonical `2bce33576b315ececbb6406837bf51f5`); diagnosed by computing G's expected y-coordinate via Python `pow(rhs, (p+1)/4, p)` Tonelli-Shanks and comparing. The canonical G_y is now in place; pilot 99/99 tests including 2G against the known canonical 2G_x value, n*G yielding identity, G satisfying the curve equation, sign+verify round-trip, wrong-message rejection, tampered-signature rejection, off-curve public-key rejection. Host exposes `ecdsaP256Sha256SignBytes` + `ecdsaP256Sha256VerifyBytes` Rust bindings; JS-side importKey JWK path extended for EC keys (parses x, y, d from base64url; rejects non-P-256 curves); crypto.subtle.sign/verify dispatch ECDSA alongside HMAC and RSA-PSS; nonce k sourced from /dev/urandom via crypto.getRandomValues. consumer-ecdsa-p256-suite Tier-J fixture verified 6/6 byte-identical against Bun 1.3.11.
- **EXTENDED 2026-05-11 (RSA-PSS sign/verify over SHA-1/256/384/512)**: pilot gains EMSA-PSS-ENCODE + EMSA-PSS-VERIFY (RFC 8017 §9.1) + rsa_pss_sign + rsa_pss_verify (§8.1). EM = maskedDB ‖ H ‖ 0xbc with maskedDB = MGF1(H) ⊕ DB and DB = PS ‖ 0x01 ‖ salt. Reuses MGF1 + bigint substrate; ~150 LOC of new pilot code. Verify checks: trailer 0xbc, leftmost-bits-zero per emBits, all-zero PS, 0x01 separator, recomputed H matches via timing_safe_equal. Host exposes `rsaPssSignBytes` + `rsaPssVerifyBytes` Rust bindings with hash-name dispatch; JS-side importKey JWK path extended to accept name="RSA-PSS"; `crypto.subtle.sign` + `crypto.subtle.verify` dispatch RSA-PSS alongside HMAC; salt sourced via crypto.getRandomValues. consumer-rsa-pss-suite Tier-J fixture verified 7/7 byte-identical against Bun 1.3.11: JWK import, sign+verify round-trip, wrong-message rejection, tampered-signature rejection, randomized-signature property (sLen=32), deterministic mode (sLen=0), salt-length-binding check (sign sLen=32 must not verify with sLen=64).
- **EXTENDED 2026-05-11 (RSA-OAEP closure over SHA-1/256/384/512)**: pilot gains MGF1 mask-generation (RFC 8017 §B.2.1) + rsa_oaep_encrypt + rsa_oaep_decrypt (RFC 8017 §7.1). Hash-parameterized via `impl Fn(&[u8]) -> Vec<u8>` so all four SHA variants dispatch through the same code path. Host exposes `rsaOaepEncryptBytes` + `rsaOaepDecryptBytes` Rust bindings with hash-name dispatch; JS-side importKey accepts JWK form for RSA-OAEP (parses n, e, d via base64url; CRT components ignored — pilot uses non-CRT decrypt); `crypto.subtle.encrypt({name:"RSA-OAEP", label?}, publicKey, data)` + `crypto.subtle.decrypt(...)` wire the OAEP path; `crypto.getRandomValues(typedArray)` JS-side now backed by Rust `getRandomBytes` binding for seed generation. consumer-rsa-oaep-suite Tier-J fixture verified 6/6 byte-identical against Bun 1.3.11 (note: OAEP encrypt is randomized — differential signal is the engine's summary string, not byte-equal ciphertext; both engines produce "6/6"). Performance: 2048-bit modexp without CRT optimization is ~1-1.5s in release per operation; adequate for correctness verification, not for production hot paths.
- **EXTENDED 2026-05-11 (RSA bigint foundation)**: pilot gains BigUInt struct (little-endian Vec<u32> limbs) + I2OSP/OS2IP byte conversions + add/sub/mul (schoolbook) + binary long-division divmod + square-and-multiply mod_pow + plain-RSA primitives rsaep (m^e mod n) and rsadp (c^d mod n) per RFC 8017 §5. Verified against carrying limb additions, multi-limb multiplications, textbook RSA (n=33, e=3, d=7), and a real RSA-2048 key round-trip (m → c → m via mod_pow with 2048-bit modulus). 81/81 pilot tests. Foundation for RSA-OAEP / RSA-PSS landing in subsequent rounds — this round is math-layer only, no host wiring yet (per Doc 581 D5 staging: substantial substrate work warrants isolation).
- **EXTENDED 2026-05-11 (AES-CBC + AES-CTR + AES-KW)**: pilot gains AES inverse cipher (FIPS 197 §5.3: inverse S-box, InvShiftRows, InvSubBytes, InvMixColumns) + AES-CBC encrypt/decrypt (SP 800-38A §6.2 with PKCS#7 padding per RFC 5652 §6.3) + AES-CTR keystream (SP 800-38A §6.5 with WebCrypto counter-block layout: nonce-prefix + length-bit counter incremented modulo 2^length) + AES-KW wrap/unwrap (RFC 3394 with 0xA6A6A6A6A6A6A6A6 IV integrity check). Verified against FIPS 197 §C.1+§C.3 inverse vectors + SP 800-38A F.2.1 (CBC-AES128) + F.5.1 (CTR-AES128) + RFC 3394 §4.1, §4.3, §4.6 (KW vectors covering 128-bit KEK and 256-bit KEK + 256-bit-key wrap). Host exposes `aesCbcEncryptBytes` / `aesCbcDecryptBytes` / `aesCtrXorBytes` / `aesKwWrapBytes` / `aesKwUnwrapBytes` Rust bindings; JS-side importKey extended for AES-CBC/CTR/KW algorithm names; `encrypt`/`decrypt` dispatch CBC/CTR alongside GCM; new `wrapKey`/`unwrapKey` wrappers for AES-KW (format="raw" pilot scope) chain through importKey on unwrap. consumer-aes-modes-suite Tier-J fixture verified 10/10 byte-identical Bun 1.3.11 ↔ rusty-bun-host across all three modes.
- **EXTENDED 2026-05-11 (HKDF deriveBits, all four hashes)**: pilot gains `hkdf_inner` generic helper + `hkdf_sha{1,256,384,512}` (RFC 5869 Extract-and-Expand). Verified against RFC 5869 Appendix A Test Cases 1, 2, 3 (SHA-256, including longer inputs/outputs + zero-length salt/info) + Test Case 4 (SHA-1). Host exposes `hkdfSha{1,256,384,512}Bytes` bindings; JS-side `importKey` accepts `{name:"HKDF"}`; `deriveBits` refactored to dispatch HKDF and PBKDF2 via the HASHES table's `hkdf` and `pbkdf2` fields. consumer-hkdf-suite Tier-J fixture verified 10/10 byte-identical against Bun 1.3.11, including all four RFC 5869 vectors, hash isolation, info sensitivity, length-truncation prefix property, multi-block expansion, algorithm normalization.
- **EXTENDED 2026-05-11 (AES-GCM encrypt/decrypt)**: pilot gains AES-128/192/256 block cipher (FIPS 197 reference) + AES-GCM mode (NIST SP 800-38D) with authenticated encryption, 12-byte IV, 16-byte tag. Verified against FIPS 197 Appendix C.1 (AES-128) + C.3 (AES-256) block-cipher vectors + SP 800-38D Appendix B Test Cases 2, 3, 4 (the canonical GCM vectors including AAD coverage). Host exposes `aesGcmEncryptBytes` + `aesGcmDecryptBytes` Rust bindings; JS-side `importKey` accepts `{name:"AES-GCM"}` (16/24/32-byte raw keys); new `crypto.subtle.encrypt({name:"AES-GCM", iv, additionalData?, tagLength=128}, key, data)` + `crypto.subtle.decrypt` wrappers. Same-round M8(a) reconciliation: `TextDecoder.decode` extended to accept ArrayBuffer (was only handling typed-array views). consumer-aes-gcm-suite Tier-J fixture verified 11/11 byte-identical against Bun 1.3.11, including all three NIST vectors, AAD-bound round-trips, tampered-ciphertext/tag/AAD rejection, key-size and IV isolation, algorithm-name normalization.
- **EXTENDED 2026-05-11 (PBKDF2 deriveBits, all four hashes)**: pilot gains `pbkdf2_inner` generic helper + `pbkdf2_hmac_sha1/256/384/512` (RFC 2898 §5.2 with the canonical T_i = U_1 ⊕ U_2 ⊕ … construction). Verified against RFC 6070 Test Cases 1, 2, 3, 5 (PBKDF2-HMAC-SHA-1) + RFC 7914 §11 (PBKDF2-HMAC-SHA-256, 64-byte output). Host exposes `pbkdf2HmacSha{1,256,384,512}Bytes` bindings; JS-side `crypto.subtle.importKey` now accepts `{name:"PBKDF2"}` and `crypto.subtle.deriveBits({name:"PBKDF2", hash, salt, iterations}, key, lengthBits)` dispatches via the HASHES table's `pbkdf2` field. consumer-pbkdf2-suite Tier-J fixture verified 10/10 byte-identical against Bun 1.3.11, including the canonical RFC 6070 vectors, multi-block output, salt/iteration sensitivity, and short-output truncation.
- **STILL OPEN**: none on the canonical WebCrypto algorithm surface. All hash, MAC, KDF, AEAD, RSA, EC algorithms specified by W3C WebCrypto over the three NIST curves are now closed. E.8 is **fully closed on its original scope**. Remaining surface gaps are platform-level (DER/PEM key parsing as additional importKey formats, ECDH-derived-key via deriveKey with HKDF chain) — secondary to the algorithmic closure. RSA/ECDSA/ECDH need big-integer + finite-field-curve primitives. AES-KW/CBC/CTR are easy follow-ons to the existing AES block cipher. HKDF is straightforward given the HMAC family.
- **Hash-algorithm coverage post-2026-05-11**: SHA-1 ✓, SHA-256 ✓, SHA-384 ✓, SHA-512 ✓ (digest + HMAC + PBKDF2). The four SHA variants are the full HMAC algorithm space WebCrypto specifies; pilot scope on hash-based MAC + password-based KDF is complete.
- **Symmetric block-cipher coverage post-2026-05-11**: AES-GCM ✓ (128/256). The dominant AEAD on the web — JWE A128GCM/A256GCM, cookie sealing, encrypted-at-rest envelope crypto.

- **Source.** Direct probe 2026-05-10. Bun 1.3.11 has `crypto.subtle.importKey`, `crypto.subtle.sign`, `crypto.subtle.verify` as functions (full WebCrypto via JavaScriptCore). rusty-bun-host has `crypto.subtle.digest` (added in the consumer-request-signer round) and `crypto.subtle.digestSha256Hex`/`digestSha256Bytes` but no key-handle APIs.
- **Severity.** Apparatus-side scope-limit; web-crypto pilot scope-bounded.
- **What it means.** Consumer code performing HMAC / signature verification / asymmetric crypto operations on rusty-bun-host will fail. Real Bun supports the full surface. The rusty-bun web-crypto pilot covered SHA-256 + UUID v4 + getRandomValues + timing-safe — the constructive crypto primitives — but did not extend to the key-management surface.
- **Re-open condition.** Either (i) extend the rusty-bun web-crypto pilot with importKey/sign/verify for HMAC-SHA-256 minimum (the most-used variant; spec'd in WebCrypto), OR (ii) wire a different web-crypto pilot covering the full surface.
- **Per M8(b):** scope-limit recorded; no Tier-J fixture has been built against this surface (consumer-request-signer used `digest`, which IS supported); future fixture-author attempts on this axis must check this entry first.

### E9. Bun's host-integration globals absent from rusty-bun-host (compound basin boundary)
- **Source.** Direct probe 2026-05-10. Compound finding: five Bun-platform globals/namespaces absent from rusty-bun-host while present in Bun 1.3.11:
  - `Intl` (+ `Intl.NumberFormat`, `Intl.DateTimeFormat`, `Intl.Collator`): internationalization namespace. QuickJS can be built with Intl support via ICU but rquickjs default does not link it.
  - `Bun.password` (and `Bun.sql`): Bun extension APIs for password hashing / SQLite. Not part of the rusty-bun Bun-namespace wiring (which covers `Bun.serve`, `Bun.spawn`, `Bun.file` data-layer).
  - ~~`node:os`~~ **CLOSED 2026-05-10**: wired in host's `wire_os` covering platform/arch/type/tmpdir/homedir/hostname/endianness/EOL via std::env + cfg!(); CJS/ESM `require("node:os")` and `import os from "node:os"` both work; consumer-system-info fixture differentially verified.
  - `WebSocket`: transport-tier global. Requires socket binding (Tier-G).
  - `BroadcastChannel`: messaging-between-contexts. Requires shared-state infrastructure.
  - `Worker`: threading global. Requires actual thread spawning.
- **Severity.** Apparatus-side scope-limits at several distinct surfaces.
- **Re-open conditions.** Per-surface:
  - Intl: link an ICU-equipped QuickJS build, or wire a minimal Intl pilot covering NumberFormat/DateTimeFormat with limited locales.
  - Bun.password: wire a `Bun.password.hash`/`verify` pilot over Argon2/bcrypt.
  - WebSocket / BroadcastChannel / Worker: each requires substantial transport-or-threading infrastructure; deferred to engagement scope beyond current Tier-G.
- **In-basin counterparts confirmed in same probe:** `Atomics`, `SharedArrayBuffer`, `WeakMap`, `WeakSet`, `Symbol.asyncIterator` — these were also probed and are PRESENT in rusty-bun-host's QuickJS. Lock-free primitives are available even though threading globals are not.
- **Per M8(b):** scope-limits recorded; no Tier-J fixture has been built against these surfaces; future fixture-author attempts must check this entry first.

### E11. `process` global absent from rusty-bun-host — CLOSED 2026-05-10
- **Source.** Direct probe 2026-05-10. Bun 1.3.11 has `process` as object with full surface (argv array, env object, platform string, version string, cwd function, exit function, stdout/stderr write functions, hrtime). rusty-bun-host had `typeof process === "undefined"` — process global was entirely missing.
- **Severity at the time.** Apparatus-side scope-limit; very high-impact since `process` is used pervasively by real consumer code (config via env, CLI args, OS detection, runtime termination).
- **Historical impact.** Tier-J fixtures had been silently bypassing the `process.stdout.write` first branch in their dual-path emission pattern; the fallback `globalThis.__esmResult` path was carrying the result, making the gap invisible from test outcomes until probed directly.
- **CLOSED**: same commit closes via `wire_process` (Rust-side) covering argv/env/platform/arch/version/versions/cwd/exit/stdout.write/stderr.write plus `hrtime` and `hrtime.bigint`. `node:process` builtin resolution added to is_node_builtin + node_builtin_esm_source + CJS NODE_BUILTINS table for symmetric ESM/CJS support. consumer-cli-tool Tier-J fixture differentially verified.

### E10. Set.prototype.union / .intersection / .difference (ES2025) absent from rusty-bun-host — CLOSED 2026-05-10
- **Source.** Direct probe 2026-05-10. Bun 1.3.11 has `Set.prototype.union`, `.intersection`, `.difference` as functions (ES2025 / TC39 proposal-set-methods Stage 4). rusty-bun-host had them as `undefined` — its embedded QuickJS predates the merge.
- **CLOSED**: same commit closes via `install_set_methods_polyfill` (JS-side at host init) installing all seven ES2025 set-methods on Set.prototype: union, intersection, difference, symmetricDifference, isSubsetOf, isSupersetOf, isDisjointFrom. ~90 LOC JS, no Rust changes. consumer-set-algebra Tier-J fixture differentially verified 10/10 byte-identical.
- **In-basin counterparts confirmed in same probe:** Promise.withResolvers (ES2024), Array.prototype.toSorted/toReversed/toSpliced/with (ES2023), Object.groupBy (ES2024), structuredClone on Uint8Array, Atomics.wait/notify all PRESENT.

## Category F — Fixture-author Mode-5 findings (rusty-bun engagement-internal)

Author-side typos and spec-misunderstandings surfaced during M9 spec-first fixture authoring. NOT Bun bugs — these are cases where the author wrote spec-violating JS and the runtime (Bun and rusty-bun-host alike) correctly threw. The category is kept as a trace of what spec-strictness Bun enforces and what the author should remember when authoring future fixtures. Each entry implicitly attests Bun's spec compliance on the surface where the author tripped.

### F2. Symbol.toPrimitive hint semantics ("default" vs "number")
- **Source.** consumer-sequence-id fixture initial Bun run, 2026-05-10. Author wrote `Symbol.toPrimitive(hint)` returning numeric for `"number"` only and string for `"default"`. `id + 8` produced `"id-428"` (string concatenation), not `50`.
- **Spec.** ECMAScript spec: arithmetic `+` on non-Date uses hint `"default"`, not `"number"`. `String()` / template-literal coercion uses hint `"string"`. The `"number"` hint is rare in practice — chiefly used by explicit `Number(x)` and certain comparison contexts.
- **What it attests.** Bun is spec-correct on the toPrimitive hint protocol — the failure was the author's misunderstanding of which hint `+` uses.
- **Author rule.** When writing `Symbol.toPrimitive(hint)`, `"default"` should typically return the same as `"number"` (most consumer code wants numeric coercion for `+`); reserve `"string"` for the template-literal/String-conversion path. The branch order matters: handle `"string"` first, then numeric for the rest.

### F3. Library-semantics misread without reading source
- **Source.** consumer-vendored-pkg fixture initial Bun run, 2026-05-10. Author expected `clsx(1, 2, 3) === "123"` (concatenation). Actual: `clsx(1, 2, 3) === "1 2 3"` (space-joined per the library's top-level arg loop).
- **Spec.** clsx's documented behavior: each truthy argument's toVal output is space-separated from the running string. The author's "concatenation" assumption was a misread of the library's semantics.
- **What it attests.** Both Bun and rusty-bun-host execute clsx identically to its documented semantics. The failure was the fixture author's expectation, not either runtime.
- **Author rule.** When vendoring third-party code, write expected values by *running the library and copying the actual output*, not by inferring semantics from the function name. The library's source is authoritative; one's intuition about what `clsx` "should do" is not.

### F1. BigInt-arithmetic operand-type strictness
- **Source.** consumer-batch-loader fixture initial Bun run, 2026-05-10. Author wrote `id % 2 === 0n` (mixing Number `2` with BigInt `id`). Bun threw `TypeError: Invalid mix of BigInt and other type in remainder.`
- **Spec.** ECMAScript spec: BigInt operators require both operands BigInt; no implicit coercion.
- **What it attests.** Bun is spec-strict on BigInt mixing — the throw is correct behavior.
- **Author rule.** When using BigInt anywhere in an arithmetic expression, all operands must be `n`-suffixed literals or BigInt-valued. The shortcut: write `id % 2n === 0n`, not `id % 2`.

## How this catalogue is maintained

The catalogue is updated as new pilots run. Each pilot's RUN-NOTES.md cross-references findings here. Categories A and C grow with consumer-regression pilots (per Doc 707's bidirectional reading); B is rare and stable; D is provisional and items move to A or get deleted as they're investigated; E grows with verifier-caught derivation bugs **and runtime-integration-pin findings** (the JS-host iteration on 2026-05-10 was the first session that contributed E entries from a non-pilot source).

Bun maintainers are welcome to use any of these directly, link to them in commits/issues, or ignore them. The apparatus produces them as a by-product of derivation work; the cost to surface them is low; their value is whatever Bun's discretion finds.

For background on the apparatus that produced this catalogue, see [Doc 706 (Three-Pilot Evidence Chain)](https://jaredfoy.com/resolve/doc/706-three-pilot-evidence-chain-derivation-from-constraints) and [Doc 707 (Pin-Art at the Behavioral Surface)](https://jaredfoy.com/resolve/doc/707-pin-art-at-the-behavioral-surface-bidirectional-probes).
