# Edge → Substrate Stratum Map

Pin-Art bilateral reading at the substrate-stratum level. Edges E.1–E.56
clustered by the engine primitive each edge probes. Per §A8.18, the
geometry is K substrate strata exposed through N edge observations.

## Section 1 — Stratum map (dependency order, engine-foundation first)

### S1. Regex /u strict-escape semantics
**Axis:** QuickJS rejects some backslash-escape sequences under /u flag
that the ECMAScript spec and V8 permit.
- E.29 camelcase ^9 — `\-` in `[...]` after re-compile with `'gu'` — **CLOSED** (RegExp wrapper)
- E.43 camelcase-keys ^10 — transitive — **CLOSED**
- E.56 ajv-regex — different /u escape in Ajv internals — **OPEN**
- *Suspected members:* E.12 hono arrow-field, E.13 strict-mode reserved

### S2. Wall-clock timer scheduling
**Axis:** setTimeout(fn, ms) honoring ms instead of microtask-immediate.
- E.54 delay ^7 — **CLOSED** (deadline-based + tick pump)
- E.55 p-series — transitive — **CLOSED**
- E.18 polka cooperative-loop — likely subsumed (re-test needed)
- E.19 megastack same — likely subsumed
- E.44 p-throttle ^8 — likely subsumed
- E.41 p-wait-for ^6 — partial (TimeoutError naming still diverges)

### S3. Node-builtin module namespace
**Axis:** which node:* identifiers resolve to a JS object via the loader.
- E.17 fastify ^5 — was missing 11 node:* modules — **MOSTLY CLOSED** (stops at S6 Ajv regex)
- *Beneficiaries:* any modern Node-targeted lib (pino, megastack, polka, etc.) — gain landed by induction even without individual fixtures

### S4. Conditional-exports resolver (CJS path)
**Axis:** recursive walk of nested `"import"/"require"/"default"` keys.
- toad-cache (substrate-only; no E.NN assigned) — **CLOSED**
- *Beneficiaries:* any package shipping nested-conditional `exports` field

### S5. http.Server API-surface shape
**Axis:** `createServer(opts, handler)` two-arg form + Server.setTimeout +
event-emitter shape stubs.
- E.17 fastify constructor — **CLOSED**

### S6. Regex /u strict-escape semantics (deep)
**Axis:** same as S1 but inside engine-bundled deps (Ajv); current
RegExp wrapper preprocessor doesn't catch all cases.
- E.56 ajv-regex — **OPEN**, blocks fastify operation

### S7. ESM/CJS class-inheritance boundary
**Axis:** CJS bridge preserving prototype chain across require/import.
- E.14 — **OPEN**

### S8. Π2.6.b cooperative-loop (legacy, pre-timers)
**Axis:** non-blocking-TCP yield budget before real wall-clock timers.
- Most members likely subsumed by S2 — needs re-test of E.18/E.19

### S9. fs/promises async surface
**Axis:** node:fs Promise-returning surface beyond sync subset.
- E.20 glob ^10 — **OPEN** (readdir, real recursive walk)

### S10. Stream Transform / TransformStream
**Axis:** WHATWG TransformStream wire + node:stream.Transform parity.
- E.21 csv-parse — **OPEN**
- E.37 ndjson-parser — **OPEN**

### S11. Intl namespace
**Axis:** locale-aware NumberFormat / DateTimeFormat / Collator.
- E.9 compound — **OPEN**
- E.22 luxon — **OPEN**
- E.35 pretty-bytes — **OPEN**

### S12. WebCrypto asymmetric primitives
**Axis:** ECDSA / ECDH / Ed25519 elliptic-curve big-integer arithmetic.
- E.8 partial (HMAC/PBKDF2/HKDF/AES/RSA closed; EC/Ed25519 open)

### S13. globalThis aliases (window/self UMD)
**Axis:** browser-shaped global object writes.
- E.24 prism — **OPEN**

### S14. ES2022+ syntax density
**Axis:** private class fields + decorator-style annotations.
- E.25 p-limit — **OPEN**
- E.26 chalk — **OPEN**

### S15. GC reference object model
**Axis:** WeakRef / FinalizationRegistry.
- E.7 — **OPEN** (QuickJS structural absence)

### S16. Numeric / bytes formatting divergence
**Axis:** Buffer.from raw octet path, hex output formatting.
- E.40 base64url — **OPEN**
- E.51 md5-hex — **OPEN**

### S17. Error stack format (V8 vs QuickJS)
**Axis:** stack-string shape consumers parse.
- E.45 clean-stack — **OPEN**

### S18. ANSI-aware string width (Unicode emoji widths)
**Axis:** east-asian + emoji width tables.
- E.47 wrap-ansi — **OPEN**
- E.50 cli-truncate — **OPEN**

### S19. Heterogeneous declare-entry exceptions (unresolved)
**Axis:** distinct top-level module-evaluation throws lumped by surface
symptom; each likely a distinct stratum.
- E.27 dotenv ^16, E.28 pug ^3, E.30 jsonpath-plus, E.31 jsonschema,
  E.32 cbor-x, E.33 msgpackr, E.34 ohash, E.36 yargs-parser, E.38 shortid,
  E.39 supports-color, E.42 iconv-lite, E.46 node-emoji, E.48 bson,
  E.49 tiny-pinyin, E.52 normal-distribution, E.53 unraw
- *Sub-strata not yet bisected.*

## Section 2 — Axis count

- **Identified strata:** S1–S18 = 18 (S19 is a placeholder bucket pending
  per-edge bisection; honest K is 18 + however many distinct primitives
  hide inside S19).
- **N (recorded edges):** 56
- **K/N ratio:** 18/56 ≈ **0.32** at current resolution.
- **Conjecture refinement:** raw K/N at this resolution does NOT show
  K << N. But three of the 18 named strata (S1, S2, S3) account for
  10+ edges and 3 transitive closures landed for free in this session.
  Cluster sizes are bimodal: a few large clusters (S1, S2, S3, S19)
  and many singletons.

**Highest-leverage open strata:**
1. S6 Ajv regex /u-deep — single fix unblocks fastify's full operation
2. S19 declare-entry bucket — bisecting yields 5–10 sub-strata, probably
   reveals several large clusters
3. S11 Intl — closes E.9 + E.22 + E.35 + likely several S19 members
4. S2 timers second pass — confirms E.18/E.19/E.44 collapsed

**Most engine-deep open strata:**
- S15 WeakRef (QuickJS absent — engine upgrade)
- S12 EC arithmetic (rusty-bigint extension)
- S14 ES2022+ syntax (QuickJS parser features)

## Section 3 — Pin-Art reading

K/N ≈ 0.32 at current resolution **does not refute K << N**; it confirms
that the *upper bound* on stratum count is much smaller than edge count
but the *current resolution* on the declare-entry bucket (S19, 16 edges)
is too coarse — each S19 member is a separate pin reading awaiting
bisection. The engagement's actual position on the SIPE-T curve: the
in-basin axes (S1–S5 closed/mostly-closed this session) hit cluster
ratios of 2–11×; the open frontier (S6–S18) is genuine engine work
where one substrate widening still retires a cluster but the strata
sit deeper in the engine stack (regex semantics, Intl namespace, EC
arithmetic, GC model). Total work to telos closure is K' (after S19
bisection), and K' is bounded by the number of *primitive substrate
points where Bun and rusty-bun-host's QuickJS-derived engine diverge*,
not by N — a finite, enumerable count, deeper than wide.
