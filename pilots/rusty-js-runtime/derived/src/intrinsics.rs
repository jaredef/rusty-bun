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
use crate::value::{FunctionInternals, InternalKind, NativeFn, Object, ObjectRef, PropertyDescriptor, Value};
use std::collections::HashMap;
use std::rc::Rc;

impl Runtime {
    pub fn install_intrinsics(&mut self) {
        // Prototype intrinsics must install first so subsequent alloc_object
        // calls (Math/JSON/console hosts, Promise) inherit from
        // Object.prototype. Tier-Ω.5.a.
        self.install_prototypes();
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
        let math = self.alloc_object(Object::new_ordinary());
        register_method(self, math, "abs", |_rt, args|Ok(Value::Number(num_arg(args, 0).abs())));
        register_method(self, math, "floor", |_rt, args|Ok(Value::Number(num_arg(args, 0).floor())));
        register_method(self, math, "ceil", |_rt, args|Ok(Value::Number(num_arg(args, 0).ceil())));
        register_method(self, math, "round", |_rt, args|{
            // JS Math.round rounds half-to-positive-infinity, not Rust's
            // half-to-even. Reimplement.
            let x = num_arg(args, 0);
            Ok(Value::Number((x + 0.5).floor()))
        });
        register_method(self, math,"trunc", |_rt, args|Ok(Value::Number(num_arg(args, 0).trunc())));
        register_method(self, math,"sqrt", |_rt, args|Ok(Value::Number(num_arg(args, 0).sqrt())));
        register_method(self, math,"cbrt", |_rt, args|Ok(Value::Number(num_arg(args, 0).cbrt())));
        register_method(self, math,"pow", |_rt, args|{
            Ok(Value::Number(num_arg(args, 0).powf(num_arg(args, 1))))
        });
        register_method(self, math,"max", |_rt, args|{
            let mut m = f64::NEG_INFINITY;
            for a in args {
                let n = abstract_ops::to_number(a);
                if n.is_nan() { return Ok(Value::Number(f64::NAN)); }
                if n > m { m = n; }
            }
            Ok(Value::Number(m))
        });
        register_method(self, math,"min", |_rt, args|{
            let mut m = f64::INFINITY;
            for a in args {
                let n = abstract_ops::to_number(a);
                if n.is_nan() { return Ok(Value::Number(f64::NAN)); }
                if n < m { m = n; }
            }
            Ok(Value::Number(m))
        });
        register_method(self, math,"sign", |_rt, args|{
            let x = num_arg(args, 0);
            Ok(Value::Number(if x.is_nan() { f64::NAN } else if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { x }))
        });
        register_method(self, math,"exp", |_rt, args|Ok(Value::Number(num_arg(args, 0).exp())));
        register_method(self, math,"log", |_rt, args|Ok(Value::Number(num_arg(args, 0).ln())));
        register_method(self, math,"log2", |_rt, args|Ok(Value::Number(num_arg(args, 0).log2())));
        register_method(self, math,"log10", |_rt, args|Ok(Value::Number(num_arg(args, 0).log10())));
        register_method(self, math,"sin", |_rt, args|Ok(Value::Number(num_arg(args, 0).sin())));
        register_method(self, math,"cos", |_rt, args|Ok(Value::Number(num_arg(args, 0).cos())));
        register_method(self, math,"tan", |_rt, args|Ok(Value::Number(num_arg(args, 0).tan())));
        register_method(self, math,"atan", |_rt, args|Ok(Value::Number(num_arg(args, 0).atan())));
        register_method(self, math,"atan2", |_rt, args|Ok(Value::Number(num_arg(args, 0).atan2(num_arg(args, 1))))) ;
        register_method(self, math,"random", |_rt, _|{
            // v1: simple LCG-style PRNG seeded from time. Not crypto-grade.
            use std::time::{SystemTime, UNIX_EPOCH};
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
            let pseudo = ((nanos as u64).wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)) as f64;
            Ok(Value::Number((pseudo / u64::MAX as f64).abs().fract()))
        });
        // Constants
        self.object_set(math, "PI".into(), Value::Number(std::f64::consts::PI));
        self.object_set(math, "E".into(), Value::Number(std::f64::consts::E));
        self.object_set(math, "LN2".into(), Value::Number(std::f64::consts::LN_2));
        self.object_set(math, "LN10".into(), Value::Number(std::f64::consts::LN_10));
        self.object_set(math, "LOG2E".into(), Value::Number(std::f64::consts::LOG2_E));
        self.object_set(math, "LOG10E".into(), Value::Number(std::f64::consts::LOG10_E));
        self.object_set(math, "SQRT2".into(), Value::Number(std::f64::consts::SQRT_2));

