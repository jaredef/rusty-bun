//! Tier-Omega.5.p.parse — trailing comma in import/export specifier lists.
//!
//! Spec: ImportClause / NamedExports both allow an optional trailing comma
//! before the closing `}`. Real-world usage: entities and many others.

use rusty_js_parser::parse_module;

#[test]
fn export_named_trailing_comma_from() {
    let src = r#"export { a, b, } from "./m";"#;
    parse_module(src).expect("export trailing comma");
}

#[test]
fn import_named_trailing_comma() {
    let src = r#"import { a, b, c, } from "./m";"#;
    parse_module(src).expect("import trailing comma");
}

#[test]
fn mixed_trailing_and_non_trailing() {
    let src = r#"export { a, b, } from "./m"; export { c, d } from "./n";"#;
    parse_module(src).expect("mixed trailing/non-trailing");
}
