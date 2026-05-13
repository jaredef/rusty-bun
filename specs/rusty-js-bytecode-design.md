# rusty-js-bytecode — Design Spec

[surface] Bytecode instruction set + compiler
[reference] QuickJS (Bellard) opcodes; ECMA-262 §6 (Algorithm Conventions) for value semantics; ECMA-262 §10 (Ordinary Objects + Functions) for closure form
[engagement role] The compilation target of rusty-js-parser's AST and the input of rusty-js-runtime's interpreter. Tier-Ω.3.c per the engine-selection decision artifact (host/tools/omega-3-engine-selection.md §III).

This is a design document, not a spec extraction. No external spec governs JavaScript bytecode — every engine designs its own. QuickJS serves as the architectural reference per the keeper directive; this document records the design choices made for rusty-js-bytecode.

## I. Design rationale

The rusty-js-bytecode pilot occupies the cut-rung between AST and runtime. Three constraints set the design:

1. **Single-pass compilation.** Walk the AST once and emit bytecode. No optimization passes in v1; the engine's correctness-first stance matches QuickJS's architectural commitment. Optimization is successor-engagement scope.

2. **Stack-based dispatch.** A stack machine. Values are pushed and popped; opcodes consume their operands from the stack top and push their result back. Matches the spec's algorithm-conventions language directly (`Let v be ? ToNumber(x)` — pop, transform, push). Register-based dispatch (Lua-style) would be faster but harder to derive from spec text.

3. **Forward-referencing patches.** Jump targets are resolved by emitting placeholder offsets and patching them when the target offset becomes known. Standard for single-pass forward jumps (if-else, while loop exits).

## II. Instruction set (v1 subset)

Instructions are u8 opcode + variable-length operand bytes. The interpreter dispatches on the opcode and decodes operands inline. Operands: u8, u16, u32 (little-endian), or constants-pool index (u16).

### Stack ops
- `PUSH_NULL` — push the null value
- `PUSH_UNDEF` — push undefined
- `PUSH_TRUE` / `PUSH_FALSE` — push the boolean literal
- `PUSH_I32 <i32>` — push a small integer literal
- `PUSH_CONST <u16>` — push the constants-pool entry (number, string, bigint, regex, function)
- `POP` — discard top
- `DUP` — duplicate top
- `SWAP` — swap top two values

### Variable / scope
- `LOAD_LOCAL <u16>` / `STORE_LOCAL <u16>` — local slot by index
- `LOAD_ARG <u16>` / `STORE_ARG <u16>` — function argument by index
- `LOAD_GLOBAL <u16>` / `STORE_GLOBAL <u16>` — global name (string constant index)
- `LOAD_UPVALUE <u16>` / `STORE_UPVALUE <u16>` — closure-captured upvalue
- `DEFINE_LOCAL <u16>` — initialize a local slot (let/const semantics)

