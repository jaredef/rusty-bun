//! Spec golden tests for the expression grammar (Tier-Ω.3.b round 3a subset).
//!
//! Tests `parse_assignment_expression` by wrapping each test source in
//! `export default <expr>;` and asserting on the resulting AST node.

use rusty_js_ast::*;
use rusty_js_parser::parse_module;

fn expr_of(src: &str) -> Expr {
    let wrapped = format!("export default {};", src);
    let m = parse_module(&wrapped).expect(&format!("parse failed: {:?}", wrapped));
    let first = &m.body[0];
    match first {
        ModuleItem::Export(ExportDeclaration::Default { body: DefaultExportBody::Expression { expr }, .. }) => expr.clone(),
        _ => panic!("expected default export expression"),
    }
}

// ─────────── Literals ───────────

#[test]
fn null_literal() {
    assert!(matches!(expr_of("null"), Expr::NullLiteral { .. }));
}

#[test]
fn bool_literal() {
    assert!(matches!(expr_of("true"), Expr::BoolLiteral { value: true, .. }));
    assert!(matches!(expr_of("false"), Expr::BoolLiteral { value: false, .. }));
}

#[test]
fn number_literal() {
    if let Expr::NumberLiteral { value, .. } = expr_of("42") {
        assert_eq!(value, 42.0);
    } else { panic!("expected number"); }
}

#[test]
fn string_literal() {
    if let Expr::StringLiteral { value, .. } = expr_of("'hello'") {
        assert_eq!(value, "hello");
    } else { panic!("expected string"); }
}

#[test]
fn bigint_literal() {
    if let Expr::BigIntLiteral { digits, .. } = expr_of("123n") {
        assert_eq!(digits, "123");
    } else { panic!("expected bigint"); }
}

// ─────────── Identifier + meta-properties ───────────

#[test]
fn identifier() {
    if let Expr::Identifier { name, .. } = expr_of("foo") {
        assert_eq!(name, "foo");
    } else { panic!("expected identifier"); }
}

#[test]
fn this_expression() {
    assert!(matches!(expr_of("this"), Expr::This { .. }));
}

#[test]
fn new_target() {
    if let Expr::MetaProperty { meta, property, .. } = expr_of("new.target") {
        assert_eq!(meta, "new");
        assert_eq!(property, "target");
    } else { panic!("expected meta-property"); }
}

// ─────────── Member + Call ───────────

#[test]
fn member_dot() {
    if let Expr::Member { object, property, optional, .. } = expr_of("a.b") {
        assert!(matches!(*object, Expr::Identifier { .. }));
        assert!(matches!(*property, MemberProperty::Identifier { .. }));
        assert!(!optional);
    } else { panic!("expected member"); }
}

#[test]
fn member_optional() {
    if let Expr::Member { optional, .. } = expr_of("a?.b") {
        assert!(optional);
    } else { panic!("expected optional member"); }
}

#[test]
fn member_computed() {
    if let Expr::Member { property, .. } = expr_of("a[0]") {
        assert!(matches!(*property, MemberProperty::Computed { .. }));
    } else { panic!("expected computed"); }
}

#[test]
fn member_chained() {
    // a.b.c.d
    let e = expr_of("a.b.c.d");
    let mut depth = 0;
    let mut cur = &e;
    while let Expr::Member { object, .. } = cur {
        depth += 1;
        cur = object;
    }
    assert_eq!(depth, 3);
    assert!(matches!(cur, Expr::Identifier { .. }));
}

#[test]
fn call_expression() {
    if let Expr::Call { callee, arguments, optional, .. } = expr_of("f(1, 2)") {
        assert!(matches!(*callee, Expr::Identifier { .. }));
        assert_eq!(arguments.len(), 2);
        assert!(!optional);
    } else { panic!("expected call"); }
}

#[test]
fn call_spread() {
    if let Expr::Call { arguments, .. } = expr_of("f(...args)") {
        assert_eq!(arguments.len(), 1);
        assert!(matches!(&arguments[0], Argument::Spread { .. }));
    } else { panic!("expected call"); }
}

