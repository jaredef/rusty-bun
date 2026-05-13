//! ECMAScript AssignmentExpression parser.
//!
//! Per specs/ecma262-expressions.spec.md (queued; this is the round-3a
//! implementation against the slice of the grammar needed for typed
//! declaration-body parsing).
//!
//! v1 subset: literals + identifier + member + call + new + unary + update +
//! binary (full precedence climbing) + conditional + assignment + sequence +
//! array + object + parenthesized. FunctionExpression / ClassExpression /
//! ArrowFunction / TemplateLiteral-with-substitutions fall back to
//! Expr::Opaque via balanced-brace skip.

use crate::parser::{ParseError, Parser};
use crate::token::{Punct, TokenKind};
use rusty_js_ast::{
    Argument, ArrayElement, ArrowBody, AssignOp, BinaryOp, Expr, MemberProperty,
    ObjectKey, ObjectProperty, Span, UnaryOp, UpdateOp,
};

impl<'src> Parser<'src> {
    // ───────────────── AssignmentExpression entry ─────────────────

    /// Parse a single AssignmentExpression (per ECMA-262 §13.15).
    pub fn parse_assignment_expression(&mut self) -> Result<Expr, ParseError> {
        // `async` disambiguation MUST come before the generic arrow-head
        // probe — otherwise `async (x) => x` matches the Identifier-=>-x
        // arrow shape with `async` as the parameter name.
        if self.is_ident("async") {
            let pos = self.lookahead_span().end;
            let bytes = self.source().as_bytes();
            let mut p = pos;
            while p < bytes.len() && bytes[p].is_ascii_whitespace() { p += 1; }
            if bytes[p..].starts_with(b"function") {
                self.bump()?; // consume `async`
                return self.parse_function_expression(true);
            }
            let starts_paren = bytes.get(p) == Some(&b'(');
            let starts_ident = p < bytes.len() &&
                (bytes[p].is_ascii_alphabetic() || bytes[p] == b'_' || bytes[p] == b'$');
            // Only treat as async-arrow if the next token after the
            // `(...)` / Identifier head is `=>` — avoids capturing the
            // bare `async()` call expression.
            if starts_paren || starts_ident {
                let mut q = p;
                if bytes.get(q) == Some(&b'(') {
                    let mut depth = 1i32;
                    q += 1;
                    while q < bytes.len() && depth > 0 {
                        match bytes[q] {
                            b'(' => depth += 1,
                            b')' => depth -= 1,
                            _ => {}
                        }
                        q += 1;
                    }
                } else {
                    while q < bytes.len() && (bytes[q].is_ascii_alphanumeric() || bytes[q] == b'_' || bytes[q] == b'$') { q += 1; }
                }
                while q < bytes.len() && (bytes[q] == b' ' || bytes[q] == b'\t') { q += 1; }
                if bytes.get(q) == Some(&b'=') && bytes.get(q + 1) == Some(&b'>') {
                    self.bump()?; // consume `async`
                    return self.parse_arrow_function(true);
                }
            }
        }
        // ArrowFunction is recognized at this precedence level per spec.
        if self.looks_like_arrow_function_head() {
            return self.parse_arrow_function(false);
        }
        // FunctionExpression / ClassExpression.
        if self.is_ident("function") {
            return self.parse_function_expression(false);
        }
        if self.is_ident("class") {
            return self.parse_class_expression();
        }

        let left = self.parse_conditional_expression()?;
        if let Some(op) = self.peek_assign_op() {
            self.bump()?;
            let value = self.parse_assignment_expression()?;
            let span = Span::new(left.span().start, value.span().end);
            return Ok(Expr::Assign { operator: op, target: Box::new(left), value: Box::new(value), span });
        }
        Ok(left)
    }

