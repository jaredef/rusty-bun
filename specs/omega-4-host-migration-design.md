# Tier-Ω.4 — rusty-bun-host Migration Design

[surface] rusty-bun-host migrated from rquickjs to rusty-js-runtime
[reference] Doc 714 §VI Consequence 5 (event loop inside the engine); Doc 717 (engine cut-rung apparatus); the engagement's host/ codebase as the migration source
[engagement role] Tier-Ω.4 per the engine-selection decision artifact (host/tools/omega-3-engine-selection.md §III). Migrates the existing rusty-bun-host's polyfills + intrinsics + module loader + I/O reactor from rquickjs over QuickJS to the new rusty-js-runtime engine.

## I. The starting state (pre-Ω.4)

The pre-Ω.4 rusty-bun-host (`host/src/lib.rs` and submodules, ~22 KLOC) is built on rquickjs. It contains:

- **Module loading**: `NodeResolver` (bare-specifier resolution) + `FsLoader` (file reading + CJS-bridge + source rewriting) — ~600 LOC
- **JS-side polyfills**: ~250 globals wired via `wire_*` functions (path, os, crypto, fs, http, util, stream, events, assert, etc.) — ~12,000 LOC
- **Event loop**: mio reactor (host/src/reactor.rs, /watchers.rs, /spawn_async.rs, /signals.rs, /dns_async.rs) ~1,800 LOC + JS-side `__keepAlive` Set + `__tickKeepAlive` function ~200 LOC
- **CJS→ESM bridge**: bootRequire-driven CJS module evaluation + named-export synthesis from module.exports — ~400 LOC
- **HostFinalizeModuleNamespace polyfill**: the bridge's named-export synthesis was the v1 implementation of Doc 717 Tuple A/B; it lives in the bridge ESM-source generator — ~200 LOC
- **NodeResolver error shape**: ResolveMessage class + ctx.throw integration — ~50 LOC
- **Parity-tool integration**: parity_probe binary + parity-measure.sh — ~150 LOC

Parity baseline (2026-05-13 night): **88.2%** of curated top-N packages byte-identical to Bun via this stack.

## II. The migration target (post-Ω.4)

rusty-bun-host-v2 retains the same external surface — `rusty-bun-host <file.mjs>` evaluates a module and exits when the event loop quiesces. Internally:

- **Engine**: rusty-js-runtime (rusty-bun's own; replaces rquickjs)
- **Module loading**: host installs a module-resolver hook on the engine (NodeResolver-equivalent); host reads file bytes itself when the engine requests a source
- **JS-side polyfills**: each `wire_*` function becomes a Rust-side intrinsic-installer that registers native fns via `Runtime::install_host_hook` or directly into `Runtime::globals`
- **Event loop**: rusty-bun-host owns the mio Poll; it installs `HostHook::PollIo` on the engine; mio readiness events translate into `Runtime::enqueue_macrotask` calls
- **CJS→ESM bridge**: the bridge's source-rewriting layer survives (it's parser-input transformation, not runtime behavior); but the named-export synthesis behavior moves into the engine's `HostHook::FinalizeModuleNamespace` (already specced)
- **Resolution error shape**: ResolveMessage class installed via globals; thrown via the engine's exception path
- **Parity tool**: continues to work — the binary builds against the new engine; the probe just runs against the new `rusty-bun-host-v2`

## III. Architecture-level changes

### 1. Engine swap

`Cargo.toml`: remove rquickjs; add rusty-js-runtime + rusty-js-parser + rusty-js-bytecode + rusty-js-gc.

