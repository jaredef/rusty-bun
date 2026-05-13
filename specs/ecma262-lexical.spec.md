# ECMA-262 — Lexical Grammar

[surface] Lexer
[spec] https://tc39.es/ecma262/#sec-ecmascript-language-lexical-grammar
[engagement role] Substrate seed for rusty-js-parser per Tier-Ω.3.a; the lexical layer required to recognize ECMA-262 module-level grammar (see ecma262-module.spec.md). Subset bounded by Bun-parity targets per [Doc 717](https://jaredfoy.com/resolve/doc/717-the-apparatus-above-the-engine-boundary-the-three-projections-lifted-to-engine-substrate-and-the-pure-abstraction-point) — surfaces not exercised by the parity-119 corpus are bracketed for follow-on.

## SourceCharacter and input element division

- ECMAScript source text is a sequence of Unicode code points
- The lexer divides input into a sequence of InputElements: WhiteSpace, LineTerminator, Comment, CommonToken, DivPunctuator, RightBracePunctuator, RegularExpressionLiteral, TemplateSubstitutionTail
- Goal symbol selection is context-dependent: InputElementDiv / InputElementRegExp / InputElementRegExpOrTemplateTail / InputElementTemplateTail / InputElementHashbangOrRegExp

## WhiteSpace

- WhiteSpace is one of: TAB (U+0009), VT (U+000B), FF (U+000C), SP (U+0020), NBSP (U+00A0), ZWNBSP (U+FEFF), or any code point with Unicode "Space_Separator" general category
- Multiple WhiteSpace code points form a single WhiteSpace token; they are insignificant except as separators

## LineTerminator

- LineTerminator is one of: LF (U+000A), CR (U+000D), LS (U+2028), PS (U+2029)
- CR LF sequence is a single LineTerminator
- LineTerminators are significant to automatic semicolon insertion and to single-line comment termination

## Comment

- Comment is MultiLineComment (`/* ... */`) or SingleLineComment (`// ...` to end of line)
- MultiLineComment containing a LineTerminator counts as a LineTerminator for ASI purposes
- HashbangComment (`#!` followed by characters to end of line) is permitted only at the very start of the source text

## CommonToken

- CommonToken is one of: IdentifierName, PrivateIdentifier, Punctuator, NumericLiteral, StringLiteral, Template
- NumericLiteralSeparator (U+005F LOW LINE) is permitted between digits in numeric literals (ES2021+)

## IdentifierName

- IdentifierName starts with an IdentifierStartCharacter; subsequent characters are IdentifierPartCharacters
- IdentifierStartCharacter: any code point with the Unicode "ID_Start" property, plus `$`, `_`, and `\u`-prefixed UnicodeEscapeSequence
- IdentifierPartCharacter: any code point with the Unicode "ID_Continue" property, plus `$`, ZWNJ (U+200C), ZWJ (U+200D), and `\u`-prefixed UnicodeEscapeSequence
- A `\u`-escape evaluates to its represented code point before identifier validity is checked
- IdentifierName covers all reserved words (keyword, future-reserved, contextually-reserved); whether a particular IdentifierName is a ReservedWord is a parser-level concern, not a lexer-level concern

## Reserved words

- Per §13.2 Keywords-and-Reserved-Words:
  - Always reserved: `await` (in module bodies and async functions), `break`, `case`, `catch`, `class`, `const`, `continue`, `debugger`, `default`, `delete`, `do`, `else`, `enum`, `export`, `extends`, `false`, `finally`, `for`, `function`, `if`, `import`, `in`, `instanceof`, `new`, `null`, `return`, `super`, `switch`, `this`, `throw`, `true`, `try`, `typeof`, `var`, `void`, `while`, `with`, `yield`
  - Strict-mode reserved: `implements`, `interface`, `package`, `private`, `protected`, `public`, `static`, `let` (let strict-reserved as identifier; allowed contextually as binding)
  - Contextually-reserved: `as`, `async`, `from`, `get`, `meta`, `of`, `set`, `target`
- ES2024 reserved-word list is the operative reference; older editions had additional reserved words now released (boolean, byte, char, double, final, float, goto, int, long, native, short, synchronized, throws, transient, volatile)

## PrivateIdentifier

- PrivateIdentifier is `#` followed by IdentifierName
- Permitted as a property name within a class body; not permitted at script/module top level

## Punctuator

- Punctuator covers: `{` `(` `)` `[` `]` `.` `...` `;` `,` `<` `>` `<=` `>=` `==` `!=` `===` `!==` `+` `-` `*` `%` `**` `++` `--` `<<` `>>` `>>>` `&` `|` `^` `!` `~` `&&` `||` `??` `?` `:` `=` `+=` `-=` `*=` `%=` `**=` `<<=` `>>=` `>>>=` `&=` `|=` `^=` `&&=` `||=` `??=` `=>`
- DivPunctuator: `/` `/=`
- RightBracePunctuator: `}` (split out because it terminates template substitutions in addition to closing blocks)
- OptionalChainingPunctuator: `?.`

## NumericLiteral

- DecimalLiteral: optional integer part + optional fractional part + optional ExponentPart
- DecimalBigIntegerLiteral: integer part + `n` suffix
- BinaryIntegerLiteral: `0b` or `0B` + binary digits (`0` `1`), with optional `n` suffix for BigInt
- OctalIntegerLiteral: `0o` or `0O` + octal digits, with optional `n` suffix
- HexIntegerLiteral: `0x` or `0X` + hex digits, with optional `n` suffix
- LegacyOctalIntegerLiteral: `0` followed by octal digits (non-strict only; rejected in strict mode and in module code)
- NonOctalDecimalIntegerLiteral: `0` followed by digits including `8` or `9` (non-strict only)
- NumericLiteralSeparator (`_`) permitted between digits; cannot lead, trail, or appear adjacent to other separators or radix prefix

## StringLiteral

- StringLiteral: `"..."` or `'...'`
- LineContinuation: `\` + LineTerminatorSequence inside a string is permitted and contributes nothing to the value
- EscapeSequence: `\n`, `\t`, `\r`, `\b`, `\f`, `\v`, `\0` (NUL only when not followed by digit), `\'`, `\"`, `\\`, `\xHH`, `\uHHHH`, `\u{H..H}`
- LegacyOctalEscapeSequence (`\NNN`): permitted only outside strict mode and template literals
- ZWNBSP and other LineTerminators are not permitted unescaped in StringLiteral

## Template literals

- NoSubstitutionTemplate: `` `...` `` with no `${` substitutions
- TemplateHead: `` `...${ ``
- TemplateMiddle: `}...${`
- TemplateTail: `}...` ``
- TemplateCharacter: the character set permitted within a template, including LineTerminators (preserved literally in cooked values per spec; `\` + LineTerminator is line continuation)
- Cooked vs raw values: escape sequences in cooked form are processed; raw form preserves the source characters
- TaggedTemplate detection is a parser-level concern; the lexer emits the substring tokens (NoSubstitutionTemplate, TemplateHead/Middle/Tail) and the surrounding Expression tokens

## RegularExpressionLiteral

- RegularExpressionLiteral: `/RegularExpressionBody/RegularExpressionFlags`
- RegularExpressionBody: characters that are not line terminators, with escape sequences (`\.`) and character class brackets (`[...]`) handled specially
- RegularExpressionFlags: zero or more IdentifierPartCharacters (the parser later validates against the set `g i m s u y d v`)
- Whether a `/` begins a RegularExpressionLiteral or DivPunctuator is determined by goal-symbol context (InputElementRegExp vs InputElementDiv)

## Automatic semicolon insertion (ASI) — lexer-relevant signals

- ASI is a parser-level mechanism but depends on lexer-emitted LineTerminator markers
- The lexer preserves LineTerminator positions through to the parser; the parser inserts virtual semicolons per the three ASI rules
- The lexer emits a "preceded-by-LineTerminator" bit on each token so the parser can apply ASI without re-walking whitespace

## Out-of-scope for v1 (engagement-bounded)

- Unicode general-category property tables (parser ships with a precomputed subset table indexed by code point; full Unicode tables are a successor refinement)
- LegacyOctalIntegerLiteral and LegacyOctalEscapeSequence (both forbidden in module code; rusty-js-parser is module-only in v1)
- HashbangComment only at source start (implemented; not extended to nested files)
- Annex B web extensions (HTMLOpenComment, HTMLCloseComment) — rejected in module code per spec
