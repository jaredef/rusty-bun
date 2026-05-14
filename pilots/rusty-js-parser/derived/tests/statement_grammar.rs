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

fn declarator_names(d: &VariableDeclarator) -> Vec<String> {
    d.target.collect_names().into_iter().map(|n| n.name.clone()).collect()
}

fn param_names(p: &Parameter) -> Vec<String> {
    p.target.collect_names().into_iter().map(|n| n.name.clone()).collect()
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
        assert_eq!(declarator_names(&v.declarators[0]), vec!["x"]);
        assert_eq!(declarator_names(&v.declarators[1]), vec!["y"]);
        assert_eq!(declarator_names(&v.declarators[2]), vec!["z"]);
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
        let names = declarator_names(&v.declarators[0]);
        // a is bare; b is renamed to c — local binding is `c`.
        assert!(names.iter().any(|n| n == "a"));
        assert!(names.iter().any(|n| n == "c"));
    } else { panic!(); }
}

#[test]
fn var_destructure_array() {
    if let Stmt::Variable(v) = first_stmt("const [a, b, c] = arr;") {
        let names = declarator_names(&v.declarators[0]);
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
    if let Stmt::FunctionDecl { name, is_async, is_generator, params, body, .. } = first_stmt("function foo() {}") {
        assert_eq!(name.unwrap().name, "foo");
        assert!(!is_async);
        assert!(!is_generator);
        assert!(params.is_empty());
        assert!(body.is_empty());
    } else { panic!(); }
}

#[test]
fn function_with_parameters() {
    if let Stmt::FunctionDecl { params, .. } = first_stmt("function add(a, b) { return a + b; }") {
        assert_eq!(params.len(), 2);
        assert_eq!(param_names(&params[0]), vec!["a"]);
        assert_eq!(param_names(&params[1]), vec!["b"]);
        assert!(!params[0].rest);
    } else { panic!(); }
}

#[test]
fn function_with_default_parameter() {
    if let Stmt::FunctionDecl { params, .. } = first_stmt("function f(x = 1, y = x + 2) {}") {
        assert_eq!(params.len(), 2);
        assert!(params[0].default.is_some());
        assert!(params[1].default.is_some());
    } else { panic!(); }
}

#[test]
fn function_with_rest_parameter() {
    if let Stmt::FunctionDecl { params, .. } = first_stmt("function f(a, ...rest) {}") {
        assert_eq!(params.len(), 2);
        assert!(!params[0].rest);
        assert!(params[1].rest);
        assert_eq!(param_names(&params[1]), vec!["rest"]);
    } else { panic!(); }
}

#[test]
fn function_with_destructure_parameter() {
    if let Stmt::FunctionDecl { params, .. } = first_stmt("function f({a, b: c}) {}") {
        assert_eq!(params.len(), 1);
        let names = param_names(&params[0]);
        assert!(names.iter().any(|n| n == "a"));
        assert!(names.iter().any(|n| n == "c"));
    } else { panic!(); }
}

#[test]
fn function_typed_body() {
    if let Stmt::FunctionDecl { body, .. } = first_stmt("function f(x) { let y = x + 1; return y; }") {
        assert_eq!(body.len(), 2);
        assert!(matches!(&body[0], Stmt::Variable(_)));
        assert!(matches!(&body[1], Stmt::Return { .. }));
    } else { panic!(); }
}

#[test]
fn function_recursive_body() {
    // Function body containing another function body — Block-of-statements
    // recursion through parse_statement.
    if let Stmt::FunctionDecl { body, .. } = first_stmt(
        "function outer() { function inner() { return 1; } return inner; }"
    ) {
        assert_eq!(body.len(), 2);
        if let Stmt::FunctionDecl { name, .. } = &body[0] {
            assert_eq!(name.as_ref().unwrap().name, "inner");
        } else { panic!("expected nested function"); }
        assert!(matches!(&body[1], Stmt::Return { .. }));
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

// ─────────── Control-flow (now typed in round 3c) ───────────

#[test]
fn if_statement_typed() {
    if let Stmt::If { test, consequent, alternate, .. } = first_stmt("if (x) { y(); }") {
        assert!(matches!(test, Expr::Identifier { .. }));
        assert!(matches!(*consequent, Stmt::Block { .. }));
        assert!(alternate.is_none());
    } else { panic!("expected if"); }
}

#[test]
fn if_else_statement() {
    if let Stmt::If { alternate, .. } = first_stmt("if (x) a(); else b();") {
        assert!(alternate.is_some());
    } else { panic!("expected if-else"); }
}

#[test]
fn for_c_style() {
    if let Stmt::For { init, test, update, body, .. } = first_stmt("for (let i = 0; i < 10; i++) {}") {
        assert!(init.is_some());
        assert!(test.is_some());
        assert!(update.is_some());
        assert!(matches!(*body, Stmt::Block { .. }));
    } else { panic!("expected for"); }
}

#[test]
fn for_in() {
    if let Stmt::ForIn { left, .. } = first_stmt("for (const k in obj) f(k);") {
        if let ForBinding::Decl { target, .. } = left {
            if let BindingPattern::Identifier(id) = target {
                assert_eq!(id.name, "k");
            } else { panic!("expected identifier target"); }
        } else { panic!("expected decl binding"); }
    } else { panic!("expected for-in"); }
}

#[test]
fn for_of() {
    if let Stmt::ForOf { left, await_, .. } = first_stmt("for (const x of arr) f(x);") {
        if let ForBinding::Decl { target, .. } = left {
            if let BindingPattern::Identifier(id) = target {
                assert_eq!(id.name, "x");
            } else { panic!("expected identifier target"); }
        } else { panic!("expected decl binding"); }
        assert!(!await_);
    } else { panic!("expected for-of"); }
}

#[test]
fn for_await_of() {
    if let Stmt::ForOf { await_, .. } = first_stmt("for await (const x of asyncIter) f(x);") {
        assert!(await_);
    } else { panic!("expected for-await-of"); }
}

#[test]
fn while_statement() {
    if let Stmt::While { test, body, .. } = first_stmt("while (x) f();") {
        assert!(matches!(test, Expr::Identifier { .. }));
        assert!(matches!(*body, Stmt::Expression { .. }));
    } else { panic!("expected while"); }
}

#[test]
fn do_while_statement() {
    if let Stmt::DoWhile { test, body, .. } = first_stmt("do { f(); } while (cond);") {
        assert!(matches!(*body, Stmt::Block { .. }));
        assert!(matches!(test, Expr::Identifier { .. }));
    } else { panic!("expected do-while"); }
}

#[test]
fn switch_statement() {
    if let Stmt::Switch { discriminant, cases, .. } = first_stmt("switch (x) { case 1: a(); break; case 2: b(); break; default: c(); }") {
        assert!(matches!(discriminant, Expr::Identifier { .. }));
        assert_eq!(cases.len(), 3);
        assert!(cases[0].test.is_some());
        assert!(cases[2].test.is_none());
    } else { panic!("expected switch"); }
}

#[test]
fn try_catch_finally() {
    if let Stmt::Try { handler, finalizer, .. } = first_stmt("try { f(); } catch (e) { g(e); } finally { h(); }") {
        let h = handler.expect("handler");
        assert_eq!(h.param.unwrap().name, "e");
        assert!(finalizer.is_some());
    } else { panic!("expected try"); }
}

#[test]
fn try_optional_catch_binding() {
    if let Stmt::Try { handler, .. } = first_stmt("try { f(); } catch { g(); }") {
        let h = handler.expect("handler");
        assert!(h.param.is_none());
    } else { panic!("expected try"); }
}

#[test]
fn return_no_argument() {
    if let Stmt::Return { argument, .. } = first_stmt("return;") {
        assert!(argument.is_none());
    } else { panic!("expected return"); }
}

#[test]
fn return_with_argument() {
    if let Stmt::Return { argument, .. } = first_stmt("return 42;") {
        assert!(argument.is_some());
    } else { panic!("expected return"); }
}

#[test]
fn throw_statement() {
    if let Stmt::Throw { argument, .. } = first_stmt("throw new Error('boom');") {
        assert!(matches!(argument, Expr::New { .. }));
    } else { panic!("expected throw"); }
}

#[test]
fn break_with_label() {
    if let Stmt::Break { label, .. } = first_stmt("break outer;") {
        assert_eq!(label.unwrap().name, "outer");
    } else { panic!("expected break"); }
}

#[test]
fn continue_unlabelled() {
    if let Stmt::Continue { label, .. } = first_stmt("continue;") {
        assert!(label.is_none());
    } else { panic!("expected continue"); }
}

#[test]
fn debugger_statement() {
    assert!(matches!(first_stmt("debugger;"), Stmt::Debugger { .. }));
}

#[test]
fn labelled_statement() {
    if let Stmt::Labelled { label, body, .. } = first_stmt("outer: for (;;) { break outer; }") {
        assert_eq!(label.name, "outer");
        assert!(matches!(*body, Stmt::For { .. }));
    } else { panic!("expected labelled"); }
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

// ─────────── Tier-Ω.5.g.2: typed BindingPattern parser tests ───────────

#[test]
fn destructure_array_with_rest() {
    if let Stmt::Variable(v) = first_stmt("const [a, b, ...rest] = arr;") {
        let target = &v.declarators[0].target;
        if let BindingPattern::Array(arr) = target {
            assert_eq!(arr.elements.len(), 2);
            // first two are named elements
            for (i, expected) in ["a", "b"].iter().enumerate() {
                if let Some(e) = &arr.elements[i] {
                    if let BindingPattern::Identifier(id) = &e.target {
                        assert_eq!(id.name, *expected);
                    } else { panic!("expected ident at {}", i); }
                } else { panic!("expected element at {}", i); }
            }
            // rest
            if let Some(rest) = &arr.rest {
                if let BindingPattern::Identifier(id) = rest.as_ref() {
                    assert_eq!(id.name, "rest");
                } else { panic!("expected ident rest"); }
            } else { panic!("expected rest"); }
        } else { panic!("expected array pattern"); }
        let names: Vec<String> = target.collect_names().into_iter().map(|n| n.name.clone()).collect();
        assert_eq!(names, vec!["a", "b", "rest"]);
    } else { panic!("expected variable"); }
}

#[test]
fn destructure_object_with_rename_and_rest() {
    if let Stmt::Variable(v) = first_stmt("const {x, y: alias, ...rest} = obj;") {
        let target = &v.declarators[0].target;
        if let BindingPattern::Object(obj) = target {
            assert_eq!(obj.properties.len(), 2);
            // Property 0: x — shorthand
            assert!(obj.properties[0].shorthand);
            if let BindingPattern::Identifier(id) = &obj.properties[0].value.target {
                assert_eq!(id.name, "x");
            } else { panic!("expected ident"); }
            // Property 1: y: alias — renamed
            assert!(!obj.properties[1].shorthand);
            if let BindingPattern::Identifier(id) = &obj.properties[1].value.target {
                assert_eq!(id.name, "alias");
            } else { panic!("expected alias"); }
            // Rest
            let rest = obj.rest.as_ref().expect("rest");
            assert_eq!(rest.name, "rest");
        } else { panic!("expected object pattern"); }
        let names: Vec<String> = target.collect_names().into_iter().map(|n| n.name.clone()).collect();
        assert_eq!(names, vec!["x", "alias", "rest"]);
    } else { panic!("expected variable"); }
}

#[test]
fn function_with_two_destructure_params() {
    if let Stmt::FunctionDecl { params, .. } = first_stmt("function f({a, b}, [c, d]) {}") {
        assert_eq!(params.len(), 2);
        if let BindingPattern::Object(_) = &params[0].target {} else { panic!("expected object pattern param 0"); }
        if let BindingPattern::Array(_) = &params[1].target {} else { panic!("expected array pattern param 1"); }
        let n0: Vec<String> = params[0].target.collect_names().into_iter().map(|n| n.name.clone()).collect();
        let n1: Vec<String> = params[1].target.collect_names().into_iter().map(|n| n.name.clone()).collect();
        assert_eq!(n0, vec!["a", "b"]);
        assert_eq!(n1, vec!["c", "d"]);
    } else { panic!("expected function decl"); }
}

#[test]
fn template_literal_typed() {
    // The template appears in expression-statement position.
    if let Stmt::Expression { expr, .. } = first_stmt("`hello ${x} world ${y}!`;") {
        if let Expr::TemplateLiteral { quasis, expressions, .. } = expr {
            let qs: Vec<String> = quasis.iter().map(|s| (**s).clone()).collect();
            assert_eq!(qs, vec!["hello ".to_string(), " world ".to_string(), "!".to_string()]);
            assert_eq!(expressions.len(), 2);
            if let Expr::Identifier { name, .. } = &expressions[0] {
                assert_eq!(name, "x");
            } else { panic!("expected ident x"); }
            if let Expr::Identifier { name, .. } = &expressions[1] {
                assert_eq!(name, "y");
            } else { panic!("expected ident y"); }
        } else { panic!("expected template literal, got {:?}", expr); }
    } else { panic!("expected expression statement"); }
}