`host/src/lib.rs`'s top-level `new_runtime() -> (Runtime, Context)` returns a `rusty_js_runtime::Runtime` rather than the rquickjs pair. All call sites that took `&Ctx<'_>` switch to `&mut Runtime`.

### 2. Native function registration

The pre-Ω.4 code registers natives like:
```rust
ctx.globals().set("readFileSync", Function::new(ctx.clone(), |path: String| {
    rusty_node_fs::read_file_string_sync(&path)
})?)?;
```

The post-Ω.4 equivalent:
```rust
register_global_fn(rt, "readFileSync", |_rt, args| {
    let path = abstract_ops::to_string(args.first().unwrap_or(&Value::Undefined));
    rusty_node_fs::read_file_string_sync(&path)
        .map(|s| Value::String(Rc::new(s)))
        .map_err(|e| RuntimeError::TypeError(format!("{}", e)))
});
```

The pattern is mechanical: each `Function::new(ctx, |args| { ... })` rquickjs callback becomes `register_global_fn(rt, name, |_rt, args| { ... })` with explicit `to_string` / `to_number` coercion at argument boundaries.

### 3. JS-side polyfill retirement

The pre-Ω.4 architecture has a substantial JS-side polyfill layer — `__keepAlive`, `__tickKeepAlive`, `__cjsBridge`, `__reactor`, etc. — that lives as inline JS strings evaluated at runtime startup. Post-Ω.4:

- `__keepAlive` + `__tickKeepAlive` → retired (engine's JobQueue replaces them)
- `__reactor.register/deregister/...` → retired (PollIo host hook replaces them)
- `__cjsBridge` → mostly retired (the bridge's source-rewriting survives at the parser-input layer; the bootRequire eval-then-stash mechanism migrates to engine-side module-evaluation with the engine's existing Module Record machinery)
- `__tz`, `__keepAliveUnref`, `__dnsPending`, etc. → retired or migrated as host-side state

Estimated JS-side polyfill LOC reduction: **-3,000 to -4,000 LOC**.

### 4. mio reactor migration

The pre-Ω.4 mio reactor exposes JS-facing primitives (`TCP.waitReadable`, `__reactor.register`, etc.) that the bytecode driver consults at idle. Post-Ω.4, the host owns the mio Poll directly:

```rust
// host/src/lib.rs (post-Ω.4)
fn main() {
    let mut rt = Runtime::new();
    rt.install_intrinsics();
    install_bun_host(&mut rt);  // wire path/os/crypto/fs/http/etc.
    let mio_poll = mio::Poll::new()?;
    rt.install_host_hook(HostHook::PollIo(Box::new(move |rt| {
        // consult mio.poll(timeout); translate ready events to macrotasks
        // ... see below
    })));
    rt.evaluate_module(&source, &url)?;
    rt.run_to_completion()?;
}
```

The PollIo hook does the work the old eval_esm_module loop did:
1. Compute next deadline from any pending timer set
2. mio.Poll::poll(events, Some(deadline))
3. For each ready event, look up the registered callback + enqueue a macrotask
4. Return true if any macrotask was enqueued; false if all I/O sources are quiescent

The five reactor submodules (reactor.rs / watchers.rs / spawn_async.rs / signals.rs / dns_async.rs) retire to a single ~300 LOC mio-integration in the host. The pre-Ω.4 nine sub-rounds' work product collapses to a thin registration layer.

Estimated reactor-related LOC reduction: **-1,500 LOC**.

### 5. CJS→ESM bridge

The pre-Ω.4 bridge has two responsibilities:
1. **Source rewriting** — generate ESM-shape wrapper around module.exports keys
2. **Runtime evaluation** — bootRequire creates a CJS-mode evaluation context, runs the source, stashes module.exports

Source rewriting moves to the parser layer (no change — it's input transformation).

Runtime evaluation re-implements against the new engine's module API. The engine's evaluate_module takes a module source; the bridge calls evaluate_module on the rewritten ESM source.

Named-export synthesis moves into HostFinalizeModuleNamespace (already specced; the engine fires the hook between Link and Evaluate; the host's hook walks module.exports as the source of names).

### 6. ResolveMessage error shape

Pre-Ω.4: a JS-side class injected via wire_globals + thrown via ctx.eval-then-Err(Exception). Post-Ω.4: a Rust-side `ResolveMessage` Value (an Error-kind object) constructed in the host's module-resolver hook and thrown via `Err(RuntimeError::Thrown(value))`. The parity probe's `e.constructor.name === "ResolveMessage"` check works the same.

## IV. Migration phases

The migration ships in 5-7 rounds:

### Ω.4.a — substrate-introduction (this design + AUDIT.md)
No code yet. Documents the migration plan + the LOC-delta predictions for the falsifier in Doc 714 §VI Consequence 5.

### Ω.4.b — minimal host-v2 with core intrinsics
- New crate: `host-v2/` parallel to `host/` (keep both during migration)
- Cargo bin: `rusty-bun-host-v2`
- Wires rusty-js-runtime + Math + JSON + console + Promise (free via engine's install_intrinsics)
- Adds path / os / crypto-random / process minimal surface (~600 LOC)
- Runs a small subset (~10) of the parity-119 corpus
- Falsifier: tests pass against the new engine

### Ω.4.c — fs + http + node:* breadth
- Migrate the load-bearing node:* surface: fs (sync subset), path, os, http (data layer), util, events, stream
- ~3,000 LOC of register_* calls
- Cumulative parity: ~50% of corpus

### Ω.4.d — TLS + WebSocket + crypto.subtle
- Migrate the Tier-G pilots' wirings: TLS (composes on Π1.4 substrate), WebSocket, crypto.subtle (HMAC + AES + EC)
- ~2,000 LOC
- Cumulative parity: ~75% of corpus

### Ω.4.e — mio reactor integration
- Replace the JS-side __reactor / __keepAlive with a host-side mio Poll wired through HostHook::PollIo
- Migrate the nine reactor sub-rounds' work product into the host-side mio registration
- Cumulative parity: ~85% (matches pre-Ω.4 baseline)
- **Critical falsifier:** measure host LOC at this point vs pre-Ω.4. Per Doc 714 §VI Consequence 5, host LOC should be substantially smaller. If not, the cut-rung diagnosis was wrong.

### Ω.4.f — CJS↔ESM bridge + Tuple A/B host hooks
- Migrate the bridge's source-rewriting layer (no change at parser input)
- Wire HostFinalizeModuleNamespace with the Bun-parity behavior (default = namespace for Tuple A; named-from-default-properties for Tuple B; reserved-key alias for the parser-already-retired Tuple C)
- Cumulative parity: 88.2%+ (recovers + extends the 2026-05-13 baseline)

### Ω.4.g — final cleanup + parity remeasurement
- Retire pre-Ω host/ crate
- Final parity measurement against the new engine
- Report LOC delta vs pre-Ω.4 baseline
- Headline KPI update

## V. LOC-delta prediction (the falsifier)

Per Doc 714 §VI Consequence 5:

| Component | Pre-Ω.4 LOC | Post-Ω.4 LOC | Δ |
|---|---:|---:|---:|
| Module loading | 600 | 300 | -300 |
| JS-side polyfills | 12,000 | 8,000 | -4,000 |
| mio reactor + JS surface | 2,000 | 300 | -1,700 |
| CJS bridge | 400 | 250 | -150 |
| HostFinalizeModuleNamespace polyfill | 200 | 80 | -120 |
| Resolution error shape | 50 | 30 | -20 |
| Parity-tool integration | 150 | 150 | 0 |
| Intrinsic wirings (rest) | 6,000 | 5,500 | -500 |
| **Total** | **21,400** | **14,610** | **-6,790** |

Predicted host LOC reduction: **~32%** (consistent with the Consequence 5 prediction of 30-50%).

The reduction comes from three sources:
1. The engine owns more (microtask queue, intrinsics, Promise + Math + JSON / console) — work the host previously polyfilled retires
2. JS-side glue layer (__keepAlive, __reactor, __cjsBridge) retires entirely
3. The nine reactor sub-rounds collapse to a single ~300 LOC mio integration

## VI. Parity falsifier per Doc 714 §VI Consequence 5

The Consequence 5 amendment locked a specific falsifier: **post-Ω.3.f + Ω.4, the host LOC delta required per parity-percentage point should be smaller than the pre-recognition trajectory.**

Concrete test: at Ω.4.e (matches pre-Ω parity), measure host LOC. If the LOC is roughly equal to pre-Ω.4 ~21,400, the architectural shift didn't deliver — the cut-rung diagnosis (E5 host-defined behavior) was operationally wrong even though it was structurally correct per Doc 717. If LOC is ~14,500 (the prediction), the shift delivered.

Per Doc 715 P1's heavy-tail prediction at the consumer-substrate DAG: the event-loop node has very high in-degree. Pulling it inside the engine boundary should flip the architectural ratio. The Ω.4.e measurement IS the empirical test of that prediction.

## VII. Out of scope for Ω.4

- **GC migration to ObjectId** (deferred from Ω.3.e.d; per the substrate-amortization split, the Value::Object → ObjectId migration is its own focused work and isn't required for Bun parity)
- **Closure upvalue binding** (deferred from Ω.3.c.d in the bytecode compiler)
- **Generator / async-function suspension** (out of v1 engine scope)
- **Worker threads / Atomics / SharedArrayBuffer** (out of v1 engine scope)
- **WebAssembly** (permanent out-of-scope)

## VIII. Composition with Ω.5 (parity re-baseline)

After Ω.4.g, the engagement re-runs the parity-measurement tool against rusty-bun-host-v2. Predicted result per Doc 717 P3 + Consequence 5 falsifier:

- Tuple A retires (7 packages: yup, io-ts, superstruct, neverthrow, jsonc-parser, fp-ts, yargs cascade) — closed by HostFinalizeModuleNamespace
- Tuple B retires (3 packages: dayjs, date-fns, node-fetch) — closed by same hook
- Tuple C already retired by the parser
- D-class items (got, enquirer) remain as independent per-package investigations

**Predicted post-migration baseline: 117/119 ≈ 98.3%** (matching the P3 prediction from 2026-05-13 night).

Ω.5 ratifies the measurement + locks the new headline KPI.

## IX. Sub-round roadmap

- **Ω.4.a** (this) — design + AUDIT.md
- **Ω.4.b** — host-v2 skeleton + Math/JSON/console/Promise free + path/os/process minimal (~5% parity)
- **Ω.4.c** — fs + http + node:* breadth (~50% parity)
- **Ω.4.d** — TLS + WebSocket + crypto.subtle (~75% parity)
- **Ω.4.e** — mio reactor integration + LOC measurement (~85% parity; falsifier test)
- **Ω.4.f** — CJS↔ESM bridge + Tuple A/B host hooks (~98% parity per P3)
- **Ω.4.g** — final cleanup + parity remeasurement; Ω.5 lock
