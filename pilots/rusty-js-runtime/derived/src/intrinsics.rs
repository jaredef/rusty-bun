//! Built-in intrinsics — minimal v1 surface for the parity-119 corpus.
//! Per specs/rusty-js-runtime-design.md §V.
//!
//! Round 3.d.e scope:
//! - Global functions: parseInt, parseFloat, isNaN, isFinite
//! - Math intrinsic: abs, floor, ceil, round, trunc, sqrt, pow, max, min,
//!   sign, exp, log, sin, cos, tan, random, PI, E, LN2, LN10
//! - JSON intrinsic: stringify (limited), parse (limited)
//! - Number static: parseInt, parseFloat, isNaN, isFinite, isInteger,
//!   isSafeInteger, MAX_SAFE_INTEGER, MAX_VALUE, etc.
//! - Console.log

use crate::abstract_ops;
use crate::interp::{Runtime, RuntimeError};
use crate::value::{FunctionInternals, InternalKind, NativeFn, Object, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

impl Runtime {
    pub fn install_intrinsics(&mut self) {
        self.install_globals();
        self.install_math();
        self.install_json();
        self.install_console();
        self.install_promise();
        self.install_test_record();
    }

    fn install_globals(&mut self) {
        register_global_fn(self, "parseInt", |_rt, args|{
            let s = if args.is_empty() { return Ok(Value::Number(f64::NAN)); } else { abstract_ops::to_string(&args[0]) };
            let radix = args.get(1).map(|v| abstract_ops::to_number(v) as i32).unwrap_or(10);
            let radix = if radix == 0 { 10 } else { radix };
            let trimmed = s.trim_start();
            let (sign, body) = if let Some(rest) = trimmed.strip_prefix('-') { (-1.0, rest) }
                else if let Some(rest) = trimmed.strip_prefix('+') { (1.0, rest) }
                else { (1.0, trimmed) };
            let mut acc: u64 = 0;
            let mut any = false;
            for c in body.chars() {
                let d = match c {
                    '0'..='9' => c as u32 - '0' as u32,
                    'a'..='z' => c as u32 - 'a' as u32 + 10,
                    'A'..='Z' => c as u32 - 'A' as u32 + 10,
                    _ => break,
                };
                if (d as i32) >= radix { break; }
                acc = acc.saturating_mul(radix as u64).saturating_add(d as u64);
                any = true;
            }
            if !any { return Ok(Value::Number(f64::NAN)); }
            Ok(Value::Number(sign * acc as f64))
        });
        register_global_fn(self, "parseFloat", |_rt, args|{
            if args.is_empty() { return Ok(Value::Number(f64::NAN)); }
            let s = abstract_ops::to_string(&args[0]);
            let trimmed = s.trim_start();
            // Find longest numeric prefix
            let mut end = 0;
            let mut saw_digit = false;
            let mut saw_dot = false;
            let mut saw_e = false;
            for (i, c) in trimmed.char_indices() {
                if i == 0 && (c == '+' || c == '-') { end = i + 1; continue; }
                match c {
                    '0'..='9' => { saw_digit = true; end = i + 1; }
                    '.' if !saw_dot && !saw_e => { saw_dot = true; end = i + 1; }
                    'e' | 'E' if saw_digit && !saw_e => { saw_e = true; end = i + 1; }
                    '+' | '-' if saw_e && trimmed[..i].chars().last() == Some('e' as char) => { end = i + 1; }
                    _ => break,
                }
            }
            if end == 0 { return Ok(Value::Number(f64::NAN)); }
            Ok(Value::Number(trimmed[..end].parse().unwrap_or(f64::NAN)))
        });
        register_global_fn(self, "isNaN", |_rt, args|{
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            let n = abstract_ops::to_number(&v);
            Ok(Value::Boolean(n.is_nan()))
        });
        register_global_fn(self, "isFinite", |_rt, args|{
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            let n = abstract_ops::to_number(&v);
            Ok(Value::Boolean(n.is_finite()))
        });
    }

    fn install_math(&mut self) {
        let math = new_object();
        register_method(&math, "abs", |_rt, args|Ok(Value::Number(num_arg(args, 0).abs())));
        register_method(&math, "floor", |_rt, args|Ok(Value::Number(num_arg(args, 0).floor())));
        register_method(&math, "ceil", |_rt, args|Ok(Value::Number(num_arg(args, 0).ceil())));
        register_method(&math, "round", |_rt, args|{
            // JS Math.round rounds half-to-positive-infinity, not Rust's
            // half-to-even. Reimplement.
            let x = num_arg(args, 0);
            Ok(Value::Number((x + 0.5).floor()))
        });
        register_method(&math, "trunc", |_rt, args|Ok(Value::Number(num_arg(args, 0).trunc())));
        register_method(&math, "sqrt", |_rt, args|Ok(Value::Number(num_arg(args, 0).sqrt())));
        register_method(&math, "cbrt", |_rt, args|Ok(Value::Number(num_arg(args, 0).cbrt())));
        register_method(&math, "pow", |_rt, args|{
            Ok(Value::Number(num_arg(args, 0).powf(num_arg(args, 1))))
        });
        register_method(&math, "max", |_rt, args|{
            let mut m = f64::NEG_INFINITY;
            for a in args {
                let n = abstract_ops::to_number(a);
                if n.is_nan() { return Ok(Value::Number(f64::NAN)); }
                if n > m { m = n; }
            }
            Ok(Value::Number(m))
        });
        register_method(&math, "min", |_rt, args|{
            let mut m = f64::INFINITY;
            for a in args {
                let n = abstract_ops::to_number(a);
                if n.is_nan() { return Ok(Value::Number(f64::NAN)); }
                if n < m { m = n; }
            }
            Ok(Value::Number(m))
        });
        register_method(&math, "sign", |_rt, args|{
            let x = num_arg(args, 0);
            Ok(Value::Number(if x.is_nan() { f64::NAN } else if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { x }))
        });
        register_method(&math, "exp", |_rt, args|Ok(Value::Number(num_arg(args, 0).exp())));
        register_method(&math, "log", |_rt, args|Ok(Value::Number(num_arg(args, 0).ln())));
        register_method(&math, "log2", |_rt, args|Ok(Value::Number(num_arg(args, 0).log2())));
        register_method(&math, "log10", |_rt, args|Ok(Value::Number(num_arg(args, 0).log10())));
        register_method(&math, "sin", |_rt, args|Ok(Value::Number(num_arg(args, 0).sin())));
        register_method(&math, "cos", |_rt, args|Ok(Value::Number(num_arg(args, 0).cos())));
        register_method(&math, "tan", |_rt, args|Ok(Value::Number(num_arg(args, 0).tan())));
        register_method(&math, "atan", |_rt, args|Ok(Value::Number(num_arg(args, 0).atan())));
        register_method(&math, "atan2", |_rt, args|Ok(Value::Number(num_arg(args, 0).atan2(num_arg(args, 1))))) ;
        register_method(&math, "random", |_rt, _|{
            // v1: simple LCG-style PRNG seeded from time. Not crypto-grade.
            use std::time::{SystemTime, UNIX_EPOCH};
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
            let pseudo = ((nanos as u64).wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)) as f64;
            Ok(Value::Number((pseudo / u64::MAX as f64).abs().fract()))
        });
        // Constants
        math.borrow_mut().set_own("PI".into(), Value::Number(std::f64::consts::PI));
        math.borrow_mut().set_own("E".into(), Value::Number(std::f64::consts::E));
        math.borrow_mut().set_own("LN2".into(), Value::Number(std::f64::consts::LN_2));
        math.borrow_mut().set_own("LN10".into(), Value::Number(std::f64::consts::LN_10));
        math.borrow_mut().set_own("LOG2E".into(), Value::Number(std::f64::consts::LOG2_E));
        math.borrow_mut().set_own("LOG10E".into(), Value::Number(std::f64::consts::LOG10_E));
        math.borrow_mut().set_own("SQRT2".into(), Value::Number(std::f64::consts::SQRT_2));

        self.globals.insert("Math".into(), Value::Object(math));
    }

    fn install_json(&mut self) {
        let json = new_object();
        register_method(&json, "stringify", |_rt, args|{
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            Ok(Value::String(Rc::new(json_stringify(&v))))
        });
        register_method(&json, "parse", |_rt, args|{
            let s = if let Some(v) = args.first() { abstract_ops::to_string(v) } else {
                return Err(RuntimeError::TypeError("JSON.parse requires a string".into()));
            };
            json_parse(s.as_str())
        });
        self.globals.insert("JSON".into(), Value::Object(json));
    }

    fn install_test_record(&mut self) {
        // __record(value) - testing-only intrinsic that stores its
        // argument into runtime.globals["__last_recorded"]. Used by the
        // test harness to verify side effects from microtask reactions.
        register_global_fn(self, "__record", |rt, args| {
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            rt.globals.insert("__last_recorded".into(), v);
            Ok(Value::Undefined)
        });
    }

    fn install_console(&mut self) {
        let console = new_object();
        register_method(&console, "log", |_rt, args|{
            let mut out = String::new();
            for (i, a) in args.iter().enumerate() {
                if i > 0 { out.push(' '); }
                out.push_str(&abstract_ops::to_string(a));
            }
            println!("{}", out);
            Ok(Value::Undefined)
        });
        register_method(&console, "error", |_rt, args|{
            let mut out = String::new();
            for (i, a) in args.iter().enumerate() {
                if i > 0 { out.push(' '); }
                out.push_str(&abstract_ops::to_string(a));
            }
            eprintln!("{}", out);
            Ok(Value::Undefined)
        });
        register_method(&console, "warn", |_rt, args|{
            let mut out = String::new();
            for (i, a) in args.iter().enumerate() {
                if i > 0 { out.push(' '); }
                out.push_str(&abstract_ops::to_string(a));
            }
            eprintln!("{}", out);
            Ok(Value::Undefined)
        });
        self.globals.insert("console".into(), Value::Object(console));
    }
}

