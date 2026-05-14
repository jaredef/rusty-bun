# rusty-js-gc — Design Spec

[surface] Managed heap + mark-sweep garbage collector
[reference] ECMA-262 §9.10 (Cleanup Finalization), Wilson "Uniprocessor Garbage Collection Techniques" (1992) for tri-color terminology; QuickJS-NG's mark-sweep allocator for architectural reference
[engagement role] The managed-heap layer beneath rusty-js-runtime. Tier-Ω.3.e per the engine-selection decision artifact. Replaces v1's Rc<RefCell<Object>> with handle-based heap + mark-sweep GC.

## I. Design rationale

The v1 runtime uses `Rc<RefCell<Object>>` for heap-allocated values. Cycles leak. Real consumer code creates cycles routinely (parent ↔ child references, doubly-linked lists, event emitters with closures over self). For the engagement's parity-119 corpus the leak is benign — modules are short-lived and the cycle footprint per module is bounded — but a long-running rusty-bun-host (after Ω.4 migration) needs a real collector.

Three constraints set the design:

1. **Conservative, stop-the-world mark-sweep.** The simplest correct collector. No tricolor incremental, no compaction, no generational. Pauses are O(live objects) per cycle; the runtime briefly halts during collection. Acceptable for v1; matches QuickJS's architectural posture.

2. **Handle-based references.** Heap-allocated values reference each other via ObjectId (u32 index into a heap-side slot table), not Rust pointers. This decouples object identity from memory address and lets the collector compact or reorder slots without invalidating handles. (v1 does not compact; the decoupling is for v2.)

3. **Roots-from-Runtime-and-frames.** The mark phase walks roots: Runtime.globals, the active call-stack frames (operand stacks, locals, this), the host_hooks closures' captured state. Reachability traces through each object's `proto` + `properties` + InternalKind-specific fields.

## II. Heap structure

```rust
pub struct Heap {
    slots: Vec<Slot>,
    free_list: Vec<u32>,
    /// Allocation count since last collection. Triggers collection when
    /// exceeds threshold.
    alloc_count: usize,
    /// Threshold for next collection. Adaptive: starts at 1024, doubles
    /// after each cycle until we hit a high-water-mark or live set
    /// stabilizes.
    threshold: usize,
}

pub enum Slot {
    Object(Object),
    Free,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(u32);
```

Object allocation: pop from free_list if non-empty; else push to slots; return ObjectId.

Object access: `heap.get(id) -> &Object`; `heap.get_mut(id) -> &mut Object`.

## III. Mark phase

Color-bit per object (stored alongside Slot, or in a parallel BitVec for cache friendliness). Three states:
- WHITE: candidate for collection
- GRAY: reached, but its references not yet traced
- BLACK: reached and fully traced

The mark phase:
1. Reset all colors to WHITE
2. From each root, mark GRAY
3. While there are GRAY objects:
   - Pick a GRAY object O
   - Mark O BLACK
   - For each reference R in O (proto, property values, InternalKind fields):
     - If R points to WHITE, mark GRAY
4. After the worklist drains, all reachable objects are BLACK and unreachable are WHITE

## IV. Sweep phase

Walk slots. Each WHITE slot is freed (transition Slot::Object to Slot::Free, push index to free_list). Each BLACK slot is reset to WHITE for the next cycle.

## V. Roots enumeration

The collector needs every "edge" into the heap from outside the heap. v1 roots:

- **Runtime.globals** — a HashMap<String, Value>; each Value::Object holds an ObjectId
- **Active call stack** — Frame's operand_stack, locals, args, upvalues, this_binding, try_stack
- **Constants pool** — Function-constant entries hold their own FunctionProto (with its own constants); the constants pool transitively reaches more bytecode-level state. v1 treats FunctionProto as a Rust-owned, never-collected value.
- **HostHooks closures** — Rust closures captured by host_hooks may capture ObjectIds. The host_hooks API will require a "root callback" that the host implements to declare its retained handles. v1 assumes host hooks don't retain ObjectIds across calls; documented as a v1 limitation.

## VI. Allocation API