    fn peek_assign_op(&self) -> Option<AssignOp> {
        match self.current_kind() {
            TokenKind::Punct(Punct::Assign) => Some(AssignOp::Assign),
            TokenKind::Punct(Punct::PlusAssign) => Some(AssignOp::AddAssign),
            TokenKind::Punct(Punct::MinusAssign) => Some(AssignOp::SubAssign),
            TokenKind::Punct(Punct::StarAssign) => Some(AssignOp::MulAssign),
            TokenKind::Punct(Punct::SlashAssign) => Some(AssignOp::DivAssign),
            TokenKind::Punct(Punct::PercentAssign) => Some(AssignOp::ModAssign),
            TokenKind::Punct(Punct::StarStarAssign) => Some(AssignOp::PowAssign),
            TokenKind::Punct(Punct::ShlAssign) => Some(AssignOp::ShlAssign),
            TokenKind::Punct(Punct::ShrAssign) => Some(AssignOp::ShrAssign),
            TokenKind::Punct(Punct::UShrAssign) => Some(AssignOp::UShrAssign),
            TokenKind::Punct(Punct::BitAndAssign) => Some(AssignOp::BitAndAssign),
            TokenKind::Punct(Punct::BitOrAssign) => Some(AssignOp::BitOrAssign),
            TokenKind::Punct(Punct::BitXorAssign) => Some(AssignOp::BitXorAssign),
            TokenKind::Punct(Punct::LogicalAndAssign) => Some(AssignOp::LogicalAndAssign),
            TokenKind::Punct(Punct::LogicalOrAssign) => Some(AssignOp::LogicalOrAssign),
            TokenKind::Punct(Punct::NullishAssign) => Some(AssignOp::NullishAssign),
            _ => None,
        }
    }

    // ───────────────── ConditionalExpression ─────────────────

    fn parse_conditional_expression(&mut self) -> Result<Expr, ParseError> {
        let test = self.parse_binary_expression(0)?;
        if matches!(self.current_kind(), TokenKind::Punct(Punct::Question)) {
            self.bump()?;
            let consequent = self.parse_assignment_expression()?;
            self.expect_punct(Punct::Colon)?;
            let alternate = self.parse_assignment_expression()?;
            let span = Span::new(test.span().start, alternate.span().end);
            return Ok(Expr::Conditional {
                test: Box::new(test),
                consequent: Box::new(consequent),
                alternate: Box::new(alternate),
                span,
            });
        }
        Ok(test)
    }

    // ───────────────── BinaryExpression with precedence climbing ─────────────────

    /// Precedence-climbing per ECMA-262 §13 binary-operator hierarchy.
    /// Returns when no operator at >= min_prec remains.
    fn parse_binary_expression(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary_expression()?;
        loop {
            let Some((op, prec, right_assoc)) = self.peek_binary_op() else { break };
            if prec < min_prec { break; }
            self.bump()?;
            let next_min = if right_assoc { prec } else { prec + 1 };
            let right = self.parse_binary_expression(next_min)?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary { operator: op, left: Box::new(left), right: Box::new(right), span };
        }
        Ok(left)
    }

    /// (BinaryOp, precedence, right-associative?)
    fn peek_binary_op(&self) -> Option<(BinaryOp, u8, bool)> {
        match self.current_kind() {
            // Lowest: nullish, logical-or, logical-and
            TokenKind::Punct(Punct::NullishCoalesce) => Some((BinaryOp::NullishCoalesce, 3, false)),
            TokenKind::Punct(Punct::LogicalOr) => Some((BinaryOp::LogicalOr, 4, false)),
            TokenKind::Punct(Punct::LogicalAnd) => Some((BinaryOp::LogicalAnd, 5, false)),
            // Bitwise
            TokenKind::Punct(Punct::BitOr) => Some((BinaryOp::BitOr, 6, false)),
            TokenKind::Punct(Punct::BitXor) => Some((BinaryOp::BitXor, 7, false)),
            TokenKind::Punct(Punct::BitAnd) => Some((BinaryOp::BitAnd, 8, false)),
            // Equality
            TokenKind::Punct(Punct::Eq) => Some((BinaryOp::Eq, 9, false)),
            TokenKind::Punct(Punct::Ne) => Some((BinaryOp::Ne, 9, false)),
            TokenKind::Punct(Punct::StrictEq) => Some((BinaryOp::StrictEq, 9, false)),
            TokenKind::Punct(Punct::StrictNe) => Some((BinaryOp::StrictNe, 9, false)),
            // Relational
            TokenKind::Punct(Punct::Lt) => Some((BinaryOp::Lt, 10, false)),
            TokenKind::Punct(Punct::Gt) => Some((BinaryOp::Gt, 10, false)),
            TokenKind::Punct(Punct::Le) => Some((BinaryOp::Le, 10, false)),
            TokenKind::Punct(Punct::Ge) => Some((BinaryOp::Ge, 10, false)),
            // instanceof / in
            TokenKind::Ident(s) if s == "instanceof" => Some((BinaryOp::Instanceof, 10, false)),
            TokenKind::Ident(s) if s == "in" => Some((BinaryOp::In, 10, false)),
            // Shift
            TokenKind::Punct(Punct::Shl) => Some((BinaryOp::Shl, 11, false)),
            TokenKind::Punct(Punct::Shr) => Some((BinaryOp::Shr, 11, false)),
            TokenKind::Punct(Punct::UShr) => Some((BinaryOp::UShr, 11, false)),
            // Additive
            TokenKind::Punct(Punct::Plus) => Some((BinaryOp::Add, 12, false)),
            TokenKind::Punct(Punct::Minus) => Some((BinaryOp::Sub, 12, false)),
            // Multiplicative
            TokenKind::Punct(Punct::Star) => Some((BinaryOp::Mul, 13, false)),
            TokenKind::Punct(Punct::Slash) => Some((BinaryOp::Div, 13, false)),
            TokenKind::Punct(Punct::Percent) => Some((BinaryOp::Mod, 13, false)),
            // Exponentiation (right-associative)
            TokenKind::Punct(Punct::StarStar) => Some((BinaryOp::Pow, 14, true)),
            _ => None,
        }
    }

