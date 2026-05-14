//! Tier-Ω.5.d acceptance: compound assignment + prefix/postfix update.
//!
//! Covers all AssignOp variants the parser emits (arithmetic, bitwise,
//! shift, logical/nullish with short-circuit), plus prefix/postfix
//! increment and decrement, across the four legal assignment targets:
//! bare identifier, static member (`o.p`), computed member (`o[k]`),
//! and index (`a[i]`).

use rusty_js_runtime::{run_module, Value};

fn run(src: &str) -> Value {
    run_module(src).unwrap_or_else(|e| panic!("run failed for {:?}: {:?}", src, e))
}

// ───────── Arithmetic compound ─────────

#[test]
fn t01_add_assign() {
    assert_eq!(run("let n = 5; n += 3; return n;"), Value::Number(8.0));
}

#[test]
fn t02_sub_assign() {
    assert_eq!(run("let n = 5; n -= 3; return n;"), Value::Number(2.0));
}

#[test]
fn t03_mul_assign() {
    assert_eq!(run("let n = 5; n *= 4; return n;"), Value::Number(20.0));
}

#[test]
fn t04_div_assign() {
    assert_eq!(run("let n = 20; n /= 4; return n;"), Value::Number(5.0));
}

#[test]
fn t05_mod_assign() {
    assert_eq!(run("let n = 17; n %= 5; return n;"), Value::Number(2.0));
}

#[test]
fn t06_pow_assign() {
    assert_eq!(run("let n = 2; n **= 8; return n;"), Value::Number(256.0));
}

// ───────── Bitwise / shift compound ─────────

#[test]
fn t07_bitor_assign() {
    assert_eq!(run("let n = 12; n |= 3; return n;"), Value::Number(15.0));
}

#[test]
fn t08_bitand_assign() {
    assert_eq!(run("let n = 14; n &= 11; return n;"), Value::Number(10.0));
}

#[test]
fn t09_bitxor_assign() {
    assert_eq!(run("let n = 10; n ^= 15; return n;"), Value::Number(5.0));
}

#[test]
fn t10_shl_assign() {
    assert_eq!(run("let n = 1; n <<= 3; return n;"), Value::Number(8.0));
}

#[test]
fn t11_shr_assign() {
    assert_eq!(run("let n = 8; n >>= 2; return n;"), Value::Number(2.0));
}

// ───────── Logical / nullish compound (short-circuit) ─────────

#[test]
fn t12a_or_assign_falsy_assigns() {
    assert_eq!(run("let n = 0; n ||= 7; return n;"), Value::Number(7.0));
}

#[test]
fn t12b_or_assign_truthy_keeps() {
    assert_eq!(run("let n = 9; n ||= 7; return n;"), Value::Number(9.0));
}

#[test]
fn t13a_and_assign_truthy_assigns() {
    assert_eq!(run("let n = 1; n &&= 7; return n;"), Value::Number(7.0));
}

#[test]
fn t13b_and_assign_falsy_keeps() {
    assert_eq!(run("let n = 0; n &&= 7; return n;"), Value::Number(0.0));
}

#[test]
fn t14a_nullish_assign_null_assigns() {
    assert_eq!(run("let n = null; n ??= 7; return n;"), Value::Number(7.0));
}

#[test]
fn t14b_nullish_assign_zero_keeps() {
    assert_eq!(run("let n = 0; n ??= 7; return n;"), Value::Number(0.0));
}

// ───────── Member targets ─────────

#[test]
fn t15_static_member_add_assign() {
    assert_eq!(run("let o = {n: 5}; o.n += 3; return o.n;"), Value::Number(8.0));
}

#[test]
fn t16_index_mul_assign() {
    assert_eq!(run("let a = [10, 20, 30]; a[1] *= 2; return a[1];"), Value::Number(40.0));
}

#[test]
fn t17_computed_member_sub_assign() {
    assert_eq!(run("let o = {n: 5}; let k = \"n\"; o[k] -= 2; return o.n;"), Value::Number(3.0));
}

// ───────── Update expressions ─────────

#[test]
fn t18_prefix_inc_identifier_value_and_effect() {
    // Expression evaluates to new value; n becomes 6.
    assert_eq!(run("let n = 5; let r = ++n; return [r, n][0];"), Value::Number(6.0));
    assert_eq!(run("let n = 5; ++n; return n;"), Value::Number(6.0));
}

#[test]
fn t19_postfix_inc_identifier_value_and_effect() {
    // Expression evaluates to old value; n becomes 6.
    assert_eq!(run("let n = 5; let r = n++; return r;"), Value::Number(5.0));
    assert_eq!(run("let n = 5; n++; return n;"), Value::Number(6.0));
}

#[test]
fn t20_prefix_dec() {
    assert_eq!(run("let n = 5; --n; return n;"), Value::Number(4.0));
    assert_eq!(run("let n = 5; let r = --n; return r;"), Value::Number(4.0));
}

#[test]
fn t21_postfix_dec_value_and_effect() {
    assert_eq!(run("let n = 5; let r = n--; return r;"), Value::Number(5.0));
    assert_eq!(run("let n = 5; n--; return n;"), Value::Number(4.0));
}

#[test]
fn t22_prefix_inc_static_member() {
    assert_eq!(run("let o = {n: 5}; ++o.n; return o.n;"), Value::Number(6.0));
    assert_eq!(run("let o = {n: 5}; let r = ++o.n; return r;"), Value::Number(6.0));
}

#[test]
fn t23_postfix_inc_index() {
    assert_eq!(run("let a = [10, 20]; a[0]++; return a[0];"), Value::Number(11.0));
    assert_eq!(run("let a = [10, 20]; let r = a[0]++; return r;"), Value::Number(10.0));
}

// ───────── For-of with compound (canonical smoke) ─────────

#[test]
fn t24_for_of_compound_accumulator() {
    let src = r#"
        let n = 0;
        for (const x of [10, 20, 30]) n += x;
        return n;
    "#;
    assert_eq!(run(src), Value::Number(60.0));
}
