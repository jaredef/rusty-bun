//! Spec abstract operations per ECMA-262 §7. Each operation is named
//! verbatim from the spec where reasonable. v1 implements the subset
//! exercised by the round-3.d.b opcode handlers.

use crate::value::{Value};
use std::rc::Rc;

/// ToBoolean per §7.1.2.
pub fn to_boolean(v: &Value) -> bool {
    match v {
        Value::Undefined | Value::Null => false,
        Value::Boolean(b) => *b,
        Value::Number(n) => !(n.is_nan() || *n == 0.0),
        Value::String(s) => !s.is_empty(),
        Value::BigInt(s) => s.as_str() != "0",
        Value::Object(_) => true,
    }
}

/// ToNumber per §7.1.4. v1 supports the primitive cases; Object → primitive
/// → number coercion lands when intrinsics + Symbol.toPrimitive arrive.
pub fn to_number(v: &Value) -> f64 {
    match v {
        Value::Undefined => f64::NAN,
        Value::Null => 0.0,
        Value::Boolean(true) => 1.0,
        Value::Boolean(false) => 0.0,
        Value::Number(n) => *n,
        Value::String(s) => parse_string_to_number(s.as_str()),
        Value::BigInt(_) => f64::NAN,  // BigInt -> Number is a TypeError per spec; v1 returns NaN as placeholder
        Value::Object(_) => f64::NAN,  // Object -> primitive deferred
    }
}

fn parse_string_to_number(s: &str) -> f64 {
    let trimmed = s.trim();
    if trimmed.is_empty() { return 0.0; }
    if let Some(rest) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
        return u64::from_str_radix(rest, 16).map(|n| n as f64).unwrap_or(f64::NAN);
    }
    trimmed.parse::<f64>().unwrap_or(f64::NAN)
}

/// ToString per §7.1.17. v1 supports primitives.
pub fn to_string(v: &Value) -> Rc<String> {
    Rc::new(match v {
        Value::Undefined => "undefined".to_string(),
        Value::Null => "null".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => number_to_string(*n),
        Value::String(s) => return s.clone(),
        Value::BigInt(s) => return s.clone(),
        Value::Object(_) => "[object Object]".to_string(),  // Object ToString deferred
    })
}

/// Number::toString per §6.1.6.1.20. v1 uses Rust's default f64 formatter
/// with special-cases for integer numbers + NaN + Infinity per spec.
pub fn number_to_string(n: f64) -> String {
    if n.is_nan() { return "NaN".to_string(); }
    if n == f64::INFINITY { return "Infinity".to_string(); }
    if n == f64::NEG_INFINITY { return "-Infinity".to_string(); }
    if n == 0.0 { return "0".to_string(); }
    if n.fract() == 0.0 && n.abs() < 1e21 {
        return format!("{}", n as i64);
    }
    format!("{}", n)
}

/// Strict equality per §7.2.15.
pub fn is_strictly_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Undefined, Value::Undefined) => true,
        (Value::Null, Value::Null) => true,
        (Value::Boolean(x), Value::Boolean(y)) => x == y,
        (Value::Number(x), Value::Number(y)) => {
            // NaN !== NaN per IEEE 754 and spec
            if x.is_nan() || y.is_nan() { return false; }
            x == y
        }
        (Value::String(x), Value::String(y)) => x.as_str() == y.as_str(),
        (Value::BigInt(x), Value::BigInt(y)) => x == y,
        (Value::Object(x), Value::Object(y)) => Rc::ptr_eq(x, y),
        _ => false,
    }
}

/// Loose equality per §7.2.13. v1 handles the primitive cases; full
/// type-coercion table including Object-to-primitive lands later.
pub fn is_loosely_equal(a: &Value, b: &Value) -> bool {
    // Same-type fast path: defer to strict equality.
    if std::mem::discriminant(a) == std::mem::discriminant(b) {
        return is_strictly_equal(a, b);
    }
    match (a, b) {
        (Value::Null, Value::Undefined) | (Value::Undefined, Value::Null) => true,
        (Value::Number(x), Value::String(s)) | (Value::String(s), Value::Number(x)) => {
            let y = parse_string_to_number(s.as_str());
            !x.is_nan() && !y.is_nan() && *x == y
        }
        // Boolean -> Number, then re-compare loosely.
        (Value::Boolean(b), other) | (other, Value::Boolean(b)) => {
            let nb = if *b { 1.0 } else { 0.0 };
            is_loosely_equal(&Value::Number(nb), other)
        }
        _ => false,
    }
}

/// Abstract Relational Comparison per §7.2.14, returning Ordering.
/// Used by <, >, <=, >= opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelOrder { Less, Greater, Equal, Undefined }

pub fn abstract_relational_compare(x: &Value, y: &Value) -> RelOrder {
    // v1 simplified: ToPrimitive → if both String, lex compare; else ToNumber.
    if let (Value::String(a), Value::String(b)) = (x, y) {
        use std::cmp::Ordering::*;
        return match a.as_str().cmp(b.as_str()) {
            Less => RelOrder::Less,
            Greater => RelOrder::Greater,
            Equal => RelOrder::Equal,
        };
    }
    let nx = to_number(x);
    let ny = to_number(y);
    if nx.is_nan() || ny.is_nan() { return RelOrder::Undefined; }
    if nx < ny { RelOrder::Less }
    else if nx > ny { RelOrder::Greater }
    else { RelOrder::Equal }
}

/// Apply `+` semantics per §13.15. ToPrimitive-coerces operands; if either
/// is a String, concatenate; else arithmetic add.
pub fn op_add(x: &Value, y: &Value) -> Value {
    if matches!(x, Value::String(_)) || matches!(y, Value::String(_)) {
        let xs = to_string(x);
        let ys = to_string(y);
        let mut concat = String::with_capacity(xs.len() + ys.len());
        concat.push_str(&xs);
        concat.push_str(&ys);
        return Value::String(Rc::new(concat));
    }
    Value::Number(to_number(x) + to_number(y))
}