    // ───────────────── UnaryExpression / UpdateExpression ─────────────────

    fn parse_unary_expression(&mut self) -> Result<Expr, ParseError> {
        let start = self.lookahead_span().start;
        match self.current_kind() {
            TokenKind::Punct(Punct::Plus) => {
                self.bump()?;
                let arg = self.parse_unary_expression()?;
                let span = Span::new(start, arg.span().end);
                Ok(Expr::Unary { operator: UnaryOp::Plus, argument: Box::new(arg), span })
            }
            TokenKind::Punct(Punct::Minus) => {
                self.bump()?;
                let arg = self.parse_unary_expression()?;
                let span = Span::new(start, arg.span().end);
                Ok(Expr::Unary { operator: UnaryOp::Minus, argument: Box::new(arg), span })
            }
            TokenKind::Punct(Punct::BitNot) => {
                self.bump()?;
                let arg = self.parse_unary_expression()?;
                let span = Span::new(start, arg.span().end);
                Ok(Expr::Unary { operator: UnaryOp::BitNot, argument: Box::new(arg), span })
            }
            TokenKind::Punct(Punct::LogicalNot) => {
                self.bump()?;
                let arg = self.parse_unary_expression()?;
                let span = Span::new(start, arg.span().end);
                Ok(Expr::Unary { operator: UnaryOp::LogicalNot, argument: Box::new(arg), span })
            }
            TokenKind::Ident(s) if s == "typeof" => {
                self.bump()?;
                let arg = self.parse_unary_expression()?;
                let span = Span::new(start, arg.span().end);
                Ok(Expr::Unary { operator: UnaryOp::Typeof, argument: Box::new(arg), span })
            }
            TokenKind::Ident(s) if s == "void" => {
                self.bump()?;
                let arg = self.parse_unary_expression()?;
                let span = Span::new(start, arg.span().end);
                Ok(Expr::Unary { operator: UnaryOp::Void, argument: Box::new(arg), span })
            }
            TokenKind::Ident(s) if s == "delete" => {
                self.bump()?;
                let arg = self.parse_unary_expression()?;
                let span = Span::new(start, arg.span().end);
                Ok(Expr::Unary { operator: UnaryOp::Delete, argument: Box::new(arg), span })
            }
            TokenKind::Ident(s) if s == "await" => {
                self.bump()?;
                let arg = self.parse_unary_expression()?;
                let span = Span::new(start, arg.span().end);
                Ok(Expr::Unary { operator: UnaryOp::Await, argument: Box::new(arg), span })
            }
            TokenKind::Punct(Punct::Inc) => {
                self.bump()?;
                let arg = self.parse_unary_expression()?;
                let span = Span::new(start, arg.span().end);
                Ok(Expr::Update { operator: UpdateOp::Inc, argument: Box::new(arg), prefix: true, span })
            }
            TokenKind::Punct(Punct::Dec) => {
                self.bump()?;
                let arg = self.parse_unary_expression()?;
                let span = Span::new(start, arg.span().end);
                Ok(Expr::Update { operator: UpdateOp::Dec, argument: Box::new(arg), prefix: true, span })
            }
            _ => self.parse_postfix_expression(),
        }
    }