fn new_object() -> Rc<RefCell<Object>> {
    Rc::new(RefCell::new(Object::new_ordinary()))
}

fn num_arg(args: &[Value], i: usize) -> f64 {
    args.get(i).map(abstract_ops::to_number).unwrap_or(f64::NAN)
}

fn register_method<F>(host: &Rc<RefCell<Object>>, name: &str, f: F)
where F: Fn(&mut Runtime, &[Value]) -> Result<Value, RuntimeError> + 'static {
    let native: NativeFn = Rc::new(f);
    let fn_obj = Object {
        proto: None,
        extensible: true,
        properties: HashMap::new(),
        internal_kind: InternalKind::Function(FunctionInternals {
            name: name.to_string(),
            native,
        }),
    };
    host.borrow_mut().set_own(name.into(), Value::Object(Rc::new(RefCell::new(fn_obj))));
}

fn register_global_fn<F>(rt: &mut Runtime, name: &str, f: F)
where F: Fn(&mut Runtime, &[Value]) -> Result<Value, RuntimeError> + 'static {
    let native: NativeFn = Rc::new(f);
    let fn_obj = Object {
        proto: None,
        extensible: true,
        properties: HashMap::new(),
        internal_kind: InternalKind::Function(FunctionInternals {
            name: name.to_string(),
            native,
        }),
    };
    rt.globals.insert(name.into(), Value::Object(Rc::new(RefCell::new(fn_obj))));
}

