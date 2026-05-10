//! Rust test-block extractor. Walks the AST via syn 2 and pulls out:
//!   - Functions annotated with `#[test]` (top-level fns, impl methods, trait fns)
//!   - The assertion macros invoked inside each test body
//!   - `#[ignore]` markers
//!
//! The output mirrors the shared `TestFile` shape so the downstream
//! cluster/invert/predict phases consume the unified structure regardless
//! of source language.

use super::{ConstraintClause, ConstraintKind, Language, TestCase, TestFile, TestKind};
use anyhow::Result;
use syn::visit::{self, Visit};
use syn::{Attribute, ImplItemFn, ItemFn, TraitItemFn};

const ASSERT_MACROS: &[&str] = &[
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "panic",
    "unreachable",
    "todo",
];

pub fn extract(path: &str, src: &str) -> Result<TestFile> {
    let file = match syn::parse_file(src) {
        Ok(f) => f,
        Err(e) => {
            return Ok(TestFile {
                path: path.to_string(),
                language: Language::Rust,
                loc: src.lines().count() as u32,
                tests: Vec::new(),
                parse_failure: Some(format!("syn parse error: {}", e)),
            });
        }
    };
    let mut v = Visitor {
        tests: Vec::new(),
    };
    v.visit_file(&file);
    Ok(TestFile {
        path: path.to_string(),
        language: Language::Rust,
        loc: src.lines().count() as u32,
        tests: v.tests,
        parse_failure: None,
    })
}

struct Visitor {
    tests: Vec<TestCase>,
}

impl Visitor {
    fn try_record_fn(&mut self, name: &str, attrs: &[Attribute], block: &syn::Block, kind_span: proc_macro2::Span) {
        if !is_test_fn(attrs) {
            return;
        }
        let span = kind_span;
        let line_start = span.start().line as u32;
        // syn's spans for the fn signature don't reach the closing brace; use the
        // block's brace-token span for the end line.
        let line_end = block.brace_token.span.close().end().line as u32;
        let mut constraints = Vec::new();
        let mut clause_visitor = ClauseVisitor {
            out: &mut constraints,
        };
        clause_visitor.visit_block(block);
        self.tests.push(TestCase {
            name: name.to_string(),
            kind: TestKind::Test,
            line_start,
            line_end,
            constraints,
            skip: has_attr(attrs, "ignore"),
            todo: false,
            failing: false,
        });
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.try_record_fn(
            &node.sig.ident.to_string(),
            &node.attrs,
            &node.block,
            node.sig.fn_token.span,
        );
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        self.try_record_fn(
            &node.sig.ident.to_string(),
            &node.attrs,
            &node.block,
            node.sig.fn_token.span,
        );
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast TraitItemFn) {
        if let Some(ref block) = node.default {
            self.try_record_fn(
                &node.sig.ident.to_string(),
                &node.attrs,
                block,
                node.sig.fn_token.span,
            );
        }
        visit::visit_trait_item_fn(self, node);
    }
}

/// Walks a test fn's block looking for assertion macro calls. Records
/// each as a ConstraintClause with the raw token-stream rendering.
struct ClauseVisitor<'a> {
    out: &'a mut Vec<ConstraintClause>,
}

impl<'ast, 'a> Visit<'ast> for ClauseVisitor<'a> {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let macro_name = mac
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        if ASSERT_MACROS.iter().any(|n| *n == macro_name) {
            let span = mac.path.span();
            let line = span.start().line as u32;
            let raw = format!("{}!({})", macro_name, mac.tokens.to_string());
            // Heuristic subject: first identifier in the macro tokens.
            let subject = first_identifier(&mac.tokens.to_string());
            let authority_tier = crate::extract::classify_authority_tier(
                subject.as_deref(),
                ConstraintKind::AssertMacro,
            );
            self.out.push(ConstraintClause {
                line,
                raw: collapse(&raw),
                kind: ConstraintKind::AssertMacro,
                subject,
                authority_tier,
            });
        }
        visit::visit_macro(self, mac);
    }
}

fn is_test_fn(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path()
            .segments
            .last()
            .map(|s| s.ident == "test")
            .unwrap_or(false)
    })
}

fn has_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|a| {
        a.path()
            .segments
            .last()
            .map(|s| s.ident == name)
            .unwrap_or(false)
    })
}

fn first_identifier(s: &str) -> Option<String> {
    let trimmed = s.trim_start();
    let end = trimmed
        .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':'))
        .unwrap_or(trimmed.len());
    if end == 0 {
        None
    } else {
        Some(trimmed[..end].to_string())
    }
}

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// `proc_macro2::Span::span` extension shim — syn doesn't reexport `Spanned` here
// in a way that's free of feature flags, so we provide a thin trait import.
trait HasSpan {
    fn span(&self) -> proc_macro2::Span;
}
impl HasSpan for syn::Path {
    fn span(&self) -> proc_macro2::Span {
        self.segments
            .first()
            .map(|s| s.ident.span())
            .unwrap_or_else(proc_macro2::Span::call_site)
    }
}
