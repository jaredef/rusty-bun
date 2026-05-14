# rusty-js-gc — coverage audit

**Fourth sub-pilot of Tier-Ω.3**, after rusty-js-parser, rusty-js-bytecode, and rusty-js-runtime. Per the [Ω.3 engine-selection decision artifact](../../host/tools/omega-3-engine-selection.md) §III + the [design spec](../../specs/rusty-js-gc-design.md).

## Engagement role

Migrates the runtime's heap from `Rc<RefCell<Object>>` to a handle-based managed heap with mark-sweep garbage collection. The migration is mechanical but invasive — every Value::Object access on the v1 runtime routes through `heap.get(id)` / `heap.get_mut(id)` in v2.

Per design spec §IX: the GC contributes load-bearing infrastructure for **post-Ω.4 production deployment**, where rusty-bun-host's long-running module evaluation would otherwise leak. It is not on the critical path to Bun parity per the parity-119 baseline.

## Pilot scope (v1)

Five composed surfaces in a single crate:

### Heap representation (round Ω.3.e.b)
- `Heap` struct: slots: Vec<Slot>, free_list: Vec<u32>, alloc_count, threshold
- `Slot` enum: Object(Object), Free
- `ObjectId(u32)` handle type
- Color bits per slot (parallel Vec or inline in Slot)

### Allocation API (round Ω.3.e.b)
- `alloc_object` / `alloc_array` — push to slots or pop free_list, return ObjectId
- `get(id)` / `get_mut(id)` — borrow by index
- Threshold-driven `maybe_collect` triggered at safe points

### Mark phase (round Ω.3.e.b)
- Tri-color marking: WHITE → GRAY → BLACK
- Worklist-driven trace from roots
- Per-object reachability: proto + properties + InternalKind-specific fields (FunctionInternals, ClosureInternals, ModuleNamespace)

### Sweep phase (round Ω.3.e.b)
- Walk slots; WHITE → Free; BLACK → WHITE for next cycle
- Free-list maintenance

### Runtime migration (rounds Ω.3.e.c, Ω.3.e.d)
- Value::Object switches from Rc<RefCell<Object>> to ObjectId
- Frame retains reachability via operand_stack + locals + args + upvalues + this_binding + try_stack
- Runtime gains heap: Heap field
- ~150 call sites across interp.rs + intrinsics.rs + module.rs touch Object accessors; each is mechanical

## Roots enumeration (per design spec §V)

- Runtime.globals — HashMap<String, Value>
- Active call stack — Frame's operand_stack / locals / args / upvalues / this_binding / try_stack
- Constants pool — FunctionProto entries are Rust-owned (not GC'd)
- HostHooks closures — v1 assumes host hooks don't retain ObjectIds across calls

## Out of scope for v1 (per design spec §VIII)

- Incremental marking
- Generational hypothesis + write barriers
- Compaction (would invalidate ObjectIds)
- Concurrent / parallel mark
- WeakRef / FinalizationRegistry (Doc 715 §X.b basin boundary)
- BigInt arena (Rust Drop suffices)

## Test corpus

Three layers:

1. **Standalone heap tests** — Rust-only tests of Heap / Slot / ObjectId. Allocate, mutate, mark from synthetic roots, sweep, verify free-list state.

2. **Cycle-collection golden test** — construct A → B → A cycle, drop external root, collect, assert both slots are Free.

3. **Runtime regression suite** — all 313 engine tests continue to pass after the migration. The migration's correctness is verified by the existing test suite.

## First-round scope (this commit — Ω.3.e.a)

Substrate-introduction round only:
- specs/rusty-js-gc-design.md
- pilots/rusty-js-gc/AUDIT.md (this file)

No Cargo crate yet. Next round (Ω.3.e.b) creates pilots/rusty-js-gc/derived/ + the standalone Heap + mark-sweep implementation.

## Estimated pilot size

Per design spec §XI: ~1,800 LOC across 3-4 sub-rounds:
- Ω.3.e.a: substrate (this commit)
- Ω.3.e.b: Heap + Slot + ObjectId + alloc + mark + sweep, standalone
- Ω.3.e.c: migrate Value::Object from Rc<RefCell<Object>> to ObjectId
- Ω.3.e.d: wire collector into runtime hot loop; verify all 313 tests pass

## Doc 717 cross-reference

The GC layer is neutral on the parity-blocking tuples:
- Tuple A / B close at runtime's HostFinalizeModuleNamespace hook (already landed in 3.d.f)
- Tuple C retired at the parser layer

The GC is on the critical path to a **non-leaky long-running engine**, not on the critical path to Bun parity. The parity-119 baseline doesn't depend on it.

## Composition with downstream tiers

- **Ω.3.d (runtime)**: migration target. The runtime's Value::Object access routes through the Heap.
- **Ω.4 (host migration)**: rusty-bun-host's long-running deployments need the GC to avoid memory growth across many module loads.
