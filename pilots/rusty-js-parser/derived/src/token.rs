//! Token types emitted by the lexer.
//!
//! Per specs/ecma262-lexical.spec.md. The TokenKind enum spans every
//! InputElement category the module-goal grammar can consume.

// Span is shared with rusty-js-ast so the parser can flow a single type
// across lexer + AST. Re-exported here for the lexer's convenience.
pub use rusty_js_ast::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    /// Per ECMA-262 ASI: a LineTerminator (or MultiLineComment containing
    /// one) immediately before this token sets this bit. Parser consults
    /// it for automatic semicolon insertion.
    pub preceded_by_line_terminator: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// IdentifierName, including all reserved-word lexemes. Whether a
    /// particular identifier is reserved is a parser-layer concern; the
    /// lexer simply emits the cooked name.
    Ident(String),

    /// PrivateIdentifier — `#` followed by IdentifierName (no leading `#`).
    PrivateIdent(String),

    /// Numeric literals. `NumberKind` discriminates the radix and BigInt suffix.
    Number(f64, NumberKind),

    /// BigInt literal. The string form is the digit-portion only (no `n` suffix),
    /// matching the form a BigInt constructor would accept.
    BigInt(String, NumberKind),

    /// String literal — cooked value with escapes resolved.
    String(String),

    /// Template literal token. `TemplatePart` discriminates the position.
    Template {
        cooked: Option<String>, // None when a forbidden escape would have erred per tagged-template-strict rules
        raw: String,
        part: TemplatePart,
    },

    /// Regular expression literal: body + flags as separate strings.
    /// The lexer does not validate flags or body grammar.
    Regex { body: String, flags: String },

    /// Punctuator (operators + structural symbols).
    Punct(Punct),

    /// Hashbang at source start (`#!...` to end of line). Emitted once,
    /// at the start, when present.
    Hashbang(String),

    /// End of file.
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberKind {
    Decimal,
    Hex,
    Binary,
    Octal,
    /// Legacy octal (leading `0` then octal digits, no `o`). Permitted only
    /// in sloppy script-goal source; lexer emits it but parser rejects
    /// when in module/strict mode. v1 of the parser is module-only so the
    /// lexer can also reject; kept here for future script-goal extension.
    LegacyOctal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplatePart {
    NoSubstitution,
    Head,
    Middle,
    Tail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Punct {
    // Structural
    LBrace, RBrace, LParen, RParen, LBracket, RBracket,
    Semicolon, Comma, Colon, Dot, Spread,
    Arrow, OptionalChain,

    // Comparison + equality
    Lt, Gt, Le, Ge, Eq, Ne, StrictEq, StrictNe,

    // Arithmetic + bitwise
    Plus, Minus, Star, Percent, StarStar, Slash,
    Inc, Dec,
    Shl, Shr, UShr,
    BitAnd, BitOr, BitXor, BitNot,
    LogicalNot, LogicalAnd, LogicalOr, NullishCoalesce,
    Question,

    // Assignments
    Assign,
    PlusAssign, MinusAssign, StarAssign, PercentAssign, StarStarAssign, SlashAssign,
    ShlAssign, ShrAssign, UShrAssign,
    BitAndAssign, BitOrAssign, BitXorAssign,
    LogicalAndAssign, LogicalOrAssign, NullishAssign,
}