        self.globals.insert("Math".into(), Value::Object(math));
    }

    fn install_json(&mut self) {
        let json = self.alloc_object(Object::new_ordinary());
        register_method(self, json, "stringify", |rt, args|{
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            Ok(Value::String(Rc::new(json_stringify(rt, &v))))
        });
        register_method(self, json, "parse", |rt, args|{
            let s = if let Some(v) = args.first() { abstract_ops::to_string(v) } else {
                return Err(RuntimeError::TypeError("JSON.parse requires a string".into()));
            };
            json_parse(rt, s.as_str())
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
        let console = self.alloc_object(Object::new_ordinary());
        register_method(self, console, "log", |_rt, args|{
            let mut out = String::new();
            for (i, a) in args.iter().enumerate() {
                if i > 0 { out.push(' '); }
                out.push_str(&abstract_ops::to_string(a));
            }
            println!("{}", out);
            Ok(Value::Undefined)
        });
        register_method(self, console,"error", |_rt, args|{
            let mut out = String::new();
            for (i, a) in args.iter().enumerate() {
                if i > 0 { out.push(' '); }
                out.push_str(&abstract_ops::to_string(a));
            }
            eprintln!("{}", out);
            Ok(Value::Undefined)
        });
        register_method(self, console,"warn", |_rt, args|{
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

fn num_arg(args: &[Value], i: usize) -> f64 {
    args.get(i).map(abstract_ops::to_number).unwrap_or(f64::NAN)
}

pub(crate) fn make_native(name: &str, f: impl Fn(&mut Runtime, &[Value]) -> Result<Value, RuntimeError> + 'static) -> Object {
    let native: NativeFn = Rc::new(f);
    Object {
        proto: None,
        extensible: true,
        properties: HashMap::new(),
        internal_kind: InternalKind::Function(FunctionInternals {
            name: name.to_string(),
            native,
        }),
    }
}

fn register_method<F>(rt: &mut Runtime, host: ObjectRef, name: &str, f: F)
where F: Fn(&mut Runtime, &[Value]) -> Result<Value, RuntimeError> + 'static {
    let fn_obj = make_native(name, f);
    let fn_id = rt.alloc_object(fn_obj);
    rt.object_set(host, name.into(), Value::Object(fn_id));
}

fn register_global_fn<F>(rt: &mut Runtime, name: &str, f: F)
where F: Fn(&mut Runtime, &[Value]) -> Result<Value, RuntimeError> + 'static {
    let fn_obj = make_native(name, f);
    let fn_id = rt.alloc_object(fn_obj);
    rt.globals.insert(name.into(), Value::Object(fn_id));
}

// ──────────────── JSON.stringify (limited) ────────────────

fn json_stringify(rt: &Runtime, v: &Value) -> String {
    match v {
        Value::Undefined => "undefined".into(),
        Value::Null => "null".into(),
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => {
            if n.is_finite() { abstract_ops::number_to_string(*n) } else { "null".into() }
        }
        Value::String(s) => json_quote_string(s.as_str()),
        Value::BigInt(_) => "null".into(),
        Value::Object(id) => {
            // Snapshot the props (clones Value) to avoid recursive borrow.
            let (is_array, props): (bool, Vec<(String, PropertyDescriptor)>) = {
                let obj = rt.obj(*id);
                let is_array = matches!(obj.internal_kind, InternalKind::Array);
                let v: Vec<_> = obj.properties.iter()
                    .map(|(k, d)| (k.clone(), d.clone())).collect();
                (is_array, v)
            };
            if is_array {
                let mut entries: Vec<(usize, String)> = props.iter()
                    .filter_map(|(k, d)| k.parse::<usize>().ok().map(|i| (i, json_stringify(rt, &d.value))))
                    .collect();
                entries.sort_by_key(|(i, _)| *i);
                let body: Vec<String> = entries.into_iter().map(|(_, s)| s).collect();
                format!("[{}]", body.join(","))
            } else {
                let entries: Vec<String> = props.iter()
                    .filter(|(_, d)| d.enumerable && !matches!(d.value, Value::Undefined))
                    .map(|(k, d)| format!("{}:{}", json_quote_string(k), json_stringify(rt, &d.value)))
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

fn json_parse(rt: &mut Runtime, s: &str) -> Result<Value, RuntimeError> {
    let bytes = s.as_bytes();
    let mut p = 0;
    skip_ws(bytes, &mut p);
    let v = json_parse_value(rt, bytes, &mut p)?;
    skip_ws(bytes, &mut p);
    if p != bytes.len() {
        return Err(RuntimeError::TypeError("JSON.parse: trailing characters".into()));
    }
    Ok(v)
}

fn skip_ws(b: &[u8], p: &mut usize) {
    while *p < b.len() && matches!(b[*p], b' ' | b'\t' | b'\n' | b'\r') { *p += 1; }
}

fn json_parse_value(rt: &mut Runtime, b: &[u8], p: &mut usize) -> Result<Value, RuntimeError> {
    skip_ws(b, p);
    if *p >= b.len() { return Err(RuntimeError::TypeError("JSON.parse: unexpected end".into())); }
    match b[*p] {
        b'{' => json_parse_object(rt, b, p),
        b'[' => json_parse_array(rt, b, p),
        b'"' => json_parse_string(b, p).map(|s| Value::String(Rc::new(s))),
        b't' if b[*p..].starts_with(b"true") => { *p += 4; Ok(Value::Boolean(true)) }
        b'f' if b[*p..].starts_with(b"false") => { *p += 5; Ok(Value::Boolean(false)) }
        b'n' if b[*p..].starts_with(b"null") => { *p += 4; Ok(Value::Null) }
        b'-' | b'0'..=b'9' => json_parse_number(b, p),
        _ => Err(RuntimeError::TypeError(format!("JSON.parse: unexpected character at offset {}", p))),
    }
}

fn json_parse_object(rt: &mut Runtime, b: &[u8], p: &mut usize) -> Result<Value, RuntimeError> {
    *p += 1; // consume '{'
    let obj = rt.alloc_object(Object::new_ordinary());
    skip_ws(b, p);
    if *p < b.len() && b[*p] == b'}' { *p += 1; return Ok(Value::Object(obj)); }
    loop {
        skip_ws(b, p);
        let key = json_parse_string(b, p)?;
        skip_ws(b, p);
        if *p >= b.len() || b[*p] != b':' { return Err(RuntimeError::TypeError("JSON.parse: expected ':'".into())); }
        *p += 1;
        let value = json_parse_value(rt, b, p)?;
        rt.object_set(obj, key, value);
        skip_ws(b, p);
        match b.get(*p) {
            Some(&b',') => { *p += 1; continue; }
            Some(&b'}') => { *p += 1; return Ok(Value::Object(obj)); }
            _ => return Err(RuntimeError::TypeError("JSON.parse: expected ',' or '}'".into())),
        }
    }
}

fn json_parse_array(rt: &mut Runtime, b: &[u8], p: &mut usize) -> Result<Value, RuntimeError> {
    *p += 1; // consume '['
    let arr = rt.alloc_object(Object::new_array());
    skip_ws(b, p);
    if *p < b.len() && b[*p] == b']' { *p += 1; return Ok(Value::Object(arr)); }
    let mut i = 0u32;
    loop {
        let value = json_parse_value(rt, b, p)?;
        rt.object_set(arr, i.to_string(), value);
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