```rust
impl Heap {
    pub fn alloc_object(&mut self) -> ObjectId;
    pub fn alloc_array(&mut self) -> ObjectId;
    pub fn alloc_string(&mut self, s: String) -> StringId;  // strings interned separately
    pub fn maybe_collect(&mut self, runtime: &Runtime);
    pub fn collect(&mut self, runtime: &Runtime);
}
```

The runtime's hot loop calls `heap.maybe_collect(self)` at safe points (loop back-edges, before allocations, at function-call boundaries). Collection runs only when alloc_count exceeds threshold.

## VII. Migration of Value::Object

```rust
// v1 (current):
pub enum Value {
    ...
    Object(Rc<RefCell<Object>>),
}

// v2 (Ω.3.e):
pub enum Value {
    ...
    Object(ObjectId),
}
```

Every code path that accesses an Object goes through `heap.get(id)` / `heap.get_mut(id)`. The change is invasive but mechanical. Per the substrate-amortization discipline, the migration ships in 3-4 sub-rounds:

- **Ω.3.e.a** — design + AUDIT.md (this commit)
- **Ω.3.e.b** — standalone rusty-js-gc pilot: Heap + Slot + ObjectId + alloc + mark + sweep + threshold-driven collection. Tested in isolation (Rust-only tests, no JS).
- **Ω.3.e.c** — Migrate rusty-js-runtime's Value::Object from Rc<RefCell<Object>> to ObjectId. The migration touches ~150 call sites; mechanical.
- **Ω.3.e.d** — Wire the collector into the runtime's hot loop. Verify all 313 engine tests pass on the new heap.

## VIII. Out of scope for v1

- **Incremental marking.** v2 work; needed when pause times become noticeable at consumer scale.
- **Generational hypothesis.** Young/old generations + write barriers. v2 work.
- **Compaction.** ObjectId stability through compaction requires updating every reference on compact; v2 work after we have a need for it.
- **Concurrent / parallel mark.** v3 territory.
- **WeakRef / FinalizationRegistry.** Spec'd at §26.1; needs finalizer scheduling integrated with sweep. Documented as basin boundary in the engagement (Doc 715 §X.b).
- **Bigint arena.** BigInts have no internal references but are heap-allocated for arbitrary precision. v1 lets Rust's Drop manage them.

## IX. Doc 717 cross-reference

The GC layer is neutral on the parity-blocking tuples:
- **Tuple A / B** close at runtime's HostFinalizeModuleNamespace hook (3.d.f); the host hook may allocate via the heap but doesn't need GC-specific support
- **Tuple C** retired at the parser layer

The GC contributes load-bearing infrastructure for **post-Ω.4 production deployment**, where rusty-bun-host's long-running module evaluation would otherwise leak. It is not on the critical path to Bun parity per the parity-119 baseline; it's on the critical path to a non-leaky engine.

## X. Composition with rusty-js-runtime

The runtime's `Value::Object(ObjectId)` reads + writes go through a Heap handle. Frames retain reachability of their operand-stack contents; the collector walks the live frame chain. The Runtime owns the Heap; the runtime's dispatch loop checks `heap.maybe_collect(self)` at safe points.

## XI. Estimated pilot size

- Heap + Slot + ObjectId types: ~200 LOC
- Allocation API + free-list management: ~150 LOC
- Mark + sweep algorithm: ~250 LOC
- Threshold-driven collection trigger: ~100 LOC
- Runtime migration (touches every Value::Object access): ~400 LOC delta in interp.rs + ~150 LOC delta in intrinsics.rs + ~50 LOC in module.rs
- Tests: ~500 LOC (standalone heap tests + post-migration regression suite)

Total ~1,800 LOC across 3-4 sub-rounds. Smaller than the parser or runtime pilots; comparable to bytecode.

## XII. Falsifier

The migration's correctness is verified by **all 313 engine tests continuing to pass post-migration**. Cycle-collection correctness is verified by a dedicated test that constructs a cyclic object graph + drops the only external reference + invokes collect + asserts the cycle's slots are now Free.

Per Doc 715 P2: a small-N test corpus verifying memory invariants under controlled allocations confirms the substrate. The engagement's existing test suite is the regression-detection mechanism for the migration.
