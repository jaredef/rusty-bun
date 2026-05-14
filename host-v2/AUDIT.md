# host-v2 — rusty-bun-host on the new engine

**Tier-Ω.4 — migration of rusty-bun-host from rquickjs to rusty-js-runtime.** Per [the migration design spec](../specs/omega-4-host-migration-design.md) + [Doc 714 §VI Consequence 5](/resolve/doc/714-the-rusty-bun-engagement-read-through-the-lattice-extension-basin-expansion-at-the-l2m-saturation-point#consequence-5--the-event-loop-belongs-inside-the-engine-amendment-2026-05-14).

## Engagement role

host-v2 replaces the existing host/ crate's rquickjs backend with rusty-js-runtime. The migration is the engagement's largest substrate work product after the engine itself — predicted at 5-7 rounds.

Per the substrate-amortization discipline (seed §A8.13), host-v2 ships in phases:

- **Ω.4.a** — substrate-introduction (this commit): design spec + AUDIT.md
- **Ω.4.b** — host-v2 Cargo skeleton + Math/JSON/console/Promise free via engine + path/os/process minimal
- **Ω.4.c** — fs + http + node:* breadth
- **Ω.4.d** — TLS + WebSocket + crypto.subtle
- **Ω.4.e** — mio reactor integration as PollIo host hook; **falsifier measurement**
- **Ω.4.f** — CJS↔ESM bridge + Tuple A/B host hooks
- **Ω.4.g** — final cleanup + parity remeasurement; Ω.5 lock

## What lives where post-migration

- **rusty-js-runtime** (engine, ~2,070 LOC): JS execution + JobQueue + run-to-completion + Math + JSON + console + Promise + HostFinalizeModuleNamespace hook + PollIo hook
- **host-v2** (host, predicted ~14,500 LOC): mio Poll integration + node:* intrinsics + fs/path/os/etc. wirings + CJS↔ESM bridge (source rewriting) + ResolveMessage error shape + module-resolver hook

The pre-Ω.4 host's nine reactor sub-rounds (Π2.6.c.a-e + Π2.6.d.a-d) collapse to a single ~300 LOC mio integration in host-v2.

## Migration-cost falsifier (per Doc 714 §VI Consequence 5)

| Component | Pre-Ω.4 LOC | Post-Ω.4 LOC | Δ |
|---|---:|---:|---:|
| Module loading | 600 | 300 | -300 |
| JS-side polyfills | 12,000 | 8,000 | -4,000 |
| mio reactor + JS surface | 2,000 | 300 | -1,700 |
| CJS bridge | 400 | 250 | -150 |
| HostFinalize hook polyfill | 200 | 80 | -120 |
| Resolution error shape | 50 | 30 | -20 |
| Parity tool | 150 | 150 | 0 |
| Intrinsic wirings (rest) | 6,000 | 5,500 | -500 |
| **Total** | **21,400** | **14,610** | **-6,790 (-32%)** |

Per the Consequence 5 falsifier: if Ω.4.e's measurement shows roughly equal LOC to the pre-Ω baseline, the architectural shift didn't deliver and the cut-rung diagnosis was operationally wrong.

## Predicted post-migration parity baseline

Per Doc 717 P3's tuple classification + the host-hook surface:

- Tuple A (7 packages) retires via HostFinalizeModuleNamespace
- Tuple B (3 packages) retires via same hook
- Tuple C (2 packages) already retired by the parser
- D-class (got, enquirer) remain as independent investigations

**Predicted baseline: 117/119 ≈ 98.3%** (vs 88.2% pre-Ω).

## First-round scope (this commit — Ω.4.a)

Substrate-introduction only:
- specs/omega-4-host-migration-design.md
- host-v2/AUDIT.md (this file)

No Cargo crate yet. Next round (Ω.4.b) creates host-v2/Cargo.toml + the bin scaffold + the first intrinsics.
