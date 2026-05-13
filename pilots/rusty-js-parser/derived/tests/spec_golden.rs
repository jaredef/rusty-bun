//! Spec golden tests per specs/ecma262-lexical.spec.md.
//!
//! Each test names the spec clause it verifies. Coverage is per-clause:
//! one positive and (where applicable) one negative.

use rusty_js_parser::{Lexer, LexerGoal, NumberKind, Punct, TemplatePart, TokenKind, LexErrorKind};

fn tokens(src: &str) -> Vec<TokenKind> {
    let mut lx = Lexer::new(src);
    let mut out = Vec::new();
    loop {
        let t = lx.next_token(LexerGoal::Div).expect("lex error");
        if matches!(t.kind, TokenKind::Eof) { break; }
        out.push(t.kind);
    }
    out
}

fn tokens_regex_goal(src: &str) -> Vec<TokenKind> {
    let mut lx = Lexer::new(src);
    let mut out = Vec::new();
    loop {
        let t = lx.next_token(LexerGoal::RegExp).expect("lex error");
        if matches!(t.kind, TokenKind::Eof) { break; }
        out.push(t.kind);
    }
    out
}

// ─────────── WhiteSpace + LineTerminator + Comment ───────────

#[test]
fn whitespace_skipped() {
    assert_eq!(tokens("  \tx"), vec![TokenKind::Ident("x".into())]);
}

#[test]
fn line_terminator_sets_preceded_bit() {
    let mut lx = Lexer::new("a\nb");
    let a = lx.next_token(LexerGoal::Div).unwrap();
    let b = lx.next_token(LexerGoal::Div).unwrap();
    assert!(!a.preceded_by_line_terminator);
    assert!(b.preceded_by_line_terminator);
}

#[test]
fn cr_lf_single_line_terminator() {
    let mut lx = Lexer::new("a\r\nb");
    lx.next_token(LexerGoal::Div).unwrap();
    let b = lx.next_token(LexerGoal::Div).unwrap();
    assert!(b.preceded_by_line_terminator);
}

#[test]
fn single_line_comment() {
    assert_eq!(tokens("a // comment\nb"), vec![TokenKind::Ident("a".into()), TokenKind::Ident("b".into())]);
}

#[test]
fn multi_line_comment() {
    assert_eq!(tokens("a /* in\nside */ b"), vec![TokenKind::Ident("a".into()), TokenKind::Ident("b".into())]);
}

#[test]
fn multi_line_comment_with_lt_sets_preceded() {
    let mut lx = Lexer::new("a /* \n */ b");
    lx.next_token(LexerGoal::Div).unwrap();
    let b = lx.next_token(LexerGoal::Div).unwrap();
    assert!(b.preceded_by_line_terminator);
}

#[test]
fn unterminated_block_comment_errors() {
    let mut lx = Lexer::new("/* unterminated");
    let e = lx.next_token(LexerGoal::Div).unwrap_err();
    assert_eq!(e.kind, LexErrorKind::UnterminatedComment);
}

#[test]
fn hashbang_only_at_start() {
    let mut lx = Lexer::new("#!/usr/bin/env bun\nfoo");
    let h = lx.next_token(LexerGoal::Div).unwrap();
    assert!(matches!(h.kind, TokenKind::Hashbang(_)));
    let id = lx.next_token(LexerGoal::Div).unwrap();
    assert_eq!(id.kind, TokenKind::Ident("foo".into()));
}

// ─────────── IdentifierName ───────────

#[test]
fn identifier_ascii() {
    assert_eq!(tokens("foo $bar _baz"),
        vec![TokenKind::Ident("foo".into()), TokenKind::Ident("$bar".into()), TokenKind::Ident("_baz".into())]);
}

#[test]
fn identifier_unicode_escape() {
    assert_eq!(tokens("\\u0061"), vec![TokenKind::Ident("a".into())]);
}

#[test]
fn identifier_unicode_braced_escape() {
    assert_eq!(tokens("\\u{61}"), vec![TokenKind::Ident("a".into())]);
}

#[test]
fn private_identifier() {
    assert_eq!(tokens("#foo"), vec![TokenKind::PrivateIdent("foo".into())]);
}

// ─────────── NumericLiteral ───────────

#[test]
fn decimal_integer() {
    assert_eq!(tokens("42"), vec![TokenKind::Number(42.0, NumberKind::Decimal)]);
}

#[test]
fn decimal_fractional() {
    assert_eq!(tokens("3.14"), vec![TokenKind::Number(3.14, NumberKind::Decimal)]);
}

#[test]
fn decimal_leading_dot() {
    assert_eq!(tokens(".5"), vec![TokenKind::Number(0.5, NumberKind::Decimal)]);
}