    fn parse_postfix_expression(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_left_hand_side_expression()?;
        // Postfix ++/-- — but only if not preceded by a line terminator
        // (no-LT-before per spec to permit ASI).
        if !self.lookahead_preceded_by_lt() {
            let start = expr.span().start;
            if matches!(self.current_kind(), TokenKind::Punct(Punct::Inc)) {
                let end = self.lookahead_span().end;
                self.bump()?;
                return Ok(Expr::Update {
                    operator: UpdateOp::Inc,
                    argument: Box::new(expr),
                    prefix: false,
                    span: Span::new(start, end),
                });
            }
            if matches!(self.current_kind(), TokenKind::Punct(Punct::Dec)) {
                let end = self.lookahead_span().end;
                self.bump()?;
                return Ok(Expr::Update {
                    operator: UpdateOp::Dec,
                    argument: Box::new(expr),
                    prefix: false,
                    span: Span::new(start, end),
                });
            }
        }
        Ok(expr)
    }

    // ───────────────── LeftHandSideExpression: member + call + new ─────────────────

    pub(crate) fn parse_left_hand_side_expression(&mut self) -> Result<Expr, ParseError> {
        let mut expr = if self.is_ident("new") {
            self.parse_new_expression()?
        } else {
            self.parse_primary_expression()?
        };

        // CallExpression / MemberExpression continuation.
        loop {
            match self.current_kind() {
                TokenKind::Punct(Punct::Dot) => {
                    self.bump()?;
                    expr = self.consume_member_property(expr, false)?;
                }
                TokenKind::Punct(Punct::OptionalChain) => {
                    self.bump()?;
                    // After ?. we expect either Identifier (member), `[` (computed),
                    // or `(` (call).
                    if matches!(self.current_kind(), TokenKind::Punct(Punct::LParen)) {
                        let start = expr.span().start;
                        let arguments = self.parse_arguments()?;
                        let end = self.last_span_end();
                        expr = Expr::Call {
                            callee: Box::new(expr),
                            arguments,
                            optional: true,
                            span: Span::new(start, end),
                        };
                    } else if matches!(self.current_kind(), TokenKind::Punct(Punct::LBracket)) {
                        expr = self.consume_computed_member(expr, true)?;
                    } else {
                        expr = self.consume_member_property(expr, true)?;
                    }
                }
                TokenKind::Punct(Punct::LBracket) => {
                    expr = self.consume_computed_member(expr, false)?;
                }
                TokenKind::Punct(Punct::LParen) => {
                    let start = expr.span().start;
                    let arguments = self.parse_arguments()?;
                    let end = self.last_span_end();
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        arguments,
                        optional: false,
                        span: Span::new(start, end),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_new_expression(&mut self) -> Result<Expr, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_keyword("new")?;
        // `new.target` meta-property
        if matches!(self.current_kind(), TokenKind::Punct(Punct::Dot)) {
            self.bump()?;
            if let TokenKind::Ident(p) = self.current_kind() {
                if p == "target" {
                    let end = self.lookahead_span().end;
                    let p_clone = p.to_string();
                    self.bump()?;
                    return Ok(Expr::MetaProperty {
                        meta: "new".into(), property: p_clone, span: Span::new(start, end),
                    });
                }
            }
            return Err(self.err_here("expected `target` after `new.`".into()));
        }
        let callee = if self.is_ident("new") {
            self.parse_new_expression()?
        } else {
            self.parse_primary_expression()?
        };
        // Optional argument list.
        let arguments = if matches!(self.current_kind(), TokenKind::Punct(Punct::LParen)) {
            self.parse_arguments()?
        } else { vec![] };
        let end = self.last_span_end();
        Ok(Expr::New { callee: Box::new(callee), arguments, span: Span::new(start, end) })
    }

    fn consume_member_property(&mut self, object: Expr, optional: bool) -> Result<Expr, ParseError> {
        let start = object.span().start;
        let prop = match self.current_kind().clone() {
            TokenKind::Ident(name) => {
                let span = self.lookahead_span();
                self.bump()?;
                MemberProperty::Identifier { name, span }
            }
            TokenKind::PrivateIdent(name) => {
                let span = self.lookahead_span();
                self.bump()?;
                MemberProperty::Private { name, span }
            }
            _ => return Err(self.err_here("expected property name".into())),
        };
        let end = match &prop {
            MemberProperty::Identifier { span, .. } => span.end,
            MemberProperty::Private { span, .. } => span.end,
            MemberProperty::Computed { span, .. } => span.end,
        };
        Ok(Expr::Member {
            object: Box::new(object),
            property: Box::new(prop),
            optional,
            span: Span::new(start, end),
        })
    }

    fn consume_computed_member(&mut self, object: Expr, optional: bool) -> Result<Expr, ParseError> {
        let start = object.span().start;
        self.expect_punct(Punct::LBracket)?;
        let computed = self.parse_assignment_expression()?;
        let computed_span = computed.span();
        self.expect_punct(Punct::RBracket)?;
        let end = self.last_span_end();
        Ok(Expr::Member {
            object: Box::new(object),
            property: Box::new(MemberProperty::Computed { expr: computed, span: computed_span }),
            optional,
            span: Span::new(start, end),
        })
    }

    fn parse_arguments(&mut self) -> Result<Vec<Argument>, ParseError> {
        self.expect_punct(Punct::LParen)?;
        let mut out = Vec::new();
        while !matches!(self.current_kind(), TokenKind::Punct(Punct::RParen)) {
            if matches!(self.current_kind(), TokenKind::Punct(Punct::Spread)) {
                let start = self.lookahead_span().start;
                self.bump()?;
                let expr = self.parse_assignment_expression()?;
                let end = expr.span().end;
                out.push(Argument::Spread { expr, span: Span::new(start, end) });
            } else {
                out.push(Argument::Expr(self.parse_assignment_expression()?));
            }
            if matches!(self.current_kind(), TokenKind::Punct(Punct::Comma)) {
                self.bump()?;
            } else { break; }
        }
        self.expect_punct(Punct::RParen)?;
        Ok(out)
    }

    // ───────────────── PrimaryExpression ─────────────────

    fn parse_primary_expression(&mut self) -> Result<Expr, ParseError> {
        let span = self.lookahead_span();
        match self.current_kind().clone() {
            TokenKind::Ident(name) => {
                match name.as_str() {
                    "null" => { self.bump()?; Ok(Expr::NullLiteral { span }) }
                    "true" => { self.bump()?; Ok(Expr::BoolLiteral { value: true, span }) }
                    "false" => { self.bump()?; Ok(Expr::BoolLiteral { value: false, span }) }
                    "this" => { self.bump()?; Ok(Expr::This { span }) }
                    "super" => { self.bump()?; Ok(Expr::Super { span }) }
                    "import" => {
                        // import.meta or fall through to opaque for dynamic import.
                        let look_end = span.end;
                        let bytes = self.source().as_bytes();
                        let mut p = look_end;
                        while p < bytes.len() && bytes[p].is_ascii_whitespace() { p += 1; }
                        if bytes.get(p) == Some(&b'.') {
                            self.bump()?; // consume `import`
                            self.bump()?; // consume `.`
                            if let TokenKind::Ident(p) = self.current_kind() {
                                if p == "meta" {
                                    let prop = p.clone();
                                    let end = self.lookahead_span().end;
                                    self.bump()?;
                                    return Ok(Expr::MetaProperty {
                                        meta: "import".into(), property: prop, span: Span::new(span.start, end),
                                    });
                                }
                            }
                            return Err(self.err_here("expected `meta` after `import.`".into()));
                        }
                        // dynamic import(...) — opaque for v1
                        self.opaque_until_top_terminator()
                    }
                    _ => { self.bump()?; Ok(Expr::Identifier { name, span }) }
                }
            }
            TokenKind::Number(value, _) => { self.bump()?; Ok(Expr::NumberLiteral { value, span }) }
            TokenKind::BigInt(digits, _) => { self.bump()?; Ok(Expr::BigIntLiteral { digits, span }) }
            TokenKind::String(value) => { self.bump()?; Ok(Expr::StringLiteral { value, span }) }
            TokenKind::Template { cooked, .. } => {
                // Templates with substitutions are deferred to opaque; a
                // NoSubstitution template can be represented as a string.
                if let Some(c) = cooked {
                    self.bump()?;
                    Ok(Expr::StringLiteral { value: c, span })
                } else {
                    self.opaque_until_top_terminator()
                }
            }
            TokenKind::Regex { .. } => {
                // Opaque until regex AST node lands.
                self.bump()?;
                Ok(Expr::Opaque { span })
            }
            TokenKind::Punct(Punct::LBracket) => self.parse_array_literal(),
            TokenKind::Punct(Punct::LBrace) => self.parse_object_literal(),
            TokenKind::Punct(Punct::LParen) => self.parse_parenthesized(),
            _ => Err(self.err_here(format!("unexpected token in expression: {:?}", self.current_kind()))),
        }
    }

    fn parse_array_literal(&mut self) -> Result<Expr, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_punct(Punct::LBracket)?;
        let mut elements = Vec::new();
        while !matches!(self.current_kind(), TokenKind::Punct(Punct::RBracket)) {
            if matches!(self.current_kind(), TokenKind::Punct(Punct::Comma)) {
                let span = self.lookahead_span();
                elements.push(ArrayElement::Elision { span });
                self.bump()?;
                continue;
            }
            if matches!(self.current_kind(), TokenKind::Punct(Punct::Spread)) {
                let sp_start = self.lookahead_span().start;
                self.bump()?;
                let expr = self.parse_assignment_expression()?;
                let end = expr.span().end;
                elements.push(ArrayElement::Spread { expr, span: Span::new(sp_start, end) });
            } else {
                elements.push(ArrayElement::Expr(self.parse_assignment_expression()?));
            }
            if matches!(self.current_kind(), TokenKind::Punct(Punct::Comma)) {
                self.bump()?;
            } else { break; }
        }
        self.expect_punct(Punct::RBracket)?;
        let end = self.last_span_end();
        Ok(Expr::Array { elements, span: Span::new(start, end) })
    }

    fn parse_object_literal(&mut self) -> Result<Expr, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_punct(Punct::LBrace)?;
        let mut properties = Vec::new();
        while !matches!(self.current_kind(), TokenKind::Punct(Punct::RBrace)) {
            if matches!(self.current_kind(), TokenKind::Punct(Punct::Spread)) {
                let sp_start = self.lookahead_span().start;
                self.bump()?;
                let expr = self.parse_assignment_expression()?;
                let end = expr.span().end;
                properties.push(ObjectProperty::Spread { expr, span: Span::new(sp_start, end) });
            } else {
                let prop_start = self.lookahead_span().start;
                let key = self.parse_object_key()?;
                if matches!(self.current_kind(), TokenKind::Punct(Punct::Colon)) {
                    self.bump()?;
                    let value = self.parse_assignment_expression()?;
                    let end = value.span().end;
                    properties.push(ObjectProperty::Property {
                        key, value, shorthand: false, span: Span::new(prop_start, end),
                    });
                } else if matches!(self.current_kind(), TokenKind::Punct(Punct::LParen)) {
                    // Method shorthand — opaque-fallback for v1.
                    let expr = self.opaque_until_top_terminator_within_braces()?;
                    let end = expr.span().end;
                    properties.push(ObjectProperty::Property {
                        key, value: expr, shorthand: false, span: Span::new(prop_start, end),
                    });
                } else {
                    // Bare shorthand `{ x }` — value is Identifier with same name.
                    let (name, key_span) = match &key {
                        ObjectKey::Identifier { name, span } => (name.clone(), *span),
                        _ => return Err(self.err_here("only identifier keys support shorthand".into())),
                    };
                    let value = Expr::Identifier { name, span: key_span };
                    properties.push(ObjectProperty::Property {
                        key, value, shorthand: true, span: Span::new(prop_start, key_span.end),
                    });
                }
            }
            if matches!(self.current_kind(), TokenKind::Punct(Punct::Comma)) {
                self.bump()?;
            } else { break; }
        }
        self.expect_punct(Punct::RBrace)?;
        let end = self.last_span_end();
        Ok(Expr::Object { properties, span: Span::new(start, end) })
    }

    fn parse_object_key(&mut self) -> Result<ObjectKey, ParseError> {
        let span = self.lookahead_span();
        match self.current_kind().clone() {
            TokenKind::Ident(name) => { self.bump()?; Ok(ObjectKey::Identifier { name, span }) }
            TokenKind::String(value) => { self.bump()?; Ok(ObjectKey::String { value, span }) }
            TokenKind::Number(value, _) => { self.bump()?; Ok(ObjectKey::Number { value, span }) }
            TokenKind::Punct(Punct::LBracket) => {
                self.bump()?;
                let expr = self.parse_assignment_expression()?;
                self.expect_punct(Punct::RBracket)?;
                let end = self.last_span_end();
                Ok(ObjectKey::Computed { expr, span: Span::new(span.start, end) })
            }
            _ => Err(self.err_here("expected object key".into())),
        }
    }

    fn parse_parenthesized(&mut self) -> Result<Expr, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_punct(Punct::LParen)?;
        let expr = self.parse_expression()?;
        self.expect_punct(Punct::RParen)?;
        let end = self.last_span_end();
        Ok(Expr::Parenthesized { expr: Box::new(expr), span: Span::new(start, end) })
    }

