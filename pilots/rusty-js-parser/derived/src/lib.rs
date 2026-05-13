//! rusty-js-parser — ECMAScript module-goal parser.
//!
//! v1 scope: lexer only. Subsequent sub-rounds add the parser proper and
//! the diagnostics layer. Spec corpus: specs/ecma262-lexical.spec.md +
//! specs/ecma262-module.spec.md.

pub mod token;
pub mod lexer;

pub use token::{Token, TokenKind, Punct, NumberKind, TemplatePart, Span};
pub use lexer::{Lexer, LexError, LexErrorKind, LexerGoal};