#[test]
fn decimal_exponent() {
    let t = tokens("1e3");
    assert_eq!(t.len(), 1);
    if let TokenKind::Number(v, _) = &t[0] { assert!((v - 1000.0).abs() < 1e-9); }
    else { panic!("expected number"); }
}

#[test]
fn hex_literal() {
    assert_eq!(tokens("0xff"), vec![TokenKind::Number(255.0, NumberKind::Hex)]);
}

#[test]
fn binary_literal() {
    assert_eq!(tokens("0b1010"), vec![TokenKind::Number(10.0, NumberKind::Binary)]);
}

#[test]
fn octal_literal() {
    assert_eq!(tokens("0o17"), vec![TokenKind::Number(15.0, NumberKind::Octal)]);
}

#[test]
fn bigint_decimal() {
    assert_eq!(tokens("9007199254740993n"),
        vec![TokenKind::BigInt("9007199254740993".into(), NumberKind::Decimal)]);
}

#[test]
fn bigint_hex() {
    assert_eq!(tokens("0xffn"), vec![TokenKind::BigInt("ff".into(), NumberKind::Hex)]);
}

#[test]
fn numeric_separator() {
    assert_eq!(tokens("1_000_000"), vec![TokenKind::Number(1_000_000.0, NumberKind::Decimal)]);
}

#[test]
fn legacy_octal_rejected() {
    let mut lx = Lexer::new("07");
    let e = lx.next_token(LexerGoal::Div).unwrap_err();
    assert_eq!(e.kind, LexErrorKind::LegacyOctalInModule);
}

#[test]
fn identifier_after_numeric_rejected() {
    let mut lx = Lexer::new("3abc");
    let e = lx.next_token(LexerGoal::Div).unwrap_err();
    assert_eq!(e.kind, LexErrorKind::InvalidNumeric);
}

// ─────────── StringLiteral ───────────

#[test]
fn string_double_quote() {
    assert_eq!(tokens("\"hello\""), vec![TokenKind::String("hello".into())]);
}

#[test]
fn string_single_quote() {
    assert_eq!(tokens("'hello'"), vec![TokenKind::String("hello".into())]);
}

