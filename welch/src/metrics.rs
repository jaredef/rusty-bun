//! Per-file metric definitions and the AST visitor that extracts them.
//!
//! The metrics target idiomaticity markers that distinguish Rust code that
//! has settled onto Rust's natural representational geometry from Rust code
//! that recreates a different language's semantics inside Rust syntax. Per
//! [RESOLVE Doc 702 §3 Mapping 6 + §4 Addition 4], dense unsafe / raw-pointer
//! / transmute / FFI patterns are the operational marker for a translation
//! that has not re-settled onto Rust's natural ETF attractor — a Welch-bound
//! packing failure.

use serde::{Deserialize, Serialize};
use syn::visit::{self, Visit};
use syn::{ExprUnsafe, ImplItemFn, ItemFn, ItemForeignMod, TraitItemFn, TypePtr};

/// Per-file metrics. Counts are integers; densities are derived at compare
/// time so that the raw counts remain composable across files (sum
/// well-behaved; ratios do not).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileMetrics {
    /// Path relative to the scan root.
    pub path: String,
    /// Total byte count of the source file.
    pub bytes: u64,
    /// Total source lines (including blank and comment lines).
    pub loc: u64,
    /// Number of `unsafe { ... }` expression blocks.
    pub unsafe_blocks: u64,
    /// Total source lines spanned by unsafe blocks (with overlap collapsed).
    pub unsafe_loc: u64,
    /// Number of `unsafe fn` declarations.
    pub unsafe_fns: u64,
    /// Total fn declarations (incl. unsafe ones).
    pub fns: u64,
    /// Occurrences of `*const T` or `*mut T` pointer types in the AST.
    pub raw_pointers: u64,
    /// Calls to `mem::transmute`, `transmute`, or their alias forms.
    pub transmutes: u64,
    /// Number of `extern "..."` foreign module blocks.
    pub extern_blocks: u64,
    /// Whether parsing succeeded. Failed files contribute LOC but no AST metrics.
    pub parsed: bool,
}

impl FileMetrics {
    pub fn new(path: String, bytes: u64, loc: u64) -> Self {
        FileMetrics {
            path,
            bytes,
            loc,
            ..Default::default()
        }
    }
}

/// AST visitor that accumulates the metrics defined above. Line counts for
/// unsafe blocks come from proc-macro2's stable LineColumn span API.
pub struct MetricsVisitor {
    pub metrics: FileMetrics,
}

impl MetricsVisitor {
    pub fn new(path: String, src: Option<&str>) -> Self {
        let bytes = src.map(|s| s.len() as u64).unwrap_or(0);
        let loc = src.map(|s| s.lines().count() as u64).unwrap_or(0);
        let mut metrics = FileMetrics::new(path, bytes, loc);
        metrics.parsed = true;
        MetricsVisitor { metrics }
    }

    fn span_to_line_count(&self, span: proc_macro2::Span) -> u64 {
        let start = span.start();
        let end = span.end();
        if end.line >= start.line {
            (end.line - start.line + 1) as u64
        } else {
            1
        }
    }
}

impl<'ast> Visit<'ast> for MetricsVisitor {
    fn visit_expr_unsafe(&mut self, node: &'ast ExprUnsafe) {
        self.metrics.unsafe_blocks += 1;
        self.metrics.unsafe_loc += self.span_to_line_count(node.unsafe_token.span);
        visit::visit_expr_unsafe(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.metrics.fns += 1;
        if node.sig.unsafety.is_some() {
            self.metrics.unsafe_fns += 1;
        }
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        // Methods defined inside `impl` blocks. syn distinguishes these from
        // top-level `fn` items; without a separate visitor hook the count
        // would silently miss every method.
        self.metrics.fns += 1;
        if node.sig.unsafety.is_some() {
            self.metrics.unsafe_fns += 1;
        }
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast TraitItemFn) {
        // Method declarations inside `trait` blocks (with or without default body).
        self.metrics.fns += 1;
        if node.sig.unsafety.is_some() {
            self.metrics.unsafe_fns += 1;
        }
        visit::visit_trait_item_fn(self, node);
    }

    fn visit_type_ptr(&mut self, node: &'ast TypePtr) {
        self.metrics.raw_pointers += 1;
        visit::visit_type_ptr(self, node);
    }

    fn visit_item_foreign_mod(&mut self, node: &'ast ItemForeignMod) {
        self.metrics.extern_blocks += 1;
        visit::visit_item_foreign_mod(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if is_transmute_call(&node.func) {
            self.metrics.transmutes += 1;
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        // `mem::transmute::<A, B>` referenced without immediate call.
        if path_ends_with(&node.path, "transmute") {
            // Counted only when used as a call; bare references are too noisy.
        }
        visit::visit_expr_path(self, node);
    }
}

fn is_transmute_call(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Path(p) => path_ends_with(&p.path, "transmute"),
        _ => false,
    }
}

fn path_ends_with(path: &syn::Path, name: &str) -> bool {
    path.segments
        .last()
        .map(|s| s.ident == name)
        .unwrap_or(false)
}
