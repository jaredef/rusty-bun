# rusty-js-runtime — coverage audit

**Third sub-pilot of Tier-Ω.3** (after rusty-js-parser and rusty-js-bytecode). Per the [Ω.3 engine-selection decision artifact](../../host/tools/omega-3-engine-selection.md) §III and [the design spec](../../specs/rusty-js-runtime-design.md).

## Engagement role

The runtime is the execution layer. Reads `rusty_js_bytecode::CompiledModule`, executes against a Value representation rooted in ECMA-262 §6, manages call stack + scope chain, implements the spec's abstract operations, and exposes the host-hook API where Tier-Ω.4's parity-blocking tuples (Doc 717 A and B) close.

QuickJS-architectural reference per the keeper directive (2026-05-13 19:53Z) and the Ω.3 decision artifact: single-threaded interpreter, no JIT, conservative mark-sweep GC (migrated in Ω.3.e), embedding API directly in Rust.

## Tier-Ω.4 cross-reference

The runtime is where the engine-side cut closure for the parity residual attaches:

| Tuple | Spec § | Cut rung | Runtime contact point |
|---|---|---|---|
| A | §16.2.1.10 [[OwnPropertyKeys]] | E5 host-defined | HostFinalizeModuleNamespace hook |
| B | §16.2.3.4 default export binding | E5 host-defined | Same hook, applied after default evaluates |
| C | §16.2.2 grammar | E1 (retired) | Inherited from rusty-js-parser |

Per design spec §X. The runtime's load-bearing design decision is to expose namespace augmentation as a named host hook called *between Link and Evaluate*. That positioning lets the host (rusty-bun-host or any other embedder) install Bun's E3 spec-extension behavior without modifying the engine internals.

## Pilot scope (v1)

Seven composed surfaces in a single crate (large; subdivided into sub-rounds):

### Value representation (round Ω.3.d.b)
- `Value` enum per spec §6.1 (Undefined, Null, Boolean, Number, BigInt, String, Symbol, Object)
- `Object` struct: prototype + extensible + property map + internal kind (Ordinary, Array, Function, Closure, ModuleNamespace, Arguments, Error, ...)
- `PropertyDescriptor` per §6.1.7.1
- `JsString` as Vec<u16> per §6.1.4
- `JsSymbol` with description + unique id

### Frame + dispatch loop (round Ω.3.d.b)
- `Frame` struct: bytecode + constants + locals + args + upvalues + operand_stack + pc + try_stack + this
- `run_frame` dispatch loop on Op opcode discriminator
- Inline operand decode (u8, u16, i32, u32)

### Per-opcode handlers (rounds Ω.3.d.b – Ω.3.d.e)
- Round b: stack ops + integer-fast-path + arithmetic + comparison + local variables
- Round c: control flow + short-circuit + conditional + try-stack mechanics
- Round d: function call + closure materialization + property access + this binding
- Round e: object/array literals + iterator protocol + try-catch dispatch

### Abstract operations (rounds Ω.3.d.b – Ω.3.d.f)
Per spec §7. Each opcode handler invokes one or more abstract ops:
- ToBoolean, ToNumber, ToString, ToObject, ToPropertyKey
- IsCallable, IsConstructor, IsLooselyEqual, IsStrictlyEqual, SameValue
- GetV, Set, CreateDataProperty
- OrdinaryGet, OrdinarySet, OrdinaryDefineOwnProperty

### Built-in intrinsics (round Ω.3.d.f)
Per spec §20–§29. v1 minimum surface for the parity-119 corpus to load:
- %Object%, %Array%, %Function%, %String%, %Number%, %Boolean%, %Symbol%, %BigInt%
- %Error% + subclasses (TypeError/RangeError/SyntaxError/ReferenceError)
- %RegExp%, %Math%, %JSON%
- %Promise% minimum surface

### Module Record + linking (round Ω.3.d.g)
- `ModuleRecord` per §16.2.1.6 — bytecode + namespace + import/export entries
- Linking phases: Unlinked → Linking → Linked → Evaluating → Evaluated
- HostFinalizeModuleNamespace called between Link and Evaluate per Doc 717

### Host integration API (round Ω.3.d.g)
- `Runtime::new` / `install_intrinsics` / `install_host_hook` / `evaluate_module` / `call` / `get_global`
- `HostHookKind`: FinalizeModuleNamespace, SynthesizeNamedExportsFromDefault, ResolveModule, FetchModuleSource

## Out of scope for v1 (per design spec §IX)

- Generator / async function suspension
- Tail-call optimization
- Proxy / Reflect
- WeakMap / WeakSet / WeakRef / FinalizationRegistry (needs GC)
- WebAssembly (permanent)
- Eval / new Function dynamic compilation
- Atomics / SharedArrayBuffer
- Intl

## Test corpus

Three layers:

1. **Per-opcode behavior tests** — for each opcode, compose a small bytecode program and assert it produces the expected Value. Verifies the runtime executes the bytecode correctly.

2. **End-to-end source tests** — `parse + compile + run` for source-level test cases. Asserts that source-level behavior matches expected results.

3. **Spec-clause golden tests** — for each abstract operation in spec §7, a test that exercises the operation through the bytecode dispatch path and asserts spec-conformant behavior.

## First-round scope (this commit — Ω.3.d.a)

Substrate-introduction round only:
- specs/rusty-js-runtime-design.md
- pilots/rusty-js-runtime/AUDIT.md (this file)

No Cargo crate yet. Next round (Ω.3.d.b) creates pilots/rusty-js-runtime/derived/ + the Value enum + the dispatch loop skeleton + first 20 opcode handlers.

## Estimated pilot size

Per design spec §XII: ~7,100 LOC over 5-7 sub-rounds. Largest pilot in the engagement.

## Composition with downstream pilots

- **rusty-js-gc** (Ω.3.e): when the v2 mark-sweep heap migrates, the runtime's heap-allocating operations route through the GC. v1 uses Rc<RefCell<Object>>; the migration is mechanical.
- **rusty-bun-host** (Ω.4): the engagement's existing host wires its intrinsics + NodeResolver + FsLoader + ResolveMessage class + all the wire_* polyfills into the runtime's API.
