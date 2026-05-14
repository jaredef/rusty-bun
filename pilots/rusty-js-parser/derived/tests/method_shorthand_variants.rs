//! Tier-Omega.5.p.parse — object-literal method shorthand variants.
//!
//! Coverage: getter / setter / generator method shorthand in object
//! literals. v1 deviation (documented in expr.rs): getter/setter drop
//! accessor-descriptor semantics — the accessor function is stored as
//! a plain function-valued property. Real getter/setter behavior is
//! queued for a follow-on substrate round when Object.defineProperty
//! accessor descriptors land. The acceptance bar is parser-level only:
//! `parse_module` returns Ok.

use rusty_js_parser::parse_module;

#[test]
fn object_getter_shorthand() {
    let src = "const o = { get foo() { return 1; } };";
    parse_module(src).expect("getter shorthand");
}

#[test]
fn object_setter_shorthand() {
    let src = "const o = { set bar(v) { this.x = v; } };";
    parse_module(src).expect("setter shorthand");
}

#[test]
fn object_generator_shorthand() {
    let src = "const o = { *gen() { yield 1; } };";
    parse_module(src).expect("generator shorthand");
}

#[test]
fn object_mixed_method_kinds() {
    let src = "const o = { plain() { return 1; }, get acc() { return 2; }, set sett(v) {} };";
    parse_module(src).expect("mixed method kinds");
}

#[test]
fn object_get_as_method_name() {
    // Disambiguation: `get` as method name (followed directly by `(`),
    // not as an accessor marker. Must still parse as plain method shorthand.
    let src = "const o = { get() { return 1; } };";
    parse_module(src).expect("get as method name");
}

#[test]
fn object_get_as_plain_key() {
    // Disambiguation: `get` as a plain shorthand key followed by `,` or `}`.
    let src = "const get = 1; const o = { get };";
    parse_module(src).expect("get as plain shorthand key");
}
