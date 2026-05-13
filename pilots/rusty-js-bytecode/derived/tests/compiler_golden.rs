//! Golden tests for the bytecode compiler per the design spec.
//!
//! Each test compiles a small module and asserts the disassembly matches
//! an expected shape. Verifies the compiler emits the correct opcode
//! sequence for each AST form.

use rusty_js_bytecode::{compile_module, disassemble, Constant};

fn disasm(src: &str) -> String {
    let m = compile_module(src).expect(&format!("compile failed: {:?}", src));
    disassemble(&m)
}

// ─────────── Literals ───────────

#[test]
fn push_null() {
    let d = disasm("null;");
    assert!(d.contains("PushNull"));
    assert!(d.contains("ReturnUndef"));
}

#[test]
fn push_bool() {
    let d = disasm("true;");
    assert!(d.contains("PushTrue"));
}

#[test]
fn push_small_int_fast_path() {
    let d = disasm("42;");
    assert!(d.contains("PushI32 42"));
}

#[test]
fn push_large_number_via_constants() {
    let d = disasm("1e100;");
    assert!(d.contains("PushConst"));
}

#[test]
fn push_string_via_constants() {
    let d = disasm("'hello';");
    assert!(d.contains("PushConst"));
    assert!(d.contains("\"hello\""));
}

// ─────────── Arithmetic ───────────

#[test]
fn add() {
    let d = disasm("1 + 2;");
    assert!(d.contains("PushI32 1"));
    assert!(d.contains("PushI32 2"));
    assert!(d.contains("Add"));
}

#[test]
fn precedence_in_emission() {
    // 1 + 2 * 3 -> Push 1, Push 2, Push 3, Mul, Add
    let d = disasm("1 + 2 * 3;");
    let mul_idx = d.find("Mul").unwrap();
    let add_idx = d.find("Add").unwrap();
    assert!(mul_idx < add_idx, "Mul should come before Add (RHS evaluated first)");
}

#[test]
fn unary_neg() {
    let d = disasm("-x;");
    assert!(d.contains("LoadGlobal"));
    assert!(d.contains("Neg"));
}

#[test]
fn typeof_operator() {
    let d = disasm("typeof x;");
    assert!(d.contains("Typeof"));
}

// ─────────── Comparison ───────────

#[test]
fn strict_equality() {
    let d = disasm("a === b;");
    assert!(d.contains("StrictEq"));
}

#[test]
fn relational() {
    let d = disasm("a < b;");
    assert!(d.contains("Lt"));
}

#[test]
fn instanceof_operator() {
    let d = disasm("x instanceof Foo;");
    assert!(d.contains("Instanceof"));
}

// ─────────── Bitwise / shift ───────────

#[test]
fn bitwise_ops() {
    let d = disasm("a & b | c ^ ~d;");
    assert!(d.contains("BitAnd"));
    assert!(d.contains("BitOr"));
    assert!(d.contains("BitXor"));
    assert!(d.contains("BitNot"));
}

#[test]
fn shift_ops() {
    let d = disasm("a << b;");
    assert!(d.contains("Shl"));
    let d = disasm("a >>> b;");
    assert!(d.contains("UShr"));
}

// ─────────── Variables ───────────

#[test]
fn identifier_loads_global() {
    let d = disasm("x;");
    assert!(d.contains("LoadGlobal"));
    assert!(d.contains("\"x\""));
}

#[test]
fn variable_declaration_stores_local() {
    let d = disasm("let x = 1;");
    assert!(d.contains("PushI32 1"));
    assert!(d.contains("StoreLocal"));
}

#[test]
fn variable_without_initializer() {
    let d = disasm("let x;");
    assert!(d.contains("PushUndef"));
    assert!(d.contains("StoreLocal"));
}

#[test]
fn multiple_declarators() {
    let d = disasm("const a = 1, b = 2;");
    let count = d.matches("StoreLocal").count();
    assert_eq!(count, 2);
}

#[test]
fn local_identifier_resolves_to_local_slot() {
    let d = disasm("let x = 1; x;");
    assert!(d.contains("LoadLocal"));
}

#[test]
fn undeclared_identifier_falls_to_global() {
    let d = disasm("y;");
    assert!(d.contains("LoadGlobal"));
}

// ─────────── Control flow ───────────

#[test]
fn if_statement_emits_conditional_jump() {
    let d = disasm("if (x) { y; }");
    assert!(d.contains("JumpIfFalse"));
}

#[test]
fn if_else_emits_both_jumps() {
    let d = disasm("if (x) a; else b;");
    assert!(d.contains("JumpIfFalse"));
    assert!(d.contains("Jump "));
}