#[test]
fn new_expression() {
    if let Expr::New { callee, arguments, .. } = expr_of("new Foo(1, 2)") {
        assert!(matches!(*callee, Expr::Identifier { .. }));
        assert_eq!(arguments.len(), 2);
    } else { panic!("expected new"); }
}

// ─────────── Unary + Update ───────────

#[test]
fn unary_minus() {
    if let Expr::Unary { operator, .. } = expr_of("-x") {
        assert_eq!(operator, UnaryOp::Minus);
    } else { panic!("expected unary"); }
}

#[test]
fn typeof_operator() {
    if let Expr::Unary { operator, .. } = expr_of("typeof x") {
        assert_eq!(operator, UnaryOp::Typeof);
    } else { panic!("expected typeof"); }
}

#[test]
fn prefix_increment() {
    if let Expr::Update { operator, prefix, .. } = expr_of("++x") {
        assert_eq!(operator, UpdateOp::Inc);
        assert!(prefix);
    } else { panic!("expected update"); }
}

// ─────────── Binary precedence ───────────

#[test]
fn additive() {
    if let Expr::Binary { operator, .. } = expr_of("1 + 2") {
        assert_eq!(operator, BinaryOp::Add);
    } else { panic!("expected binary"); }
}

#[test]
fn precedence_climbing_correct() {
    // 1 + 2 * 3 → 1 + (2 * 3)
    if let Expr::Binary { operator, left, right, .. } = expr_of("1 + 2 * 3") {
        assert_eq!(operator, BinaryOp::Add);
        assert!(matches!(*left, Expr::NumberLiteral { value, .. } if value == 1.0));
        if let Expr::Binary { operator: op2, .. } = *right {
            assert_eq!(op2, BinaryOp::Mul);
        } else { panic!("expected nested binary"); }
    } else { panic!("expected binary"); }
}

#[test]
fn left_associativity() {
    // 1 - 2 - 3 → (1 - 2) - 3
    if let Expr::Binary { operator, left, .. } = expr_of("1 - 2 - 3") {
        assert_eq!(operator, BinaryOp::Sub);
        assert!(matches!(*left, Expr::Binary { operator: BinaryOp::Sub, .. }));
    } else { panic!("expected binary"); }
}

#[test]
fn exponentiation_right_associative() {
    // 2 ** 3 ** 2 → 2 ** (3 ** 2)
    if let Expr::Binary { operator, right, .. } = expr_of("2 ** 3 ** 2") {
        assert_eq!(operator, BinaryOp::Pow);
        assert!(matches!(*right, Expr::Binary { operator: BinaryOp::Pow, .. }));
    } else { panic!("expected binary"); }
}

#[test]
fn comparison_then_logical() {
    // a < b && c < d → (a<b) && (c<d)
    if let Expr::Binary { operator, .. } = expr_of("a < b && c < d") {
        assert_eq!(operator, BinaryOp::LogicalAnd);
    } else { panic!("expected logical and"); }
}

#[test]
fn nullish_coalesce() {
    if let Expr::Binary { operator, .. } = expr_of("a ?? b") {
        assert_eq!(operator, BinaryOp::NullishCoalesce);
    } else { panic!("expected nullish"); }
}

#[test]
fn instanceof_operator() {
    if let Expr::Binary { operator, .. } = expr_of("x instanceof Foo") {
        assert_eq!(operator, BinaryOp::Instanceof);
    } else { panic!("expected instanceof"); }
}

// ─────────── Conditional + Assignment ───────────

#[test]
fn conditional_expression() {
    if let Expr::Conditional { .. } = expr_of("a ? b : c") {
        // ok
    } else { panic!("expected conditional"); }
}

#[test]
fn assignment_basic() {
    if let Expr::Assign { operator, .. } = expr_of("x = 1") {
        assert_eq!(operator, AssignOp::Assign);
    } else { panic!("expected assignment"); }
}

#[test]
fn compound_assignment() {
    if let Expr::Assign { operator, .. } = expr_of("x += 1") {
        assert_eq!(operator, AssignOp::AddAssign);
    } else { panic!("expected compound assignment"); }
}

