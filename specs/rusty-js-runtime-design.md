# rusty-js-runtime — Design Spec

[surface] Bytecode interpreter + Value representation + heap + intrinsics
[reference] ECMA-262 §6 (ECMAScript Data Types and Values), §7 (Abstract Operations), §10 (Ordinary and Exotic Objects Behaviours), §16 (ECMAScript Language: Scripts and Modules); QuickJS for VM architecture
[engagement role] The execution layer for compiled bytecode emitted by rusty-js-bytecode. Tier-Ω.3.d per the engine-selection decision artifact (host/tools/omega-3-engine-selection.md §III).

The runtime is where ECMA-262's abstract operations realize as concrete code. Each abstract operation in the spec (ToNumber, ToString, GetValue, Call, OrdinaryGet, etc.) corresponds to a runtime function. The interpreter dispatches on bytecode and consults these abstract operations at each step.

Per Doc 717: the runtime is where Bun-parity Tuple A and Tuple B attach. Module Namespace augmentation (Tuple A) happens at HostFinalizeModuleNamespace, called between Link and Evaluate. Named-export synthesis from default's own properties (Tuple B) happens at the same host hook. The runtime exposes these as named callbacks the host installs.

## I. Value representation

Per ECMA-262 §6.1 ECMAScript Language Types: Undefined, Null, Boolean, String, Symbol, Number, BigInt, Object.

```rust
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    BigInt(Rc<BigInt>),  // arbitrary precision; v1 stores as decimal string
    String(Rc<JsString>),
    Symbol(Rc<JsSymbol>),
    Object(ObjectRef),
}
```

`ObjectRef` is a handle into the heap. v1 uses `Rc<RefCell<Object>>` for simplicity; GC migration to a managed heap is Ω.3.e.

### Number representation

f64 with IEEE 754 semantics. The integer-fast-path the compiler emits (PushI32) materializes as Number(f64) — no separate integer type. ECMA-262 §6.1.6.1 defines the Number Type as IEEE 754 doubles; the spec's "Number" maps directly.

NaN canonicalization: incoming NaNs (from arithmetic or ToNumber) are coerced to the canonical quiet NaN bit-pattern. The IEEE comparisons (Number-Less-Than per §7.1.13) handle NaN comparisons correctly.

### BigInt representation

Wrapper around `num-bigint` crate or hand-rolled arbitrary-precision. v1 stores as the parser-produced digit-string and defers arithmetic to a Bigint crate.

### String representation

UTF-16-encoded internally to match ECMA-262 §6.1.4 String Type (sequences of UTF-16 code units). v1 stores as `Vec<u16>` to match the spec exactly; UTF-8 conversion happens at the FFI boundary. JS string lengths report UTF-16 code-unit count, not Unicode code-point count, per spec.

### Symbol representation

Per §6.1.5: each Symbol value has a [[Description]] String. v1 uses `JsSymbol { description: Option<JsString>, id: u32 }` where `id` is a process-unique identifier.

### Object representation

Per §6.1.7 and §10.1: an ordinary object has:
- `[[Prototype]]`: another Object or Null
- `[[Extensible]]`: Boolean
- Property storage (own keys → property descriptors)

```rust
pub struct Object {
    proto: Option<ObjectRef>,
    extensible: bool,
    properties: PropertyMap,
    internal_kind: InternalKind,  // Ordinary, Array, Function, ModuleNamespace, etc.
}

pub enum InternalKind {
    Ordinary,
    Array,
    Function(FunctionInternals),
    Closure(ClosureInternals),
    BoundFunction(BoundFunctionInternals),
    Error,
    ModuleNamespace(ModuleNamespaceInternals),
    Arguments,
    // ... plus exotic types
}
```

PropertyMap is a hash-table of key (String or Symbol) → PropertyDescriptor (value, writable, enumerable, configurable, optional getter/setter).

### Value vs Reference

Per §6.2.5 Reference Record: lvalue contexts (the LHS of assignment, the operand of `delete`, the operand of `typeof` for undeclared names) need a Reference, not a Value. The runtime's GetValue / PutValue / ToReference abstract operations handle the V↔R distinction.