#[test]
fn while_loop_emits_back_jump() {
    let d = disasm("while (x) { y; }");
    assert!(d.contains("JumpIfFalse"));
    // backward Jump to loop-start
    assert!(d.contains("Jump "));
}

#[test]
fn for_c_style_emits_init_test_body_update() {
    let d = disasm("for (let i = 0; i < 10; i = i + 1) { i; }");
    // Test should emit the comparison
    assert!(d.contains("Lt"));
    assert!(d.contains("JumpIfFalse"));
}

#[test]
fn do_while_emits_jump_if_true_back() {
    let d = disasm("do { x; } while (cond);");
    assert!(d.contains("JumpIfTrue"));
}

#[test]
fn break_in_while_loop() {
    let d = disasm("while (x) { break; }");
    // A break inside the loop should compile (no error)
    assert!(d.contains("Jump"));
}

#[test]
fn continue_in_while_loop() {
    let d = disasm("while (x) { continue; }");
    assert!(d.contains("Jump"));
}

// ─────────── Short-circuit + conditional ───────────

#[test]
fn logical_and_short_circuits() {
    let d = disasm("a && b;");
    assert!(d.contains("JumpIfFalseKeep"));
}

#[test]
fn logical_or_short_circuits() {
    let d = disasm("a || b;");
    assert!(d.contains("JumpIfTrueKeep"));
}

#[test]
fn nullish_coalesce() {
    let d = disasm("a ?? b;");
    assert!(d.contains("JumpIfNullish"));
}

#[test]
fn conditional_expression_emits_jumps() {
    let d = disasm("a ? b : c;");
    assert!(d.contains("JumpIfFalse"));
}

#[test]
fn assignment_to_local() {
    let d = disasm("let x = 0; x = 1;");
    // After the let, x is a local. The assignment should emit StoreLocal.
    let stores = d.matches("StoreLocal").count();
    assert!(stores >= 2);
}

#[test]
fn assignment_to_global() {
    // x is undeclared; assignment goes to global.
    let d = disasm("x = 1;");
    assert!(d.contains("StoreGlobal"));
}

// ─────────── Statements ───────────

#[test]
fn expression_statement_pops_result() {
    let d = disasm("1 + 2;");
    assert!(d.contains("Pop"));
}

#[test]
fn return_with_argument() {
    // Wrap in a synthetic module-level Stmt::Return — return at module
    // top-level is unusual but the parser permits it; compiler emits
    // the Return opcode.
    let d = disasm("return 42;");
    // Return statement at top level routes through Stmt::Opaque?
    // Actually parse_statement handles "return" as a typed Stmt::Return.
    assert!(d.contains("PushI32 42"));
    assert!(d.contains("Return"));
}

#[test]
fn return_without_argument() {
    let d = disasm("return;");
    assert!(d.contains("ReturnUndef"));
}

#[test]
fn throw_statement() {
    let d = disasm("throw 1;");
    assert!(d.contains("PushI32 1"));
    assert!(d.contains("Throw"));
}

#[test]
fn debugger_statement() {
    let d = disasm("debugger;");
    assert!(d.contains("Debugger"));
}

#[test]
fn empty_statement() {
    // Just emits the module's trailing ReturnUndef.
    let d = disasm(";");
    assert!(d.contains("ReturnUndef"));
}

// ─────────── Block ───────────

#[test]
fn block_compiles_children_in_order() {
    let d = disasm("{ 1; 2; 3; }");
    let one = d.find("PushI32 1").unwrap();
    let two = d.find("PushI32 2").unwrap();
    let three = d.find("PushI32 3").unwrap();
    assert!(one < two && two < three);
}

// ─────────── Constants pool deduplication ───────────

#[test]
fn constants_pool_dedups_strings() {
    let m = compile_module("'x'; 'x'; 'x';").unwrap();
    let string_xs = m.constants.entries().iter()
        .filter(|c| matches!(c, Constant::String(s) if s == "x"))
        .count();
    assert_eq!(string_xs, 1, "string constants should dedupe");
}

#[test]
fn constants_pool_dedups_numbers() {
    let m = compile_module("1e100; 1e100;").unwrap();
    let nums = m.constants.entries().iter()
        .filter(|c| matches!(c, Constant::Number(v) if (*v - 1e100).abs() < 1e90))
        .count();
    assert_eq!(nums, 1, "number constants should dedupe");
}

// ─────────── Source-map ───────────

#[test]
fn source_map_populated() {
    let m = compile_module("let x = 1 + 2;").unwrap();
    assert!(!m.source_map.is_empty(), "source-map should be populated");
}
