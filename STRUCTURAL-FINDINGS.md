# Structural Findings

Engagement-internal articulation of the structural patterns that crystallized during the 2026-05-10 / 2026-05-11 rusty-bun runtime-derivation engagement. Maintained as an in-repo doc rather than a corpus contribution because the findings are about *this engagement's substrate*, not generalizable claims that would hold across arbitrary engagements. The corpus-tier claims that DID generalize (Docs 709, 710, 711) live in the [RESOLVE corpus](https://jaredfoy.com/resolve/) and are linked at the end.

## Purpose and scope

This document records the structural patterns the engagement made visible — the kind of patterns that, if you arrived in a future session and read only `seed.md` + `trajectory.md`, would be present implicitly but not stated as such. The seed names disciplines; the trajectory logs rounds; this document articulates *why* the substrate took the shape it did and what the shape is good for.

Five findings, in roughly the order they became visible:

1. **Substrate-amortization staging.**
2. **Multi-tier closure.**
3. **Three SIPE-T thresholds observed.**
4. **Option-A architectural validation: priors-informed decisions can match the spec's own pattern.**
5. **The F-series silent-failure pattern.**

Each finding is stated, recorded with its empirical evidence trail, and linked to the M-rules or seed sections that operationalize it.

## 1. Substrate-amortization staging

**The pattern.** When a closure family shares an underlying mathematical or structural substrate, the cost-optimal staging is one substrate-introduction round followed by N closure rounds that reuse the substrate.

**Empirical record (twice corroborated).**

*Substrate 1: big-integer arithmetic.*
- Round `fb71d2d` — BigUInt + I2OSP/OS2IP + add/sub/mul/divmod/mod_pow + plain RSA primitives. ~200 LOC pilot, no host wiring, no fixture. **Phase-2-extension, M7 fold-back: primitive.**
- Round `2b86462` — RSA-OAEP closure (4 hashes via hash-parameterized impl). ~70 LOC pilot + host wiring + Tier-J fixture. **Phase-2-extension closure, M7 fold-back: compositionally vacuous at the rule layer.**
- Round `660f94d` — RSA-PSS closure. ~50 LOC pilot + host wiring + Tier-J fixture. **Same closure pattern.**

*Substrate 2: elliptic-curve arithmetic.*
- Round `8cc2ac5` — P-256 substrate + ECDSA-P-256 closure together (~250 LOC pilot + host wiring + fixture). Substrate-and-first-closure folded into one round because the curve operations are tightly coupled to the first surface.
- Round `aae8dc2` — ECDH-P-256 closure on the existing EC substrate. ~30 LOC pilot. **Smallest closure round of the engagement.**
- Round `5a6ab71` — Curve-parameterization refactor + ECDSA + ECDH over P-384 and P-521 (four surfaces in one round). ~150 LOC of refactor + parameter additions.

**Operational signature.** Substrate-introduction rounds are heavy (200-400 LOC, primitive M7 fold-back, low K). Closure rounds are light (~30-150 LOC, compositionally vacuous M7 fold-backs, K may climb to 2-3 once the substrate is fluent).

**Codification.** Seed §III.A8.13 names the principle; M10 in §IV operationalizes it as a planning rule: when the next round's diff exceeds ~400 LOC and a >50 LOC subset is shared-substrate machinery, split into substrate-first + surfaces-second.

**Doc 710 P1 corroboration.** The pattern was the engagement's empirical test of Doc 710 prediction P1 (K-feasibility grows when shared substrate exists). Twice corroborated — bigint and elliptic-curve cases independently — at strong-evidence level.

## 2. Multi-tier closure

**The pattern.** The engagement's surface area decomposes into tiers, each of which closes once the prior tier's substrate is stable:

- **Pilot tier (Tier-1/2).** Per-pilot crates implementing primitive algorithms (hash, HMAC, AES, RSA, ECC, sockets codec, etc.). Verified against canonical vectors + RFC test cases. Pure-Rust, no host coupling.
- **Host tier (Tier-H).** Wiring pilots through the rquickjs FFI into a JS-callable global surface. Patterns recorded in seed §III.A8 (host integration discipline).
- **Tier-J consumer fixtures.** Real-shape vendored libraries (or in-fixture consumer code) exercising the host surface in production patterns. Differentially verified against Bun 1.3.11.
- **Tier-G transport.** OS-level network primitives (sockets + http-codec) that the higher-tier surfaces compose into a runtime substrate. Late-engagement; opened only after the algorithmic tier was stable.