#[test]
fn string_escape_sequences() {
    assert_eq!(tokens(r#""\n\t\"\\""#), vec![TokenKind::String("\n\t\"\\".into())]);
}

#[test]
fn string_hex_escape() {
    assert_eq!(tokens(r#""\x41""#), vec![TokenKind::String("A".into())]);
}

#[test]
fn string_unicode_escape() {
    assert_eq!(tokens(r#""A""#), vec![TokenKind::String("A".into())]);
}

#[test]
fn string_unicode_braced_escape() {
    assert_eq!(tokens(r#""\u{1F600}""#), vec![TokenKind::String("😀".into())]);
}

#[test]
fn string_line_continuation() {
    let src = "\"line1\\\nline2\"";
    assert_eq!(tokens(src), vec![TokenKind::String("line1line2".into())]);
}

#[test]
fn string_octal_escape_rejected() {
    let mut lx = Lexer::new(r#""\7""#);
    let e = lx.next_token(LexerGoal::Div).unwrap_err();
    assert_eq!(e.kind, LexErrorKind::InvalidEscape);
}

#[test]
fn unterminated_string_errors() {
    let mut lx = Lexer::new(r#""hello"#);
    let e = lx.next_token(LexerGoal::Div).unwrap_err();
    assert_eq!(e.kind, LexErrorKind::UnterminatedString);
}

#[test]
fn line_terminator_in_string_rejected() {
    let mut lx = Lexer::new("\"line\nend\"");
    let e = lx.next_token(LexerGoal::Div).unwrap_err();
    assert_eq!(e.kind, LexErrorKind::UnterminatedString);
}

// ─────────── Template literal ───────────

#[test]
fn no_substitution_template() {
    let t = tokens("`hello`");
    assert_eq!(t.len(), 1);
    if let TokenKind::Template { cooked, raw, part } = &t[0] {
        assert_eq!(cooked.as_deref(), Some("hello"));
        assert_eq!(raw, "hello");
        assert_eq!(*part, TemplatePart::NoSubstitution);
    } else { panic!("expected template"); }
}

#[test]
fn template_head_and_tail() {
    // `a${x}b`
    let mut lx = Lexer::new("`a${x}b`");
    let head = lx.next_token(LexerGoal::Div).unwrap();
    let x = lx.next_token(LexerGoal::Div).unwrap();
    let tail = lx.next_token(LexerGoal::TemplateTail).unwrap();
    assert!(matches!(&head.kind, TokenKind::Template { part: TemplatePart::Head, .. }));
    assert_eq!(x.kind, TokenKind::Ident("x".into()));
    assert!(matches!(&tail.kind, TokenKind::Template { part: TemplatePart::Tail, .. }));
}

// ─────────── Regex literal ───────────

#[test]
fn regex_simple() {
    let t = tokens_regex_goal("/abc/g");
    assert_eq!(t.len(), 1);
    if let TokenKind::Regex { body, flags } = &t[0] {
        assert_eq!(body, "abc");
        assert_eq!(flags, "g");
    } else { panic!("expected regex"); }
}

#[test]
fn regex_with_class() {
    let t = tokens_regex_goal("/[abc/def]/");
    assert_eq!(t.len(), 1);
    if let TokenKind::Regex { body, .. } = &t[0] {
        assert_eq!(body, "[abc/def]");
    } else { panic!("expected regex"); }
}

#[test]
fn regex_escape() {
    let t = tokens_regex_goal(r"/\d+/");
    assert_eq!(t.len(), 1);
    if let TokenKind::Regex { body, .. } = &t[0] {
        assert_eq!(body, r"\d+");
    } else { panic!("expected regex"); }
}

#[test]
fn unterminated_regex_errors() {
    let mut lx = Lexer::new("/abc");
    let e = lx.next_token(LexerGoal::RegExp).unwrap_err();
    assert_eq!(e.kind, LexErrorKind::UnterminatedRegex);
}

// ─────────── Punctuators ───────────

#[test]
fn structural_punctuators() {
    assert_eq!(tokens("(){}[],;:"), vec![
        TokenKind::Punct(Punct::LParen), TokenKind::Punct(Punct::RParen),
        TokenKind::Punct(Punct::LBrace), TokenKind::Punct(Punct::RBrace),
        TokenKind::Punct(Punct::LBracket), TokenKind::Punct(Punct::RBracket),
        TokenKind::Punct(Punct::Comma), TokenKind::Punct(Punct::Semicolon),
        TokenKind::Punct(Punct::Colon),
    ]);
}

#[test]
fn arrow_and_optional_chain() {
    assert_eq!(tokens("=> ?."), vec![
        TokenKind::Punct(Punct::Arrow),
        TokenKind::Punct(Punct::OptionalChain),
    ]);
}

#[test]
fn strict_equality() {
    assert_eq!(tokens("=== !=="), vec![
        TokenKind::Punct(Punct::StrictEq), TokenKind::Punct(Punct::StrictNe),
    ]);
}

#[test]
fn unsigned_right_shift_assign() {
    assert_eq!(tokens(">>>="), vec![TokenKind::Punct(Punct::UShrAssign)]);
}

#[test]
fn logical_assignment() {
    assert_eq!(tokens("&&= ||= ??="), vec![
        TokenKind::Punct(Punct::LogicalAndAssign),
        TokenKind::Punct(Punct::LogicalOrAssign),
        TokenKind::Punct(Punct::NullishAssign),
    ]);
}

#[test]
fn spread_punct() {
    assert_eq!(tokens("..."), vec![TokenKind::Punct(Punct::Spread)]);
}

#[test]
fn optional_chain_vs_decimal() {
    // `?.5` is `?` then `.5` (per spec — no optional-chain when followed by digit)
    assert_eq!(tokens("?.5"), vec![
        TokenKind::Punct(Punct::Question),
        TokenKind::Number(0.5, NumberKind::Decimal),
    ]);
}

// ─────────── Composed ───────────

#[test]
fn import_declaration_token_stream() {
    let t = tokens("import { x as 'm-search' } from 'pkg';");
    assert_eq!(t, vec![
        TokenKind::Ident("import".into()),
        TokenKind::Punct(Punct::LBrace),
        TokenKind::Ident("x".into()),
        TokenKind::Ident("as".into()),
        TokenKind::String("m-search".into()),
        TokenKind::Punct(Punct::RBrace),
        TokenKind::Ident("from".into()),
        TokenKind::String("pkg".into()),
        TokenKind::Punct(Punct::Semicolon),
    ]);
}

#[test]
fn export_default_function() {
    let t = tokens("export default function fetch() {}");
    assert_eq!(t, vec![
        TokenKind::Ident("export".into()),
        TokenKind::Ident("default".into()),
        TokenKind::Ident("function".into()),
        TokenKind::Ident("fetch".into()),
        TokenKind::Punct(Punct::LParen),
        TokenKind::Punct(Punct::RParen),
        TokenKind::Punct(Punct::LBrace),
        TokenKind::Punct(Punct::RBrace),
    ]);
}