For v1, the bytecode compiler emits explicit Load/Store opcodes that subsume the GetValue/PutValue distinction; the runtime doesn't need a first-class Reference value. Eval / with / dynamic-property-name lookups will need first-class References in a follow-on round.

## II. Heap + GC

### v1: reference-counted

Use `Rc<RefCell<Object>>` for heap objects. Simple, deterministic destruction. Cycles leak — that's accepted for v1 since the engagement's parity-119 corpus doesn't exercise cycle-heavy patterns at module load time.

### v2 (Ω.3.e): mark-sweep

Migrate to a managed heap with mark-sweep GC. Per design spec for rusty-js-gc (future): conservative root scan from the call stack + global object + module namespace registry; mark from roots; sweep unreachable. No incremental marking, no compaction — v1 GC is the simplest correct collector.

## III. Bytecode dispatch

Single-threaded interpreter. Reads bytecode from a `CompiledModule` (or `FunctionProto` for nested functions) in a linear scan, decoding each opcode and dispatching to a handler.

### Frame structure

```rust
pub struct Frame {
    bytecode: Rc<Vec<u8>>,
    constants: Rc<ConstantsPool>,
    locals: Vec<Value>,
    args: Vec<Value>,
    upvalues: Vec<UpvalueRef>,
    operand_stack: Vec<Value>,
    pc: usize,
    try_stack: Vec<TryFrame>,
    this_binding: Value,
}

pub struct TryFrame {
    catch_offset: usize,
    sp_at_entry: usize,
}
```

The runtime maintains a Vec<Frame> as the call stack. Each function call pushes a frame; Return pops it.

### Dispatch loop shape

```rust
fn run_frame(&mut self, frame: &mut Frame) -> Result<Value, RuntimeError> {
    loop {
        let op = decode(frame);
        match op {
            Op::PushI32 => { let v = decode_i32(frame); push(Value::Number(v as f64)); }
            Op::Add => { let r = pop(); let l = pop(); push(add(l, r)?); }
            Op::JumpIfFalse => { let v = pop(); let off = decode_i32(frame); if !to_boolean(v) { jump(frame, off); } }
            Op::Call => { let n = decode_u8(frame); call_n(n)?; }
            Op::Return => return Ok(pop()),
            // ... per opcode
        }
    }
}
```

