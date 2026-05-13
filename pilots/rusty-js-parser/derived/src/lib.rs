//! rusty-js-parser — ECMAScript module-goal parser.
//!
//! Spec corpus: specs/ecma262-lexical.spec.md + specs/ecma262-module.spec.md.
//! Module-goal only in v1. Statement and expression bodies are captured as
//! opaque byte-spans until the expression-grammar sub-round.

pub mod token;
pub mod lexer;
pub mod parser;

pub use token::{Token, TokenKind, Punct, NumberKind, TemplatePart, Span};
pub use lexer::{Lexer, LexError, LexErrorKind, LexerGoal};
pub use parser::{Parser, ParseError};

/// Convenience: parse a complete module from a source string. Returns the
/// AST or the first parse error.
pub fn parse_module(src: &str) -> Result<rusty_js_ast::Module, ParseError> {
    let mut p = Parser::new(src)?;
    p.parse_module()
}