// ──────────────── JSON.stringify (limited) ────────────────

fn json_stringify(v: &Value) -> String {
    match v {
        Value::Undefined => "undefined".into(),  // technically JSON.stringify(undefined) -> undefined, but for v1 return string
        Value::Null => "null".into(),
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => {
            if n.is_finite() { abstract_ops::number_to_string(*n) } else { "null".into() }
        }
        Value::String(s) => json_quote_string(s.as_str()),
        Value::BigInt(_) => "null".into(),  // BigInt not valid JSON
        Value::Object(o) => {
            let obj = o.borrow();
            if matches!(obj.internal_kind, InternalKind::Array) {
                let mut entries: Vec<(usize, String)> = obj.properties.iter()
                    .filter_map(|(k, d)| k.parse::<usize>().ok().map(|i| (i, json_stringify(&d.value))))
                    .collect();
                entries.sort_by_key(|(i, _)| *i);
                let body: Vec<String> = entries.into_iter().map(|(_, s)| s).collect();
                format!("[{}]", body.join(","))
            } else {
                let entries: Vec<String> = obj.properties.iter()
                    .filter(|(_, d)| d.enumerable && !matches!(d.value, Value::Undefined))
                    .map(|(k, d)| format!("{}:{}", json_quote_string(k), json_stringify(&d.value)))
                    .collect();
                format!("{{{}}}", entries.join(","))
            }
        }
    }
}

fn json_quote_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ──────────────── JSON.parse (limited recursive-descent) ────────────────

fn json_parse(s: &str) -> Result<Value, RuntimeError> {
    let bytes = s.as_bytes();
    let mut p = 0;
    skip_ws(bytes, &mut p);
    let v = json_parse_value(bytes, &mut p)?;
    skip_ws(bytes, &mut p);
    if p != bytes.len() {
        return Err(RuntimeError::TypeError("JSON.parse: trailing characters".into()));
    }
    Ok(v)
}

fn skip_ws(b: &[u8], p: &mut usize) {
    while *p < b.len() && matches!(b[*p], b' ' | b'\t' | b'\n' | b'\r') { *p += 1; }
}

fn json_parse_value(b: &[u8], p: &mut usize) -> Result<Value, RuntimeError> {
    skip_ws(b, p);
    if *p >= b.len() { return Err(RuntimeError::TypeError("JSON.parse: unexpected end".into())); }
    match b[*p] {
        b'{' => json_parse_object(b, p),
        b'[' => json_parse_array(b, p),
        b'"' => json_parse_string(b, p).map(|s| Value::String(Rc::new(s))),
        b't' if b[*p..].starts_with(b"true") => { *p += 4; Ok(Value::Boolean(true)) }
        b'f' if b[*p..].starts_with(b"false") => { *p += 5; Ok(Value::Boolean(false)) }
        b'n' if b[*p..].starts_with(b"null") => { *p += 4; Ok(Value::Null) }
        b'-' | b'0'..=b'9' => json_parse_number(b, p),
        _ => Err(RuntimeError::TypeError(format!("JSON.parse: unexpected character at offset {}", p))),
    }
}