Computed-goto-style dispatch (Rust doesn't have computed goto, but match in a hot loop with `#[inline(always)]` helpers is close enough). Tail-call-recursive frames avoid Rust stack overflow for deep call chains.

### Exception propagation

`throw` opcode raises a RuntimeError with the thrown value. The dispatch loop catches RuntimeError, walks the try_stack of the current frame, jumps to the catch_offset if present (pushing the thrown value as the stack top), else pops the frame and re-raises to the caller.

## IV. Abstract operations (per §7)

Each abstract op is a Rust function on the runtime. The opcode handlers invoke them:

- `to_primitive(value, hint)` per §7.1.1
- `to_boolean(value)` per §7.1.2
- `to_number(value)` per §7.1.4 (delegates to ToNumeric for BigInt-mixed)
- `to_integer(value)` per §7.1.5
- `to_string(value)` per §7.1.17
- `to_object(value)` per §7.1.18
- `to_property_key(value)` per §7.1.19
- `is_callable(value)` per §7.2.3
- `is_constructor(value)` per §7.2.4
- `same_value(x, y)` per §7.2.11
- `same_value_zero(x, y)` per §7.2.12
- `is_loosely_equal(x, y)` per §7.2.13
- `is_strictly_equal(x, y)` per §7.2.15
- `get_v(value, key)` per §7.3.2
- `set(o, key, value, throw_)` per §7.3.4
- `create_data_property(o, key, value)` per §7.3.5
- `ordinary_get(o, key, receiver)` per §10.1.8.1
- `ordinary_set(o, key, value, receiver)` per §10.1.9.1
- `ordinary_define_own_property(o, key, descriptor)` per §10.1.6.1

Most opcode handlers correspond 1:1 to a small set of abstract ops. The dispatch loop stays narrow; the abstract ops do the work.

## V. Built-in intrinsics (v1 minimum)

Per §20–§29. v1 ships the minimum surface for the parity-119 corpus to load:

- **%Object%** with prototype methods: hasOwnProperty, isPrototypeOf, propertyIsEnumerable, toString, valueOf, toLocaleString
- **%Array%** with prototype methods: push, pop, shift, unshift, slice, splice, concat, indexOf, lastIndexOf, includes, join, reverse, sort, map, filter, reduce, reduceRight, forEach, find, findIndex, some, every, flat, flatMap, fill, copyWithin, at, entries, keys, values
- **%Function%** with prototype methods: call, apply, bind, toString
- **%String%** with prototype methods: charAt, charCodeAt, codePointAt, concat, includes, endsWith, startsWith, indexOf, lastIndexOf, match, matchAll, normalize, padEnd, padStart, repeat, replace, replaceAll, search, slice, split, substring, toLowerCase, toUpperCase, trim, trimStart, trimEnd
- **%Number%** with prototype methods + statics (parseInt, parseFloat, isFinite, isInteger, isNaN, isSafeInteger)
- **%Boolean%**, **%Symbol%** (with .iterator + .asyncIterator), **%BigInt%**
- **%Error%** + subclasses (TypeError, RangeError, SyntaxError, ReferenceError)
- **%RegExp%** with a wrapper around the host's regex engine (v1 may defer regex.exec to host)
- **%Math%** with all the §21.3 functions
- **%JSON%** with parse + stringify
- **%Promise%** — substantial; v1 wires a minimal Promise.{resolve,reject,all,then,catch} synchronously where possible
- **globalThis** with the above attached

The intrinsic surface is large but mechanical. Each intrinsic is a small Rust function registered into the global object at runtime startup.

## VI. Module Record + linking

Per §16.2.1 (Module Records) and §16.2.1.7 (Source Text Module Records):

```rust
pub struct ModuleRecord {
    pub status: ModuleStatus,
    pub bytecode: Rc<CompiledModule>,
    pub namespace: Option<ObjectRef>,
    pub requested_modules: Vec<String>,
    pub import_entries: Vec<rusty_js_ast::ImportEntry>,
    pub export_entries_local: Vec<rusty_js_ast::ExportEntry>,
    pub export_entries_indirect: Vec<rusty_js_ast::ExportEntry>,
    pub export_entries_star: Vec<rusty_js_ast::ExportEntry>,
}

pub enum ModuleStatus { Unlinked, Linking, Linked, Evaluating, Evaluated, Failed }
```

Module linking phases:
1. **Parse** — rusty-js-parser produces the AST + ImportEntries/ExportEntries
2. **Link** — runtime walks the ImportEntries, resolves each request to another ModuleRecord, recursively links transitively
3. **HostFinalizeModuleNamespace** — the Doc 717 Tuple A/B hook. Called for each Module Record before evaluation. Default behavior is no-op (spec-conformant). The host can install a hook that adds synthetic bindings (default = namespace; named-from-default-props; etc).
4. **Evaluate** — the bytecode dispatch loop runs the module's top-level statements

## VII. Host integration

The runtime exposes a minimal API for the host (rusty-bun-host or any other embedder):

```rust
pub struct Runtime { ... }

impl Runtime {
    pub fn new() -> Self;
    pub fn install_intrinsics(&mut self);
    pub fn install_host_hook<F>(&mut self, kind: HostHookKind, hook: F);
    pub fn evaluate_module(&mut self, source: &str, url: &str) -> Result<Value, RuntimeError>;
    pub fn call(&mut self, callable: Value, this: Value, args: Vec<Value>) -> Result<Value, RuntimeError>;
    pub fn get_global(&self) -> ObjectRef;
}

pub enum HostHookKind {
    FinalizeModuleNamespace,        // Doc 717 Tuple A
    SynthesizeNamedExportsFromDefault,  // Doc 717 Tuple B
    ResolveModule,                   // bare-specifier → URL/path
    FetchModuleSource,               // URL → source bytes
}
```

The host wires its FsLoader, NodeResolver, console object, fs/path/etc into the runtime's intrinsics or global before calling evaluate_module. The runtime itself has no knowledge of the filesystem, the network, or any other Node/Bun surface.

## VIII. Compilation invariants (continued from bytecode spec)

The runtime enforces the spec's value-type invariants:
- All arithmetic ToNumber-coerces non-number operands
- All comparison Abstract Relational Comparison per §7.2.14
- String concatenation: + with string operand routes through ToString
- == per Abstract Equality (§7.2.13); === per Strict Equality (§7.2.15)
- Property access ToPropertyKey-coerces non-string-non-symbol keys
- Function invocation ToCallable-checks the callee; non-callable throws TypeError

The bytecode compiler trusts the runtime to enforce these; opcodes don't carry redundant guards.

## IX. Out of scope for v1

- **Generator / async function suspension** — requires save/restore of the operand stack and PC across yield boundaries; major engineering work; v2
- **Tail-call optimization** — spec-permitted but optional; not v1
- **Proxy / Reflect** — Proxy involves trap-redirection at every internal-method invocation; ~1500 LOC of additional handler code; v2
- **WeakMap / WeakSet / WeakRef / FinalizationRegistry** — needs the managed-heap GC (Ω.3.e) for finalizer scheduling
- **WebAssembly** — permanent out-of-scope per engagement decision
- **Eval / new Function** — requires the runtime to embed the parser+compiler dynamically; doable but defer to a follow-on
- **Indirect-eval-of-ESM (eval-ESM)** — closed by rusty-bun-host's data: URL path; not runtime-resident
- **Atomics / SharedArrayBuffer** — multi-threaded; out of v1 scope
- **Internationalization (Intl)** — host-layer concern, not runtime-resident

## X. Doc 717 cross-reference

The runtime is the layer at which Tier-Ω.4's parity-blocking tuples close.

- **Tuple A (Module Namespace augmentation at E2 spec-relaxation):** runtime's HostFinalizeModuleNamespace hook is the closure point. Default behavior: spec-conformant (no augmentation). Host installs a non-default hook to match Bun.
- **Tuple B (named-export synthesis at E5 spec-extension):** same hook, called after default-export evaluates. Host hook enumerates default's own properties + adds them as named exports on the Module Namespace.
- **Tuple C (parser grammar):** already retired by rusty-js-parser; runtime inherits the typed AST.

The cuts for A and B in the design itself sit at E5 (realm host-defined behavior) per Doc 717 §VII. This is the design's most important architectural decision: by exposing namespace augmentation as a named host hook called before evaluation, the runtime makes Bun's E3 spec-extension reachable from above without modifying the engine internals.

## XI. Composition with downstream pilots

- **rusty-js-gc** (Ω.3.e): when the v2 mark-sweep heap migrates, the runtime's heap-allocating operations route through the GC. v1 uses Rc<RefCell<Object>>; the migration is mechanical.
- **rusty-bun-host** (Ω.4): the engagement's existing host wires its intrinsics + NodeResolver + FsLoader + ResolveMessage class + all the wire_* polyfills into the runtime's API. The migration is per-pilot mechanical: each rusty-bun Rust function exposed through rquickjs's FFI re-exposes through rusty-js-runtime's call API.

## XII. Estimated pilot size

- Value enum + heap object representation: ~400 LOC
- Frame + dispatch loop: ~800 LOC
- Per-opcode handlers: ~600 LOC (60+ opcodes, average ~10 LOC each)
- Abstract operations: ~600 LOC
- Built-in intrinsics (v1 minimum): ~2,500 LOC
- Module Record + linking: ~400 LOC
- Host hooks + Runtime public API: ~300 LOC
- Tests: ~1,500 LOC

Total ~7,100 LOC. Largest pilot in the engagement, comparable to rusty-bun-host itself. Per the substrate-amortization discipline (seed §A8.13), the pilot ships in 5-7 sub-rounds:

- Ω.3.d.a: spec + AUDIT.md (this commit)
- Ω.3.d.b: Value + Object + Frame + dispatch loop skeleton (first ~20 opcodes: stack ops, arithmetic, comparison, local-slot variables)
- Ω.3.d.c: control flow + short-circuit + conditional dispatch
- Ω.3.d.d: function call frames + closures + this binding + property access
- Ω.3.d.e: object / array literals + iteration + try-catch dispatch
- Ω.3.d.f: built-in intrinsics (Object/Array/String/Number/Function/Math/JSON minimum)
- Ω.3.d.g: module record + linking + Tuple-A/B host hooks
