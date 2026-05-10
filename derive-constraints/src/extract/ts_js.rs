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
use std::collections::HashMap;
use tree_sitter::{Node, Parser};

/// Per-test scope for `const|let|var X = expr` bindings. Used to substitute
/// the initializer expression for the variable name when canonicalizing
/// expect-subjects: `const server = Bun.serve(opts); expect(server.port).toBe(3000)`
/// canonicalizes the subject `server.port` to `Bun.serve(opts).port` so the
/// architectural surface (Bun.serve) is visible to the cluster phase.
///
/// Single flat scope per test body — does not honor nested block scoping.
/// Most test bodies are flat enough that the loss of fidelity is small.
type BindingMap = HashMap<String, String>;

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
                        let mut bindings: BindingMap = HashMap::new();
                        walk_test_body(&body, src, &mut bindings, &mut constraints);
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
    // Special-case: Deno's `Deno.test(name, fn)` and `Deno.test({name, fn})`
    // shape. The callee is a member_expression with object="Deno" and
    // property="test"; the test API has no `it` / `describe` analogues
    // (Deno uses subTest grouping inside the test body), so we treat
    // `Deno.test` as a Test role with no Describe variant.
    let is_deno_test = head == "Deno" && suffix.as_deref() == Some("test");
    let role = if TEST_NAMES.contains(&head.as_str()) || is_deno_test {
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
            "object" => {
                // Deno.test({name: "...", fn: () => {...}}) form.
                // Walk the object's pair children for `name:` and `fn:`.
                let (n, b) = extract_options_object(&arg, src);
                if name.is_none() {
                    name = n;
                }
                if body.is_none() {
                    body = b;
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

/// Extract `(name, body)` from a Deno.test options object: walks the
/// object's `pair` children, returning the value of any `name:` field
/// (string literal) and the value of any `fn:` field (function/arrow).
fn extract_options_object<'tree>(
    obj: &Node<'tree>,
    src: &[u8],
) -> (Option<String>, Option<Node<'tree>>) {
    let mut name = None;
    let mut body = None;
    let mut cursor = obj.walk();
    for pair in obj.children(&mut cursor) {
        if pair.kind() != "pair" {
            continue;
        }
        let key = pair.child_by_field_name("key");
        let value = pair.child_by_field_name("value");
        let (Some(k), Some(v)) = (key, value) else {
            continue;
        };
        let key_text = text(&k, src);
        let unquoted = strip_string_quotes(&key_text);
        match unquoted.as_str() {
            "name" => {
                if name.is_none() && matches!(v.kind(), "string" | "template_string") {
                    name = Some(string_literal_text(&v, src));
                }
            }
            "fn" => {
                if body.is_none()
                    && matches!(
                        v.kind(),
                        "arrow_function"
                            | "function"
                            | "function_expression"
                            | "async_function"
                            | "function_declaration"
                    )
                {
                    body = v.child_by_field_name("body").or(Some(v));
                }
            }
            _ => {}
        }
    }
    (name, body)
}

fn strip_string_quotes(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 {
        let first = s.chars().next().unwrap();
        let last = s.chars().last().unwrap();
        if (first == '"' || first == '\'' || first == '`') && first == last {
            return s[1..s.len() - 1].to_string();
        }
    }
    s.to_string()
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

/// Walk a test body in source order. Captures `const|let|var` bindings
/// into `bindings` before they are referenced (preorder traversal) so
/// that subject substitution operates on identifiers introduced earlier
/// in the same scope.
fn walk_test_body(
    node: &Node,
    src: &[u8],
    bindings: &mut BindingMap,
    out: &mut Vec<ConstraintClause>,
) {
    match node.kind() {
        "lexical_declaration" | "variable_declaration" => {
            capture_bindings(node, src, bindings);
        }
        "call_expression" => {
            if let Some(c) = classify_constraint(node, src, bindings) {
                out.push(c);
            }
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_test_body(&child, src, bindings, out);
    }
}

/// Extract `name = value` pairs from a `const|let|var` declaration node.
/// Multi-binding declarations (`const a = 1, b = 2`) are handled because
/// each `variable_declarator` is captured independently.
fn capture_bindings(node: &Node, src: &[u8], bindings: &mut BindingMap) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() != "variable_declarator" {
            continue;
        }
        let name_node = child.child_by_field_name("name");
        let value_node = child.child_by_field_name("value");
        if let (Some(name), Some(value)) = (name_node, value_node) {
            if name.kind() == "identifier" {
                let name_text = text(&name, src);
                let value_text = collapse(&text(&value, src));
                // Only record bindings whose RHS is a simple call/member
                // chain — `new Foo()`, `Bun.file(p).text()`,
                // `await x.y()`. Reject RHS values containing top-level
                // binary operators or boolean expressions: those are
                // computed values whose later use as a subject would
                // mis-attribute architectural identity (the TextDecoder
                // false-positive seen in the TextEncoder pilot:
                // `const headerEnd = new TextDecoder()…> 0;
                //  assert(headerEnd > 0)` would substitute the
                // TextDecoder-bearing RHS into the unrelated assertion).
                if !is_simple_call_chain(&value_text) {
                    continue;
                }
                // Don't overwrite an existing binding with later one in
                // the same scope — first definition wins. This matches
                // the common test pattern where setup-time bindings
                // dominate and rebindings are uncommon.
                bindings.entry(name_text).or_insert(value_text);
            }
        }
    }
}

/// If the leading identifier of `s` is bound, splice the binding's value
/// in for that identifier. Operates on the textual prefix only; preserves
/// trailing `.member`/`[...]`/etc. text. First strips common prefix
/// keywords (`await `, `new `, `typeof `) so `await server.exited` resolves
/// the underlying binding rather than treating `await` as the head.
fn resolve_binding(s: &str, bindings: &BindingMap) -> String {
    if bindings.is_empty() {
        return s.to_string();
    }
    // Only substitute when the input is itself a simple call/member chain.
    // If the input contains a comparison (`headerEnd > 0`), arithmetic
    // (`offset + 4`), or boolean operator, the bound value is being used
    // as a scalar, not as the architectural object — substituting the
    // chain head misattributes the subject (the TextDecoder false-positive
    // from the TextEncoder pilot).
    if !is_simple_call_chain(s) {
        return s.to_string();
    }
    let mut work: &str = s;
    loop {
        let trimmed = work.trim_start();
        if let Some(rest) = trimmed.strip_prefix("await ") {
            work = rest;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("new ") {
            work = rest;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("typeof ") {
            work = rest;
            continue;
        }
        break;
    }
    let trimmed = work.trim_start();
    let leading = take_identifier_prefix(trimmed);
    if leading.is_empty() {
        return s.to_string();
    }
    let head = leading.split('.').next().unwrap_or("");
    if let Some(replacement) = bindings.get(head) {
        let prefix_len = head.len();
        let rest = &trimmed[prefix_len..];
        // Only substitute when the trailing tail after the bound head
        // does not invoke a method call (`.method(...)`). A method call
        // shifts the architectural identity to the method's return-type,
        // not the binding's chain head — substituting would attribute the
        // assertion to the wrong surface (the TextDecoder false-positive
        // pattern: `const r = new TextDecoder().decode(buf);
        //  assert(r.startsWith("..."))` — `.startsWith` is a String
        // method, not a TextDecoder property). Bare-identifier tails and
        // pure-getter tails (`.url`, `.pathname`) remain substitutable.
        if rest.contains('(') {
            return s.to_string();
        }
        return format!("{}{}", replacement, rest);
    }
    work.to_string()
}

/// Return the longest leading prefix that looks like an identifier-path
/// `[A-Za-z_$][A-Za-z0-9_$]*(\.[A-Za-z_$][A-Za-z0-9_$]*)*`. Used to find
/// the head identifier of a possibly-substitutable subject text.
fn take_identifier_prefix(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut end = 0;
    let mut state = 0u8; // 0 start, 1 in_ident, 2 after_dot
    let mut last_id_end = 0;
    while end < bytes.len() {
        let c = bytes[end];
        match state {
            0 | 2 => {
                if matches!(c, b'_' | b'$' | b'A'..=b'Z' | b'a'..=b'z') {
                    state = 1;
                    last_id_end = end + 1;
                } else {
                    break;
                }
            }
            1 => {
                if matches!(c, b'_' | b'$' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z') {
                    last_id_end = end + 1;
                } else if c == b'.' {
                    state = 2;
                } else {
                    break;
                }
            }
            _ => break,
        }
        end += 1;
    }
    s[..last_id_end].to_string()
}

/// True if `value` looks like a pure call/member chain — i.e. an identifier
/// path optionally followed by balanced `(...)`, `[...]`, and `.member` tails,
/// optionally preceded by `await `, `new `, `typeof `. Rejects values
/// containing top-level binary operators (`>`, `<`, `==`, `&&`, `?`, `+`, …)
/// because such RHS values are computed booleans/numbers whose architectural
/// identity is not the prefix call expression they happen to contain.
fn is_simple_call_chain(value: &str) -> bool {
    let mut s = value.trim();
    loop {
        let t = s.trim_start();
        if let Some(r) = t.strip_prefix("await ") { s = r; continue; }
        if let Some(r) = t.strip_prefix("new ") { s = r; continue; }
        if let Some(r) = t.strip_prefix("typeof ") { s = r; continue; }
        s = t;
        break;
    }
    let bytes = s.as_bytes();
    if bytes.is_empty() { return false; }
    let is_id_start = |c: u8| matches!(c, b'_' | b'$' | b'A'..=b'Z' | b'a'..=b'z');
    let is_id_cont  = |c: u8| matches!(c, b'_' | b'$' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z');
    if !is_id_start(bytes[0]) { return false; }
    let mut i = 0;
    while i < bytes.len() && (is_id_cont(bytes[i]) || bytes[i] == b'.') {
        // Disallow consecutive dots (would mean an operator-like construct).
        if bytes[i] == b'.' && (i + 1 >= bytes.len() || !is_id_start(bytes[i + 1])) {
            break;
        }
        i += 1;
    }
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b'(' | b'[' => {
                let close = if c == b'(' { b')' } else { b']' };
                let open = c;
                let mut depth = 1i32;
                i += 1;
                let mut in_str: Option<u8> = None;
                let mut prev_back = false;
                while i < bytes.len() && depth > 0 {
                    let cc = bytes[i];
                    if let Some(q) = in_str {
                        if cc == b'\\' && !prev_back { prev_back = true; i += 1; continue; }
                        if cc == q && !prev_back { in_str = None; }
                        prev_back = false;
                    } else if matches!(cc, b'"' | b'\'' | b'`') {
                        in_str = Some(cc);
                    } else if cc == open { depth += 1; }
                    else if cc == close { depth -= 1; }
                    i += 1;
                }
                if depth != 0 { return false; }
            }
            b'.' => {
                i += 1;
                let start = i;
                while i < bytes.len() && is_id_cont(bytes[i]) { i += 1; }
                if i == start { return false; }
            }
            b' ' | b'\t' | b'\n' | b'\r' => { i += 1; }
            _ => return false,
        }
    }
    true
}

fn classify_constraint(node: &Node, src: &[u8], bindings: &BindingMap) -> Option<ConstraintClause> {
    let func = node.child_by_field_name("function")?;
    // Detect `expect(x).toY(z)` — a member_expression whose deepest object
    // is itself a call to `expect`.
    if func.kind() == "member_expression" {
        if let Some(object) = func.child_by_field_name("object") {
            if let Some(inner_func) = object.child_by_field_name("function") {
                if text(&inner_func, src) == "expect" {
                    let raw = collapse(&text(node, src));
                    let subject = expect_subject(&object, src, bindings);
                    return Some(ConstraintClause {
                        line: node.start_position().row as u32 + 1,
                        raw,
                        kind: ConstraintKind::ExpectChain,
                        subject,
                    });
                }
            }
            // `assert.equal(x, y)` style. Subject is the first argument
            // of the call — the value being asserted on — rather than
            // the assert.* function itself, which is just the test
            // framework. With first-arg extraction the architectural
            // surface becomes visible upstream.
            if text(&object, src) == "assert" {
                let raw = collapse(&text(node, src));
                let subject = first_call_arg_subject(node, src, bindings);
                return Some(ConstraintClause {
                    line: node.start_position().row as u32 + 1,
                    raw,
                    kind: ConstraintKind::AssertCall,
                    subject,
                });
            }
        }
    }
    if func.kind() == "identifier" {
        let head = text(&func, src);
        if ASSERT_NAMES.contains(&head.as_str()) {
            let raw = collapse(&text(node, src));
            let subject = first_call_arg_subject(node, src, bindings);
            return Some(ConstraintClause {
                line: node.start_position().row as u32 + 1,
                raw,
                kind: ConstraintKind::AssertCall,
                subject,
            });
        }
    }
    None
}

/// For `assert(x, ...)`, `assert.equal(a, b, ...)`, etc., return the
/// first call argument as the subject (the value being asserted on),
/// resolved through the binding map.
fn first_call_arg_subject(node: &Node, src: &[u8], bindings: &BindingMap) -> Option<String> {
    let args = node.child_by_field_name("arguments")?;
    let mut cursor = args.walk();
    for arg in args.children(&mut cursor) {
        match arg.kind() {
            "(" | ")" | "," => continue,
            _ => {
                let raw = collapse(&text(&arg, src));
                return Some(resolve_binding(&raw, bindings));
            }
        }
    }
    None
}

fn expect_subject(object: &Node, src: &[u8], bindings: &BindingMap) -> Option<String> {
    // object is the `expect(x)` call_expression. Subject is the first
    // argument of that call. We resolve any leading binding so the
    // architectural surface (e.g. Bun.serve) becomes visible upstream
    // instead of the local variable name.
    let args = object.child_by_field_name("arguments")?;
    let mut cursor = args.walk();
    for arg in args.children(&mut cursor) {
        match arg.kind() {
            "(" | ")" | "," => continue,
            _ => {
                let raw = collapse(&text(&arg, src));
                return Some(resolve_binding(&raw, bindings));
            }
        }
    }
    None
}

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
