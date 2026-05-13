//! Spec golden tests for the module-goal parser per specs/ecma262-module.spec.md.
//!
//! Coverage: each ImportDeclaration form, each ExportDeclaration form,
//! the Doc 717 Tuple-C closure (string-literal ModuleExportName), and the
//! Module-Record entry-table derivation.

use rusty_js_ast::*;
use rusty_js_parser::parse_module;

fn module(src: &str) -> Module {
    parse_module(src).expect(&format!("parse failed for {:?}", src))
}

// ─────────── ImportDeclaration forms ───────────

#[test]
fn import_bare_specifier() {
    let m = module("import 'side-effect';");
    assert_eq!(m.body.len(), 1);
    if let ModuleItem::Import(d) = &m.body[0] {
        assert_eq!(d.specifier.value, "side-effect");
        assert!(d.default_binding.is_none());
        assert!(d.namespace_binding.is_none());
        assert!(d.named_imports.is_empty());
    } else { panic!("expected import"); }
    assert!(m.import_entries.is_empty());
}

#[test]
fn import_default() {
    let m = module("import x from 'pkg';");
    assert_eq!(m.import_entries, vec![
        ImportEntry { module_request: "pkg".into(), import_name: ImportName::Default, local_name: "x".into() },
    ]);
}

#[test]
fn import_namespace() {
    let m = module("import * as ns from 'pkg';");
    assert_eq!(m.import_entries, vec![
        ImportEntry { module_request: "pkg".into(), import_name: ImportName::Namespace, local_name: "ns".into() },
    ]);
}

#[test]
fn import_named() {
    let m = module("import { a, b as c } from 'pkg';");
    assert_eq!(m.import_entries, vec![
        ImportEntry { module_request: "pkg".into(), import_name: ImportName::Single("a".into()), local_name: "a".into() },
        ImportEntry { module_request: "pkg".into(), import_name: ImportName::Single("b".into()), local_name: "c".into() },
    ]);
}

#[test]
fn import_default_plus_named() {
    let m = module("import D, { a, b } from 'pkg';");
    assert_eq!(m.import_entries, vec![
        ImportEntry { module_request: "pkg".into(), import_name: ImportName::Default, local_name: "D".into() },
        ImportEntry { module_request: "pkg".into(), import_name: ImportName::Single("a".into()), local_name: "a".into() },
        ImportEntry { module_request: "pkg".into(), import_name: ImportName::Single("b".into()), local_name: "b".into() },
    ]);
}

#[test]
fn import_default_plus_namespace() {
    let m = module("import D, * as ns from 'pkg';");
    assert_eq!(m.import_entries, vec![
        ImportEntry { module_request: "pkg".into(), import_name: ImportName::Default, local_name: "D".into() },
        ImportEntry { module_request: "pkg".into(), import_name: ImportName::Namespace, local_name: "ns".into() },
    ]);
}

