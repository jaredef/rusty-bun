# ECMA-262 — Module-Level Grammar

[surface] Parser (module goal symbol)
[spec] https://tc39.es/ecma262/#sec-modules
[engagement role] The module-level grammar slice required to recognize the import/export forms surfaced by the parity-119 corpus. Together with ecma262-lexical.spec.md and ecma262-expressions.spec.md, this constitutes the rusty-js-parser pilot's constraint corpus per Tier-Ω.3.a. Bounded by the (abstract-op × rung) tuple analysis in Doc 717 + the P3 classification.

## Module goal symbol

- The Module production is the top-level goal symbol when parsing ECMAScript module text
- Module: ModuleBody?
- ModuleBody: ModuleItemList
- ModuleItemList: ModuleItem | ModuleItemList ModuleItem
- ModuleItem: ImportDeclaration | ExportDeclaration | StatementListItem

## ImportDeclaration

Form 1 — bare import (side-effect):
- `import ModuleSpecifier ;`

Form 2 — default import:
- `import ImportedDefaultBinding from ModuleSpecifier ;`
- ImportedDefaultBinding: an IdentifierName that is bound at module evaluation start

Form 3 — namespace import:
- `import * as ImportedBinding from ModuleSpecifier ;`

Form 4 — named imports:
- `import { NamedImports } from ModuleSpecifier ;`
- NamedImports: `{}` | `{ ImportsList ,? }`
- ImportSpecifier: ModuleExportName | ModuleExportName as ImportedBinding
- ModuleExportName: IdentifierName | StringLiteral  (ES2022+)

Form 5 — combined default + named or default + namespace:
- `import ImportedDefaultBinding, NamedImports from ModuleSpecifier ;`
- `import ImportedDefaultBinding, * as ImportedBinding from ModuleSpecifier ;`

Form 6 — dynamic import (call-expression form, ES2020+):
- `import ( AssignmentExpression , AssertClause? )` — returns a Promise resolving to the Module Namespace Object
- Permitted in any expression context, not only at top level

Form 7 — import.meta (ES2020+):
- `import . meta` — evaluates to the Module's meta object; the host provides its properties (url, resolve, etc. per the WHATWG HTML integration)

## ExportDeclaration

Form 1 — export of a single declaration:
- `export VariableStatement` — exports each VariableDeclaration name
- `export Declaration` where Declaration is one of:
  - HoistableDeclaration (function / generator / async function / async generator)
  - ClassDeclaration
  - LexicalDeclaration (let / const)

Form 2 — re-export from another module:
- `export * from ModuleSpecifier ;` — re-exports all named exports of the source module except `default`
- `export * as IdentifierName from ModuleSpecifier ;` (ES2020+) — re-exports the source module's Module Namespace as a single named export
- `export NamedExports from ModuleSpecifier ;` — re-exports a selected set

Form 3 — export from local bindings:
- `export NamedExports ;` — exports each name from the local lexical environment
- NamedExports: `{}` | `{ ExportsList ,? }`
- ExportSpecifier: ModuleExportName | ModuleExportName as ModuleExportName
- ModuleExportName: IdentifierName | StringLiteral  (ES2022+ — direct relevance to Tuple C parity)

Form 4 — default export:
- `export default HoistableDeclaration[~Yield, ~Await, +Default]`
- `export default ClassDeclaration[~Yield, ~Await, +Default]`
- `export default [lookahead ∉ {function, async function*, class}] AssignmentExpression ;`
- The default declaration may be anonymous; the resulting export binding is named "default"
- When the declaration is a HoistableDeclaration or ClassDeclaration with a name, the name IS visible as a local binding within the module but is NOT exported as a named export — only the default-export binding is created per ECMA-262 §16.2.3.4. This is the spec-conformant behavior. **Bun's behavior (per Doc 717 Tuple B) extends this with a host-defined hook that exposes the name as a named export.**

## ImportAttributes / ImportAssertions

- `assert { type: "json" }` (Stage-3 ES2024, formerly "import assertions") is recognized as an AssertClause in ImportDeclaration and dynamic import call-expressions
- ES2024 renamed to "Import Attributes" with `with { type: "json" }` syntax; both forms tolerated for transition

## ModuleSpecifier

- ModuleSpecifier is a StringLiteral
- The lexer-emitted StringLiteral is passed through to the host's module-resolution hook unchanged
- Bare specifiers (no `./`, no `/`, no `file:`) are host-defined; rusty-js-parser passes them through to the host's NodeResolver

## Linking semantics (parser-relevant)

- Each ImportSpecifier creates an import-binding entry on the Module Record
- Each ExportSpecifier creates an export-binding entry on the Module Record
- `export *` creates a star-export entry; resolution at link time walks the target module's exports
- Per ECMA-262 §16.2.1.6 (Cyclic Module Records) and §16.2.1.7 (Source Text Module Records), the parser is responsible for constructing the ImportEntry and ExportEntry lists for the Module Record; actual resolution happens at link time

## Module Namespace Object (parser-adjacent)

- Per ECMA-262 §16.2.1.10 (Module Namespace Objects), the Module Namespace Object is an exotic object with custom [[OwnPropertyKeys]], [[Get]], [[Set]], [[DefineOwnProperty]], [[Delete]] internal methods
- The spec specifies these internal methods completely; engines must implement them per the algorithm
- **Doc 717 Tuple A finding:** the spec does NOT specify *when* the namespace's exports table is finalized — engines may augment between ParseModule and the first access. Bun augments; rquickjs freezes at construction. rusty-js (the v1 engine) cuts at E5: the namespace's exports table is finalized at the host-defined `HostFinalizeModuleNamespace` hook, called between Link and Evaluate phases, after the host has had an opportunity to inject synthetic bindings per the Tuple A/B closure path.

## Out-of-scope for v1 (engagement-bounded)

- HoistableDeclaration: GeneratorDeclaration and AsyncGeneratorDeclaration parser-side recognition is in scope; full runtime semantics deferred to a successor round
- Top-level await: parser recognizes; runtime semantics deferred until the engine's microtask + reactor integration lands
- Decorators (Stage-3 proposal): out of scope until Stage-4 advance
- Type annotations (TypeScript syntax): permanently out of scope — rusty-js is an ECMAScript engine, not a TypeScript one
- Source phase imports (`import source ...`): out of scope until Stage-4 advance

## Tuple-A/B/C parity targets (cross-reference to Doc 717)

| Tuple | Spec section | Engine cut |
|---|---|---|
| A (Module Namespace augmentation) | §16.2.1.10 [[OwnPropertyKeys]] | E5 host hook at HostFinalizeModuleNamespace |
| B (named-export synthesis from default) | §16.2.3.4 default export binding | E5 host hook at HostFinalizeModuleNamespace, applied after default evaluates |
| C (ModuleExportName as StringLiteral) | §16.2.2 ImportDeclaration / §16.2.3 ExportDeclaration | E1 grammar production — direct implementation per ES2022+ spec text |