fn json_parse_object(b: &[u8], p: &mut usize) -> Result<Value, RuntimeError> {
    *p += 1; // consume '{'
    let obj = new_object();
    skip_ws(b, p);
    if *p < b.len() && b[*p] == b'}' { *p += 1; return Ok(Value::Object(obj)); }
    loop {
        skip_ws(b, p);
        let key = json_parse_string(b, p)?;
        skip_ws(b, p);
        if *p >= b.len() || b[*p] != b':' { return Err(RuntimeError::TypeError("JSON.parse: expected ':'".into())); }
        *p += 1;
        let value = json_parse_value(b, p)?;
        obj.borrow_mut().set_own(key, value);
        skip_ws(b, p);
        match b.get(*p) {
            Some(&b',') => { *p += 1; continue; }
            Some(&b'}') => { *p += 1; return Ok(Value::Object(obj)); }
            _ => return Err(RuntimeError::TypeError("JSON.parse: expected ',' or '}'".into())),
        }
    }
}

fn json_parse_array(b: &[u8], p: &mut usize) -> Result<Value, RuntimeError> {
    *p += 1; // consume '['
    let arr = Rc::new(RefCell::new(Object::new_array()));
    skip_ws(b, p);
    if *p < b.len() && b[*p] == b']' { *p += 1; return Ok(Value::Object(arr)); }
    let mut i = 0u32;
    loop {
        let value = json_parse_value(b, p)?;
        arr.borrow_mut().set_own(i.to_string(), value);
        i += 1;
        skip_ws(b, p);
        match b.get(*p) {
            Some(&b',') => { *p += 1; continue; }
            Some(&b']') => { *p += 1; return Ok(Value::Object(arr)); }
            _ => return Err(RuntimeError::TypeError("JSON.parse: expected ',' or ']'".into())),
        }
    }
}

fn json_parse_string(b: &[u8], p: &mut usize) -> Result<String, RuntimeError> {
    if *p >= b.len() || b[*p] != b'"' {
        return Err(RuntimeError::TypeError("JSON.parse: expected string".into()));
    }
    *p += 1;
    let mut out = String::new();
    while *p < b.len() {
        let c = b[*p];
        if c == b'"' { *p += 1; return Ok(out); }
        if c == b'\\' {
            *p += 1;
            if *p >= b.len() { return Err(RuntimeError::TypeError("JSON.parse: dangling \\".into())); }
            match b[*p] {
                b'"' => out.push('"'),
                b'\\' => out.push('\\'),
                b'/' => out.push('/'),
                b'n' => out.push('\n'),
                b'r' => out.push('\r'),
                b't' => out.push('\t'),
                b'b' => out.push('\u{0008}'),
                b'f' => out.push('\u{000C}'),
                b'u' if *p + 4 < b.len() => {
                    let hex = std::str::from_utf8(&b[*p+1..*p+5]).map_err(|_|RuntimeError::TypeError("JSON.parse: bad \\u".into()))?;
                    let cp = u32::from_str_radix(hex, 16).map_err(|_|RuntimeError::TypeError("JSON.parse: bad \\u".into()))?;
                    if let Some(ch) = char::from_u32(cp) { out.push(ch); }
                    *p += 4;
                }
                _ => return Err(RuntimeError::TypeError("JSON.parse: bad escape".into())),
            }
            *p += 1;
        } else {
            out.push(c as char);
            *p += 1;
        }
    }
    Err(RuntimeError::TypeError("JSON.parse: unterminated string".into()))
}

fn json_parse_number(b: &[u8], p: &mut usize) -> Result<Value, RuntimeError> {
    let start = *p;
    if b[*p] == b'-' { *p += 1; }
    while *p < b.len() && b[*p].is_ascii_digit() { *p += 1; }
    if *p < b.len() && b[*p] == b'.' {
        *p += 1;
        while *p < b.len() && b[*p].is_ascii_digit() { *p += 1; }
    }
    if *p < b.len() && (b[*p] == b'e' || b[*p] == b'E') {
        *p += 1;
        if *p < b.len() && (b[*p] == b'+' || b[*p] == b'-') { *p += 1; }
        while *p < b.len() && b[*p].is_ascii_digit() { *p += 1; }
    }
    let s = std::str::from_utf8(&b[start..*p]).map_err(|_|RuntimeError::TypeError("JSON.parse: bad number".into()))?;
    let n = s.parse::<f64>().map_err(|_|RuntimeError::TypeError("JSON.parse: bad number".into()))?;
    Ok(Value::Number(n))
}
