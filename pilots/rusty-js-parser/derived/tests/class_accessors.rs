//! Tier-Omega.5.u — class-member getter / setter parser coverage.
//!
//! The compiler-side v1 deviation (semantics dropped, lowered as plain
//! function-valued properties) is documented in compiler.rs. The parser
//! itself produces ClassMember::Method with the correct MethodKind tag.
//! These assertions pin that contract.

use rusty_js_ast::{ClassMember, ClassMemberName, MethodKind, ModuleItem, Stmt};
use rusty_js_parser::parse_module;

fn class_members(src: &str) -> Vec<ClassMember> {
    let module = parse_module(src).expect("parse_module");
    for item in module.body {
        if let ModuleItem::Statement(Stmt::ClassDecl { members, .. }) = item {
            return members;
        }
    }
    panic!("expected ClassDecl in {:?}", src);
}

#[test]
fn class_getter_method_kind() {
    let members = class_members("class C { get foo() { return 1; } }");
    let m = members.first().expect("one member");
    if let ClassMember::Method { kind, name, .. } = m {
        assert_eq!(*kind, MethodKind::Getter);
        if let ClassMemberName::Identifier { name: n, .. } = name {
            assert_eq!(n, "foo");
        } else { panic!("expected identifier name"); }
    } else { panic!("expected Method, got {:?}", m); }
}

#[test]
fn class_setter_method_kind() {
    let members = class_members("class C { set bar(v) { this.x = v; } }");
    if let Some(ClassMember::Method { kind, name, .. }) = members.first() {
        assert_eq!(*kind, MethodKind::Setter);
        if let ClassMemberName::Identifier { name: n, .. } = name {
            assert_eq!(n, "bar");
        } else { panic!("expected identifier name"); }
    } else { panic!("expected Method"); }
}

#[test]
fn class_get_as_method_name_disambiguation() {
    // `get()` followed directly by `(` is a plain method named "get",
    // not a getter modifier — same disambiguation as Ω.5.p.parse's
    // object-literal side.
    let members = class_members("class C { get() { return 1; } }");
    if let Some(ClassMember::Method { kind, name, .. }) = members.first() {
        assert_eq!(*kind, MethodKind::Method);
        if let ClassMemberName::Identifier { name: n, .. } = name {
            assert_eq!(n, "get");
        } else { panic!("expected identifier name"); }
    } else { panic!("expected Method"); }
}