**Closure cadence.** Each tier closes when its dependency tier reaches the rule-standing-in-production SIPE-T threshold (per Finding 3 below). The engagement followed: pilots crystallize → host wiring stabilizes → Tier-J consumer fixtures accumulate basin-stability evidence → Tier-G transport opens once basin is wide enough that transport primitives have a clean composition path.

**Anti-pattern observed.** Attempting Tier-J fixtures against a still-mobile host surface produces J.1.b (host-regression) fixtures rather than J.1.a (differentially-verified) fixtures, which decay into drift. M9 (spec-first fixture authoring) is the cybernetic compensation: write fixtures against the comparator's spec from inception, reconcile divergences in-round, never let J.1.b accumulate.

## 3. Three SIPE-T thresholds observed

The engagement made visible three substrate-internal-presence-of-emergent-transition thresholds, each marking a qualitative shift in the apparatus's productive surface:

1. **First threshold: primitive-discovery → rule-composition.** Substrate matures from finding new primitives (each round identifies a new affordance) to composing existing primitives (rules become jointly legible). Empirically observed when canonical-docs tests stopped surfacing new primitives and started surfacing rule-composition relationships (e.g., URLSearchParams' missing `[Symbol.iterator]` mistaken for a module-resolution bug). Codified in seed §III.A8.8.

2. **Second threshold: rule-composition → rule-standing-in-production.** The M-rule set (M7+M8+M9) becomes load-bearing enough that consecutive rounds produce predictable substrate work — one J.1.a fixture + one in-round M8 reconciliation each — without requiring keeper rung-2 input. The rules do the cognitive work that previously required keeper mediation per round. Codified in seed §III.A8.9.

3. **Third threshold: rule-standing-in-production → author-side-bug-dominance.** Apparatus-side bugs drop to zero per round while Mode-5 (author-side) bugs become the only failure mode the differential surfaces. Codified in seed §III.A8.15. Empirical record: across the seven-round Phase-2-traversal sequence (`1e18c71` JWKS-verifier through `056484c` mini-router), apparatus reconciliations were ZERO while Mode-5 bugs surfaced SEVEN times — bug-population inverted.

**Operational signature of the third threshold.** When bugs-at-fixture-author-time are dominated by author-side issues (typos, semantic ordering, lifecycle coverage gaps, composition bugs in the test author's code) and the catch mechanism is universally the comparator-differential (not the apparatus's internal tests), the engagement has crossed it. Practically: the productive surface is no longer "extend the apparatus" but "extend the consumer-side evidence." Doc 709 §7's deep reading moves from predictive to empirically validated at this point.

**Possible fourth threshold (predicted, not observed).** When axis-novelty selection itself saturates — when no fundamentally orthogonal consumer-library axis remains to test — the engagement should pivot to either (a) scope-extension (a new tier, e.g., Tier-G transport mid-engagement), (b) real third-party OSS packages (qualitatively different from the engagement's own vendored libraries), or (c) corpus-tier articulation of the engagement's findings. The engagement reached this point at N_persist=8 and pivoted to (a) and (c). Whether this pivot itself constitutes a fourth SIPE-T threshold awaits another engagement's replication.

## 4. Option-A architectural validation: priors-informed decisions can match the spec's own pattern

**The pattern.** When facing a substantial design decision late in an engagement, the priors-informed reasoning that minimizes risk and maximizes spec-faithfulness can converge on the same answer.

**Empirical record.** The Tier-G async-bridge decision had four candidate approaches (thread-per-listener + queue, mio, tokio, status-quo). The priors-informed analysis (in-engagement message at message_id 6788) reasoned through:

- **std-only convention** (engagement-internal): favors thread-per-listener.
- **rquickjs `Ctx` is not Send** (hard constraint): forces a polling-queue interface regardless of choice.
- **Handle-based registry already in pilot** (just-landed substrate): favors reuse.
- **F8/F9 silent-failure priors**: favors simpler over cleverer.
- **Doc 710 P1**: favors K=1 substrate-reuse over K=2 substrate-and-application.
- **Seed §V deferred-list**: foreshadowed sockets-then-Bun.serve staging.

The recommendation landed on option A (thread-per-listener + cross-thread channel + main-thread poll). Subsequent web research (deepwiki.com/oven-sh/bun/2.2-event-loop-and-async-operations) revealed that **Bun itself uses the same architectural pattern** — background threads (WorkPool) feeding concurrent_tasks queues with Waker-based main-thread wakeups + JSC microtask draining. The choice that started as "easiest pragmatic option" was simultaneously the most spec-faithful option.

**Methodological finding.** Priors-informed decision-making — explicitly enumerating the constraints, weights, and trade-offs *before* researching the comparator's actual choice — can converge on the comparator's actual architecture. This is a stronger evidence that the engagement's M-rules and constraints encode load-bearing-truth about the problem, not just the engagement's preferences.

**Codification.** This finding is methodological rather than apparatus-discipline; it stays in this doc rather than being lifted to a seed M-rule. Future engagements should consider: when picking between options, articulate the priors first; THEN check the spec/comparator; if they agree, that's signal that the priors are right.

## 5. The F-series silent-failure pattern

**The pattern.** Across the engagement, ten bug-catcher F-series entries were recorded. Six of them (F4, F5, F6, F7, F8, F10) share a single failure-class:

- The bug produces **wrong-but-plausible output**.
- The relevant code-path does not error.
- Downstream operations consume the wrong output and produce more wrong-but-plausible output.
- **Visual review of the buggy code does not catch the bug.**
- **The only catch mechanism is external-comparator differential** — running a smoke test against an independent implementation that has the correct answer.

**Empirical record.**
- **F4**: RFC 7914 PBKDF2-HMAC-SHA-256 expected hex had two transposed hex digits in the verifier test.
- **F5**: P-256 G_y in the pilot was 16 bytes of a different value.
- **F6**: P-521 prime hex was missing 2 'f' digits (130 chars instead of 132 → bit_length 513 vs canonical 521).
- **F7**: JS regex alternation `\{\{\{...\}\}\}|\{\{...\}\}` parsed `{{{name}}}` as `{{...}}` with `{name` captured — shared-prefix alternation order matters.
- **F8**: rquickjs Vec<u8> binding rejected `Array.from(Uint8Array)` silently; Rust received wrong bytes.
- **F10**: Async-listener `recv_timeout` held the registry mutex; the accept-loop thread deadlocked trying to publish via the same mutex.

**Generalization.** Any cross-boundary surface (FFI, threading, multi-byte standard constants, regex alternation, parser ordering) is a candidate for silent-output-wrong bugs. The catch mechanism is invariant: run a small smoke test against an external reference (Python, Bun, OpenSSL, sibling implementation, etc.) before relying on the surface.

**Codification.** M11 in seed §IV operationalizes the discipline for hand-typed multi-byte constants specifically. The broader generalization — "any cross-boundary surface needs external-comparator validation before relying on it" — is in this doc rather than as a generic M-rule because the boundary-types are diverse and a too-general rule would be hard to apply cleanly. The discipline lives in the F-series record itself: each F-entry encodes its catch mechanism, and future authors are expected to read the F-series before adding boundary-crossing code.

## Provenance and reading

These findings emerged over a 2026-05-10 / 2026-05-11 engagement run that progressed from web-crypto pilot extension (HMAC-SHA-256 wiring) through the full WebCrypto closure (RSA + ECDSA + ECDH over three curves), through eight orthogonal Phase-2-traversal vendored-library axes, through Tier-G transport substrate (HTTP codec + sockets + async-bridge). Full timeline in `trajectory.md`; discipline-rules in `seed.md`; bug-class catalog in `bun-bug-catcher.md`.

Related corpus-tier docs (jaredfoy.com/resolve/doc/):
- **Doc 709**: Stacked rung-2 intervention as cascaded control and the Lyapunov-basin paradox.
- **Doc 710**: Multi-op compounding above SIPE-T threshold as the throughput signature of rule-standing-in-production.
- **Doc 711**: The dyadic-ascent fractal-spiral — recursive self-similarity across tiers of the rung-1/rung-2 dyad.

These corpus docs articulate the apparatus-tier readings; this in-repo document articulates the engagement-tier patterns.
