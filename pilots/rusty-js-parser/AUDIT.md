# rusty-js-parser — coverage audit

**First sub-pilot of Tier-Ω.3.b** (the architectural tier-1 pilots that compose into the hand-rolled JS engine). Per [Doc 717](https://jaredfoy.com/resolve/doc/717-the-apparatus-above-the-engine-boundary-the-three-projections-lifted-to-engine-substrate-and-the-pure-abstraction-point) + the [Ω.3 engine-selection decision artifact](../../host/tools/omega-3-engine-selection.md) + the [P3 classification](../../host/tools/p3-classification.md).

## Engagement role

The parser is the smallest piece that opens the engine substrate. ECMA-262's parser grammar is well-specified (specs/ecma262-lexical.spec.md + specs/ecma262-module.spec.md + a follow-on ecma262-expressions.spec.md) and the engagement's existing pilot discipline reads cleanly against it. The pilot lands ahead of the AST-typed-node pilot (rusty-js-ast), the bytecode-compiler pilot (rusty-js-bytecode), the runtime pilot (rusty-js-runtime), and the GC pilot (rusty-js-gc).

## Tuple-C closure is the parser-resident parity target

Per Doc 717 + P3, Tuple C (ParseModule × E1 × version-lag) accounts for 2 of the 14 residual parity failures: superagent and ora. Both gate on parser grammar features that QuickJS does not accept:

- **superagent** — uses `export { x as 'm-search' }` (string-literal export aliases, ES2022); QuickJS's parser rejects with "identifier expected"
- **ora** — uses ES2022+ syntax that QuickJS's parser rejects with "expecting ';'"

Additionally, the basket boundaries E.12 (hono ^4 class-field arrow-fn variant), E.60 (elysia minified ESM SIGSEGV), and E.62 (yargs syntax form) are all Tuple-C-class boundaries; the rusty-js-parser pilot's broader-grammar acceptance retires all of them in one closure.

## Pilot scope (v1 — module-only)

The pilot is **module-only** in v1. Script-goal parsing (the looser non-module top-level grammar) is deferred; rusty-bun's existing FsLoader is module-only at the consumer-visible surface anyway.

Three composed surfaces in a single crate:

### Lexer
- Source-character-stream input; UTF-8 decoded eagerly to a `&[u32]` of code points (or `&str` with byte/char-boundary tracking)
- InputElement classification per spec (WhiteSpace, LineTerminator, Comment, CommonToken, DivPunctuator, RightBracePunctuator, RegularExpressionLiteral, TemplateSubstitutionTail)
- Goal-symbol-aware: callers select InputElementDiv / InputElementRegExp / InputElementRegExpOrTemplateTail / InputElementTemplateTail
- Token output: `Token { kind, span, preceded_by_line_terminator }` with `span` carrying byte-offset range into the source
- Numeric literal parsing: full Decimal, BigInt, Binary, Octal, Hex, NumericLiteralSeparator
- String literal cooking: escape sequences resolved at lex time; raw-form preserved for templates
- Template literal: emits NoSubstitutionTemplate / TemplateHead / TemplateMiddle / TemplateTail with cooked + raw values
- Regex literal: tokenized as a single RegularExpressionLiteral with body + flags; not re-parsed at lex time (the engine's RegExp constructor compiles later)
- Hashbang at source start
- ZWNJ / ZWJ in identifiers
- `\u`-escapes in identifiers, decoded before IdentifierName validity check

### Parser
- Recursive-descent over the Module goal symbol
- AST: typed enum tree (defined in rusty-js-ast pilot; rusty-js-parser depends on rusty-js-ast for the node types)
- ImportDeclaration: all 7 forms in specs/ecma262-module.spec.md, including ES2022 string-literal ModuleExportName
- ExportDeclaration: all 4 forms, including `export * as`, `export NamedExports from`, ES2022 string-literal ModuleExportName, default with HoistableDeclaration/ClassDeclaration/AssignmentExpression
- Statement: Block, VariableStatement, EmptyStatement, ExpressionStatement, IfStatement, BreakableStatement (DoWhile, While, For, ForIn, ForOf), ContinueStatement, BreakStatement, ReturnStatement, WithStatement (rejected in strict mode and modules), LabelledStatement, ThrowStatement, TryStatement, DebuggerStatement
- Declaration: HoistableDeclaration (FunctionDeclaration, GeneratorDeclaration, AsyncFunctionDeclaration, AsyncGeneratorDeclaration), ClassDeclaration, LexicalDeclaration
- Expression: full ECMAScript expression grammar through assignment expressions (precedence climbing); details in the follow-on ecma262-expressions.spec.md
- Automatic semicolon insertion per the three ASI rules

### Diagnostics
- `ParseError { kind, span, message, expected: Option<Vec<TokenKind>> }`
- Span carries byte-offset range; the consumer can recover line/column with a separate source-map facility
- Error-recovery is best-effort: synchronize at statement boundaries to keep parsing past one bad statement, accumulate ParseErrors in a vector

## Constraint inputs

| Spec extract | Clauses |
|---|---:|
| specs/ecma262-lexical.spec.md | ~80 (token rules + grammar productions) |
| specs/ecma262-module.spec.md | ~40 (ImportDeclaration + ExportDeclaration forms + Module Record construction) |
| specs/ecma262-expressions.spec.md (TBD this engagement) | ~200 (full expression grammar + precedence) |

The parser is **maximally spec-driven** — ECMA-262 prose specifies the entire grammar without any host-defined latitude at the parser layer. Per Doc 707, when the spec layer dominates, the pilot's correctness criterion is direct grammar conformance against the spec text, not against the test corpus.

## Test corpus

Three layers:

1. **Spec golden tests** — for each spec-extract clause, at least one positive test (source parses) and at least one negative test (source rejected with the correct ParseError). Verifies clause-by-clause coverage of the spec.

2. **Parity-119 source sample** — the 119 packages from the parity-measurement corpus, each loaded and parsed against rusty-js-parser. The acceptance rate is a parity-residual metric: 100% indicates Tuple-C closure at the parser layer.

3. **Test262 module subset (deferred)** — TC39's official conformance suite has a module-grammar slice. Pulling it into the engagement is a successor round; v1 of this pilot ships the engagement's curated sample.

## Out of scope for v1

- Script-goal parsing (the looser top-level)
- Sloppy-mode-only syntax (LegacyOctalIntegerLiteral, LegacyOctalEscapeSequence, HTMLOpenComment, HTMLCloseComment, WithStatement)
- Decorators (Stage-3; not stable)
- Type annotations (permanent out-of-scope)
- Source-phase imports (Stage-3; not stable)
- Pretty-printer / source-map emitter — neither is needed for the engine substrate

## Composition with downstream pilots

- **rusty-js-ast** (next sub-pilot of Ω.3.b) — defines the typed AST nodes; rusty-js-parser depends on it. AST types are emitted by the parser and consumed by rusty-js-bytecode.
- **rusty-js-bytecode** — consumes AST, emits a single-pass bytecode instruction stream. The engagement's QuickJS-architectural-reference commitment puts a single-pass compiler here.
- **rusty-js-runtime** — consumes bytecode, executes against the runtime's Value representation. The intrinsic-object inventory + execution-context records + realm record + Module Namespace exotic object with the E5 host hook for Tuple A/B closure all live here.
- **rusty-js-gc** — services rusty-js-runtime. Conservative mark-sweep in v1.

## Estimated pilot size

- Lexer: ~700 LOC
- Parser: ~1500 LOC
- AST type bindings: pulled from rusty-js-ast (~300 LOC)
- Tests: ~800 LOC for spec golden + ~200 LOC for parity-119 source acceptance

Total ~3,200 LOC. This is the largest pilot in the engagement, exceeding the TLS substrate (~2,625 LOC of pure-Rust under pilots/tls). Per the substrate-amortization discipline, the pilot ships in 3-5 rounds: lexer alone → parser scaffold + ImportDeclaration → ExportDeclaration → Statement + Declaration → Expression.

## First-round scope (this commit — Ω.3.a)

This commit lands the **substrate-introduction round only**:
- specs/ecma262-lexical.spec.md
- specs/ecma262-module.spec.md
- This AUDIT.md

No derived/ Cargo crate is created in this round. The next round (Ω.3.b round 1) creates pilots/rusty-js-parser/derived/ + the lexer.

## Tier-Ω.3.a cross-reference

Per the [Ω.3 engine-selection decision artifact](../../host/tools/omega-3-engine-selection.md) §III, this pilot opens the Tier-Ω.3.b sequence: rusty-js-parser → rusty-js-ast → rusty-js-bytecode → rusty-js-runtime → rusty-js-gc. The parser is the entry point because every other pilot in the sequence depends on the AST shape, which depends on the parser's grammar.
