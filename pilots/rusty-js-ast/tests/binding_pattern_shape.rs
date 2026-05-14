//! Tier-Ω.5.g.2 substrate tests for BindingPattern.
//!
//! Asserts that `BindingPattern::collect_names()` walks the pattern and
//! returns each introduced BindingIdentifier in source order. These tests
//! exercise the pattern constructors directly (no parser dependency); the
//! parser-population tests live in rusty-js-parser/derived/tests.

use rusty_js_ast::*;

fn id(name: &str) -> BindingIdentifier {
    BindingIdentifier { name: name.to_string(), span: Span::new(0, 0) }
}

fn elem(target: BindingPattern) -> BindingElement {
    BindingElement { target, default: None, span: Span::new(0, 0) }
}

fn elem_with_default(target: BindingPattern, default: Expr) -> BindingElement {
    BindingElement { target, default: Some(default), span: Span::new(0, 0) }
}

fn names_of(pat: &BindingPattern) -> Vec<String> {
    pat.collect_names().into_iter().map(|n| n.name.clone()).collect()
}

#[test]
fn t01_identifier_collect_names() {
    let p = BindingPattern::Identifier(id("x"));
    assert_eq!(names_of(&p), vec!["x"]);
}

#[test]
fn t02_simple_array_pattern() {
    // `[a, b]`
    let p = BindingPattern::Array(ArrayPattern {
        elements: vec![
            Some(elem(BindingPattern::Identifier(id("a")))),
            Some(elem(BindingPattern::Identifier(id("b")))),
        ],
        rest: None,
        span: Span::new(0, 0),
    });
    assert_eq!(names_of(&p), vec!["a", "b"]);
}

#[test]
fn t03_array_pattern_with_rest() {
    // `[a, ...rest]`
    let p = BindingPattern::Array(ArrayPattern {
        elements: vec![Some(elem(BindingPattern::Identifier(id("a"))))],
        rest: Some(Box::new(BindingPattern::Identifier(id("rest")))),
        span: Span::new(0, 0),
    });
    assert_eq!(names_of(&p), vec!["a", "rest"]);
}

#[test]
fn t04_array_pattern_with_elision() {
    // `[a,,b]`
    let p = BindingPattern::Array(ArrayPattern {
        elements: vec![
            Some(elem(BindingPattern::Identifier(id("a")))),
            None,
            Some(elem(BindingPattern::Identifier(id("b")))),
        ],
        rest: None,
        span: Span::new(0, 0),
    });
    assert_eq!(names_of(&p), vec!["a", "b"]);
}

#[test]
fn t05_array_pattern_with_default() {
    // `[a = 99]`
    let default = Expr::NumberLiteral { value: 99.0, span: Span::new(0, 0) };
    let p = BindingPattern::Array(ArrayPattern {
        elements: vec![Some(elem_with_default(BindingPattern::Identifier(id("a")), default))],
        rest: None,
        span: Span::new(0, 0),
    });
    assert_eq!(names_of(&p), vec!["a"]);
}

#[test]
fn t06_simple_object_pattern() {
    // `{x, y}` (shorthand)
    let p = BindingPattern::Object(ObjectPattern {
        properties: vec![
            ObjectPatternProperty {
                key: PropertyKey::Identifier(id("x")),
                value: elem(BindingPattern::Identifier(id("x"))),
                shorthand: true,
                span: Span::new(0, 0),
            },
            ObjectPatternProperty {
                key: PropertyKey::Identifier(id("y")),
                value: elem(BindingPattern::Identifier(id("y"))),
                shorthand: true,
                span: Span::new(0, 0),
            },
        ],
        rest: None,
        span: Span::new(0, 0),
    });
    assert_eq!(names_of(&p), vec!["x", "y"]);
}

#[test]
fn t07_object_pattern_with_rename() {
    // `{a: alias}` — local binding is `alias`, not `a`.
    let p = BindingPattern::Object(ObjectPattern {
        properties: vec![ObjectPatternProperty {
            key: PropertyKey::Identifier(id("a")),
            value: elem(BindingPattern::Identifier(id("alias"))),
            shorthand: false,
            span: Span::new(0, 0),
        }],
        rest: None,
        span: Span::new(0, 0),
    });
    assert_eq!(names_of(&p), vec!["alias"]);
}

#[test]
fn t08_object_pattern_with_rest() {
    // `{a, ...rest}`
    let p = BindingPattern::Object(ObjectPattern {
        properties: vec![ObjectPatternProperty {
            key: PropertyKey::Identifier(id("a")),
            value: elem(BindingPattern::Identifier(id("a"))),
            shorthand: true,
            span: Span::new(0, 0),
        }],
        rest: Some(Box::new(id("rest"))),
        span: Span::new(0, 0),
    });
    assert_eq!(names_of(&p), vec!["a", "rest"]);
}

#[test]
fn t09_nested_pattern() {
    // `[a, {b, c: d}]` — nested object inside array.
    let p = BindingPattern::Array(ArrayPattern {
        elements: vec![
            Some(elem(BindingPattern::Identifier(id("a")))),
            Some(elem(BindingPattern::Object(ObjectPattern {
                properties: vec![
                    ObjectPatternProperty {
                        key: PropertyKey::Identifier(id("b")),
                        value: elem(BindingPattern::Identifier(id("b"))),
                        shorthand: true,
                        span: Span::new(0, 0),
                    },
                    ObjectPatternProperty {
                        key: PropertyKey::Identifier(id("c")),
                        value: elem(BindingPattern::Identifier(id("d"))),
                        shorthand: false,
                        span: Span::new(0, 0),
                    },
                ],
                rest: None,
                span: Span::new(0, 0),
            }))),
        ],
        rest: None,
        span: Span::new(0, 0),
    });
    assert_eq!(names_of(&p), vec!["a", "b", "d"]);
}
