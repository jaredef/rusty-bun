//! ECMAScript module-goal parser.
//!
//! Per specs/ecma262-module.spec.md. v1 covers the Module goal symbol's
//! ImportDeclaration + ExportDeclaration forms in full; statement and
//! expression bodies are captured as opaque byte-spans (balanced-brace
//! skip) until the expression-grammar sub-round.
//!
//! This parser produces a `rusty_js_ast::Module` with fully-populated
//! ImportEntries / ExportEntries lists per §16.2.1.6, sufficient to
//! drive the engine's link phase (Tier-Ω.4.a Module Namespace
//! augmentation hooks).

use crate::lexer::{Lexer, LexerGoal, LexError};
use crate::token::{Punct, Token, TokenKind};
use rusty_js_ast::{
    BindingIdentifier, DefaultExportBody, ExportDeclaration, ExportEntry,
    ExportImportName, ExportSpecifier, ImportAttribute, ImportDeclaration,
    ImportEntry, ImportName, ImportSpecifier, Module, ModuleExportName,
    ModuleItem, ModuleSpecifier, Span,
};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

pub struct Parser<'src> {
    src: &'src str,
    lx: Lexer<'src>,
    /// One-token lookahead, replenished by `bump`.
    lookahead: Token,
    /// Watchdog. The current v1 parser captures statement bodies as opaque
    /// byte-spans via balanced-brace skip; pathological inputs without
    /// expression-grammar handling can over-read. Cap at a generous bound
    /// proportional to source size so the parser never unboundedly allocates.
    /// The expression-grammar sub-round retires this watchdog.
    advance_count: usize,
    advance_cap: usize,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Result<Self, ParseError> {
        let mut lx = Lexer::new(src);
        let lookahead = lx.next_token(LexerGoal::RegExp).map_err(lex_to_parse)?;
        // Watchdog: cap advances at 16x source bytes. Well-formed source
        // uses at most ~1 token per ~3 bytes; this is a 48x margin and
        // catches the v1 opaque-span-skip's pathological cases without
        // affecting any well-formed parse. Retired by the expression-
        // grammar sub-round once declaration bodies become typed.
        let advance_cap = (src.len() * 16).max(2048);
        Ok(Self { src, lx, lookahead, advance_count: 0, advance_cap })
    }

    pub fn parse_module(&mut self) -> Result<Module, ParseError> {
        let module_start = self.lookahead.span.start;
        let mut body: Vec<ModuleItem> = Vec::new();
        let mut import_entries: Vec<ImportEntry> = Vec::new();
        let mut local_export_entries: Vec<ExportEntry> = Vec::new();
        let mut indirect_export_entries: Vec<ExportEntry> = Vec::new();
        let mut star_export_entries: Vec<ExportEntry> = Vec::new();

        while !self.at_eof() {
            // Skip hashbang and other trivia surfaces that the lexer may emit.
            if matches!(self.lookahead.kind, TokenKind::Hashbang(_)) {
                self.bump_regexp()?;
                continue;
            }
            if self.is_ident("import") && !self.is_dynamic_import_call_after_import() {
                let decl = self.parse_import_declaration()?;
                self.collect_import_entries(&decl, &mut import_entries);
                body.push(ModuleItem::Import(decl));
                continue;
            }
            if self.is_ident("export") {
                let decl = self.parse_export_declaration()?;
                self.collect_export_entries(&decl, &mut local_export_entries,
                    &mut indirect_export_entries, &mut star_export_entries);
                body.push(ModuleItem::Export(decl));
                continue;
            }
            // Statement or declaration — skip its byte-span with balanced braces.
            let span = self.skip_statement_or_decl()?;
            body.push(ModuleItem::StatementOrDecl(span));
        }

        Ok(Module {
            span: Span::new(module_start, self.lookahead.span.start),
            body,
            import_entries,
            local_export_entries,
            indirect_export_entries,
            star_export_entries,
        })
    }

    // ───────────────────────────── ImportDeclaration ─────────────────────────────

    fn parse_import_declaration(&mut self) -> Result<ImportDeclaration, ParseError> {
        let start = self.lookahead.span.start;
        self.expect_keyword("import")?;

        let mut default_binding: Option<BindingIdentifier> = None;
        let mut namespace_binding: Option<BindingIdentifier> = None;
        let mut named_imports: Vec<ImportSpecifier> = Vec::new();
        let mut specifier_required = true;

        // Form 1 — `import 'specifier' ;`
        if matches!(self.lookahead.kind, TokenKind::String(_)) {
            let specifier = self.parse_module_specifier()?;
            let attributes = self.parse_optional_attributes()?;
            self.consume_semicolon();
            return Ok(ImportDeclaration {
                span: Span::new(start, self.last_span_end()),
                specifier,
                default_binding: None,
                namespace_binding: None,
                named_imports: vec![],
                attributes,
            });
        }

        // Default binding
        if let TokenKind::Ident(name) = &self.lookahead.kind {
            if name != "*" && name != "{" {
                default_binding = Some(self.parse_binding_identifier()?);
                if self.is_punct(Punct::Comma) {
                    self.bump_regexp()?;
                } else {
                    specifier_required = true;
                }
            }
        }

        // Namespace binding or named imports
        if self.is_punct(Punct::Star) {
            self.bump_regexp()?;
            self.expect_ident("as")?;
            namespace_binding = Some(self.parse_binding_identifier()?);
        } else if self.is_punct(Punct::LBrace) {
            named_imports = self.parse_named_imports()?;
        }

        let _ = specifier_required;
        self.expect_ident("from")?;
        let specifier = self.parse_module_specifier()?;
        let attributes = self.parse_optional_attributes()?;
        self.consume_semicolon();

        Ok(ImportDeclaration {
            span: Span::new(start, self.last_span_end()),
            specifier,
            default_binding,
            namespace_binding,
            named_imports,
            attributes,
        })
    }

    fn parse_named_imports(&mut self) -> Result<Vec<ImportSpecifier>, ParseError> {
        self.expect_punct(Punct::LBrace)?;
        let mut out = Vec::new();
        while !self.is_punct(Punct::RBrace) {
            let start = self.lookahead.span.start;
            let imported = self.parse_module_export_name()?;
            let local: BindingIdentifier = if self.is_ident("as") {
                self.bump_regexp()?;
                self.parse_binding_identifier()?
            } else {
                // Bare form: `{ x }` — local equals imported as IdentifierName.
                match &imported {
                    ModuleExportName::Ident(b) => b.clone(),
                    ModuleExportName::String { span, .. } => {
                        return Err(self.err_at(*span, "string-literal imported name requires `as Local`".into()));
                    }
                }
            };
            let end = self.last_span_end();
            out.push(ImportSpecifier {
                span: Span::new(start, end),
                imported,
                local,
            });
            if self.is_punct(Punct::Comma) {
                self.bump_regexp()?;
            } else {
                break;
            }
        }
        self.expect_punct(Punct::RBrace)?;
        Ok(out)
    }

    /// Heuristic: only relevant for the very first token after `import`.
    /// `import(...)` and `import.meta` are call-expression / member-expression
    /// forms, not declarations. Detected by `(` or `.` immediately after.
    fn is_dynamic_import_call_after_import(&mut self) -> bool {
        // Without backtracking, we peek ahead one token by reading from the
        // source directly. The lexer doesn't support peek-N, so we
        // approximate by examining the immediately-following source byte
        // after the current token's end.
        let mut p = self.lookahead.span.end;
        while p < self.src.len() && self.src.as_bytes()[p].is_ascii_whitespace() { p += 1; }
        matches!(self.src.as_bytes().get(p), Some(b'(') | Some(b'.'))
    }

    // ───────────────────────────── ExportDeclaration ─────────────────────────────

    fn parse_export_declaration(&mut self) -> Result<ExportDeclaration, ParseError> {
        let start = self.lookahead.span.start;
        self.expect_keyword("export")?;

        // export default ...
        if self.is_ident("default") {
            self.bump_regexp()?;
            let body = self.parse_default_export_body()?;
            return Ok(ExportDeclaration::Default {
                span: Span::new(start, self.last_span_end()),
                body,
            });
        }
        // export * ...
        if self.is_punct(Punct::Star) {
            self.bump_regexp()?;
            if self.is_ident("as") {
                self.bump_regexp()?;
                let exported = self.parse_module_export_name()?;
                self.expect_ident("from")?;
                let source = self.parse_module_specifier()?;
                let attributes = self.parse_optional_attributes()?;
                self.consume_semicolon();
                return Ok(ExportDeclaration::StarAsFrom {
                    span: Span::new(start, self.last_span_end()),
                    exported,
                    source,
                    attributes,
                });
            }
            self.expect_ident("from")?;
            let source = self.parse_module_specifier()?;
            let attributes = self.parse_optional_attributes()?;
            self.consume_semicolon();
            return Ok(ExportDeclaration::StarFrom {
                span: Span::new(start, self.last_span_end()),
                source,
                attributes,
            });
        }
        // export { ... } [from ...]
        if self.is_punct(Punct::LBrace) {
            let specifiers = self.parse_named_exports()?;
            let source = if self.is_ident("from") {
                self.bump_regexp()?;
                Some(self.parse_module_specifier()?)
            } else { None };
            let attributes = if source.is_some() { self.parse_optional_attributes()? } else { vec![] };
            self.consume_semicolon();
            return Ok(ExportDeclaration::Named {
                span: Span::new(start, self.last_span_end()),
                specifiers,
                source,
                attributes,
            });
        }
        // export Declaration / export VariableStatement / export Const-Let-Var
        let decl_start = self.lookahead.span.start;
        let (decl_span, names) = self.parse_declaration_for_export()?;
        Ok(ExportDeclaration::Declaration {
            span: Span::new(start, decl_span.end),
            decl_span: Span::new(decl_start, decl_span.end),
            names,
        })
    }

    fn parse_named_exports(&mut self) -> Result<Vec<ExportSpecifier>, ParseError> {
        self.expect_punct(Punct::LBrace)?;
        let mut out = Vec::new();
        while !self.is_punct(Punct::RBrace) {
            let start = self.lookahead.span.start;
            let local = self.parse_module_export_name()?;
            let exported: ModuleExportName = if self.is_ident("as") {
                self.bump_regexp()?;
                self.parse_module_export_name()?
            } else {
                local.clone()
            };
            out.push(ExportSpecifier {
                span: Span::new(start, self.last_span_end()),
                local,
                exported,
            });
            if self.is_punct(Punct::Comma) {
                self.bump_regexp()?;
            } else {
                break;
            }
        }
        self.expect_punct(Punct::RBrace)?;
        Ok(out)
    }

    fn parse_default_export_body(&mut self) -> Result<DefaultExportBody, ParseError> {
        // Determine the form by lookahead.
        if self.is_ident("function") {
            return self.parse_default_function(false);
        }
        if self.is_ident("async") {
            // Disambiguate `async function ...` from `async <expr>`.
            // We need a peek-2; approximate from raw source.
            let mut p = self.lookahead.span.end;
            while p < self.src.len() && self.src.as_bytes()[p].is_ascii_whitespace() { p += 1; }
            // Match prefix "function"
            if self.src.as_bytes()[p..].starts_with(b"function") {
                self.bump_regexp()?; // consume "async"
                return self.parse_default_function(true);
            }
        }
        if self.is_ident("class") {
            return self.parse_default_class();
        }
        // export default <AssignmentExpression> ;
        let start = self.lookahead.span.start;
        let end = self.skip_expression()?;
        self.consume_semicolon();
        Ok(DefaultExportBody::Expression { expr_span: Span::new(start, end) })
    }

    fn parse_default_function(&mut self, is_async: bool) -> Result<DefaultExportBody, ParseError> {
        self.expect_keyword("function")?;
        let is_generator = if self.is_punct(Punct::Star) {
            self.bump_regexp()?;
            true
        } else { false };
        let name = if matches!(self.lookahead.kind, TokenKind::Ident(_)) && !self.is_punct(Punct::LParen) {
            Some(self.parse_binding_identifier()?)
        } else { None };
        // Skip parameter list and body via balanced parens/braces.
        let body_start = self.lookahead.span.start;
        self.skip_balanced(Punct::LParen, Punct::RParen)?;
        self.skip_balanced(Punct::LBrace, Punct::RBrace)?;
        Ok(DefaultExportBody::HoistableFunction {
            name,
            body_span: Span::new(body_start, self.last_span_end()),
            is_async,
            is_generator,
        })
    }

    fn parse_default_class(&mut self) -> Result<DefaultExportBody, ParseError> {
        self.expect_keyword("class")?;
        let name = if matches!(self.lookahead.kind, TokenKind::Ident(ref s) if s != "extends" && s != "{") && !self.is_punct(Punct::LBrace) {
            Some(self.parse_binding_identifier()?)
        } else { None };
        // optional extends <expr>
        if self.is_ident("extends") {
            self.bump_regexp()?;
            // Skip the extends expression up to `{`.
            while !self.is_punct(Punct::LBrace) && !self.at_eof() {
                self.bump_regexp()?;
            }
        }
        let body_start = self.lookahead.span.start;
        self.skip_balanced(Punct::LBrace, Punct::RBrace)?;
        Ok(DefaultExportBody::Class {
            name,
            body_span: Span::new(body_start, self.last_span_end()),
        })
    }

    // ───────────────────────────── Names + specifiers ─────────────────────────────

    fn parse_module_specifier(&mut self) -> Result<ModuleSpecifier, ParseError> {
        let tok = self.lookahead.clone();
        match &tok.kind {
            TokenKind::String(s) => {
                self.bump_regexp()?;
                Ok(ModuleSpecifier { value: s.clone(), span: tok.span })
            }
            _ => Err(self.err_here("expected module specifier (string literal)".into())),
        }
    }

    fn parse_module_export_name(&mut self) -> Result<ModuleExportName, ParseError> {
        let tok = self.lookahead.clone();
        match &tok.kind {
            TokenKind::Ident(name) => {
                self.bump_regexp()?;
                Ok(ModuleExportName::Ident(BindingIdentifier { name: name.clone(), span: tok.span }))
            }
            TokenKind::String(s) => {
                self.bump_regexp()?;
                Ok(ModuleExportName::String { value: s.clone(), span: tok.span })
            }
            _ => Err(self.err_here("expected identifier or string literal".into())),
        }
    }

    fn parse_binding_identifier(&mut self) -> Result<BindingIdentifier, ParseError> {
        let tok = self.lookahead.clone();
        if let TokenKind::Ident(name) = &tok.kind {
            // v1: do not reject reserved-word bindings here. The parser's
            // strict-mode reserved-word handling is in the expression grammar
            // sub-round.
            self.bump_regexp()?;
            Ok(BindingIdentifier { name: name.clone(), span: tok.span })
        } else {
            Err(self.err_here("expected identifier".into()))
        }
    }

    fn parse_optional_attributes(&mut self) -> Result<Vec<ImportAttribute>, ParseError> {
        // ES2024: `with { type: "json" }` form. Earlier `assert { ... }` also tolerated.
        if !(self.is_ident("with") || self.is_ident("assert")) {
            return Ok(vec![]);
        }
        self.bump_regexp()?;
        self.expect_punct(Punct::LBrace)?;
        let mut out = Vec::new();
        while !self.is_punct(Punct::RBrace) {
            let start = self.lookahead.span.start;
            let key = self.parse_module_export_name()?;
            self.expect_punct(Punct::Colon)?;
            let value = match &self.lookahead.kind {
                TokenKind::String(s) => { let s = s.clone(); self.bump_regexp()?; s }
                _ => return Err(self.err_here("expected string literal in attribute value".into())),
            };
            out.push(ImportAttribute {
                span: Span::new(start, self.last_span_end()),
                key,
                value,
            });
            if self.is_punct(Punct::Comma) {
                self.bump_regexp()?;
            } else {
                break;
            }
        }
        self.expect_punct(Punct::RBrace)?;
        Ok(out)
    }

    // ───────────────────────────── Statement / declaration skipping ─────────────────────────────

    /// Capture the byte-span of a statement or declaration. v1 uses balanced
    /// brace/bracket/paren skipping; the parser's job for now is structural
    /// recognition of import/export boundaries, not semantic statement analysis.
    fn skip_statement_or_decl(&mut self) -> Result<Span, ParseError> {
        let start = self.lookahead.span.start;
        let mut depth_brace = 0i32;
        let mut depth_paren = 0i32;
        let mut depth_bracket = 0i32;
        loop {
            if self.at_eof() { break; }
            match self.lookahead.kind {
                TokenKind::Punct(Punct::LBrace) => depth_brace += 1,
                TokenKind::Punct(Punct::RBrace) => {
                    if depth_brace == 0 { break; }
                    depth_brace -= 1;
                }
                TokenKind::Punct(Punct::LParen) => depth_paren += 1,
                TokenKind::Punct(Punct::RParen) => depth_paren -= 1,
                TokenKind::Punct(Punct::LBracket) => depth_bracket += 1,
                TokenKind::Punct(Punct::RBracket) => depth_bracket -= 1,
                TokenKind::Punct(Punct::Semicolon) => {
                    if depth_brace == 0 && depth_paren == 0 && depth_bracket == 0 {
                        let end = self.lookahead.span.end;
                        self.bump_regexp()?;
                        return Ok(Span::new(start, end));
                    }
                }
                _ => {}
            }
            // ASI sentinel: a newline-preceded `import`/`export` at top level
            // closes a statement that didn't have a trailing semicolon.
            if depth_brace == 0 && depth_paren == 0 && depth_bracket == 0
                && self.lookahead.preceded_by_line_terminator
                && (self.is_ident("import") || self.is_ident("export"))
            {
                break;
            }
            self.bump_regexp()?;
        }
        Ok(Span::new(start, self.last_span_end()))
    }

    fn parse_declaration_for_export(&mut self) -> Result<(Span, Vec<BindingIdentifier>), ParseError> {
        let start = self.lookahead.span.start;
        // Capture binding names from the declaration head as best-effort.
        let mut names: Vec<BindingIdentifier> = Vec::new();
        let is_func = self.is_ident("function") || self.is_ident("async");
        let is_class = self.is_ident("class");
        let is_let = self.is_ident("let");
        let is_const = self.is_ident("const");
        let is_var = self.is_ident("var");
        if is_func {
            if self.is_ident("async") { self.bump_regexp()?; }
            self.expect_keyword("function")?;
            if self.is_punct(Punct::Star) { self.bump_regexp()?; }
            if let TokenKind::Ident(n) = &self.lookahead.kind {
                let bi = BindingIdentifier { name: n.clone(), span: self.lookahead.span };
                names.push(bi);
                self.bump_regexp()?;
            }
            self.skip_balanced(Punct::LParen, Punct::RParen)?;
            self.skip_balanced(Punct::LBrace, Punct::RBrace)?;
        } else if is_class {
            self.bump_regexp()?;
            if let TokenKind::Ident(n) = &self.lookahead.kind {
                if n != "extends" && n != "{" {
                    names.push(BindingIdentifier { name: n.clone(), span: self.lookahead.span });
                    self.bump_regexp()?;
                }
            }
            if self.is_ident("extends") {
                self.bump_regexp()?;
                while !self.is_punct(Punct::LBrace) && !self.at_eof() {
                    self.bump_regexp()?;
                }
            }
            self.skip_balanced(Punct::LBrace, Punct::RBrace)?;
        } else if is_let || is_const || is_var {
            self.bump_regexp()?;
            loop {
                // Each declarator: BindingIdentifier or destructure.
                if let TokenKind::Ident(n) = &self.lookahead.kind {
                    names.push(BindingIdentifier { name: n.clone(), span: self.lookahead.span });
                    self.bump_regexp()?;
                } else if self.is_punct(Punct::LBrace) {
                    // Destructure: walk balanced, pull out top-level identifiers
                    // followed by `:` (rename) or comma/brace (bare).
                    let bracket_start = self.lookahead.span;
                    self.bump_regexp()?;
                    self.extract_destructure_names_object(&mut names)?;
                    let _ = bracket_start;
                } else if self.is_punct(Punct::LBracket) {
                    self.bump_regexp()?;
                    self.extract_destructure_names_array(&mut names)?;
                } else {
                    break;
                }
                // Optional initializer `= <expr>` (until comma at depth 0 or end of decl).
                if self.is_punct(Punct::Assign) {
                    self.bump_regexp()?;
                    self.skip_until_top_comma_or_terminator()?;
                }
                if self.is_punct(Punct::Comma) {
                    self.bump_regexp()?;
                    continue;
                }
                break;
            }
            self.consume_semicolon();
        } else {
            // Fallback: treat as opaque statement span.
            return Ok((self.skip_statement_or_decl()?, names));
        }
        Ok((Span::new(start, self.last_span_end()), names))
    }

    fn extract_destructure_names_object(&mut self, out: &mut Vec<BindingIdentifier>) -> Result<(), ParseError> {
        let mut depth = 1i32;
        while depth > 0 && !self.at_eof() {
            match &self.lookahead.kind {
                TokenKind::Punct(Punct::LBrace) => { depth += 1; self.bump_regexp()?; }
                TokenKind::Punct(Punct::RBrace) => { depth -= 1; self.bump_regexp()?; }
                TokenKind::Ident(n) => {
                    if depth == 1 {
                        // Peek next: `:` → renamed binding; else `n` is the binding.
                        let name = n.clone();
                        let span = self.lookahead.span;
                        self.bump_regexp()?;
                        if self.is_punct(Punct::Colon) {
                            self.bump_regexp()?;
                            // The renamed local is the next ident or pattern.
                            if let TokenKind::Ident(nn) = &self.lookahead.kind {
                                out.push(BindingIdentifier { name: nn.clone(), span: self.lookahead.span });
                                self.bump_regexp()?;
                            }
                        } else {
                            out.push(BindingIdentifier { name, span });
                        }
                    } else {
                        self.bump_regexp()?;
                    }
                }
                _ => { self.bump_regexp()?; }
            }
        }
        Ok(())
    }

    fn extract_destructure_names_array(&mut self, out: &mut Vec<BindingIdentifier>) -> Result<(), ParseError> {
        let mut depth = 1i32;
        while depth > 0 && !self.at_eof() {
            match &self.lookahead.kind {
                TokenKind::Punct(Punct::LBracket) => { depth += 1; self.bump_regexp()?; }
                TokenKind::Punct(Punct::RBracket) => { depth -= 1; self.bump_regexp()?; }
                TokenKind::Ident(n) => {
                    if depth == 1 {
                        out.push(BindingIdentifier { name: n.clone(), span: self.lookahead.span });
                    }
                    self.bump_regexp()?;
                }
                _ => { self.bump_regexp()?; }
            }
        }
        Ok(())
    }

    fn skip_until_top_comma_or_terminator(&mut self) -> Result<(), ParseError> {
        let mut depth_brace = 0i32;
        let mut depth_paren = 0i32;
        let mut depth_bracket = 0i32;
        while !self.at_eof() {
            match self.lookahead.kind {
                TokenKind::Punct(Punct::LBrace) => depth_brace += 1,
                TokenKind::Punct(Punct::RBrace) => depth_brace -= 1,
                TokenKind::Punct(Punct::LParen) => depth_paren += 1,
                TokenKind::Punct(Punct::RParen) => depth_paren -= 1,
                TokenKind::Punct(Punct::LBracket) => depth_bracket += 1,
                TokenKind::Punct(Punct::RBracket) => depth_bracket -= 1,
                TokenKind::Punct(Punct::Comma) | TokenKind::Punct(Punct::Semicolon) => {
                    if depth_brace == 0 && depth_paren == 0 && depth_bracket == 0 { return Ok(()); }
                }
                _ => {}
            }
            self.bump_regexp()?;
        }
        Ok(())
    }

    fn skip_expression(&mut self) -> Result<usize, ParseError> {
        // Stops at top-level `;` (or ASI-equivalent) — used for the
        // `export default <AssignmentExpression>` form's body span capture.
        let mut depth_brace = 0i32;
        let mut depth_paren = 0i32;
        let mut depth_bracket = 0i32;
        let mut last_end = self.lookahead.span.start;
        while !self.at_eof() {
            match self.lookahead.kind {
                TokenKind::Punct(Punct::LBrace) => depth_brace += 1,
                TokenKind::Punct(Punct::RBrace) => depth_brace -= 1,
                TokenKind::Punct(Punct::LParen) => depth_paren += 1,
                TokenKind::Punct(Punct::RParen) => depth_paren -= 1,
                TokenKind::Punct(Punct::LBracket) => depth_bracket += 1,
                TokenKind::Punct(Punct::RBracket) => depth_bracket -= 1,
                TokenKind::Punct(Punct::Semicolon) => {
                    if depth_brace == 0 && depth_paren == 0 && depth_bracket == 0 {
                        last_end = self.lookahead.span.end;
                        // Don't consume the semicolon here; caller handles ASI.
                        return Ok(last_end);
                    }
                }
                _ => {}
            }
            if depth_brace == 0 && depth_paren == 0 && depth_bracket == 0
                && self.lookahead.preceded_by_line_terminator
                && last_end != self.lookahead.span.start
            {
                return Ok(last_end);
            }
            last_end = self.lookahead.span.end;
            self.bump_regexp()?;
        }
        Ok(last_end)
    }

    fn skip_balanced(&mut self, open: Punct, close: Punct) -> Result<(), ParseError> {
        if !self.is_punct(open) {
            return Err(self.err_here(format!("expected `{:?}`", open)));
        }
        self.bump_regexp()?;
        let mut depth = 1i32;
        while depth > 0 {
            if self.at_eof() {
                return Err(self.err_here(format!("unterminated `{:?}`", open)));
            }
            match self.lookahead.kind {
                TokenKind::Punct(p) if p == open => depth += 1,
                TokenKind::Punct(p) if p == close => depth -= 1,
                _ => {}
            }
            self.bump_regexp()?;
        }
        Ok(())
    }

    // ───────────────────────────── Module record derivation ─────────────────────────────

    fn collect_import_entries(&self, decl: &ImportDeclaration, out: &mut Vec<ImportEntry>) {
        let mr = decl.specifier.value.clone();
        if let Some(b) = &decl.default_binding {
            out.push(ImportEntry {
                module_request: mr.clone(),
                import_name: ImportName::Default,
                local_name: b.name.clone(),
            });
        }
        if let Some(b) = &decl.namespace_binding {
            out.push(ImportEntry {
                module_request: mr.clone(),
                import_name: ImportName::Namespace,
                local_name: b.name.clone(),
            });
        }
        for spec in &decl.named_imports {
            let imported = match &spec.imported {
                ModuleExportName::Ident(b) => b.name.clone(),
                ModuleExportName::String { value, .. } => value.clone(),
            };
            out.push(ImportEntry {
                module_request: mr.clone(),
                import_name: ImportName::Single(imported),
                local_name: spec.local.name.clone(),
            });
        }
    }

    fn collect_export_entries(
        &self,
        decl: &ExportDeclaration,
        local: &mut Vec<ExportEntry>,
        indirect: &mut Vec<ExportEntry>,
        star: &mut Vec<ExportEntry>,
    ) {
        match decl {
            ExportDeclaration::Declaration { names, .. } => {
                for n in names {
                    local.push(ExportEntry {
                        export_name: Some(n.name.clone()),
                        module_request: None,
                        import_name: None,
                        local_name: Some(n.name.clone()),
                    });
                }
            }
            ExportDeclaration::Named { specifiers, source, .. } => {
                let mr = source.as_ref().map(|s| s.value.clone());
                for spec in specifiers {
                    let exported_name = match &spec.exported {
                        ModuleExportName::Ident(b) => b.name.clone(),
                        ModuleExportName::String { value, .. } => value.clone(),
                    };
                    let local_name = match &spec.local {
                        ModuleExportName::Ident(b) => Some(b.name.clone()),
                        ModuleExportName::String { value, .. } => Some(value.clone()),
                    };
                    let entry = ExportEntry {
                        export_name: Some(exported_name),
                        module_request: mr.clone(),
                        import_name: if mr.is_some() {
                            local_name.clone().map(ExportImportName::Single)
                        } else { None },
                        local_name: if mr.is_none() { local_name } else { None },
                    };
                    if mr.is_some() { indirect.push(entry); } else { local.push(entry); }
                }
            }
            ExportDeclaration::StarFrom { source, .. } => {
                star.push(ExportEntry {
                    export_name: None,
                    module_request: Some(source.value.clone()),
                    import_name: Some(ExportImportName::All),
                    local_name: None,
                });
            }
            ExportDeclaration::StarAsFrom { exported, source, .. } => {
                let name = match exported {
                    ModuleExportName::Ident(b) => b.name.clone(),
                    ModuleExportName::String { value, .. } => value.clone(),
                };
                indirect.push(ExportEntry {
                    export_name: Some(name),
                    module_request: Some(source.value.clone()),
                    import_name: Some(ExportImportName::All),
                    local_name: None,
                });
            }
            ExportDeclaration::Default { body, .. } => {
                let local_name = match body {
                    DefaultExportBody::HoistableFunction { name, .. } => name.as_ref().map(|b| b.name.clone()),
                    DefaultExportBody::Class { name, .. } => name.as_ref().map(|b| b.name.clone()),
                    DefaultExportBody::Expression { .. } => None,
                };
                local.push(ExportEntry {
                    export_name: Some("default".into()),
                    module_request: None,
                    import_name: None,
                    local_name: local_name.or_else(|| Some("*default*".into())),
                });
            }
        }
    }

    // ───────────────────────────── Token plumbing ─────────────────────────────

    fn bump_regexp(&mut self) -> Result<Token, ParseError> {
        self.advance_count += 1;
        if self.advance_count > self.advance_cap {
            return Err(self.err_here("parser watchdog tripped (expression grammar pending)".into()));
        }
        let cur = std::mem::replace(
            &mut self.lookahead,
            self.lx.next_token(LexerGoal::RegExp).map_err(lex_to_parse)?,
        );
        Ok(cur)
    }

    fn at_eof(&self) -> bool {
        matches!(self.lookahead.kind, TokenKind::Eof)
    }

    fn last_span_end(&self) -> usize {
        // The lookahead's start is the last consumed token's end.
        self.lookahead.span.start
    }

    fn is_punct(&self, p: Punct) -> bool {
        matches!(self.lookahead.kind, TokenKind::Punct(q) if q == p)
    }

    fn is_ident(&self, name: &str) -> bool {
        matches!(&self.lookahead.kind, TokenKind::Ident(n) if n == name)
    }

    fn expect_punct(&mut self, p: Punct) -> Result<(), ParseError> {
        if self.is_punct(p) { self.bump_regexp()?; Ok(()) }
        else { Err(self.err_here(format!("expected `{:?}`", p))) }
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), ParseError> {
        if self.is_ident(kw) { self.bump_regexp()?; Ok(()) }
        else { Err(self.err_here(format!("expected `{}`", kw))) }
    }

    fn expect_ident(&mut self, name: &str) -> Result<(), ParseError> {
        if self.is_ident(name) { self.bump_regexp()?; Ok(()) }
        else { Err(self.err_here(format!("expected `{}`", name))) }
    }

    fn consume_semicolon(&mut self) {
        if self.is_punct(Punct::Semicolon) {
            let _ = self.bump_regexp();
        }
        // Otherwise rely on ASI — the next iteration's `at_eof` / line-
        // terminator-before-next-keyword check handles it.
    }

    fn err_here(&self, message: String) -> ParseError {
        ParseError { span: self.lookahead.span, message }
    }

    fn err_at(&self, span: Span, message: String) -> ParseError {
        ParseError { span, message }
    }
}

fn lex_to_parse(e: LexError) -> ParseError {
    ParseError {
        span: e.span,
        message: format!("lex error: {} ({:?})", e.message, e.kind),
    }
}