### Arithmetic
- `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `POW` — binary; top is RHS, below is LHS
- `NEG`, `POS` — unary
- `INC`, `DEC` — increment/decrement (the runtime handles ToNumber coercion)

### Comparison / equality
- `LT`, `GT`, `LE`, `GE` — relational
- `EQ`, `NE`, `STRICT_EQ`, `STRICT_NE` — equality (spec §7.2.13, §7.2.15)
- `IN`, `INSTANCEOF` — relational ops with object semantics

### Bitwise / shift
- `BIT_AND`, `BIT_OR`, `BIT_XOR`, `BIT_NOT`
- `SHL`, `SHR`, `USHR`

### Logical
- `NOT` — unary `!`
- Logical AND/OR/?? compile to conditional jumps (no dedicated opcode)

### Control flow
- `JUMP <i32>` — unconditional relative jump
- `JUMP_IF_TRUE <i32>` / `JUMP_IF_FALSE <i32>` — conditional, pops the condition
- `JUMP_IF_TRUE_KEEP <i32>` / `JUMP_IF_FALSE_KEEP <i32>` — for `||` / `&&` short-circuit (peeks without pop on the not-taken branch)
- `JUMP_IF_NULLISH <i32>` — for `??`

### Calls / returns
- `CALL <u8>` — call with N arguments (N is operand). Pops N args + function, pushes result.
- `NEW <u8>` — `new f(...args)` — pops args + constructor, pushes new instance.
- `RETURN` — return from function with stack-top value
- `RETURN_UNDEF` — return without value

### Member access
- `GET_PROP <u16>` — get property by static name (string constant index)
- `SET_PROP <u16>` — set property by static name
- `GET_INDEX` — `obj[key]` — pops key + obj, pushes result
- `SET_INDEX` — `obj[key] = value` — pops value, key, obj

### Object / array construction
- `NEW_OBJECT` — push a new empty object
- `NEW_ARRAY <u16>` — push a new array of the given initial length
- `INIT_PROP <u16>` — set property on object below top using value at top (for object literals)
- `INIT_INDEX <u32>` — set indexed slot on array below top using value at top

### Unary / type
- `TYPEOF` — `typeof x` operator
- `VOID` — `void x` returns undefined after evaluating x
- `DELETE` — `delete x.prop` — pops property + obj, pushes boolean

### Function / closure
- `MAKE_CLOSURE <u16>` — wrap a constants-pool function entry in a closure capturing the current scope's upvalues, push the closure
- `MAKE_ARROW <u16>` — same as MAKE_CLOSURE but with lexical-`this` capture

### Exception handling
- `THROW` — pop and throw
- `TRY_ENTER <u32>` — push a try-frame with a catch offset
- `TRY_EXIT` — pop the current try-frame (after the protected block completes)

### Iteration support
- `ITER_INIT` — get iterator from top
- `ITER_NEXT` — advance the iterator on top; push (value, done) tuple
- `ITER_CLOSE` — close the iterator on top

### Miscellaneous
- `NOP` — no-op (used as a patch target for jumps)
- `DEBUGGER` — debugger statement

## III. Constants pool

Per compilation unit (one function body or one module), a constants pool holds the literal values referenced by opcodes:

```rust
pub enum Constant {
    Number(f64),
    BigInt(String),
    String(String),
    Regex { body: String, flags: String },
    Function(FunctionProto),
}
```

`FunctionProto` is the compiled prototype of a nested function: its bytecode, constants pool, parameter count, local-slot table, and upvalue descriptors. When a `MAKE_CLOSURE <u16>` opcode runs, the runtime materializes a closure binding the function prototype's upvalue slots to the current frame's captures.

## IV. Compilation unit shape

```rust
pub struct CompiledModule {
    pub bytecode: Vec<u8>,
    pub constants: Vec<Constant>,
    pub local_slots: Vec<LocalDescriptor>,
    pub upvalue_descriptors: Vec<UpvalueDescriptor>,
    pub source_map: Vec<(usize, Span)>,  // bytecode offset -> source span
}

pub struct LocalDescriptor {
    pub name: String,
    pub kind: VariableKind,  // let / const / var
    pub depth: u32,           // lexical scope depth at declaration
}

pub struct UpvalueDescriptor {
    pub source: UpvalueSource,
    pub name: String,
}

pub enum UpvalueSource {
    Local(u16),     // parent function's local slot
    Upvalue(u16),   // parent function's own upvalue
}
```

## V. Compilation invariants

- **No use-before-declare for let/const.** Bytecode tracks the temporal-dead-zone: `LOAD_LOCAL` on an uninitialized slot is a runtime error per spec §13.3.7.
- **Each operand has a stack-effect descriptor.** Documented as inline comments per opcode. The verifier (not v1 — successor pilot) walks the bytecode tracking the simulated stack depth at each point and confirms it converges at every control-flow merge.
- **No GC barrier in the bytecode.** Mark-sweep GC operates between dispatch steps; the bytecode interpreter is GC-aware at allocation points but does not itself emit barriers.
- **Source-map for diagnostics.** Each opcode optionally maps back to its source span via a parallel table.

## VI. Out of scope for v1

- Generator suspension / resumption (yield) — bytecode-level suspension requires save/restore of the operand stack and PC; v2 work
- Async/await — same suspension machinery
- Optimizing opcodes (inline caches, hidden-class shape guards, JIT) — successor engagement
- Block-scoped exception propagation across yield boundaries
- Sourcemap output to consuming tools (just stored internally for diagnostics)

## VII. Composition with downstream pilots

- **rusty-js-runtime** consumes `CompiledModule`. The interpreter dispatch loop reads bytes from `bytecode`, decodes operands, executes against the Value representation, and manages the call stack + scope chain.
- **rusty-js-gc** services rusty-js-runtime's allocations; the bytecode itself doesn't allocate (it's a `Vec<u8>` + `Vec<Constant>`).

## VIII. Tuple cross-reference (Doc 717)

Tier-Ω.4's parity-blocking tuples (per host/tools/p3-classification.md) attach in the engine layer at the following bytecode contact points:

- **Tuple A** (Module Namespace augmentation at E2): not bytecode-resident. Attaches to rusty-js-runtime's HostFinalizeModuleNamespace hook between Link and Evaluate phases.
- **Tuple B** (named-export synthesis at E5): same — runtime-layer host hook.
- **Tuple C** (ParseModule grammar): retired by rusty-js-parser at the AST layer; bytecode inherits the typed AST without ambiguity.

The bytecode pilot is therefore neutral on the Doc 717 cuts: its role is to make the parser's output executable. Tuple A/B closure happens above the bytecode in the runtime's Link-phase hooks.
