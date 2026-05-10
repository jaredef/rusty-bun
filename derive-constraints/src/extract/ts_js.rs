//! TypeScript / JavaScript test-block extractor via tree-sitter.
//!
//! Targets the `bun:test` / Jest / Vitest API surface: `describe`, `test`,
//! `it`, `expect(x).toBe(y)`, `assert(...)`, `assert.equal(...)`. For each
//! test/it call, walks the body to collect constraint clauses; tracks the
//! enclosing describe-chain so test names reflect their full context.
//!
//! The extractor is structural — it identifies call_expression nodes
//! whose function name matches the test API — and accepts both arrow-fn
//! and function-expression test bodies. It does not resolve identifier
//! aliases (`import { test as t }` → `t(...)` is not detected); the
//! universe of name-shapes seen in Bun's test corpus is narrow enough
//! that the simple match suffices for the MVP.

use super::{ConstraintClause, ConstraintKind, Language, TestCase, TestFile, TestKind};
use anyhow::Result;
use tree_sitter::{Node, Parser};

const TEST_NAMES: &[&str] = &["test", "it"];
const DESCRIBE_NAMES: &[&str] = &["describe"];
const SKIP_SUFFIXES: &[&str] = &["skip"];
const TODO_SUFFIXES: &[&str] = &["todo"];
const FAILING_SUFFIXES: &[&str] = &["failing", "fails"];
const ASSERT_NAMES: &[&str] = &["assert", "assertEquals", "assertEqual", "assertNotEqual", "ok"];

pub fn extract(path: &str, src: &str, lang: Language) -> Result<TestFile> {
    let mut parser = Parser::new();
    let language = match lang {
        Language::TypeScript => tree_sitter_typescript::language_typescript(),
        Language::JavaScript => tree_sitter_javascript::language(),
        _ => unreachable!("ts_js extractor invoked for non-JS language"),
    };
    parser
        .set_language(&language)
        .map_err(|e| anyhow::anyhow!("set_language: {}", e))?;
    let tree = match parser.parse(src, None) {
        Some(t) => t,
        None => {
            return Ok(TestFile {
                path: path.to_string(),
                language: lang,
                loc: src.lines().count() as u32,
                tests: Vec::new(),
                parse_failure: Some("tree-sitter returned None".to_string()),
            });
        }
    };

    let mut tests = Vec::new();
    walk(&tree.root_node(), src.as_bytes(), Vec::new(), &mut tests);

    Ok(TestFile {
        path: path.to_string(),
        language: lang,
        loc: src.lines().count() as u32,
        tests,
        parse_failure: None,
    })
}

fn walk(node: &Node, src: &[u8], scope: Vec<String>, out: &mut Vec<TestCase>) {
    if node.kind() == "call_expression" {
        if let Some(call) = classify_call(node, src) {
            match call.role {
                CallRole::Describe => {
                    let mut new_scope = scope.clone();
                    if let Some(name) = call.name {
                        new_scope.push(name);
                    }
                    if let Some(body) = call.body {
                        walk(&body, src, new_scope, out);
                        return;
                    }
                }
                CallRole::Test => {
                    let mut name_parts = scope.clone();
                    if let Some(n) = call.name {
                        name_parts.push(n);
                    }
                    let name = name_parts.join(" > ");
                    let line_start = node.start_position().row as u32 + 1;
                    let line_end = node.end_position().row as u32 + 1;
                    let mut constraints = Vec::new();
                    if let Some(body) = call.body {
                        collect_constraints(&body, src, &mut constraints);
                    }
                    out.push(TestCase {
                        name,
                        kind: if call.is_it {
                            TestKind::It
                        } else {
                            TestKind::Test
                        },
                        line_start,
                        line_end,
                        constraints,
                        skip: call.skip,
                        todo: call.todo,
                        failing: call.failing,
                    });
                    return;
                }
                CallRole::Other => {}
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(&child, src, scope.clone(), out);
    }
}

#[derive(Default)]
struct ClassifiedCall<'tree> {
    role: CallRole,
    name: Option<String>,
    body: Option<Node<'tree>>,
    is_it: bool,
    skip: bool,
    todo: bool,
    failing: bool,
}

#[derive(Default, PartialEq, Eq)]
enum CallRole {
    Test,
    Describe,
    #[default]
    Other,
}

fn classify_call<'tree>(node: &Node<'tree>, src: &[u8]) -> Option<ClassifiedCall<'tree>> {
    let func = node.child_by_field_name("function")?;
    let (head, suffix) = decompose_callee(&func, src);
    let role = if TEST_NAMES.contains(&head.as_str()) {
        CallRole::Test
    } else if DESCRIBE_NAMES.contains(&head.as_str()) {
        CallRole::Describe
    } else {
        return Some(ClassifiedCall::default());
    };

    let skip = suffix
        .as_deref()
        .map(|s| SKIP_SUFFIXES.contains(&s))
        .unwrap_or(false);
    let todo = suffix
        .as_deref()
        .map(|s| TODO_SUFFIXES.contains(&s))
        .unwrap_or(false);
    let failing = suffix
        .as_deref()
        .map(|s| FAILING_SUFFIXES.contains(&s))
        .unwrap_or(false);

    let args = node.child_by_field_name("arguments")?;
    let mut name = None;
    let mut body = None;
    let mut cursor = args.walk();
    for arg in args.children(&mut cursor) {
        match arg.kind() {
            "string" | "template_string" => {
                if name.is_none() {
                    name = Some(string_literal_text(&arg, src));
                }
            }
            "arrow_function" | "function" | "function_expression" | "async_function" | "function_declaration" => {
                if body.is_none() {
                    body = arg.child_by_field_name("body").or(Some(arg));
                }
            }
            _ => {}
        }
    }

    Some(ClassifiedCall {
        role,
        name,
        body,
        is_it: head == "it",
        skip,
        todo,
        failing,
    })
}

