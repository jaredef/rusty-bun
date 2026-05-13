//! Statement-grammar parser (Tier-Ω.3.b round 3b subset).
//!
//! Replaces the prior opaque-byte-span skip for top-level statements with
//! typed Stmt AST. v1 covers VariableStatement, ExpressionStatement, Block,
//! EmptyStatement, FunctionDeclaration (body-opaque), ClassDeclaration
//! (body-opaque). Control-flow forms (If/For/While/Switch/Try/Return/Throw/
//! Break/Continue/Labelled/With/Debugger) fall back to Stmt::Opaque until
//! a follow-on sub-round.

use crate::parser::{ParseError, Parser};
use crate::token::{Punct, TokenKind};
use rusty_js_ast::{
    BindingIdentifier, Span, Stmt, VariableDeclarator, VariableKind, VariableStatement,
};

impl<'src> Parser<'src> {
    pub fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.lookahead_span().start;

        // VariableStatement
        if self.is_ident("var") || self.is_ident("let") || self.is_ident("const") {
            let v = self.parse_variable_statement()?;
            return Ok(Stmt::Variable(v));
        }
        // FunctionDeclaration (sync + async)
        if self.is_ident("function") {
            return self.parse_function_decl_stmt(false);
        }
        if self.is_ident("async") {
            // Peek-2 disambiguation: `async function` vs `async <expr>`.
            let pos = self.lookahead_span().end;
            let bytes = self.source().as_bytes();
            let mut p = pos;
            while p < bytes.len() && bytes[p].is_ascii_whitespace() { p += 1; }
            if bytes[p..].starts_with(b"function") {
                self.bump()?; // consume `async`
                return self.parse_function_decl_stmt(true);
            }
        }
        // ClassDeclaration
        if self.is_ident("class") {
            return self.parse_class_decl_stmt();
        }
        // Block
        if matches!(self.current_kind(), TokenKind::Punct(Punct::LBrace)) {
            return self.parse_block_statement();
        }
        // EmptyStatement
        if matches!(self.current_kind(), TokenKind::Punct(Punct::Semicolon)) {
            let span = self.lookahead_span();
            self.bump()?;
            return Ok(Stmt::Empty { span });
        }
        // Control-flow forms → Stmt::Opaque for now.
        if self.is_ident("if") || self.is_ident("for") || self.is_ident("while")
            || self.is_ident("do") || self.is_ident("switch") || self.is_ident("try")
            || self.is_ident("return") || self.is_ident("throw") || self.is_ident("break")
            || self.is_ident("continue") || self.is_ident("debugger") || self.is_ident("with")
            || self.is_ident("yield")
        {
            let span = self.skip_to_top_terminator()?;
            return Ok(Stmt::Opaque { span: Span::new(start, span.end) });
        }
        // LabelledStatement (Identifier ':' Statement) — opaque for v1
        if let TokenKind::Ident(_) = self.current_kind() {
            let peek_pos = self.lookahead_span().end;
            let bytes = self.source().as_bytes();
            let mut p = peek_pos;
            while p < bytes.len() && bytes[p].is_ascii_whitespace() { p += 1; }
            if bytes.get(p) == Some(&b':') {
                let span = self.skip_to_top_terminator()?;
                return Ok(Stmt::Opaque { span: Span::new(start, span.end) });
            }
        }
        // ExpressionStatement
        let expr = self.parse_expression()?;
        self.consume_semicolon_pub();
        let end = self.last_span_end();
        Ok(Stmt::Expression { expr, span: Span::new(start, end) })
    }

    fn parse_variable_statement(&mut self) -> Result<VariableStatement, ParseError> {
        let start = self.lookahead_span().start;
        let kind = match self.current_kind() {
            TokenKind::Ident(s) if s == "var" => VariableKind::Var,
            TokenKind::Ident(s) if s == "let" => VariableKind::Let,
            TokenKind::Ident(s) if s == "const" => VariableKind::Const,
            _ => return Err(self.err_here("expected var/let/const".into())),
        };
        self.bump()?;
        let mut declarators = Vec::new();
        loop {
            let d_start = self.lookahead_span().start;
            let mut names: Vec<BindingIdentifier> = Vec::new();
            // BindingIdentifier or simple BindingPattern (object/array destructure).
            // Full BindingPattern AST is deferred; we extract names only.
            match self.current_kind().clone() {
                TokenKind::Ident(name) => {
                    let span = self.lookahead_span();
                    self.bump()?;
                    names.push(BindingIdentifier { name, span });
                }
                TokenKind::Punct(Punct::LBrace) => {
                    self.bump()?;
                    self.extract_obj_destructure_names_pub(&mut names)?;
                }
                TokenKind::Punct(Punct::LBracket) => {
                    self.bump()?;
                    self.extract_arr_destructure_names_pub(&mut names)?;
                }
                _ => return Err(self.err_here("expected binding identifier or pattern".into())),
            }
            let init = if matches!(self.current_kind(), TokenKind::Punct(Punct::Assign)) {
                self.bump()?;
                Some(self.parse_assignment_expression()?)
            } else { None };
            let d_end = self.last_span_end();
            declarators.push(VariableDeclarator { names, init, span: Span::new(d_start, d_end) });
            if matches!(self.current_kind(), TokenKind::Punct(Punct::Comma)) {
                self.bump()?;
            } else { break; }
        }
        self.consume_semicolon_pub();
        let end = self.last_span_end();
        Ok(VariableStatement { kind, declarators, span: Span::new(start, end) })
    }

    fn parse_function_decl_stmt(&mut self, is_async: bool) -> Result<Stmt, ParseError> {
        let start = if is_async {
            // `async` already consumed; recover start from before it.
            // The bump-tracker doesn't preserve prior span; use lookahead.
            self.lookahead_span().start
        } else {
            self.lookahead_span().start
        };
        self.expect_keyword("function")?;
        let is_generator = if matches!(self.current_kind(), TokenKind::Punct(Punct::Star)) {
            self.bump()?; true
        } else { false };
        let name = if let TokenKind::Ident(n) = self.current_kind().clone() {
            let span = self.lookahead_span();
            self.bump()?;
            Some(BindingIdentifier { name: n, span })
        } else { None };
        let body_start = self.lookahead_span().start;
        self.skip_balanced_public(Punct::LParen, Punct::RParen)?;
        self.skip_balanced_public(Punct::LBrace, Punct::RBrace)?;
        let end = self.last_span_end();
        Ok(Stmt::FunctionDecl {
            name, is_async, is_generator,
            body_span: Span::new(body_start, end),
            span: Span::new(start, end),
        })
    }

    fn parse_class_decl_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_keyword("class")?;
        let name = if let TokenKind::Ident(n) = self.current_kind().clone() {
            if n != "extends" {
                let span = self.lookahead_span();
                self.bump()?;
                Some(BindingIdentifier { name: n, span })
            } else { None }
        } else { None };
        // Optional `extends <expr>` — skip until `{`.
        if self.is_ident("extends") {
            self.bump()?;
            while !matches!(self.current_kind(), TokenKind::Punct(Punct::LBrace)) && !self.at_eof_internal() {
                self.bump()?;
            }
        }
        let body_start = self.lookahead_span().start;
        self.skip_balanced_public(Punct::LBrace, Punct::RBrace)?;
        let end = self.last_span_end();
        Ok(Stmt::ClassDecl {
            name,
            body_span: Span::new(body_start, end),
            span: Span::new(start, end),
        })
    }

    fn parse_block_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_punct(Punct::LBrace)?;
        let mut body = Vec::new();
        while !matches!(self.current_kind(), TokenKind::Punct(Punct::RBrace)) && !self.at_eof_internal() {
            body.push(self.parse_statement()?);
        }
        self.expect_punct(Punct::RBrace)?;
        let end = self.last_span_end();
        Ok(Stmt::Block { body, span: Span::new(start, end) })
    }

    /// Skip to top-level `;` / ASI / closing brace, returning the span of
    /// what was consumed. Used for the v1 opaque-control-flow fallback.
    fn skip_to_top_terminator(&mut self) -> Result<Span, ParseError> {
        let start = self.lookahead_span().start;
        let mut depth_paren = 0i32;
        let mut depth_brace = 0i32;
        let mut depth_bracket = 0i32;
        while !self.at_eof_internal() {
            let kind = self.current_kind().clone();
            match kind {
                TokenKind::Punct(Punct::LParen) => depth_paren += 1,
                TokenKind::Punct(Punct::RParen) => depth_paren -= 1,
                TokenKind::Punct(Punct::LBrace) => depth_brace += 1,
                TokenKind::Punct(Punct::RBrace) => {
                    if depth_brace == 0 { break; }
                    depth_brace -= 1;
                }
                TokenKind::Punct(Punct::LBracket) => depth_bracket += 1,
                TokenKind::Punct(Punct::RBracket) => depth_bracket -= 1,
                TokenKind::Punct(Punct::Semicolon) => {
                    if depth_paren == 0 && depth_brace == 0 && depth_bracket == 0 {
                        let end = self.lookahead_span().end;
                        self.bump()?;
                        return Ok(Span::new(start, end));
                    }
                }
                _ => {}
            }
            // ASI: line-terminator-preceded top-level token closes the stmt.
            if depth_paren == 0 && depth_brace == 0 && depth_bracket == 0
                && self.lookahead_preceded_by_lt()
                && self.lookahead_span().start != start
            {
                break;
            }
            self.bump()?;
        }
        Ok(Span::new(start, self.last_span_end()))
    }
}
