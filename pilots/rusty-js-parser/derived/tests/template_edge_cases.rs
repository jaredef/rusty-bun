//! Tier-Ω.5.gg acceptance tests for templates with substitutions inside
//! declarations whose body was previously walked by `skip_balanced`. The
//! bug was that `export function f() { return `X: ${a}`; }` (and similarly
//! for `export class`) caused `parse_declaration_for_export` to treat the
//! `${alg}` closing `}` as the function body's closing `}`, polluting the
//! lexer's brace count and emitting `UnterminatedTemplate` at the next
//! real backtick. The fix routes the function/class body through the
//! typed statement parser so the lexer's TemplateTail goal switch fires.

use rusty_js_parser::parse_module;

fn ok(src: &str) {
    match parse_module(src) {
        Ok(_) => {}
        Err(e) => panic!("expected OK, got {:?} for source:\n{}", e, src),
    }
}

#[test]
fn template_with_shift_substitution() {
    // jose: `SHA-${keySize << 1}` — the original failing pattern.
    ok("const a = `SHA-${k << 1}`;");
}

#[test]
fn template_with_multiple_substitutions() {
    ok("const a = `${x.y.z[0]} ${d * e}`;");
}

#[test]
fn template_with_escaped_backticks() {
    // chalk-template / p-map: `Bracket (\`}\`)`.
    ok(r"const m = `before \`x\` after`;");
}

#[test]
fn template_substitution_then_escape() {
    ok(r"const m = `${x} \`y\``;");
}

#[test]
fn template_in_call_argument_list() {
    ok("f(`abc`, 1);");
}

#[test]
fn template_in_conditional_arms() {
    ok("const r = x ? `yes` : `no`;");
}

#[test]
fn export_function_with_template_substitution() {
    // The Tier-Ω.5.gg regression: `export function` body walked by
    // skip_balanced. Pre-fix this errored as UnterminatedTemplate.
    ok(r#"
        export function f(alg) {
            return `X: ${alg}`;
        }
    "#);
}

#[test]
fn export_function_with_multi_substitution_body() {
    ok(r#"
        export function checkLen(cek, expected) {
            const actual = cek.byteLength << 3;
            if (actual !== expected) {
                throw new E(`Invalid length. Expected ${expected} bits, got ${actual} bits`);
            }
        }
    "#);
}

#[test]
fn export_class_method_with_template_substitution() {
    ok(r#"
        export class K {
            m(alg) {
                return `SHA-${alg << 1}`;
            }
        }
    "#);
}

#[test]
fn export_function_with_escaped_backticks_inside() {
    // chalk-template's pattern.
    ok(r#"
        export function brace(n) {
            return `Bracket${n === 1 ? '' : 's'} (\`}\`)`;
        }
    "#);
}
