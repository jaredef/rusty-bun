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
                let f = self.parse_function_expression(true)?;
                return self.continue_lhs_continuation(f);
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
            let f = self.parse_function_expression(false)?;
            return self.continue_lhs_continuation(f);
        }
        if self.is_ident("class") {
            let c = self.parse_class_expression()?;
            return self.continue_lhs_continuation(c);
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
        let expr = if self.is_ident("new") {
            self.parse_new_expression()?
        } else {
            self.parse_primary_expression()?
        };
        self.continue_lhs_continuation(expr)
    }

    /// Tier-Ω.5.xx: run the Member/Call/Tagged-Template continuation loop
    /// against an already-parsed primary expression. Lets callers that
    /// short-circuit primary-expression parsing (function expression,
    /// class expression at expression position) still pick up `(...)` /
    /// `.x` / `?.x` / `\`tpl\`` continuations — without which the UMD
    /// idiom `(function(){...}(this, factory))` and `}.constructor`-style
    /// access on a function expression fail.
    pub(crate) fn continue_lhs_continuation(&mut self, mut expr: Expr) -> Result<Expr, ParseError> {
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
                TokenKind::Template { .. } => {
                    // Tier-Ω.5.ww: tagged template literal. Lower to a Call
                    // where the first argument is an Array of the quasi
                    // strings and the remaining arguments are the
                    // interpolation expressions. v1 deviation: the strings
                    // array does not carry `.raw` (so String.raw returns
                    // cooked values), but most tag uses don't depend on
                    // `.raw`. camelcase / consola / styled-components
                    // patterns parse.
                    let start = expr.span().start;
                    expr = self.parse_tagged_template(expr, start)?;
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
        let mut callee = if self.is_ident("new") {
            self.parse_new_expression()?
        } else {
            self.parse_primary_expression()?
        };
        // Tier-Ω.5.ppp: consume MemberExpression continuation (`.x` /
        // `[x]`) so `new X.Y.Z(args)` parses as new (X.Y.Z)(args) per
        // ECMA-262 §13.3.5. Do NOT consume Call (parentheses) here —
        // the first `(args)` after the chain belongs to the new.
        // minimatch's `new Minimatch(pattern, options).match(p)` pattern
        // depends on this.
        loop {
            match self.current_kind() {
                TokenKind::Punct(Punct::Dot) => {
                    self.bump()?;
                    callee = self.consume_member_property(callee, false)?;
                }
                TokenKind::Punct(Punct::LBracket) => {
                    callee = self.consume_computed_member(callee, false)?;
                }
                _ => break,
            }
        }
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
                        // Tier-Ω.5.ff: dynamic `import(specifier)` lowered
                        // as a synthesized call to a global `__dynamic_import`
                        // helper. v1 deviation: the helper is a stub that
                        // throws on actual invocation; static-side compile
                        // succeeds so packages whose dynamic-import is inside
                        // conditional branches (never driven during shape
                        // probe) pass without erroring at compile time.
                        self.bump()?; // consume `import`
                        if !matches!(self.current_kind(), TokenKind::Punct(Punct::LParen)) {
                            return Err(self.err_here("expected '(' after `import`".into()));
                        }
                        self.bump()?; // consume '('
                        let arg = self.parse_assignment_expression()?;
                        if !matches!(self.current_kind(), TokenKind::Punct(Punct::RParen)) {
                            return Err(self.err_here("expected ')' in dynamic import()".into()));
                        }
                        let end = self.lookahead_span().end;
                        self.bump()?; // consume ')'
                        let span = Span::new(span.start, end);
                        Ok(Expr::Call {
                            callee: Box::new(Expr::Identifier {
                                name: "__dynamic_import".into(),
                                span,
                            }),
                            arguments: vec![rusty_js_ast::Argument::Expr(arg)],
                            optional: false,
                            span,
                        })
                    }
                    // Tier-Ω.5.yy: function / class expressions at any
                    // sub-expression position (not just at the head of
                    // AssignmentExpression). io-ts / rxjs's polyfill
                    // pattern `... && function(d,b){...}` and many
                    // boolean-guarded function expressions depend on
                    // this. `async function` is also recognized so
                    // `(true && async function(){})` parses.
                    "function" => self.parse_function_expression(false),
                    "class" => self.parse_class_expression(),
                    "async" => {
                        // Only consume as async-function when the next
                        // non-whitespace token is `function`. Otherwise
                        // fall through to plain identifier — bare `async`
                        // as a value is valid at lower precedence.
                        let bytes = self.source().as_bytes();
                        let mut p = span.end;
                        while p < bytes.len() && bytes[p].is_ascii_whitespace() { p += 1; }
                        if bytes[p..].starts_with(b"function") {
                            self.bump()?; // consume `async`
                            self.parse_function_expression(true)
                        } else {
                            self.bump()?;
                            Ok(Expr::Identifier { name, span })
                        }
                    }
                    _ => { self.bump()?; Ok(Expr::Identifier { name, span }) }
                }
            }
            TokenKind::Number(value, _) => { self.bump()?; Ok(Expr::NumberLiteral { value, span }) }
            TokenKind::BigInt(digits, _) => { self.bump()?; Ok(Expr::BigIntLiteral { digits, span }) }
            TokenKind::String(value) => { self.bump()?; Ok(Expr::StringLiteral { value, span }) }
            TokenKind::Template { cooked, part, .. } => {
                use crate::token::TemplatePart;
                match part {
                    TemplatePart::NoSubstitution => {
                        let value = cooked.unwrap_or_default();
                        self.bump()?;
                        Ok(Expr::StringLiteral { value, span })
                    }
                    TemplatePart::Head => self.parse_template_with_substitutions(span.start),
                    TemplatePart::Middle | TemplatePart::Tail =>
                        Err(self.err_here("unexpected template middle/tail in expression position".into())),
                }
            }
            TokenKind::Regex { body, flags } => {
                let pattern = std::rc::Rc::new(body.clone());
                let flags = std::rc::Rc::new(flags.clone());
                self.bump()?;
                Ok(Expr::RegExp { pattern, flags, span })
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
            } else if matches!(self.current_kind(), TokenKind::Punct(Punct::Star)) {
                // Tier-Ω.5.p.parse: generator method shorthand `*name(params) { body }`.
                // Lower to a plain property with a generator-flagged
                // Function expression as the value. Runtime/compile may
                // refuse generators; parsing past the syntax is the goal.
                let prop_start = self.lookahead_span().start;
                self.bump()?; // consume `*`
                let key = self.parse_object_key()?;
                let params = self.parse_function_parameters()?;
                let body = self.parse_function_body()?;
                let end = self.last_span_end();
                let func = Expr::Function {
                    name: None,
                    is_async: false,
                    is_generator: true,
                    params,
                    body,
                    span: Span::new(prop_start, end),
                };
                properties.push(ObjectProperty::Property {
                    key, value: func, shorthand: false, span: Span::new(prop_start, end),
                });
            } else if self.looks_like_async_method_shorthand() {
                // Tier-Ω.5.vv: `async name(...) { body }` object method
                // shorthand. Drop async semantics in v1 (runtime treats it
                // as a regular function); the parse-past is what unblocks
                // p-limit + many others.
                let prop_start = self.lookahead_span().start;
                self.bump()?; // consume `async`
                let key = self.parse_object_key()?;
                let params = self.parse_function_parameters()?;
                let body = self.parse_function_body()?;
                let end = self.last_span_end();
                let func = Expr::Function {
                    name: None,
                    is_async: true,
                    is_generator: false,
                    params,
                    body,
                    span: Span::new(prop_start, end),
                };
                properties.push(ObjectProperty::Property {
                    key, value: func, shorthand: false, span: Span::new(prop_start, end),
                });
            } else if self.looks_like_accessor_shorthand() {
                // Tier-Ω.5.p.parse: getter/setter shorthand `get name() { body }` /
                // `set name(v) { body }`. v1 deviation: drop accessor-descriptor
                // semantics — the accessor function is stored as a plain
                // function-valued property under the accessor's name. Real
                // getter/setter behavior is queued for a follow-on substrate
                // round when Object.defineProperty accessor descriptors land.
                let prop_start = self.lookahead_span().start;
                self.bump()?; // consume `get` or `set`
                let key = self.parse_object_key()?;
                let params = self.parse_function_parameters()?;
                let body = self.parse_function_body()?;
                let end = self.last_span_end();
                let func = Expr::Function {
                    name: None,
                    is_async: false,
                    is_generator: false,
                    params,
                    body,
                    span: Span::new(prop_start, end),
                };
                properties.push(ObjectProperty::Property {
                    key, value: func, shorthand: false, span: Span::new(prop_start, end),
                });
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
                    // Tier-Ω.5.l: method shorthand `{ name(params) { body } }` —
                    // lower to `{ name: function(params) { body } }`. The parser
                    // reads parameters + body identically to a function
                    // expression; the synthesized FunctionExpression carries
                    // an anonymous name (the method name is the property key,
                    // not the function's [[Name]] in v1).
                    let params = self.parse_function_parameters()?;
                    let body = self.parse_function_body()?;
                    let end = self.last_span_end();
                    let func = Expr::Function {
                        name: None,
                        is_async: false,
                        is_generator: false,
                        params,
                        body,
                        span: Span::new(prop_start, end),
                    };
                    properties.push(ObjectProperty::Property {
                        key, value: func, shorthand: false, span: Span::new(prop_start, end),
                    });
                } else {
                    // Bare shorthand `{ x }` — value is Identifier with same name.
                    // Tier-Ω.5.uu: also accept `{ x = expr }` (CoverInitializedName
                    // from the spec's cover grammar). This form is only meaningful
                    // when the surrounding object literal is later reinterpreted
                    // as an AssignmentPattern (destructuring); syntactically it
                    // must parse. emit_destructure_assign already consumes
                    // Expr::Assign leaves with AssignOp::Assign as default-value.
                    // p-limit / many libs depend on this.
                    let (name, key_span) = match &key {
                        ObjectKey::Identifier { name, span } => (name.clone(), *span),
                        _ => return Err(self.err_here("only identifier keys support shorthand".into())),
                    };
                    let ident = Expr::Identifier { name: name.clone(), span: key_span };
                    let value = if matches!(self.current_kind(), TokenKind::Punct(Punct::Assign)) {
                        self.bump()?;
                        let default = self.parse_assignment_expression()?;
                        let end = default.span().end;
                        Expr::Assign {
                            operator: rusty_js_ast::AssignOp::Assign,
                            target: Box::new(ident),
                            value: Box::new(default),
                            span: Span::new(key_span.start, end),
                        }
                    } else {
                        ident
                    };
                    let val_end = value.span().end;
                    properties.push(ObjectProperty::Property {
                        key, value, shorthand: true, span: Span::new(prop_start, val_end),
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

    /// Tier-Ω.5.p.parse: distinguish `get`/`set` used as accessor markers
    /// from `get`/`set` used as a plain property key. Returns true when
    /// the current token is the identifier `get` or `set` AND the next
    /// non-whitespace source byte (after the identifier's bytes) starts
    /// a property-key shape: identifier-start, string quote, digit, or `[`.
    /// If followed by `(`, `:`, `,`, `}`, or `=`, it's a plain key.
    fn parse_tagged_template(&mut self, tag: Expr, start: usize) -> Result<Expr, ParseError> {
        use crate::token::TemplatePart;
        use rusty_js_ast::{ArrayElement, Argument};
        // Parse the template literal into an Expr::TemplateLiteral first,
        // then convert into a Call with [Array(quasis), ...expressions].
        let tpl = match self.current_kind().clone() {
            TokenKind::Template { cooked, part, .. } => {
                let tspan = self.lookahead_span();
                match part {
                    TemplatePart::NoSubstitution => {
                        let value = cooked.unwrap_or_default();
                        self.bump()?;
                        Expr::TemplateLiteral {
                            quasis: vec![std::rc::Rc::new(value)],
                            expressions: Vec::new(),
                            span: tspan,
                        }
                    }
                    TemplatePart::Head => self.parse_template_with_substitutions(tspan.start)?,
                    _ => return Err(self.err_here("unexpected template part for tag".into())),
                }
            }
            _ => return Err(self.err_here("expected template after tag".into())),
        };
        let (quasis, expressions, end) = match tpl {
            Expr::TemplateLiteral { quasis, expressions, span } => (quasis, expressions, span.end),
            _ => unreachable!(),
        };
        let strings_arr = Expr::Array {
            elements: quasis.iter().map(|q| ArrayElement::Expr(Expr::StringLiteral {
                value: (**q).clone(),
                span: Span::new(start, end),
            })).collect(),
            span: Span::new(start, end),
        };
        let mut arguments: Vec<Argument> = vec![Argument::Expr(strings_arr)];
        for e in expressions {
            arguments.push(Argument::Expr(e));
        }
        Ok(Expr::Call {
            callee: Box::new(tag),
            arguments,
            optional: false,
            span: Span::new(start, end),
        })
    }

    fn looks_like_async_method_shorthand(&self) -> bool {
        let is_async_ident = matches!(self.current_kind(), TokenKind::Ident(n) if n == "async");
        if !is_async_ident { return false; }
        let src = self.source().as_bytes();
        let span = self.lookahead_span();
        let mut j = span.end;
        // Allow space/tab only; an unescaped newline after `async` makes it
        // an expression (`async\n foo` is `async; foo` per ASI).
        while j < src.len() {
            match src[j] {
                b' ' | b'\t' => j += 1,
                _ => break,
            }
        }
        match src.get(j) {
            Some(&b) if b.is_ascii_alphabetic() || b == b'_' || b == b'$' => true,
            Some(&b'"') | Some(&b'\'') => true,
            Some(&b'[') => true,
            _ => false,
        }
    }

    fn looks_like_accessor_shorthand(&self) -> bool {
        let is_accessor_ident = match self.current_kind() {
            TokenKind::Ident(n) => n == "get" || n == "set",
            _ => false,
        };
        if !is_accessor_ident { return false; }
        let src = self.source().as_bytes();
        let span = self.lookahead_span();
        // Skip whitespace and line terminators after the identifier.
        let mut j = span.end;
        while j < src.len() {
            match src[j] {
                b' ' | b'\t' | b'\n' | b'\r' => j += 1,
                _ => break,
            }
        }
        match src.get(j) {
            Some(&b) if b.is_ascii_alphabetic() || b == b'_' || b == b'$' => true,
            Some(&b'"') | Some(&b'\'') => true,
            Some(&b'[') => true,
            Some(&b) if b.is_ascii_digit() => true,
            _ => false,
        }
    }

    fn looks_like_arrow_function_head(&self) -> bool {
        // Tight detection per the AssignmentExpression-arrow grammar:
        //   - `Identifier =>` (single-parameter arrow)
        //   - `(...) =>` where `(...)` is a balanced paren group, then `=>`
        // Neither form scans past the close of its head. This avoids
        // capturing `a ? b : () => c` as an arrow starting at `b`.
        let src = self.source().as_bytes();
        let start = self.lookahead_span().start;
        match self.current_kind() {
            TokenKind::Ident(name) => {
                // Reject obvious non-arrow-head reserved words.
                if matches!(name.as_str(),
                    "typeof" | "void" | "delete" | "await" | "yield" | "new"
                    | "function" | "class" | "this" | "super" | "null"
                    | "true" | "false" | "return" | "throw" | "if" | "else"
                    | "for" | "while" | "do" | "switch" | "case" | "default"
                    | "break" | "continue" | "try" | "catch" | "finally"
                    | "var" | "let" | "const" | "import" | "export") {
                    return false;
                }
                // Skip past the identifier's bytes.
                let mut j = start;
                while j < src.len() && (src[j].is_ascii_alphanumeric() || src[j] == b'_' || src[j] == b'$') { j += 1; }
                // Then whitespace.
                while j < src.len() && (src[j] == b' ' || src[j] == b'\t') { j += 1; }
                src.get(j) == Some(&b'=') && src.get(j + 1) == Some(&b'>')
            }
            TokenKind::Punct(Punct::LParen) => {
                // Scan to the matching `)` then look for `=>` immediately after.
                let mut j = start + 1;
                let mut depth = 1i32;
                while j < src.len() && depth > 0 {
                    match src[j] {
                        b'(' => depth += 1,
                        b')' => depth -= 1,
                        b'\'' | b'"' => {
                            // Walk past a string literal.
                            let q = src[j];
                            j += 1;
                            while j < src.len() && src[j] != q {
                                if src[j] == b'\\' && j + 1 < src.len() { j += 2; continue; }
                                j += 1;
                            }
                        }
                        b'`' => {
                            // Walk past a template literal (no substitutions handled).
                            j += 1;
                            while j < src.len() && src[j] != b'`' { j += 1; }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                // After matching `)`, skip whitespace, then check `=>`.
                while j < src.len() && (src[j] == b' ' || src[j] == b'\t') { j += 1; }
                src.get(j) == Some(&b'=') && src.get(j + 1) == Some(&b'>')
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

    /// Walks a `\`head${expr}middle${expr}tail\`` template-literal token
    /// stream. Tier-Ω.5.g.2 substrate round: returns
    /// Expr::TemplateLiteral with quasis (cooked strings) and
    /// expressions captured in source order. Compiler lowering to a
    /// concat chain lands in Ω.5.g.3.
    fn parse_template_with_substitutions(&mut self, start: usize) -> Result<Expr, ParseError> {
        use crate::token::TemplatePart;
        let mut quasis: Vec<std::rc::Rc<String>> = Vec::new();
        let mut expressions: Vec<Expr> = Vec::new();
        // Consume Head, capturing its cooked text as the first quasi.
        let head_cooked = match self.current_kind().clone() {
            TokenKind::Template { cooked, part: TemplatePart::Head, .. } => cooked.unwrap_or_default(),
            _ => return Err(self.err_here("expected template head".into())),
        };
        quasis.push(std::rc::Rc::new(head_cooked));
        self.bump()?; // consume Head
        loop {
            // Parse the substitution expression.
            let expr = self.parse_expression()?;
            expressions.push(expr);
            // After the substitution, the lookahead is `}` (under Div goal
            // since the substitution completes an expression). Re-lex
            // starting at that `}` with TemplateTail goal to emit a
            // Middle/Tail token.
            self.refetch_lookahead_with_goal(crate::lexer::LexerGoal::TemplateTail)?;
            match self.current_kind().clone() {
                TokenKind::Template { cooked, part: TemplatePart::Middle, .. } => {
                    quasis.push(std::rc::Rc::new(cooked.unwrap_or_default()));
                    self.bump()?;
                    continue;
                }
                TokenKind::Template { cooked, part: TemplatePart::Tail, .. } => {
                    quasis.push(std::rc::Rc::new(cooked.unwrap_or_default()));
                    self.bump()?;
                    break;
                }
                _ => return Err(self.err_here("expected template middle/tail after substitution".into())),
            }
        }
        let end = self.last_span_end();
        Ok(Expr::TemplateLiteral { quasis, expressions, span: Span::new(start, end) })
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
                    target: rusty_js_ast::BindingPattern::Identifier(
                        rusty_js_ast::BindingIdentifier { name: n, span }
                    ),
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

