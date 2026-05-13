//! Spec golden tests for the statement grammar (Tier-Ω.3.b round 3b subset).
//!
//! Tests parse_statement at the module-top level. Covers VariableStatement,
//! ExpressionStatement, Block, EmptyStatement, FunctionDeclaration (body
//! opaque), ClassDeclaration (body opaque), and Stmt::Opaque fallback for
//! control-flow forms not yet typed.

use rusty_js_ast::*;
use rusty_js_parser::parse_module;

fn first_stmt(src: &str) -> Stmt {
    let m = parse_module(src).expect(&format!("parse failed: {:?}", src));
    match &m.body[0] {
        ModuleItem::Statement(s) => s.clone(),
        _ => panic!("expected statement"),
    }
}

// ─────────── VariableStatement ───────────

#[test]
fn var_const_let_kinds() {
    if let Stmt::Variable(v) = first_stmt("var x = 1;") {
        assert_eq!(v.kind, VariableKind::Var);
    } else { panic!(); }
    if let Stmt::Variable(v) = first_stmt("let x = 1;") {
        assert_eq!(v.kind, VariableKind::Let);
    } else { panic!(); }
    if let Stmt::Variable(v) = first_stmt("const x = 1;") {
        assert_eq!(v.kind, VariableKind::Const);
    } else { panic!(); }
}

#[test]
fn var_multiple_declarators() {
    if let Stmt::Variable(v) = first_stmt("let x = 1, y = 2, z;") {
        assert_eq!(v.declarators.len(), 3);
        assert_eq!(v.declarators[0].names[0].name, "x");
        assert_eq!(v.declarators[1].names[0].name, "y");
        assert_eq!(v.declarators[2].names[0].name, "z");
        assert!(v.declarators[2].init.is_none());
    } else { panic!(); }
}

#[test]
fn var_initializer_is_typed_expr() {
    if let Stmt::Variable(v) = first_stmt("const sum = 1 + 2 * 3;") {
        let init = v.declarators[0].init.as_ref().expect("init");
        assert!(matches!(init, Expr::Binary { operator: BinaryOp::Add, .. }));
    } else { panic!(); }
}

#[test]
fn var_destructure_object() {
    if let Stmt::Variable(v) = first_stmt("const { a, b: c } = obj;") {
        let names: Vec<&str> = v.declarators[0].names.iter().map(|n| n.name.as_str()).collect();
        // a is bare; b is renamed to c — local binding is `c`.
        assert!(names.contains(&"a"));
        assert!(names.contains(&"c"));
    } else { panic!(); }
}

#[test]
fn var_destructure_array() {
    if let Stmt::Variable(v) = first_stmt("const [a, b, c] = arr;") {
        let names: Vec<&str> = v.declarators[0].names.iter().map(|n| n.name.as_str()).collect();
        assert_eq!(names, vec!["a", "b", "c"]);
    } else { panic!(); }
}

// ─────────── ExpressionStatement ───────────

#[test]
fn expression_statement() {
    if let Stmt::Expression { expr, .. } = first_stmt("foo();") {
        assert!(matches!(expr, Expr::Call { .. }));
    } else { panic!(); }
}

#[test]
fn expression_statement_with_asi() {
    // No semicolon — relies on ASI at EOF
    if let Stmt::Expression { expr, .. } = first_stmt("foo()") {
        assert!(matches!(expr, Expr::Call { .. }));
    } else { panic!(); }
}

#[test]
fn expression_statement_chained() {
    if let Stmt::Expression { expr, .. } = first_stmt("a.b.c();") {
        assert!(matches!(expr, Expr::Call { .. }));
    } else { panic!(); }
}

// ─────────── Block ───────────

#[test]
fn block_statement() {
    if let Stmt::Block { body, .. } = first_stmt("{ let x = 1; let y = 2; }") {
        assert_eq!(body.len(), 2);
        assert!(matches!(&body[0], Stmt::Variable(_)));
        assert!(matches!(&body[1], Stmt::Variable(_)));
    } else { panic!(); }
}

#[test]
fn nested_block() {
    if let Stmt::Block { body, .. } = first_stmt("{ { let x = 1; } }") {
        assert_eq!(body.len(), 1);
        assert!(matches!(&body[0], Stmt::Block { .. }));
    } else { panic!(); }
}

// ─────────── Empty ───────────

#[test]
fn empty_statement() {
    assert!(matches!(first_stmt(";"), Stmt::Empty { .. }));
}

// ─────────── FunctionDeclaration ───────────

#[test]
fn function_declaration() {
    if let Stmt::FunctionDecl { name, is_async, is_generator, .. } = first_stmt("function foo() {}") {
        assert_eq!(name.unwrap().name, "foo");
        assert!(!is_async);
        assert!(!is_generator);
    } else { panic!(); }
}

#[test]
fn async_function_declaration() {
    if let Stmt::FunctionDecl { name, is_async, .. } = first_stmt("async function foo() {}") {
        assert_eq!(name.unwrap().name, "foo");
        assert!(is_async);
    } else { panic!(); }
}

#[test]
fn generator_function_declaration() {
    if let Stmt::FunctionDecl { is_generator, .. } = first_stmt("function* gen() {}") {
        assert!(is_generator);
    } else { panic!(); }
}

// ─────────── ClassDeclaration ───────────

#[test]
fn class_declaration() {
    if let Stmt::ClassDecl { name, .. } = first_stmt("class Foo {}") {
        assert_eq!(name.unwrap().name, "Foo");
    } else { panic!(); }
}

#[test]
fn class_with_extends() {
    if let Stmt::ClassDecl { name, .. } = first_stmt("class Foo extends Bar {}") {
        assert_eq!(name.unwrap().name, "Foo");
    } else { panic!(); }
}

// ─────────── Opaque (control-flow fallback) ───────────

#[test]
fn if_statement_falls_back_opaque() {
    assert!(matches!(first_stmt("if (x) { y(); }"), Stmt::Opaque { .. }));
}

#[test]
fn for_statement_falls_back_opaque() {
    assert!(matches!(first_stmt("for (let i = 0; i < 10; i++) {}"), Stmt::Opaque { .. }));
}

#[test]
fn return_statement_falls_back_opaque() {
    assert!(matches!(first_stmt("return 42;"), Stmt::Opaque { .. }));
}

#[test]
fn try_statement_falls_back_opaque() {
    assert!(matches!(first_stmt("try { f(); } catch (e) {}"), Stmt::Opaque { .. }));
}

// ─────────── Mixed-module integration ───────────

#[test]
fn typed_module_body() {
    let src = "import x from 'a'; const y = 2 + 3; foo(); export { y };";
    let m = parse_module(src).unwrap();
    // ImportDeclaration, Statement(Variable), Statement(Expression), ExportDeclaration
    assert!(matches!(&m.body[0], ModuleItem::Import(_)));
    assert!(matches!(&m.body[1], ModuleItem::Statement(Stmt::Variable(_))));
    assert!(matches!(&m.body[2], ModuleItem::Statement(Stmt::Expression { .. })));
    assert!(matches!(&m.body[3], ModuleItem::Export(_)));
}