#[test]
fn logical_assignment() {
    if let Expr::Assign { operator, .. } = expr_of("x ??= 1") {
        assert_eq!(operator, AssignOp::NullishAssign);
    } else { panic!("expected nullish assign"); }
}

// ─────────── Array + Object literals ───────────

#[test]
fn array_literal() {
    if let Expr::Array { elements, .. } = expr_of("[1, 2, 3]") {
        assert_eq!(elements.len(), 3);
    } else { panic!("expected array"); }
}

#[test]
fn array_with_elision() {
    if let Expr::Array { elements, .. } = expr_of("[1, , 3]") {
        // 1, elision, 3
        assert_eq!(elements.len(), 3);
        assert!(matches!(&elements[1], ArrayElement::Elision { .. }));
    } else { panic!("expected array"); }
}

#[test]
fn array_with_spread() {
    if let Expr::Array { elements, .. } = expr_of("[1, ...rest]") {
        assert_eq!(elements.len(), 2);
        assert!(matches!(&elements[1], ArrayElement::Spread { .. }));
    } else { panic!("expected array"); }
}

#[test]
fn object_literal() {
    if let Expr::Object { properties, .. } = expr_of("{ a: 1, b: 2 }") {
        assert_eq!(properties.len(), 2);
    } else { panic!("expected object"); }
}

#[test]
fn object_shorthand() {
    if let Expr::Object { properties, .. } = expr_of("{ x, y }") {
        assert_eq!(properties.len(), 2);
        if let ObjectProperty::Property { shorthand, .. } = &properties[0] {
            assert!(*shorthand);
        } else { panic!("expected shorthand property"); }
    } else { panic!("expected object"); }
}

#[test]
fn object_with_spread() {
    if let Expr::Object { properties, .. } = expr_of("{ ...rest, x: 1 }") {
        assert_eq!(properties.len(), 2);
        assert!(matches!(&properties[0], ObjectProperty::Spread { .. }));
    } else { panic!("expected object"); }
}

#[test]
fn object_string_key() {
    if let Expr::Object { properties, .. } = expr_of(r#"{ "key": 1 }"#) {
        if let ObjectProperty::Property { key, .. } = &properties[0] {
            assert!(matches!(key, ObjectKey::String { .. }));
        } else { panic!(); }
    } else { panic!("expected object"); }
}

#[test]
fn object_computed_key() {
    if let Expr::Object { properties, .. } = expr_of("{ [k]: 1 }") {
        if let ObjectProperty::Property { key, .. } = &properties[0] {
            assert!(matches!(key, ObjectKey::Computed { .. }));
        } else { panic!(); }
    } else { panic!("expected object"); }
}

// ─────────── Parenthesized + Sequence ───────────

#[test]
fn parenthesized() {
    if let Expr::Parenthesized { .. } = expr_of("(x + 1)") {
        // ok
    } else { panic!("expected parenthesized"); }
}

// Note: Sequence-at-toplevel (a, b, c) without parens would conflict with
// AssignmentExpression-only contexts; tested via parenthesized sequence.
#[test]
fn parenthesized_sequence() {
    if let Expr::Parenthesized { expr, .. } = expr_of("(a, b, c)") {
        assert!(matches!(*expr, Expr::Sequence { .. }));
    } else { panic!("expected parenthesized sequence"); }
}

// ─────────── Opaque fallback ───────────

#[test]
fn function_expression_falls_back_opaque() {
    // FunctionExpression in expression position falls back to Opaque.
    // `export default function () {}` would route to HoistableFunction in
    // parse_default_export_body; the parenthesized form hits the expression
    // path proper.
    if let Expr::Parenthesized { expr, .. } = expr_of("(function () {})") {
        assert!(matches!(*expr, Expr::Opaque { .. }));
    } else { panic!("expected parenthesized opaque"); }
}

#[test]
fn arrow_function_falls_back_opaque() {
    if let Expr::Opaque { .. } = expr_of("x => x") {
        // ok — ArrowFunction deferred
    } else { panic!("expected opaque"); }
}
