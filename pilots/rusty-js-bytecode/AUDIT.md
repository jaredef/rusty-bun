# rusty-js-bytecode — coverage audit

**Second sub-pilot of Tier-Ω.3** (after rusty-js-parser). Per the [Ω.3 engine-selection decision artifact](../../host/tools/omega-3-engine-selection.md) §III and [the design spec](../../specs/rusty-js-bytecode-design.md).

## Engagement role

The bytecode pilot occupies the cut-rung between AST and runtime. Walks the rusty-js-ast typed AST and emits a single-pass bytecode stream + constants pool + scope/upvalue descriptors. This is the unit the runtime interpreter consumes.

QuickJS as architectural reference: stack-based dispatch, single-pass compilation, no JIT, forward-reference patching for jump targets. Per the keeper directive (2026-05-13 19:53Z) and the Ω.3 decision artifact.

## Pilot scope (v1)

Three composed surfaces in a single crate:

### Bytecode + constants pool
- `Bytecode` — `Vec<u8>` instruction stream with inline operands (u8/u16/u32 LE, constant-pool indices as u16)
- `Constant` enum — Number (f64), BigInt (digit-string), String, Regex (body+flags), Function (nested compiled prototype)
- `CompiledModule` — bytecode + constants + local-slot table + upvalue descriptors + source-map (bytecode offset → AST span)

### Compiler
- Walks `rusty_js_ast::Module` / `Stmt` / `Expr` enums and emits opcodes
- Resolves scopes during the walk; allocates local slots; binds upvalues via parent-frame walk
- Forward-jump patching: emit a placeholder offset on `JUMP_*`, record the patch site, fill in the displacement when the target is known
- Single-pass: no AST mutation, no optimization passes

### Disassembler
- Reverses the bytecode into a human-readable form for debugging
- Used by the test harness to assert specific bytecode shapes

## Instruction-set scope (v1)

Per design spec §II. Subset bounded by the AST shapes rusty-js-parser emits:

- Stack ops (PUSH_*, POP, DUP, SWAP)
- Variable / scope (LOAD_LOCAL/STORE_LOCAL/LOAD_ARG/LOAD_GLOBAL/LOAD_UPVALUE/DEFINE_LOCAL)
- Arithmetic (ADD/SUB/MUL/DIV/MOD/POW/NEG/POS/INC/DEC)
- Comparison + equality (LT/GT/LE/GE/EQ/NE/STRICT_EQ/STRICT_NE)
- Relational (IN/INSTANCEOF)
- Bitwise + shift (BIT_*/SHL/SHR/USHR)
- Logical NOT (`&&` / `||` / `??` compile to conditional jumps)
- Control flow (JUMP, JUMP_IF_TRUE/FALSE, JUMP_IF_TRUE_KEEP/FALSE_KEEP, JUMP_IF_NULLISH)
- Calls + returns (CALL, NEW, RETURN, RETURN_UNDEF)
- Member access (GET_PROP, SET_PROP, GET_INDEX, SET_INDEX)
- Object/array literals (NEW_OBJECT, NEW_ARRAY, INIT_PROP, INIT_INDEX)
- Unary (TYPEOF, VOID, DELETE)
- Function/closure (MAKE_CLOSURE, MAKE_ARROW)
- Exception (THROW, TRY_ENTER, TRY_EXIT)
- Iteration (ITER_INIT, ITER_NEXT, ITER_CLOSE)
- Miscellaneous (NOP, DEBUGGER)

## Compilation invariants (per design spec §V)

- Use-before-declare for let/const is a runtime error (temporal dead zone)
- Each opcode has a documented stack-effect; verifier (not v1) confirms stack-depth convergence at control-flow merges
- Source-map populated per opcode for diagnostics
- No GC barriers emitted in bytecode (mark-sweep is between-dispatch)

## Test corpus

Three layers:

1. **Per-instruction golden tests** — for each opcode, an AST input that compiles to exactly that opcode (or a known sequence). Verifies the compiler emits the expected bytecode.

2. **AST-shape integration tests** — broader source snippets covering each expression form, statement form, and declaration form. Asserts that `parse_module → compile_module` produces well-formed bytecode (no missing patches, stack-balanced).

3. **Round-trip via disassembler** — compile + disassemble + assert the disassembly matches an expected human-readable shape.

## Out of scope for v1

- Generator/async suspension opcodes (yield, await as suspend points)
- Optimizing variants (inline caches, hidden-class shape guards, JIT)
- Sourcemap export to V3 sourcemap JSON
- Static-analysis verifier (will be a successor pilot)

## Composition with downstream pilots

- **rusty-js-runtime** (Ω.3.d, next sub-pilot) consumes `CompiledModule`. Dispatch loop reads bytes, decodes operands, executes against the Value representation.
- **rusty-js-gc** (Ω.3.e) services rusty-js-runtime; bytecode itself doesn't allocate.

## Estimated pilot size

- Constants pool + module shape: ~200 LOC
- Compiler walking expressions: ~400 LOC
- Compiler walking statements (including control-flow patches): ~350 LOC
- Compiler walking declarations + scope/upvalue resolution: ~300 LOC
- Disassembler: ~200 LOC
- Tests: ~600 LOC

Total ~2,050 LOC. Smaller than the parser pilot.

Per the substrate-amortization discipline (seed §A8.13), the pilot ships in 3-4 sub-rounds:
- Ω.3.c.a: spec + AUDIT.md (this commit)
- Ω.3.c.b: scaffold + constants pool + compiler skeleton + first 20 opcodes (literals + arithmetic + comparison)
- Ω.3.c.c: control flow + variables + scope resolution
- Ω.3.c.d: functions/closures + member access + objects/arrays + try-catch

## First-round scope (this commit — Ω.3.c.a)

Substrate-introduction round only:
- specs/rusty-js-bytecode-design.md
- pilots/rusty-js-bytecode/AUDIT.md (this file)

No Cargo crate yet. Next round (Ω.3.c.b) creates pilots/rusty-js-bytecode/derived/ + the constants pool + compiler skeleton.

## Tier-Ω.4 cross-reference

The bytecode pilot is neutral on the Doc 717 tuples (per design spec §VIII):
- Tuple A (Module Namespace augmentation): runtime-layer hook, not bytecode-resident
- Tuple B (named-export synthesis): runtime-layer hook
- Tuple C (parser grammar): retired by rusty-js-parser; bytecode inherits the typed AST

The pilot's role: make the parser's output executable. Tuple closure happens above the bytecode in the runtime's Link-phase hooks.
