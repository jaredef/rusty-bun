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
    BindingIdentifier, CatchClause, Expr, ForBinding, ForInit, Span, Stmt, SwitchCase,
    VariableDeclarator, VariableKind, VariableStatement,
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
        // Control-flow forms — typed in round 3c.
        if self.is_ident("if") { return self.parse_if_statement(); }
        if self.is_ident("for") { return self.parse_for_statement(); }
        if self.is_ident("while") { return self.parse_while_statement(); }
        if self.is_ident("do") { return self.parse_do_while_statement(); }
        if self.is_ident("switch") { return self.parse_switch_statement(); }
        if self.is_ident("try") { return self.parse_try_statement(); }
        if self.is_ident("return") { return self.parse_return_statement(); }
        if self.is_ident("throw") { return self.parse_throw_statement(); }
        if self.is_ident("break") { return self.parse_break_statement(); }
        if self.is_ident("continue") { return self.parse_continue_statement(); }
        if self.is_ident("debugger") {
            let span = self.lookahead_span();
            self.bump()?;
            self.consume_semicolon_pub();
            return Ok(Stmt::Debugger { span });
        }
        // `with` forbidden in modules; `yield` an expression at top level when
        // not in a generator. Both fall back to Stmt::Opaque.
        if self.is_ident("with") || self.is_ident("yield") {
            let span = self.skip_to_top_terminator()?;
            return Ok(Stmt::Opaque { span: Span::new(start, span.end) });
        }
        // LabelledStatement (Identifier ':' Statement) — typed.
        if let TokenKind::Ident(_) = self.current_kind() {
            let peek_pos = self.lookahead_span().end;
            let bytes = self.source().as_bytes();
            let mut p = peek_pos;
            while p < bytes.len() && bytes[p].is_ascii_whitespace() { p += 1; }
            if bytes.get(p) == Some(&b':') {
                let name = if let TokenKind::Ident(n) = self.current_kind().clone() { n } else { unreachable!() };
                let label_span = self.lookahead_span();
                self.bump()?; // consume label
                self.expect_punct(Punct::Colon)?;
                let body = self.parse_statement()?;
                let end = body.span().start.max(self.last_span_end());
                return Ok(Stmt::Labelled {
                    label: BindingIdentifier { name, span: label_span },
                    body: Box::new(body),
                    span: Span::new(start, end),
                });
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

    // ─────────────────── Typed control-flow (round 3c) ───────────────────

    fn parse_if_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_keyword("if")?;
        self.expect_punct(Punct::LParen)?;
        let test = self.parse_expression()?;
        self.expect_punct(Punct::RParen)?;
        let consequent = self.parse_statement()?;
        let alternate = if self.is_ident("else") {
            self.bump()?;
            Some(Box::new(self.parse_statement()?))
        } else { None };
        let end = self.last_span_end();
        Ok(Stmt::If { test, consequent: Box::new(consequent), alternate, span: Span::new(start, end) })
    }

    fn parse_for_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_keyword("for")?;
        // `for await (...)` (ES2018 — for-await-of)
        let await_form = if self.is_ident("await") { self.bump()?; true } else { false };
        self.expect_punct(Punct::LParen)?;

        // Head form discrimination: VariableDeclaration vs Expression vs empty.
        let head_is_var = self.is_ident("var") || self.is_ident("let") || self.is_ident("const");
        let head_is_empty = matches!(self.current_kind(), TokenKind::Punct(Punct::Semicolon));

        // Try parsing the head; then peek for `in`/`of` to disambiguate.
        // For `for (let x of arr)` and `for (let x in obj)`, the head is one
        // BindingIdentifier with no `=` and the next token is `in`/`of`.
        if head_is_var {
            // Capture kind + first binding identifier, then peek.
            let kind = match self.current_kind() {
                TokenKind::Ident(s) if s == "var" => VariableKind::Var,
                TokenKind::Ident(s) if s == "let" => VariableKind::Let,
                TokenKind::Ident(s) if s == "const" => VariableKind::Const,
                _ => unreachable!(),
            };
            let kw_span = self.lookahead_span();
            self.bump()?;
            if let TokenKind::Ident(n) = self.current_kind().clone() {
                let id_span = self.lookahead_span();
                self.bump()?;
                // for-in / for-of head
                if self.is_ident("in") || self.is_ident("of") {
                    let is_of = self.is_ident("of");
                    self.bump()?;
                    let right = self.parse_expression()?;
                    self.expect_punct(Punct::RParen)?;
                    let body = self.parse_statement()?;
                    let end = self.last_span_end();
                    let left = ForBinding::Decl {
                        kind, name: BindingIdentifier { name: n, span: id_span }, span: Span::new(kw_span.start, id_span.end),
                    };
                    return if is_of {
                        Ok(Stmt::ForOf { left, right, body: Box::new(body), await_: await_form, span: Span::new(start, end) })
                    } else {
                        Ok(Stmt::ForIn { left, right, body: Box::new(body), span: Span::new(start, end) })
                    };
                }
                // C-style with single var decl + optional initializer +
                // possibly more declarators. Recover via parse_variable_statement-like loop.
                let mut declarators = vec![{
                    let mut names = vec![BindingIdentifier { name: n, span: id_span }];
                    let init = if matches!(self.current_kind(), TokenKind::Punct(Punct::Assign)) {
                        self.bump()?;
                        Some(self.parse_assignment_expression()?)
                    } else { None };
                    let _ = &mut names;
                    VariableDeclarator { names, init, span: Span::new(id_span.start, self.last_span_end()) }
                }];
                while matches!(self.current_kind(), TokenKind::Punct(Punct::Comma)) {
                    self.bump()?;
                    let d_start = self.lookahead_span().start;
                    if let TokenKind::Ident(nn) = self.current_kind().clone() {
                        let nn_span = self.lookahead_span();
                        self.bump()?;
                        let init = if matches!(self.current_kind(), TokenKind::Punct(Punct::Assign)) {
                            self.bump()?;
                            Some(self.parse_assignment_expression()?)
                        } else { None };
                        declarators.push(VariableDeclarator {
                            names: vec![BindingIdentifier { name: nn, span: nn_span }],
                            init, span: Span::new(d_start, self.last_span_end()),
                        });
                    } else { break; }
                }
                self.expect_punct(Punct::Semicolon)?;
                let test = if !matches!(self.current_kind(), TokenKind::Punct(Punct::Semicolon)) {
                    Some(self.parse_expression()?)
                } else { None };
                self.expect_punct(Punct::Semicolon)?;
                let update = if !matches!(self.current_kind(), TokenKind::Punct(Punct::RParen)) {
                    Some(self.parse_expression()?)
                } else { None };
                self.expect_punct(Punct::RParen)?;
                let body = self.parse_statement()?;
                let end = self.last_span_end();
                let init = ForInit::Variable(VariableStatement {
                    kind, declarators, span: Span::new(kw_span.start, kw_span.end),
                });
                return Ok(Stmt::For { init: Some(init), test, update, body: Box::new(body), span: Span::new(start, end) });
            }
            // Fallback: pattern in head — opaque
        }

        // Expression-headed for / for-in / for-of
        if head_is_empty {
            self.bump()?;
        }
        let mut init_expr: Option<Expr> = None;
        let mut went_inof = false;
        if !head_is_empty && !matches!(self.current_kind(), TokenKind::Punct(Punct::Semicolon)) {
            let e = self.parse_expression()?;
            // Check for `in`/`of` after a LeftHandSideExpression head.
            if self.is_ident("in") || self.is_ident("of") {
                let is_of = self.is_ident("of");
                self.bump()?;
                let right = self.parse_expression()?;
                self.expect_punct(Punct::RParen)?;
                let body = self.parse_statement()?;
                let end = self.last_span_end();
                let left = match e {
                    Expr::Identifier { name, span } => ForBinding::Identifier(BindingIdentifier { name, span }),
                    _ => {
                        // Non-identifier LHS — represent the underlying ident
                        // via the expression's span. Round-3c keeps the type
                        // narrow.
                        let span = e.span();
                        ForBinding::Identifier(BindingIdentifier { name: String::new(), span })
                    }
                };
                went_inof = true;
                return if is_of {
                    Ok(Stmt::ForOf { left, right, body: Box::new(body), await_: await_form, span: Span::new(start, end) })
                } else {
                    Ok(Stmt::ForIn { left, right, body: Box::new(body), span: Span::new(start, end) })
                };
            }
            init_expr = Some(e);
        }
        let _ = went_inof;
        if !head_is_empty {
            self.expect_punct(Punct::Semicolon)?;
        }
        let test = if !matches!(self.current_kind(), TokenKind::Punct(Punct::Semicolon)) {
            Some(self.parse_expression()?)
        } else { None };
        self.expect_punct(Punct::Semicolon)?;
        let update = if !matches!(self.current_kind(), TokenKind::Punct(Punct::RParen)) {
            Some(self.parse_expression()?)
        } else { None };
        self.expect_punct(Punct::RParen)?;
        let body = self.parse_statement()?;
        let end = self.last_span_end();
        let init = init_expr.map(ForInit::Expression);
        Ok(Stmt::For { init, test, update, body: Box::new(body), span: Span::new(start, end) })
    }

    fn parse_while_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_keyword("while")?;
        self.expect_punct(Punct::LParen)?;
        let test = self.parse_expression()?;
        self.expect_punct(Punct::RParen)?;
        let body = self.parse_statement()?;
        let end = self.last_span_end();
        Ok(Stmt::While { test, body: Box::new(body), span: Span::new(start, end) })
    }

    fn parse_do_while_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_keyword("do")?;
        let body = self.parse_statement()?;
        self.expect_keyword("while")?;
        self.expect_punct(Punct::LParen)?;
        let test = self.parse_expression()?;
        self.expect_punct(Punct::RParen)?;
        self.consume_semicolon_pub();
        let end = self.last_span_end();
        Ok(Stmt::DoWhile { body: Box::new(body), test, span: Span::new(start, end) })
    }

    fn parse_switch_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_keyword("switch")?;
        self.expect_punct(Punct::LParen)?;
        let discriminant = self.parse_expression()?;
        self.expect_punct(Punct::RParen)?;
        self.expect_punct(Punct::LBrace)?;
        let mut cases = Vec::new();
        while !matches!(self.current_kind(), TokenKind::Punct(Punct::RBrace)) && !self.at_eof_internal() {
            let case_start = self.lookahead_span().start;
            let test = if self.is_ident("case") {
                self.bump()?;
                let t = self.parse_expression()?;
                self.expect_punct(Punct::Colon)?;
                Some(t)
            } else if self.is_ident("default") {
                self.bump()?;
                self.expect_punct(Punct::Colon)?;
                None
            } else {
                return Err(self.err_here("expected `case` or `default` in switch body".into()));
            };
            let mut consequent = Vec::new();
            while !self.is_ident("case") && !self.is_ident("default")
                && !matches!(self.current_kind(), TokenKind::Punct(Punct::RBrace))
                && !self.at_eof_internal()
            {
                consequent.push(self.parse_statement()?);
            }
            let case_end = self.last_span_end();
            cases.push(SwitchCase { test, consequent, span: Span::new(case_start, case_end) });
        }
        self.expect_punct(Punct::RBrace)?;
        let end = self.last_span_end();
        Ok(Stmt::Switch { discriminant, cases, span: Span::new(start, end) })
    }

    fn parse_try_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_keyword("try")?;
        let block = self.parse_block_statement_public()?;
        let handler = if self.is_ident("catch") {
            let h_start = self.lookahead_span().start;
            self.bump()?;
            let param = if matches!(self.current_kind(), TokenKind::Punct(Punct::LParen)) {
                self.bump()?;
                let p = if let TokenKind::Ident(n) = self.current_kind().clone() {
                    let span = self.lookahead_span();
                    self.bump()?;
                    Some(BindingIdentifier { name: n, span })
                } else {
                    // Patterned catch parameter — opaque skip for v1.
                    while !matches!(self.current_kind(), TokenKind::Punct(Punct::RParen)) && !self.at_eof_internal() {
                        self.bump()?;
                    }
                    None
                };
                self.expect_punct(Punct::RParen)?;
                p
            } else { None }; // ES2019 optional catch binding
            let body = self.parse_block_statement_public()?;
            let h_end = self.last_span_end();
            Some(CatchClause { param, body: Box::new(body), span: Span::new(h_start, h_end) })
        } else { None };
        let finalizer = if self.is_ident("finally") {
            self.bump()?;
            Some(Box::new(self.parse_block_statement_public()?))
        } else { None };
        let end = self.last_span_end();
        Ok(Stmt::Try { block: Box::new(block), handler, finalizer, span: Span::new(start, end) })
    }

    fn parse_return_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_keyword("return")?;
        // Per spec: return ASI applies if newline before next token.
        let argument = if matches!(self.current_kind(), TokenKind::Punct(Punct::Semicolon))
            || matches!(self.current_kind(), TokenKind::Punct(Punct::RBrace))
            || matches!(self.current_kind(), TokenKind::Eof)
            || self.lookahead_preceded_by_lt()
        {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume_semicolon_pub();
        let end = self.last_span_end();
        Ok(Stmt::Return { argument, span: Span::new(start, end) })
    }

    fn parse_throw_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_keyword("throw")?;
        if self.lookahead_preceded_by_lt() {
            return Err(self.err_here("no line terminator permitted between `throw` and its argument".into()));
        }
        let argument = self.parse_expression()?;
        self.consume_semicolon_pub();
        let end = self.last_span_end();
        Ok(Stmt::Throw { argument, span: Span::new(start, end) })
    }

    fn parse_break_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_keyword("break")?;
        let label = self.parse_optional_label()?;
        self.consume_semicolon_pub();
        let end = self.last_span_end();
        Ok(Stmt::Break { label, span: Span::new(start, end) })
    }

    fn parse_continue_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_keyword("continue")?;
        let label = self.parse_optional_label()?;
        self.consume_semicolon_pub();
        let end = self.last_span_end();
        Ok(Stmt::Continue { label, span: Span::new(start, end) })
    }

    fn parse_optional_label(&mut self) -> Result<Option<BindingIdentifier>, ParseError> {
        // No-LT-before rule per spec — label only if same-line identifier.
        if self.lookahead_preceded_by_lt() { return Ok(None); }
        if let TokenKind::Ident(n) = self.current_kind().clone() {
            // Excludes keywords that always terminate the statement.
            if !matches!(n.as_str(), "else") {
                let span = self.lookahead_span();
                self.bump()?;
                return Ok(Some(BindingIdentifier { name: n, span }));
            }
        }
        Ok(None)
    }

    fn parse_block_statement_public(&mut self) -> Result<Stmt, ParseError> {
        self.parse_block_statement()
    }
}