#[test]
fn import_string_literal_module_export_name() {
    // ES2022 form — Tuple-C target.
    let m = module(r#"import { "m-search" as ms } from 'pkg';"#);
    assert_eq!(m.import_entries, vec![
        ImportEntry { module_request: "pkg".into(), import_name: ImportName::Single("m-search".into()), local_name: "ms".into() },
    ]);
}

#[test]
fn import_with_attributes() {
    let m = module(r#"import data from 'data.json' with { type: "json" };"#);
    if let ModuleItem::Import(d) = &m.body[0] {
        assert_eq!(d.attributes.len(), 1);
        if let ModuleExportName::Ident(b) = &d.attributes[0].key {
            assert_eq!(b.name, "type");
        } else { panic!("expected ident key"); }
        assert_eq!(d.attributes[0].value, "json");
    } else { panic!("expected import"); }
}

#[test]
fn import_with_assert_attributes() {
    let m = module(r#"import data from 'data.json' assert { type: "json" };"#);
    if let ModuleItem::Import(d) = &m.body[0] {
        assert_eq!(d.attributes.len(), 1);
    } else { panic!("expected import"); }
}

// ─────────── ExportDeclaration forms ───────────

#[test]
fn export_named_local() {
    let m = module("const x = 1; export { x };");
    assert_eq!(m.local_export_entries, vec![
        ExportEntry { export_name: Some("x".into()), module_request: None, import_name: None, local_name: Some("x".into()) },
    ]);
}

#[test]
fn export_named_with_alias() {
    let m = module("const x = 1; export { x as y };");
    assert_eq!(m.local_export_entries, vec![
        ExportEntry { export_name: Some("y".into()), module_request: None, import_name: None, local_name: Some("x".into()) },
    ]);
}

#[test]
fn export_string_literal_export_name() {
    // ES2022 — Tuple-C target.
    let m = module(r#"const x = 1; export { x as "m-search" };"#);
    assert_eq!(m.local_export_entries, vec![
        ExportEntry { export_name: Some("m-search".into()), module_request: None, import_name: None, local_name: Some("x".into()) },
    ]);
}

#[test]
fn export_named_from() {
    let m = module("export { x } from 'pkg';");
    assert_eq!(m.indirect_export_entries, vec![
        ExportEntry {
            export_name: Some("x".into()),
            module_request: Some("pkg".into()),
            import_name: Some(ExportImportName::Single("x".into())),
            local_name: None,
        },
    ]);
}

#[test]
fn export_star_from() {
    let m = module("export * from 'pkg';");
    assert_eq!(m.star_export_entries, vec![
        ExportEntry {
            export_name: None,
            module_request: Some("pkg".into()),
            import_name: Some(ExportImportName::All),
            local_name: None,
        },
    ]);
}

#[test]
fn export_star_as_from() {
    let m = module("export * as ns from 'pkg';");
    assert_eq!(m.indirect_export_entries, vec![
        ExportEntry {
            export_name: Some("ns".into()),
            module_request: Some("pkg".into()),
            import_name: Some(ExportImportName::All),
            local_name: None,
        },
    ]);
}

#[test]
fn export_default_expression() {
    let m = module("export default 42;");
    assert_eq!(m.local_export_entries.len(), 1);
    assert_eq!(m.local_export_entries[0].export_name.as_deref(), Some("default"));
    assert_eq!(m.local_export_entries[0].local_name.as_deref(), Some("*default*"));
}

#[test]
fn export_default_named_function() {
    // Doc 717 Tuple-B target: the function's NAME is captured by the parser
    // even though spec doesn't expose it as a named export. Bun's E5 host
    // hook would synthesize a named export with this name at HostFinalize.
    let m = module("export default function fetch() {}");
    if let ModuleItem::Export(ExportDeclaration::Default { body, .. }) = &m.body[0] {
        if let DefaultExportBody::HoistableFunction { name, is_async, is_generator, .. } = body {
            assert_eq!(name.as_ref().unwrap().name, "fetch");
            assert!(!is_async);
            assert!(!is_generator);
        } else { panic!("expected function"); }
    } else { panic!("expected default export"); }
    // The spec-conformant ExportEntry has local_name = the function name.
    assert_eq!(m.local_export_entries[0].local_name.as_deref(), Some("fetch"));
}

#[test]
fn export_default_async_function() {
    let m = module("export default async function fetch() {}");
    if let ModuleItem::Export(ExportDeclaration::Default { body, .. }) = &m.body[0] {
        if let DefaultExportBody::HoistableFunction { name, is_async, .. } = body {
            assert_eq!(name.as_ref().unwrap().name, "fetch");
            assert!(is_async);
        } else { panic!("expected function"); }
    } else { panic!("expected default export"); }
}

#[test]
fn export_default_anonymous_function() {
    let m = module("export default function () {}");
    if let ModuleItem::Export(ExportDeclaration::Default { body, .. }) = &m.body[0] {
        if let DefaultExportBody::HoistableFunction { name, .. } = body {
            assert!(name.is_none());
        } else { panic!("expected function"); }
    } else { panic!("expected default export"); }
}

#[test]
fn export_default_class() {
    let m = module("export default class Foo {}");
    if let ModuleItem::Export(ExportDeclaration::Default { body, .. }) = &m.body[0] {
        if let DefaultExportBody::Class { name, .. } = body {
            assert_eq!(name.as_ref().unwrap().name, "Foo");
        } else { panic!("expected class"); }
    } else { panic!("expected default export"); }
}

#[test]
fn export_const_declaration() {
    let m = module("export const x = 1, y = 2;");
    let names: Vec<&str> = m.local_export_entries.iter()
        .map(|e| e.export_name.as_deref().unwrap()).collect();
    assert_eq!(names, vec!["x", "y"]);
}

#[test]
fn export_function_declaration() {
    let m = module("export function foo() {}");
    let names: Vec<&str> = m.local_export_entries.iter()
        .map(|e| e.export_name.as_deref().unwrap()).collect();
    assert_eq!(names, vec!["foo"]);
}

#[test]
fn export_class_declaration() {
    let m = module("export class Bar {}");
    let names: Vec<&str> = m.local_export_entries.iter()
        .map(|e| e.export_name.as_deref().unwrap()).collect();
    assert_eq!(names, vec!["Bar"]);
}

#[test]
fn multiple_imports_and_exports() {
    let src = r#"
        import a from 'a';
        import { b } from 'b';
        import * as c from 'c';
        export { a, b as renamed };
        export const d = 1;
        export default function fetch() {}
    "#;
    let m = module(src);
    assert_eq!(m.import_entries.len(), 3);
    // a (local), renamed (alias of b), d, default — local_export_entries
    let names: Vec<&str> = m.local_export_entries.iter()
        .map(|e| e.export_name.as_deref().unwrap()).collect();
    assert_eq!(names, vec!["a", "renamed", "d", "default"]);
}

#[test]
fn statement_or_decl_passes_through() {
    let m = module("const x = 1; const y = 2; export { x, y };");
    // Two const declarations + one export — body should have at least the export
    let exports: Vec<_> = m.body.iter().filter(|i| matches!(i, ModuleItem::Export(_))).collect();
    assert_eq!(exports.len(), 1);
    let names: Vec<&str> = m.local_export_entries.iter()
        .map(|e| e.export_name.as_deref().unwrap()).collect();
    assert_eq!(names, vec!["x", "y"]);
}

#[test]
fn empty_module() {
    let m = module("");
    assert!(m.body.is_empty());
    assert!(m.import_entries.is_empty());
}

#[test]
fn module_with_hashbang() {
    let m = module("#!/usr/bin/env node\nexport const x = 1;");
    let names: Vec<&str> = m.local_export_entries.iter()
        .map(|e| e.export_name.as_deref().unwrap()).collect();
    assert_eq!(names, vec!["x"]);
}