fn decompose_callee(node: &Node, src: &[u8]) -> (String, Option<String>) {
    match node.kind() {
        "identifier" | "property_identifier" => (text(node, src), None),
        "member_expression" => {
            // `test.skip` or `test.skip.failing` etc — flatten with the
            // first segment as head and the last as suffix.
            let object = node.child_by_field_name("object");
            let prop = node.child_by_field_name("property");
            let head = match object {
                Some(obj) => decompose_callee(&obj, src).0,
                None => String::new(),
            };
            let suffix = prop.map(|p| text(&p, src));
            (head, suffix)
        }
        _ => (text(node, src), None),
    }
}

fn text(node: &Node, src: &[u8]) -> String {
    src.get(node.byte_range())
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .unwrap_or_default()
}

fn string_literal_text(node: &Node, src: &[u8]) -> String {
    // Strip surrounding quotes; tree-sitter's string node includes them.
    let raw = text(node, src);
    let trimmed = raw.trim();
    if trimmed.len() >= 2 {
        let first = trimmed.chars().next().unwrap();
        let last = trimmed.chars().last().unwrap();
        if (first == '"' || first == '\'' || first == '`') && first == last {
            return trimmed[1..trimmed.len() - 1].to_string();
        }
    }
    raw
}

fn collect_constraints(node: &Node, src: &[u8], out: &mut Vec<ConstraintClause>) {
    if node.kind() == "call_expression" {
        if let Some(c) = classify_constraint(node, src) {
            out.push(c);
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_constraints(&child, src, out);
    }
}

fn classify_constraint(node: &Node, src: &[u8]) -> Option<ConstraintClause> {
    let func = node.child_by_field_name("function")?;
    // Detect `expect(x).toY(z)` — a member_expression whose deepest object
    // is itself a call to `expect`.
    if func.kind() == "member_expression" {
        if let Some(object) = func.child_by_field_name("object") {
            if let Some(inner_func) = object.child_by_field_name("function") {
                if text(&inner_func, src) == "expect" {
                    let raw = collapse(&text(node, src));
                    let subject = expect_subject(&object, src);
                    return Some(ConstraintClause {
                        line: node.start_position().row as u32 + 1,
                        raw,
                        kind: ConstraintKind::ExpectChain,
                        subject,
                    });
                }
            }
            // `assert.equal(x, y)` style.
            if text(&object, src) == "assert" {
                let raw = collapse(&text(node, src));
                return Some(ConstraintClause {
                    line: node.start_position().row as u32 + 1,
                    raw,
                    kind: ConstraintKind::AssertCall,
                    subject: None,
                });
            }
        }
    }
    if func.kind() == "identifier" {
        let head = text(&func, src);
        if ASSERT_NAMES.contains(&head.as_str()) {
            let raw = collapse(&text(node, src));
            return Some(ConstraintClause {
                line: node.start_position().row as u32 + 1,
                raw,
                kind: ConstraintKind::AssertCall,
                subject: None,
            });
        }
    }
    None
}

fn expect_subject(object: &Node, src: &[u8]) -> Option<String> {
    // object is the `expect(x)` call_expression. Subject is the first
    // argument of that call.
    let args = object.child_by_field_name("arguments")?;
    let mut cursor = args.walk();
    for arg in args.children(&mut cursor) {
        match arg.kind() {
            "(" | ")" | "," => continue,
            _ => return Some(collapse(&text(&arg, src))),
        }
    }
    None
}

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
