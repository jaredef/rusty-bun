//! rusty-js-ast — typed AST node definitions for the rusty-js engine.
//!
//! Per specs/ecma262-module.spec.md. v1 covers the Module goal's
//! ImportDeclaration and ExportDeclaration forms in full; body-of-
//! statement constructs (FunctionBody, ClassBody, expressions) are
//! represented as opaque `Span` placeholders so the parser can recognize
//! a Module's import/export structure without yet committing to a full
//! expression grammar. Subsequent sub-rounds replace placeholders with
//! typed nodes per specs/ecma262-expressions.spec.md.

/// Byte-offset range into the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self { Self { start, end } }
}

// ─────────── Module Record (per ECMA-262 §16.2.1.6) ───────────

/// A parsed module. The body retains the original ModuleItem order for
/// future Evaluate-phase walks; the ImportEntries / ExportEntries lists
/// are derived from the body at parse time per §16.2.1.7.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub span: Span,
    pub body: Vec<ModuleItem>,
    pub import_entries: Vec<ImportEntry>,
    /// Local exports — bindings declared in this module and exported.
    pub local_export_entries: Vec<ExportEntry>,
    /// Indirect exports — `export { x } from './y'` patterns.
    pub indirect_export_entries: Vec<ExportEntry>,
    /// Star re-export entries — `export * from './y'` patterns.
    pub star_export_entries: Vec<ExportEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleItem {
    Import(ImportDeclaration),
    Export(ExportDeclaration),
    /// Statement-or-Declaration. v1 stores the raw byte-span of the
    /// construct; the parser performs balanced-brace skip to find its
    /// end. Subsequent sub-rounds will replace this with typed
    /// statement/declaration nodes.
    StatementOrDecl(Span),
}

// ─────────── ImportDeclaration ───────────

#[derive(Debug, Clone, PartialEq)]
pub struct ImportDeclaration {
    pub span: Span,
    pub specifier: ModuleSpecifier,
    pub default_binding: Option<BindingIdentifier>,
    pub namespace_binding: Option<BindingIdentifier>,
    pub named_imports: Vec<ImportSpecifier>,
    pub attributes: Vec<ImportAttribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportSpecifier {
    pub span: Span,
    /// Imported name — IdentifierName or StringLiteral (ES2022+).
    pub imported: ModuleExportName,
    /// Local binding. When omitted in source (`import { x }`), this
    /// equals `imported` unless `imported` is a StringLiteral, in which
    /// case the local binding is required and the source is malformed
    /// without one.
    pub local: BindingIdentifier,
}

// ─────────── ExportDeclaration ───────────

#[derive(Debug, Clone, PartialEq)]
pub enum ExportDeclaration {
    /// `export VariableStatement` / `export Declaration` — exports each
    /// binding the declaration introduces.
    Declaration {
        span: Span,
        /// Opaque span of the underlying declaration. Future rounds
        /// replace this with the typed declaration AST.
        decl_span: Span,
        /// Names introduced by the declaration. Computed by the parser
        /// at parse time (e.g., `export const {a, b} = obj` yields ["a", "b"]).
        names: Vec<BindingIdentifier>,
    },
    /// `export { ... } [from ModuleSpecifier];` — local or indirect re-export
    /// depending on presence of `from`.
    Named {
        span: Span,
        specifiers: Vec<ExportSpecifier>,
        /// Some(specifier) = indirect re-export; None = local re-export
        source: Option<ModuleSpecifier>,
        attributes: Vec<ImportAttribute>,
    },
    /// `export * from ModuleSpecifier;` — star re-export.
    StarFrom {
        span: Span,
        source: ModuleSpecifier,
        attributes: Vec<ImportAttribute>,
    },
    /// `export * as X from ModuleSpecifier;` (ES2020+) — named-namespace re-export.
    StarAsFrom {
        span: Span,
        exported: ModuleExportName,
        source: ModuleSpecifier,
        attributes: Vec<ImportAttribute>,
    },
    /// `export default ...`
    Default {
        span: Span,
        /// What the default expression is. Subsequent sub-rounds replace
        /// the opaque span with typed nodes.
        body: DefaultExportBody,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum DefaultExportBody {
    /// `export default function NAME?(...) { ... }` — Bun's Tuple-B-relevant
    /// case is when NAME is present, in which case the engine's E5 host
    /// hook exposes NAME as a named export per Doc 717.
    HoistableFunction { name: Option<BindingIdentifier>, body_span: Span, is_async: bool, is_generator: bool },
    /// `export default class NAME? { ... }` — same Tuple-B applicability when NAME present.
    Class { name: Option<BindingIdentifier>, body_span: Span },
    /// `export default <AssignmentExpression>;` — opaque span until expressions land.
    Expression { expr_span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportSpecifier {
    pub span: Span,
    /// Local name being exported. IdentifierName for local-re-export;
    /// any ModuleExportName for indirect re-export (the local refers to
    /// the source module's export).
    pub local: ModuleExportName,
    /// Exported-as name.
    pub exported: ModuleExportName,
}

// ─────────── Binding + Names ───────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindingIdentifier {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModuleExportName {
    /// `IdentifierName` (includes any reserved word; the parser does not
    /// reject reserved-word identifiers as export aliases — they are
    /// permitted per §16.2.3 grammar).
    Ident(BindingIdentifier),
    /// `StringLiteral` (ES2022+) — `as 'm-search'`. Tuple-C-relevant.
    String { value: String, span: Span },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleSpecifier {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportAttribute {
    pub span: Span,
    pub key: ModuleExportName,
    pub value: String,
}

// ─────────── Derived entries (per §16.2.1.6) ───────────

/// One entry of the Module's [[ImportEntries]] table.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportEntry {
    pub module_request: String,
    /// One of: BindingIdentifier (for `import x from 'y'`),
    /// "*" (for `import * as x from 'y'`),
    /// the imported name (for `import { x } from 'y'`).
    pub import_name: ImportName,
    pub local_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportName {
    /// `import x from 'y'` -> ImportName::Default
    Default,
    /// `import * as x from 'y'` -> ImportName::Namespace
    Namespace,
    /// `import { x } from 'y'` -> ImportName::Single("x")
    /// `import { "string" as x } from 'y'` -> ImportName::Single("string")
    Single(String),
}

/// One entry of the Module's [[LocalExportEntries]], [[IndirectExportEntries]],
/// or [[StarExportEntries]] tables (which list it belongs to is determined
/// by the parent ExportDeclaration node + the absence of `module_request`).
#[derive(Debug, Clone, PartialEq)]
pub struct ExportEntry {
    pub export_name: Option<String>,        // None = star export without name
    pub module_request: Option<String>,     // None = local export
    pub import_name: Option<ExportImportName>,
    pub local_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportImportName {
    All,                // export * from
    AllButDefault,      // not used in v1 (relevant for export * grammar nuance)
    Default,
    Single(String),
}