    /// Comma-separated sequence (a, b, c).
    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        let first = self.parse_assignment_expression()?;
        if !matches!(self.current_kind(), TokenKind::Punct(Punct::Comma)) {
            return Ok(first);
        }
        let start = first.span().start;
        let mut expressions = vec![first];
        while matches!(self.current_kind(), TokenKind::Punct(Punct::Comma)) {
            self.bump()?;
            expressions.push(self.parse_assignment_expression()?);
        }
        let end = expressions.last().unwrap().span().end;
        Ok(Expr::Sequence { expressions, span: Span::new(start, end) })
    }

    // ───────────────── Helpers ─────────────────

    fn looks_like_arrow_function_head(&self) -> bool {
        // Identifier followed by `=>` or `(...)` followed by `=>`.
        // v1 heuristic: if current is Identifier and the byte-source has `=>`
        // before the next `;` at the same paren-depth, treat as arrow.
        match self.current_kind() {
            TokenKind::Ident(_) | TokenKind::Punct(Punct::LParen) => {
                // Crude byte-scan over source from lookahead.start; bail if too long.
                let src = self.source().as_bytes();
                let mut i = self.lookahead_span().start;
                let end = (i + 200).min(src.len());
                let mut paren = 0i32;
                let mut brace = 0i32;
                while i < end {
                    match src[i] {
                        b'(' => paren += 1,
                        b')' => paren -= 1,
                        b'{' => brace += 1,
                        b'}' => brace -= 1,
                        b';' if paren <= 0 && brace == 0 => return false,
                        b'\n' if paren <= 0 && brace == 0 => return false,
                        b'=' if i + 1 < end && src[i + 1] == b'>' => return paren <= 0,
                        _ => {}
                    }
                    i += 1;
                }
                false
            }
            _ => false,
        }
    }

    /// Fallback: skip to top-level `,`, `;`, ASI, or matching close,
    /// returning an Expr::Opaque covering the consumed span.
    fn opaque_until_top_terminator(&mut self) -> Result<Expr, ParseError> {
        let start = self.lookahead_span().start;
        let mut depth_paren = 0i32;
        let mut depth_brace = 0i32;
        let mut depth_bracket = 0i32;
        while !self.at_eof_internal() {
            let kind = self.current_kind().clone();
            match kind {
                TokenKind::Punct(Punct::LParen) => depth_paren += 1,
                TokenKind::Punct(Punct::RParen) => {
                    if depth_paren == 0 { break; }
                    depth_paren -= 1;
                }
                TokenKind::Punct(Punct::LBrace) => depth_brace += 1,
                TokenKind::Punct(Punct::RBrace) => {
                    if depth_brace == 0 { break; }
                    depth_brace -= 1;
                }
                TokenKind::Punct(Punct::LBracket) => depth_bracket += 1,
                TokenKind::Punct(Punct::RBracket) => {
                    if depth_bracket == 0 { break; }
                    depth_bracket -= 1;
                }
                TokenKind::Punct(Punct::Comma) | TokenKind::Punct(Punct::Semicolon) => {
                    if depth_paren == 0 && depth_brace == 0 && depth_bracket == 0 { break; }
                }
                _ => {}
            }
            // ASI break: top-level token preceded by line terminator.
            if depth_paren == 0 && depth_brace == 0 && depth_bracket == 0
                && self.lookahead_preceded_by_lt()
                && self.lookahead_span().start != start
            {
                break;
            }
            self.bump()?;
        }
        let end = self.last_span_end();
        Ok(Expr::Opaque { span: Span::new(start, end) })
    }

    fn opaque_until_top_terminator_within_braces(&mut self) -> Result<Expr, ParseError> {
        let start = self.lookahead_span().start;
        if matches!(self.current_kind(), TokenKind::Punct(Punct::LParen)) {
            self.skip_balanced_public(Punct::LParen, Punct::RParen)?;
        }
        if matches!(self.current_kind(), TokenKind::Punct(Punct::LBrace)) {
            self.skip_balanced_public(Punct::LBrace, Punct::RBrace)?;
        }
        let end = self.last_span_end();
        Ok(Expr::Opaque { span: Span::new(start, end) })
    }

    // ───────────────── FunctionExpression / ClassExpression / ArrowFunction ─────────────────

    fn parse_function_expression(&mut self, is_async: bool) -> Result<Expr, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_keyword("function")?;
        let is_generator = if matches!(self.current_kind(), TokenKind::Punct(Punct::Star)) {
            self.bump()?; true
        } else { false };
        let name = if let TokenKind::Ident(n) = self.current_kind().clone() {
            if !matches!(n.as_str(), "(") {
                let span = self.lookahead_span();
                self.bump()?;
                Some(rusty_js_ast::BindingIdentifier { name: n, span })
            } else { None }
        } else { None };
        let params = self.parse_function_parameters()?;
        let body = self.parse_function_body()?;
        let end = self.last_span_end();
        Ok(Expr::Function {
            name, is_async, is_generator, params, body,
            span: Span::new(start, end),
        })
    }

    fn parse_class_expression(&mut self) -> Result<Expr, ParseError> {
        let start = self.lookahead_span().start;
        self.expect_keyword("class")?;
        let name = if let TokenKind::Ident(n) = self.current_kind().clone() {
            if n != "extends" {
                let span = self.lookahead_span();
                self.bump()?;
                Some(rusty_js_ast::BindingIdentifier { name: n, span })
            } else { None }
        } else { None };
        let super_class = if self.is_ident("extends") {
            self.bump()?;
            Some(Box::new(self.parse_left_hand_side_expression()?))
        } else { None };
        let members = self.parse_class_body()?;
        let end = self.last_span_end();
        Ok(Expr::Class {
            name, super_class, members,
            span: Span::new(start, end),
        })
    }

    fn parse_arrow_function(&mut self, is_async: bool) -> Result<Expr, ParseError> {
        let start = self.lookahead_span().start;
        // Two head forms: bare Identifier or parenthesized parameter list.
        let params: Vec<rusty_js_ast::Parameter> =
            if let TokenKind::Ident(n) = self.current_kind().clone() {
                // `Identifier =>` — single-parameter arrow.
                let span = self.lookahead_span();
                self.bump()?;
                vec![rusty_js_ast::Parameter {
                    names: vec![rusty_js_ast::BindingIdentifier { name: n, span }],
                    default: None, rest: false, span,
                }]
            } else if matches!(self.current_kind(), TokenKind::Punct(Punct::LParen)) {
                self.parse_function_parameters()?
            } else {
                return Err(self.err_here("expected arrow head".into()));
            };
        self.expect_punct(Punct::Arrow)?;
        let body = if matches!(self.current_kind(), TokenKind::Punct(Punct::LBrace)) {
            ArrowBody::Block(self.parse_function_body()?)
        } else {
            ArrowBody::Expression(Box::new(self.parse_assignment_expression()?))
        };
        let end = self.last_span_end();
        Ok(Expr::Arrow { is_async, params, body, span: Span::new(start, end) })
    }
}

