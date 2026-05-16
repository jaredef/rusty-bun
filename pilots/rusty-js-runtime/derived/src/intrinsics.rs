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
        self.install_object_static();
        self.install_array_static();
        self.install_symbol_static();
        self.install_number_static();
        self.install_math();
        self.install_json();
        self.install_console();
        self.install_promise();
        self.install_regexp();
        self.install_test_record();
        self.install_destructure_helpers();
        self.install_spread_helpers();
        // Tier-Ω.5.ff: dynamic import stub.
        register_global_fn(self, "__dynamic_import", |_rt, _args| {
            Err(RuntimeError::Thrown(Value::String(Rc::new(
                "TypeError: dynamic import() not yet supported (Tier-Ω.5.ff stub)".into()
            ))))
        });
        self.install_global_this();
    }

    /// Tier-Ω.5.t: install `globalThis` as a synthetic object mirroring
    /// the current globals map. Self-references via `globalThis.globalThis`.
    /// Read-only snapshot at install time — subsequent writes to globals
    /// do NOT propagate. Acceptable v1 deviation: real spec has globalThis
    /// be the *actual* global object, but our globals are a HashMap, not
    /// an Object. Most consumer code reads from globalThis rather than
    /// writes; the snapshot is sufficient for shape probes.
    ///
    /// Hosts that add globals after install_intrinsics should call
    /// `install_global_this_refresh` once their wiring is complete so the
    /// snapshot picks up host-added bindings.
    pub fn install_global_this_refresh(&mut self) { self.install_global_this(); }

    fn install_global_this(&mut self) {
        let gt = self.alloc_object(Object::new_ordinary());
        let entries: Vec<(String, Value)> = self.globals.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (k, v) in entries {
            self.object_set(gt, k, v);
        }
        self.object_set(gt, "globalThis".into(), Value::Object(gt));
        // Tier-Ω.5.bbbb: `global` is a Node-side alias for globalThis;
        // many CJS packages do `global.foo = ...` or `global.process`.
        self.object_set(gt, "global".into(), Value::Object(gt));
        self.globals.insert("globalThis".into(), Value::Object(gt));
        self.globals.insert("global".into(), Value::Object(gt));
        // Tier-Ω.5.bbbb: Intl namespace with stub constructors. Real
        // locale-aware behavior is deferred; the stubs return objects
        // that survive shape probes and method existence checks. Lifts
        // packages that gate on `typeof Intl.X === 'function'`.
        let intl = self.alloc_object(Object::new_ordinary());
        for ctor_name in &["DateTimeFormat", "NumberFormat", "Collator", "PluralRules", "RelativeTimeFormat", "ListFormat", "Segmenter", "DisplayNames", "Locale"] {
            let name = (*ctor_name).to_string();
            let stub = make_native(&name, move |rt, _args| {
                let o = Object::new_ordinary();
                let id = rt.alloc_object(o);
                Ok(Value::Object(id))
            });
            let stub_id = self.alloc_object(stub);
            // Methods on instance proto for shape: format, formatToParts, resolvedOptions.
            register_method(self, stub_id, "supportedLocalesOf", |_rt, _args| {
                let o = Object::new_array();
                let id = _rt.alloc_object(o);
                _rt.object_set(id, "length".into(), Value::Number(0.0));
                Ok(Value::Object(id))
            });
            self.object_set(intl, ctor_name.to_string(), Value::Object(stub_id));
        }
        // getCanonicalLocales(locales) → array of canonical locale tags.
        register_method(self, intl, "getCanonicalLocales", |rt, _args| {
            let arr = Object::new_array();
            let id = rt.alloc_object(arr);
            rt.object_set(id, "length".into(), Value::Number(0.0));
            Ok(Value::Object(id))
        });
        self.globals.insert("Intl".into(), Value::Object(intl));
        // Tier-Ω.5.iiii: TextEncoder / TextDecoder per WHATWG Encoding
        // spec. v1 deviation: only UTF-8 supported; encode returns a
        // Uint8Array-shaped object (length + indexed bytes); decode
        // reads bytes back as JS string. Sufficient for jose / ky /
        // get-stream / many crypto + stream-using packages.
        let te = make_native("TextEncoder", |rt, _args| {
            let mut o = Object::new_ordinary();
            o.set_own("encoding".into(), Value::String(Rc::new("utf-8".to_string())));
            let id = rt.alloc_object(o);
            register_method(rt, id, "encode", |rt, args| {
                let s = match args.first() {
                    Some(Value::String(s)) => s.as_str().to_string(),
                    None => String::new(),
                    Some(v) => crate::abstract_ops::to_string(v).as_str().to_string(),
                };
                let bytes: Vec<u8> = s.into_bytes();
                let mut out = Object::new_array();
                out.set_own("length".into(), Value::Number(bytes.len() as f64));
                for (i, b) in bytes.iter().enumerate() {
                    out.set_own(i.to_string(), Value::Number(*b as f64));
                }
                Ok(Value::Object(rt.alloc_object(out)))
            });
            Ok(Value::Object(id))
        });
        let te_id = self.alloc_object(te);
        // Tier-Ω.5.qqqq: TextEncoder.prototype.encode for pako and any lib
        // that reaches the encode method via the prototype rather than via
        // an instance.
        let te_proto = self.alloc_object(Object::new_ordinary());
        register_method(self, te_proto, "encode", |rt, args| {
            let s = match args.first() {
                Some(Value::String(s)) => s.as_str().to_string(),
                None => String::new(),
                Some(v) => crate::abstract_ops::to_string(v).as_str().to_string(),
            };
            let bytes: Vec<u8> = s.into_bytes();
            let mut out = Object::new_array();
            out.set_own("length".into(), Value::Number(bytes.len() as f64));
            for (i, b) in bytes.iter().enumerate() {
                out.set_own(i.to_string(), Value::Number(*b as f64));
            }
            Ok(Value::Object(rt.alloc_object(out)))
        });
        self.object_set(te_id, "prototype".into(), Value::Object(te_proto));
        self.globals.insert("TextEncoder".into(), Value::Object(te_id));
        let td = make_native("TextDecoder", |rt, args| {
            let encoding = match args.first() {
                Some(Value::String(s)) => s.as_str().to_string(),
                _ => "utf-8".to_string(),
            };
            let mut o = Object::new_ordinary();
            o.set_own("encoding".into(), Value::String(Rc::new(encoding)));
            let id = rt.alloc_object(o);
            register_method(rt, id, "decode", |rt, args| {
                let bytes_id = match args.first() {
                    Some(Value::Object(id)) => *id,
                    _ => return Ok(Value::String(Rc::new(String::new()))),
                };
                let len = rt.array_length(bytes_id);
                let mut bytes: Vec<u8> = Vec::with_capacity(len);
                for i in 0..len {
                    if let Value::Number(n) = rt.object_get(bytes_id, &i.to_string()) {
                        bytes.push(n as u8);
                    }
                }
                let s = String::from_utf8_lossy(&bytes).to_string();
                Ok(Value::String(Rc::new(s)))
            });
            Ok(Value::Object(id))
        });
        let td_id = self.alloc_object(td);
        let td_proto = self.alloc_object(Object::new_ordinary());
        register_method(self, td_proto, "decode", |rt, args| {
            let bytes_id = match args.first() {
                Some(Value::Object(id)) => *id,
                _ => return Ok(Value::String(Rc::new(String::new()))),
            };
            let len = rt.array_length(bytes_id);
            let mut bytes: Vec<u8> = Vec::with_capacity(len);
            for i in 0..len {
                if let Value::Number(n) = rt.object_get(bytes_id, &i.to_string()) {
                    bytes.push(n as u8);
                }
            }
            let s = String::from_utf8_lossy(&bytes).to_string();
            Ok(Value::String(Rc::new(s)))
        });
        self.object_set(td_id, "prototype".into(), Value::Object(td_proto));
        self.globals.insert("TextDecoder".into(), Value::Object(td_id));
    }

    /// Tier-Ω.5.k: helpers the compiler emits LoadGlobal+Call into for
    /// object-literal spread and spread arguments. All return the target
    /// (array or object) so they compose without extra stack juggling.
    fn install_spread_helpers(&mut self) {
        // __object_spread(target, src) → target. Copies own enumerable
        // string-keyed properties from src to target, left-to-right.
        register_global_fn(self, "__object_spread", |rt, args| {
            let target = match args.first() {
                Some(Value::Object(id)) => *id,
                _ => return Err(RuntimeError::TypeError(
                    "__object_spread: target must be an object".into())),
            };
            if let Some(Value::Object(sid)) = args.get(1) {
                // Tier-Ω.5.bbbbb: dispatch accessor getters during spread.
                let entries: Vec<(String, Option<Value>)> = rt.obj(*sid).properties.iter()
                    .filter(|(_, d)| d.enumerable)
                    .map(|(k, d)| (k.clone(), d.getter.clone()))
                    .collect();
                for (k, getter_opt) in entries {
                    let v = if let Some(getter) = getter_opt {
                        rt.call_function(getter, Value::Object(*sid), Vec::new())?
                    } else {
                        rt.object_get(*sid, &k)
                    };
                    rt.object_set(target, k, v);
                }
            }
            // Non-object sources (null/undefined) are a no-op per ECMA-262.
            Ok(Value::Object(target))
        });
        // __array_push_single(arr, v) → arr. Appends one value.
        register_global_fn(self, "__array_push_single", |rt, args| {
            let arr = match args.first() {
                Some(Value::Object(id)) => *id,
                _ => return Err(RuntimeError::TypeError(
                    "__array_push_single: target must be an array".into())),
            };
            let v = args.get(1).cloned().unwrap_or(Value::Undefined);
            let len = rt.array_length(arr);
            rt.object_set(arr, len.to_string(), v);
            rt.object_set(arr, "length".into(), Value::Number((len + 1) as f64));
            Ok(Value::Object(arr))
        });
        // __array_extend(arr, iter) → arr. Iterates iter via @@iterator
        // protocol and appends each yielded value.
        register_global_fn(self, "__array_extend", |rt, args| {
            let arr = match args.first() {
                Some(Value::Object(id)) => *id,
                _ => return Err(RuntimeError::TypeError(
                    "__array_extend: target must be an array".into())),
            };
            let src = args.get(1).cloned().unwrap_or(Value::Undefined);
            let values = collect_iterable(rt, src)?;
            let mut len = rt.array_length(arr);
            for v in values {
                rt.object_set(arr, len.to_string(), v);
                len += 1;
            }
            rt.object_set(arr, "length".into(), Value::Number(len as f64));
            Ok(Value::Object(arr))
        });
        // __apply(callee, thisArg, argsArray) → callee.apply(thisArg, argsArray).
        // Used by the compiler to lower spread-argument calls.
        register_global_fn(self, "__apply", |rt, args| {
            let callee = args.first().cloned().unwrap_or(Value::Undefined);
            let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
            let arr = args.get(2).cloned().unwrap_or(Value::Undefined);
            let collected = match arr {
                Value::Object(id) => {
                    let n = rt.array_length(id);
                    (0..n).map(|i| rt.object_get(id, &i.to_string())).collect()
                }
                _ => Vec::new(),
            };
            rt.call_function(callee, this_arg, collected)
        });
        // __construct(callee, argsArray) → new callee(...argsArray).
        // Mirrors the Op::New handler: consults callee.prototype for the
        // new instance's [[Prototype]] and discards non-object returns.
        register_global_fn(self, "__construct", |rt, args| {
            let callee = args.first().cloned().unwrap_or(Value::Undefined);
            let arr = args.get(1).cloned().unwrap_or(Value::Undefined);
            let collected: Vec<Value> = match arr {
                Value::Object(id) => {
                    let n = rt.array_length(id);
                    (0..n).map(|i| rt.object_get(id, &i.to_string())).collect()
                }
                _ => Vec::new(),
            };
            let proto_override = if let Value::Object(cid) = &callee {
                match rt.object_get(*cid, "prototype") {
                    Value::Object(pid) => Some(pid),
                    _ => None,
                }
            } else { None };
            let mut ordinary = Object::new_ordinary();
            if proto_override.is_some() { ordinary.proto = proto_override; }
            let this_id = rt.alloc_object(ordinary);
            let this_obj = Value::Object(this_id);
            // Tier-Ω.5.s: __construct mirrors Op::New — mark new.target.
            rt.pending_new_target = Some(callee.clone());
            let ret = rt.call_function(callee, this_obj.clone(), collected)?;
            Ok(match ret {
                Value::Object(_) => ret,
                _ => this_obj,
            })
        });
    }

    /// Tier-Ω.5.g.3: helpers the compiler emits LoadGlobal+Call into for
    /// rest-collection during destructure. Installed as plain globals
    /// under `__`-prefixed names so user JS sees them.
    fn install_destructure_helpers(&mut self) {
        register_global_fn(self, "__destr_array_rest", |rt, args| {
            let src = args.first().cloned().unwrap_or(Value::Undefined);
            let start = abstract_ops::to_number(args.get(1).unwrap_or(&Value::Undefined)) as usize;
            let out_id = rt.alloc_object(Object::new_array());
            let src_id = match src {
                Value::Object(id) => id,
                _ => return Ok(Value::Object(out_id)),
            };
            let len = rt.array_length(src_id);
            let mut write_idx: usize = 0;
            for i in start..len {
                let v = rt.object_get(src_id, &i.to_string());
                rt.object_set(out_id, write_idx.to_string(), v);
                write_idx += 1;
            }
            Ok(Value::Object(out_id))
        });
        register_global_fn(self, "__destr_object_rest", |rt, args| {
            let src = args.first().cloned().unwrap_or(Value::Undefined);
            let excluded = args.get(1).cloned().unwrap_or(Value::Undefined);
            let out_id = rt.alloc_object(Object::new_ordinary());
            let src_id = match src {
                Value::Object(id) => id,
                _ => return Ok(Value::Object(out_id)),
            };
            // Build excluded-set from the array-arg.
            let mut excluded_keys: Vec<String> = Vec::new();
            if let Value::Object(ex_id) = excluded {
                let n = rt.array_length(ex_id);
                for i in 0..n {
                    let v = rt.object_get(ex_id, &i.to_string());
                    excluded_keys.push(abstract_ops::to_string(&v).as_str().to_string());
                }
            }
            // Snapshot own enumerable property keys from src.
            let entries: Vec<(String, Value)> = {
                let o = rt.obj(src_id);
                o.properties.iter()
                    .filter(|(_, d)| d.enumerable)
                    .map(|(k, d)| (k.clone(), d.value.clone()))
                    .collect()
            };
            for (k, v) in entries {
                if excluded_keys.iter().any(|e| e == &k) { continue; }
                rt.object_set(out_id, k, v);
            }
            Ok(Value::Object(out_id))
        });
    }

    fn install_globals(&mut self) {
        // Tier-Ω.5.eee: atob / btoa base64 globals (HTML living standard,
        // also exposed by Node 16+). entities + parse5 depend on atob to
        // decode their packed trie data at module load.
        register_global_fn(self, "atob", |_rt, args| {
            let s = match args.first() {
                Some(Value::String(s)) => s.as_str().to_string(),
                _ => return Err(RuntimeError::TypeError("atob: expected a string".into())),
            };
            // Standard base64 with padding tolerance.
            let cleaned: String = s.chars().filter(|c| !c.is_ascii_whitespace()).collect();
            let decoded = base64_decode(&cleaned).map_err(|e| RuntimeError::Thrown(
                Value::String(Rc::new(format!("InvalidCharacterError: {}", e)))
            ))?;
            // Per spec atob returns a binary string (one byte per char).
            let out: String = decoded.iter().map(|&b| b as char).collect();
            Ok(Value::String(Rc::new(out)))
        });
        register_global_fn(self, "btoa", |_rt, args| {
            let s = match args.first() {
                Some(Value::String(s)) => s.as_str().to_string(),
                _ => return Err(RuntimeError::TypeError("btoa: expected a string".into())),
            };
            let bytes: Vec<u8> = s.chars().map(|c| c as u8).collect();
            Ok(Value::String(Rc::new(base64_encode(&bytes))))
        });
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
        // Tier-Ω.5.j.proto: Function global as a non-constructible stub.
        // Full eval-via-Function would need parser+compiler dependency
        // injection and a Closure-from-FunctionExpression path; deferred.
        // Stub throws a clearer error than "callee is not callable".
        // Tier-Ω.5.ccc: Function constructor v1 stub. The single
        // overwhelmingly-common pattern in real code is the
        // global-detection idiom `Function('return this')()` (lodash,
        // many polyfills). Recognize that exact body and return a
        // closure that yields globalThis. Everything else still
        // throws — full eval-via-Function needs a parser+compiler
        // dependency and is deferred.
        register_global_fn(self, "Function", |rt, args| {
            let body = match args.last() {
                Some(Value::String(s)) => s.as_str().trim().to_string(),
                _ => String::new(),
            };
            if body == "return this" || body == "return this;" {
                let global_obj = rt.globals.get("globalThis").cloned().unwrap_or(Value::Undefined);
                let f_obj = make_native("<Function('return this')>", move |_rt, _args| Ok(global_obj.clone()));
                return Ok(Value::Object(rt.alloc_object(f_obj)));
            }
            Err(RuntimeError::Thrown(Value::String(Rc::new(
                "TypeError: Function constructor not yet supported in v1".into()))))
        });
        // Tier-Ω.5.yyy: expose Function.prototype on the Function
        // global. The intrinsic %Function.prototype% is the same
        // function_prototype that backs all callable instances. Adding
        // it here lets `Function.prototype.toString.call(f)` (object-
        // hash, immer-style native-function detection) resolve.
        if let Some(fp) = self.function_prototype {
            if let Some(Value::Object(fn_global)) = self.globals.get("Function").cloned() {
                self.object_set(fn_global, "prototype".into(), Value::Object(fp));
                self.object_set(fp, "constructor".into(), Value::Object(fn_global));
            }
        }
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

    fn install_object_static(&mut self) {
        let obj_ctor = self.alloc_object(Object::new_ordinary());
        register_method(self, obj_ctor, "keys", |rt, args| {
            let id = match args.first() {
                Some(Value::Object(id)) => *id,
                _ => return Ok(Value::Object(rt.alloc_object(Object::new_array()))),
            };
            let arr = rt.alloc_object(Object::new_array());
            let keys: Vec<String> = {
                let o = rt.obj(id);
                if matches!(o.internal_kind, InternalKind::Array) {
                    // Numeric keys in ascending order, length excluded.
                    let mut ks: Vec<(u64, String)> = o.properties.iter()
                        .filter_map(|(k, d)| if d.enumerable && k != "length" {
                            k.parse::<u64>().ok().map(|n| (n, k.clone()))
                        } else { None })
                        .collect();
                    ks.sort_by_key(|(n, _)| *n);
                    ks.into_iter().map(|(_, k)| k).collect()
                } else {
                    o.properties.iter().filter(|(k, d)| d.enumerable && *k != "length")
                        .map(|(k, _)| k.clone()).collect()
                }
            };
            for (i, k) in keys.iter().enumerate() {
                rt.object_set(arr, i.to_string(), Value::String(Rc::new(k.clone())));
            }
            rt.object_set(arr, "length".into(), Value::Number(keys.len() as f64));
            Ok(Value::Object(arr))
        });
        register_method(self, obj_ctor, "values", |rt, args| {
            let id = match args.first() {
                Some(Value::Object(id)) => *id,
                _ => return Ok(Value::Object(rt.alloc_object(Object::new_array()))),
            };
            let arr = rt.alloc_object(Object::new_array());
            // Tier-Ω.5.bbbbb: dispatch accessor getters for Object.values.
            let keys: Vec<(String, Option<Value>)> = {
                let o = rt.obj(id);
                let is_array = matches!(o.internal_kind, InternalKind::Array);
                let mut entries: Vec<(String, Option<Value>)> = o.properties.iter()
                    .filter(|(k, d)| d.enumerable && !(is_array && *k == "length"))
                    .map(|(k, d)| (k.clone(), d.getter.clone()))
                    .collect();
                if is_array {
                    entries.sort_by_key(|(k, _)| k.parse::<u64>().unwrap_or(u64::MAX));
                }
                entries
            };
            let mut kvs: Vec<(String, Value)> = Vec::with_capacity(keys.len());
            for (k, getter_opt) in keys {
                let v = if let Some(getter) = getter_opt {
                    rt.call_function(getter, Value::Object(id), Vec::new())?
                } else {
                    rt.object_get(id, &k)
                };
                kvs.push((k, v));
            }
            for (i, (_, v)) in kvs.iter().enumerate() {
                rt.object_set(arr, i.to_string(), v.clone());
            }
            rt.object_set(arr, "length".into(), Value::Number(kvs.len() as f64));
            Ok(Value::Object(arr))
        });
        register_method(self, obj_ctor, "entries", |rt, args| {
            let id = match args.first() {
                Some(Value::Object(id)) => *id,
                _ => return Ok(Value::Object(rt.alloc_object(Object::new_array()))),
            };
            let arr = rt.alloc_object(Object::new_array());
            // Tier-Ω.5.bbbbb: dispatch accessor getters for Object.entries.
            let keys: Vec<(String, Option<Value>)> = {
                let o = rt.obj(id);
                let is_array = matches!(o.internal_kind, InternalKind::Array);
                let mut entries: Vec<(String, Option<Value>)> = o.properties.iter()
                    .filter(|(k, d)| d.enumerable && !(is_array && *k == "length"))
                    .map(|(k, d)| (k.clone(), d.getter.clone()))
                    .collect();
                if is_array {
                    entries.sort_by_key(|(k, _)| k.parse::<u64>().unwrap_or(u64::MAX));
                }
                entries
            };
            let mut kvs: Vec<(String, Value)> = Vec::with_capacity(keys.len());
            for (k, getter_opt) in keys {
                let v = if let Some(getter) = getter_opt {
                    rt.call_function(getter, Value::Object(id), Vec::new())?
                } else {
                    rt.object_get(id, &k)
                };
                kvs.push((k, v));
            }
            for (i, (k, v)) in kvs.iter().enumerate() {
                let pair = rt.alloc_object(Object::new_array());
                rt.object_set(pair, "0".into(), Value::String(Rc::new(k.clone())));
                rt.object_set(pair, "1".into(), v.clone());
                rt.object_set(pair, "length".into(), Value::Number(2.0));
                rt.object_set(arr, i.to_string(), Value::Object(pair));
            }
            rt.object_set(arr, "length".into(), Value::Number(kvs.len() as f64));
            Ok(Value::Object(arr))
        });
        register_method(self, obj_ctor, "assign", |rt, args| {
            let target = match args.first() {
                Some(Value::Object(id)) => *id,
                // Tier-Ω.5.tttt: name the offending target type per Doc 721
                // §VI.6 / Doc 723 Layer-B. zod's "target must be an object"
                // chain was a dead-end tag; now the type is part of the chain.
                Some(other) => return Err(RuntimeError::TypeError(format!(
                    "Object.assign: target must be an object (target-type='{}')",
                    other.type_of()
                ))),
                None => return Err(RuntimeError::TypeError(
                    "Object.assign: target must be an object (target-type='missing')".into())),
            };
            for src in args.iter().skip(1) {
                if let Value::Object(sid) = src {
                    // Tier-Ω.5.bbbbb: dispatch accessor getters when reading
                    // source properties, mirror of Ω.5.aaaaa for module
                    // import-binding paths. Babel/TS-compiled libs export
                    // via Object.defineProperty getters; Object.assign was
                    // copying d.value (undefined) instead of invoking.
                    let entries: Vec<(String, Option<Value>, bool)> = rt.obj(*sid).properties.iter()
                        .filter(|(_, d)| d.enumerable)
                        .map(|(k, d)| (k.clone(), d.getter.clone(), d.getter.is_none()))
                        .collect();
                    for (k, getter_opt, is_data) in entries {
                        let v = if let Some(getter) = getter_opt {
                            rt.call_function(getter, Value::Object(*sid), Vec::new())?
                        } else if is_data {
                            rt.object_get(*sid, &k)
                        } else { continue };
                        rt.object_set(target, k, v);
                    }
                }
            }
            Ok(Value::Object(target))
        });
        register_method(self, obj_ctor, "freeze", |rt, args| {
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            if let Value::Object(id) = &v {
                let o = rt.obj_mut(*id);
                o.extensible = false;
                for d in o.properties.values_mut() {
                    d.writable = false; d.configurable = false;
                }
            }
            Ok(v)
        });
        register_method(self, obj_ctor, "isFrozen", |rt, args| {
            let id = match args.first() { Some(Value::Object(id)) => *id, _ => return Ok(Value::Boolean(true)) };
            let o = rt.obj(id);
            let frozen = !o.extensible && o.properties.values().all(|d| !d.writable && !d.configurable);
            Ok(Value::Boolean(frozen))
        });
        register_method(self, obj_ctor, "fromEntries", |rt, args| {
            let out = rt.alloc_object(Object::new_ordinary());
            let src = match args.first() { Some(v) => v.clone(), None => return Ok(Value::Object(out)) };
            // Iterate via @@iterator protocol.
            let entries = collect_iterable(rt, src)?;
            for e in entries {
                if let Value::Object(pair) = e {
                    let k = rt.object_get(pair, "0");
                    let v = rt.object_get(pair, "1");
                    let key = crate::abstract_ops::to_string(&k).as_str().to_string();
                    rt.object_set(out, key, v);
                }
            }
            Ok(Value::Object(out))
        });
        // Tier-Ω.5.j.proto: Object.defineProperty / defineProperties /
        // getOwnPropertyDescriptor / getOwnPropertyNames.
        // v1 reads only `value` from the descriptor; writable/enumerable/
        // configurable are tracked as defaults via existing object_set.
        // Accessor descriptors (get/set) are not yet honored.
        register_method(self, obj_ctor, "defineProperty", |rt, args| {
            let target = match args.first() {
                Some(Value::Object(id)) => *id,
                other => return Err(RuntimeError::TypeError(format!(
                    "Object.defineProperty: target must be an object (got {})",
                    other.map(|v| match v {
                        Value::Undefined => "undefined".to_string(),
                        Value::Null => "null".to_string(),
                        Value::Boolean(_) => "boolean".to_string(),
                        Value::Number(_) => "number".to_string(),
                        Value::String(_) => "string".to_string(),
                        _ => "other".to_string(),
                    }).unwrap_or_else(|| "missing".into())
                ))),
            };
            let key = abstract_ops::to_string(&args.get(1).cloned().unwrap_or(Value::Undefined))
                .as_str().to_string();
            let desc = args.get(2).cloned().unwrap_or(Value::Undefined);
            let desc_id = match desc {
                Value::Object(id) => id,
                _ => return Err(RuntimeError::TypeError("Object.defineProperty: descriptor must be an object".into())),
            };
            // Tier-Ω.5.nnn: accessor-descriptor support. If the descriptor
            // has a `get` or `set` function, store as accessor; else
            // treat as data descriptor (existing semantics).
            let getter = rt.object_get(desc_id, "get");
            let setter = rt.object_get(desc_id, "set");
            let has_getter = matches!(&getter, Value::Object(_));
            let has_setter = matches!(&setter, Value::Object(_));
            if has_getter || has_setter {
                rt.obj_mut(target).properties.insert(key, crate::value::PropertyDescriptor {
                    value: Value::Undefined,
                    writable: false, enumerable: true, configurable: true,
                    getter: if has_getter { Some(getter) } else { None },
                    setter: if has_setter { Some(setter) } else { None },
                });
            } else {
                let value = rt.object_get(desc_id, "value");
                rt.object_set(target, key, value);
            }
            Ok(Value::Object(target))
        });
        register_method(self, obj_ctor, "defineProperties", |rt, args| {
            let target = match args.first() {
                Some(Value::Object(id)) => *id,
                _ => return Err(RuntimeError::TypeError("Object.defineProperties: target must be an object".into())),
            };
            let props = match args.get(1) {
                Some(Value::Object(id)) => *id,
                _ => return Err(RuntimeError::TypeError("Object.defineProperties: props must be an object".into())),
            };
            // Snapshot own enumerable keys + descriptor objects.
            let entries: Vec<(String, Value)> = {
                let o = rt.obj(props);
                o.properties.iter()
                    .filter(|(_, d)| d.enumerable)
                    .map(|(k, d)| (k.clone(), d.value.clone()))
                    .collect()
            };
            for (k, dv) in entries {
                if let Value::Object(did) = dv {
                    let getter = rt.object_get(did, "get");
                    let setter = rt.object_get(did, "set");
                    let has_getter = matches!(&getter, Value::Object(_));
                    let has_setter = matches!(&setter, Value::Object(_));
                    if has_getter || has_setter {
                        rt.obj_mut(target).properties.insert(k, crate::value::PropertyDescriptor {
                            value: Value::Undefined,
                            writable: false, enumerable: true, configurable: true,
                            getter: if has_getter { Some(getter) } else { None },
                            setter: if has_setter { Some(setter) } else { None },
                        });
                    } else {
                        let value = rt.object_get(did, "value");
                        rt.object_set(target, k, value);
                    }
                }
            }
            Ok(Value::Object(target))
        });
        register_method(self, obj_ctor, "getOwnPropertyDescriptor", |rt, args| {
            let id = match args.first() {
                Some(Value::Object(id)) => *id,
                _ => return Ok(Value::Undefined),
            };
            let key = abstract_ops::to_string(&args.get(1).cloned().unwrap_or(Value::Undefined))
                .as_str().to_string();
            let (has, value, writable, enumerable, configurable) = {
                let o = rt.obj(id);
                match o.properties.get(&key) {
                    Some(d) => (true, d.value.clone(), d.writable, d.enumerable, d.configurable),
                    None => (false, Value::Undefined, false, false, false),
                }
            };
            if !has { return Ok(Value::Undefined); }
            let out = rt.alloc_object(Object::new_ordinary());
            rt.object_set(out, "value".into(), value);
            rt.object_set(out, "writable".into(), Value::Boolean(writable));
            rt.object_set(out, "enumerable".into(), Value::Boolean(enumerable));
            rt.object_set(out, "configurable".into(), Value::Boolean(configurable));
            Ok(Value::Object(out))
        });
        register_method(self, obj_ctor, "getOwnPropertyNames", |rt, args| {
            let id = match args.first() {
                Some(Value::Object(id)) => *id,
                _ => return Ok(Value::Object(rt.alloc_object(Object::new_array()))),
            };
            let arr = rt.alloc_object(Object::new_array());
            let keys: Vec<String> = {
                let o = rt.obj(id);
                let is_array = matches!(o.internal_kind, InternalKind::Array);
                if is_array {
                    let mut ks: Vec<(u64, String)> = o.properties.iter()
                        .filter_map(|(k, _)| k.parse::<u64>().ok().map(|n| (n, k.clone())))
                        .collect();
                    ks.sort_by_key(|(n, _)| *n);
                    let mut out: Vec<String> = ks.into_iter().map(|(_, k)| k).collect();
                    if o.properties.contains_key("length") { out.push("length".into()); }
                    out
                } else {
                    o.properties.keys().cloned().collect()
                }
            };
            for (i, k) in keys.iter().enumerate() {
                rt.object_set(arr, i.to_string(), Value::String(Rc::new(k.clone())));
            }
            rt.object_set(arr, "length".into(), Value::Number(keys.len() as f64));
            Ok(Value::Object(arr))
        });
        // Tier-Ω.5.v: Object.create(proto, propertiesObject?). Per
        // ECMA-262 §20.1.2.2: proto must be Object or null; otherwise
        // throw TypeError. Subset: properties handled via the `value`
        // field of each descriptor (matches our defineProperty subset).
        register_method(self, obj_ctor, "create", |rt, args| {
            let proto_arg = args.first().cloned().unwrap_or(Value::Undefined);
            let proto_id = match proto_arg {
                Value::Null => None,
                Value::Object(id) => Some(id),
                _ => return Err(RuntimeError::TypeError(
                    "Object.create: prototype must be Object or null".into())),
            };
            let mut obj = Object::new_ordinary();
            obj.proto = proto_id;
            let id = rt.alloc_object(obj);
            if let Some(Value::Object(props_id)) = args.get(1) {
                let entries: Vec<(String, Value)> = {
                    let o = rt.obj(*props_id);
                    o.properties.iter()
                        .filter(|(_, d)| d.enumerable)
                        .map(|(k, d)| (k.clone(), d.value.clone()))
                        .collect()
                };
                for (k, dv) in entries {
                    let v = match dv {
                        Value::Object(did) => rt.object_get(did, "value"),
                        _ => Value::Undefined,
                    };
                    rt.object_set(id, k, v);
                }
            }
            Ok(Value::Object(id))
        });
        // Tier-Ω.5.nn: Object.getPrototypeOf + Object.setPrototypeOf.
        // axios + many others destructure `const { getPrototypeOf } = Object;`
        // at module top level. Without these statics, getPrototypeOf is
        // undefined and `getPrototypeOf(Uint8Array)` errors. The Reflect
        // variant existed (Ω.5.cc) but consumer code uses Object.X.
        register_method(self, obj_ctor, "getPrototypeOf", |rt, args| {
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            match v {
                Value::Object(id) => match rt.obj(id).proto {
                    Some(p) => Ok(Value::Object(p)),
                    None => Ok(Value::Null),
                },
                _ => Ok(Value::Null),
            }
        });
        register_method(self, obj_ctor, "setPrototypeOf", |rt, args| {
            let target = match args.first() {
                Some(Value::Object(id)) => *id,
                _ => return Ok(args.first().cloned().unwrap_or(Value::Undefined)),
            };
            let proto = args.get(1).cloned().unwrap_or(Value::Null);
            let new_proto = match proto {
                Value::Object(id) => Some(id),
                Value::Null => None,
                _ => None,
            };
            rt.obj_mut(target).proto = new_proto;
            Ok(Value::Object(target))
        });
        register_method(self, obj_ctor, "create", |rt, args| {
            let proto_arg = args.first().cloned().unwrap_or(Value::Undefined);
            let mut obj = Object::new_ordinary();
            obj.proto = match proto_arg {
                Value::Object(id) => Some(id),
                Value::Null => None,
                _ => return Err(RuntimeError::TypeError("Object.create: prototype must be object or null".into())),
            };
            let id = rt.alloc_object(obj);
            // Properties argument (descriptor map) — implement same shape as defineProperties.
            if let Some(Value::Object(props_id)) = args.get(1) {
                let entries: Vec<(String, Value)> = rt.obj(*props_id).properties.iter()
                    .filter(|(_, d)| d.enumerable)
                    .map(|(k, d)| (k.clone(), d.value.clone()))
                    .collect();
                for (k, dv) in entries {
                    if let Value::Object(did) = dv {
                        let value = rt.object_get(did, "value");
                        rt.object_set(id, k, value);
                    }
                }
            }
            Ok(Value::Object(id))
        });
        register_method(self, obj_ctor, "is", |_rt, args| {
            let a = args.first().cloned().unwrap_or(Value::Undefined);
            let b = args.get(1).cloned().unwrap_or(Value::Undefined);
            Ok(Value::Boolean(crate::value::Value::same_value(&a, &b)))
        });
        // Tier-Ω.5.t: wire `Object.prototype` to the intrinsic %Object.prototype%
        // so consumers can read `Object.prototype.hasOwnProperty` etc.
        // Without this, `var has = Object.prototype.hasOwnProperty` (a dense
        // dequal/acorn/fast-equals idiom) errors "Cannot read property
        // 'hasOwnProperty' of undefined".
        if let Some(proto) = self.object_prototype {
            self.object_set(obj_ctor, "prototype".into(), Value::Object(proto));
            // Tier-Ω.5.lll: Object.prototype.constructor = Object. Per
            // ECMA-262 §20.1.3.1. Without this, plain-object `.constructor`
            // returns undefined, breaking type-tag idioms like dequal's
            // `(ctor=foo.constructor) === bar.constructor` followed by
            // `ctor === Date` / `ctor === RegExp` / `ctor === Array`
            // dispatch.
            self.object_set(proto, "constructor".into(), Value::Object(obj_ctor));
        }
        self.globals.insert("Object".into(), Value::Object(obj_ctor));
    }

    fn install_array_static(&mut self) {
        // Tier-Ω.5.ttt: Array is a real Function (callable) per ECMA-262
        // §23.1. `new Array(n)` produces an array of length n;
        // `new Array(v0, v1, ...)` or `Array(v0, ...)` produces an
        // array of those values. rfdc's `new Array(keys.length)` and
        // many polyfill patterns depend on this.
        let arr_proto_ref = self.array_prototype;
        let arr_ctor_native = make_native("Array", move |rt, args| {
            let mut o = Object::new_array();
            if args.len() == 1 {
                if let Value::Number(n) = &args[0] {
                    let len = *n as usize;
                    o.set_own("length".into(), Value::Number(len as f64));
                    let id = rt.alloc_object(o);
                    return Ok(Value::Object(id));
                }
            }
            // Variadic form: each arg becomes an element.
            for (i, v) in args.iter().enumerate() {
                o.set_own(i.to_string(), v.clone());
            }
            o.set_own("length".into(), Value::Number(args.len() as f64));
            let id = rt.alloc_object(o);
            let _ = arr_proto_ref;
            Ok(Value::Object(id))
        });
        let arr_ctor = self.alloc_object(arr_ctor_native);
        register_method(self, arr_ctor, "isArray", |rt, args| {
            Ok(Value::Boolean(matches!(args.first(),
                Some(Value::Object(id)) if matches!(rt.obj(*id).internal_kind, InternalKind::Array))))
        });
        register_method(self, arr_ctor, "of", |rt, args| {
            let out = rt.alloc_object(Object::new_array());
            for (i, v) in args.iter().enumerate() {
                rt.object_set(out, i.to_string(), v.clone());
            }
            rt.object_set(out, "length".into(), Value::Number(args.len() as f64));
            Ok(Value::Object(out))
        });
        register_method(self, arr_ctor, "from", |rt, args| {
            let src = args.first().cloned().unwrap_or(Value::Undefined);
            let map_fn = args.get(1).cloned();
            let out = rt.alloc_object(Object::new_array());
            // Two shapes: iterable (has @@iterator) or array-like (has length).
            let items: Vec<Value> = match &src {
                Value::Object(id) => {
                    // Try iterator first.
                    let has_iter = !matches!(rt.object_get(*id, "@@iterator"), Value::Undefined);
                    if has_iter {
                        collect_iterable(rt, src.clone())?
                    } else {
                        let len = rt.array_length(*id);
                        (0..len).map(|i| rt.object_get(*id, &i.to_string())).collect()
                    }
                }
                Value::String(s) => s.chars().map(|c| Value::String(Rc::new(c.to_string()))).collect(),
                _ => Vec::new(),
            };
            for (i, v) in items.into_iter().enumerate() {
                let mapped = if let Some(f) = &map_fn {
                    rt.call_function(f.clone(), Value::Undefined, vec![v, Value::Number(i as f64)])?
                } else { v };
                rt.object_set(out, i.to_string(), mapped);
            }
            let len = rt.array_length(out);
            rt.object_set(out, "length".into(), Value::Number(len as f64));
            Ok(Value::Object(out))
        });
        if let Some(proto) = self.array_prototype {
            self.object_set(arr_ctor, "prototype".into(), Value::Object(proto));
        }
        self.globals.insert("Array".into(), Value::Object(arr_ctor));
    }

    /// Tier-Ω.5.s: Number static surface — constants + numeric predicates.
    /// The comment at the top of this file promised this surface; the
    /// install function was never wired. semver and friends read
    /// `Number.MAX_SAFE_INTEGER` / `Number.isInteger`, so this closure
    /// is load-bearing for the parity corpus.
    fn install_number_static(&mut self) {
        // Tier-Ω.5.z: Number is also callable: `Number("3") === 3`.
        let num_obj = make_native("Number", |_rt, args| {
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            Ok(Value::Number(abstract_ops::to_number(&v)))
        });
        let num = self.alloc_object(num_obj);
        // Constants per ECMA-262 §21.1.2.
        self.object_set(num, "MAX_SAFE_INTEGER".into(), Value::Number(9007199254740991.0));
        self.object_set(num, "MIN_SAFE_INTEGER".into(), Value::Number(-9007199254740991.0));
        self.object_set(num, "MAX_VALUE".into(), Value::Number(f64::MAX));
        self.object_set(num, "MIN_VALUE".into(), Value::Number(5e-324));
        self.object_set(num, "EPSILON".into(), Value::Number(f64::EPSILON));
        self.object_set(num, "POSITIVE_INFINITY".into(), Value::Number(f64::INFINITY));
        self.object_set(num, "NEGATIVE_INFINITY".into(), Value::Number(f64::NEG_INFINITY));
        self.object_set(num, "NaN".into(), Value::Number(f64::NAN));
        // Tier-Ω.5.ggggg: global Infinity / NaN / undefined per ECMA-262
        // §19.1. acorn's tokenizer uses `Infinity` as a sentinel in
        // `for (var i=0, e=Infinity; i<e; ...)`; without the global,
        // i<undefined is false, the loop never runs, every numeric literal
        // fails to tokenize.
        self.globals.insert("Infinity".into(), Value::Number(f64::INFINITY));
        self.globals.insert("NaN".into(), Value::Number(f64::NAN));
        self.globals.insert("undefined".into(), Value::Undefined);
        // Predicates. Note: Number.isX (capital-N) differs from global
        // isX in NOT coercing — typeof check first, false otherwise.
        register_method(self, num, "isInteger", |_rt, args| {
            let n = match args.first() {
                Some(Value::Number(n)) => *n,
                _ => return Ok(Value::Boolean(false)),
            };
            Ok(Value::Boolean(n.is_finite() && n.floor() == n))
        });
        register_method(self, num, "isFinite", |_rt, args| {
            let n = match args.first() {
                Some(Value::Number(n)) => *n,
                _ => return Ok(Value::Boolean(false)),
            };
            Ok(Value::Boolean(n.is_finite()))
        });
        register_method(self, num, "isNaN", |_rt, args| {
            let n = match args.first() {
                Some(Value::Number(n)) => *n,
                _ => return Ok(Value::Boolean(false)),
            };
            Ok(Value::Boolean(n.is_nan()))
        });
        register_method(self, num, "isSafeInteger", |_rt, args| {
            let n = match args.first() {
                Some(Value::Number(n)) => *n,
                _ => return Ok(Value::Boolean(false)),
            };
            Ok(Value::Boolean(n.is_finite() && n.floor() == n && n.abs() <= 9007199254740991.0))
        });
        // Alias the global parseInt / parseFloat onto Number.
        if let Some(pi) = self.globals.get("parseInt").cloned() {
            self.object_set(num, "parseInt".into(), pi);
        }
        if let Some(pf) = self.globals.get("parseFloat").cloned() {
            self.object_set(num, "parseFloat".into(), pf);
        }
        if let Some(proto) = self.number_prototype {
            self.object_set(num, "prototype".into(), Value::Object(proto));
        }
        self.globals.insert("Number".into(), Value::Object(num));
        self.install_string_global();
        self.install_boolean_global();
    }

    /// Tier-Ω.5.z: `String(x)` callable — coerces to string per ToString.
    /// `new String(x)` (wrapper object) deferred; v1 returns the primitive.
    /// Carries `String.prototype` for the dense `String.prototype.X`
    /// access idiom (axios, etc.) used by polyfills + duck-type checks.
    fn install_string_global(&mut self) {
        let str_obj = make_native("String", |_rt, args| {
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            Ok(Value::String(Rc::new(abstract_ops::to_string(&v).as_str().to_string())))
        });
        let str_id = self.alloc_object(str_obj);
        register_method(self, str_id, "fromCharCode", |_rt, args| {
            let mut s = String::new();
            for a in args {
                let n = abstract_ops::to_number(a);
                if let Some(c) = char::from_u32(n as u32) { s.push(c); }
            }
            Ok(Value::String(Rc::new(s)))
        });
        register_method(self, str_id, "fromCodePoint", |_rt, args| {
            let mut s = String::new();
            for a in args {
                let n = abstract_ops::to_number(a);
                if let Some(c) = char::from_u32(n as u32) { s.push(c); }
            }
            Ok(Value::String(Rc::new(s)))
        });
        // Tier-Ω.5.ww.b: String.raw(template, ...subs). Spec uses
        // template.raw; v1 falls back to indexed cooked values from the
        // strings array (Tier-Ω.5.ww doesn't populate .raw yet). Sufficient
        // for the camelcase / consola / styled-components patterns where
        // .raw vs cooked agree (no escape sequences requiring raw).
        register_method(self, str_id, "raw", |rt, args| {
            let template = match args.first() {
                Some(Value::Object(id)) => *id,
                _ => return Err(RuntimeError::TypeError("String.raw: first argument must be an object".into())),
            };
            let raw = match rt.object_get(template, &"raw".to_string()) {
                Value::Undefined => Value::Object(template),
                v => v,
            };
            let raw_id = match raw {
                Value::Object(id) => id,
                _ => return Err(RuntimeError::TypeError("String.raw: raw must be an object".into())),
            };
            let length = match rt.object_get(raw_id, &"length".to_string()) {
                Value::Number(n) => n as i64,
                _ => {
                    let mut n: i64 = 0;
                    while !matches!(rt.object_get(raw_id, &n.to_string()), Value::Undefined) {
                        n += 1;
                    }
                    n
                }
            };
            let mut out = String::new();
            for i in 0..length {
                let seg = rt.object_get(raw_id, &i.to_string());
                out.push_str(&abstract_ops::to_string(&seg));
                if i + 1 < length {
                    if let Some(sub) = args.get((i as usize) + 1) {
                        out.push_str(&abstract_ops::to_string(sub));
                    }
                }
            }
            Ok(Value::String(Rc::new(out)))
        });
        if let Some(proto) = self.string_prototype {
            self.object_set(str_id, "prototype".into(), Value::Object(proto));
        }
        self.globals.insert("String".into(), Value::Object(str_id));
    }

    /// Tier-Ω.5.z: `Boolean(x)` callable — coerces to boolean per ToBoolean.
    fn install_boolean_global(&mut self) {
        let b_obj = make_native("Boolean", |_rt, args| {
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            Ok(Value::Boolean(abstract_ops::to_boolean(&v)))
        });
        let b_id = self.alloc_object(b_obj);
        self.globals.insert("Boolean".into(), Value::Object(b_id));
        // Tier-Ω.5.pp: Proxy as a stub constructor. v1 deviation: the
        // proxy doesn't actually intercept operations; it's a transparent
        // pass-through that returns the target as-is. This lets `new
        // Proxy(target, handler)` not crash; access still goes through
        // the underlying target. Many packages create proxies for
        // deprecation guards or namespace shims where the trap-handling
        // isn't actually exercised during shape probe.
        let proxy_obj = make_native("Proxy", |rt, args| {
            let target = args.first().cloned().unwrap_or(Value::Undefined);
            // Return target directly; trap-handling deferred.
            let _ = (rt, args);
            Ok(target)
        });
        let proxy_id = self.alloc_object(proxy_obj);
        self.globals.insert("Proxy".into(), Value::Object(proxy_id));

        // Tier-Ω.5.ccccc: minimal WHATWG URL global. Parses
        // scheme://[user:pass@]host[:port]/path?query#fragment and exposes
        // the standard read-only properties. Real spec parsing is intricate
        // (punycode, percent-encoding canonicalization, IDN); v1 covers
        // the URL shapes the corpus actually constructs.
        let url_ctor = make_native("URL", |rt, args| {
            let input = match args.first() {
                Some(Value::String(s)) => s.as_str().to_string(),
                Some(v) => crate::abstract_ops::to_string(v).as_str().to_string(),
                None => return Err(RuntimeError::TypeError("URL: invalid URL".into())),
            };
            let base = match args.get(1) {
                Some(Value::String(s)) => Some(s.as_str().to_string()),
                _ => None,
            };
            // Resolve against base if provided and input is relative.
            let full = match base {
                Some(b) if !input.contains("://") && !input.starts_with("//") => {
                    // Strip filename from base path, append input.
                    let cut = b.rfind('/').map(|i| &b[..=i]).unwrap_or(&b);
                    format!("{}{}", cut, input)
                }
                _ => input.clone(),
            };
            let mut rest: &str = &full;
            let (protocol, after_scheme) = if let Some(i) = rest.find("://") {
                let p = format!("{}:", &rest[..i]);
                rest = &rest[i+3..];
                (p, true)
            } else if let Some(i) = rest.find(':') {
                let p = format!("{}:", &rest[..i]);
                rest = &rest[i+1..];
                (p, false)
            } else {
                ("".to_string(), false)
            };
            let (hash, rest2) = match rest.find('#') {
                Some(i) => (rest[i..].to_string(), &rest[..i]),
                None => ("".to_string(), rest),
            };
            let (search, rest3) = match rest2.find('?') {
                Some(i) => (rest2[i..].to_string(), &rest2[..i]),
                None => ("".to_string(), rest2),
            };
            let (authority, path) = if after_scheme {
                match rest3.find('/') {
                    Some(i) => (&rest3[..i], &rest3[i..]),
                    None => (rest3, ""),
                }
            } else {
                ("", rest3)
            };
            let path_s = if path.is_empty() && after_scheme { "/".to_string() } else { path.to_string() };
            let (userinfo, hostport) = match authority.rfind('@') {
                Some(i) => (&authority[..i], &authority[i+1..]),
                None => ("", authority),
            };
            let (username, password) = match userinfo.find(':') {
                Some(i) => (&userinfo[..i], &userinfo[i+1..]),
                None => (userinfo, ""),
            };
            let (hostname, port) = if hostport.starts_with('[') {
                // IPv6 literal.
                match hostport.find("]:") {
                    Some(i) => (&hostport[..=i], &hostport[i+2..]),
                    None => (hostport, ""),
                }
            } else {
                match hostport.rfind(':') {
                    Some(i) => (&hostport[..i], &hostport[i+1..]),
                    None => (hostport, ""),
                }
            };
            let origin = if protocol.is_empty() {
                "null".to_string()
            } else {
                format!("{}//{}", protocol, hostport)
            };
            let href = full.clone();

            let url_obj = match rt.current_this() {
                Value::Object(id) => id,
                _ => rt.alloc_object(Object::new_ordinary()),
            };
            rt.object_set(url_obj, "href".into(), Value::String(Rc::new(href)));
            rt.object_set(url_obj, "protocol".into(), Value::String(Rc::new(protocol)));
            rt.object_set(url_obj, "username".into(), Value::String(Rc::new(username.into())));
            rt.object_set(url_obj, "password".into(), Value::String(Rc::new(password.into())));
            rt.object_set(url_obj, "host".into(), Value::String(Rc::new(hostport.into())));
            rt.object_set(url_obj, "hostname".into(), Value::String(Rc::new(hostname.into())));
            rt.object_set(url_obj, "port".into(), Value::String(Rc::new(port.into())));
            rt.object_set(url_obj, "pathname".into(), Value::String(Rc::new(path_s)));
            rt.object_set(url_obj, "search".into(), Value::String(Rc::new(search)));
            rt.object_set(url_obj, "hash".into(), Value::String(Rc::new(hash)));
            rt.object_set(url_obj, "origin".into(), Value::String(Rc::new(origin)));
            register_method(rt, url_obj, "toString", |rt, _args| {
                Ok(rt.object_get(match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::String(Rc::new(String::new()))) }, "href"))
            });
            register_method(rt, url_obj, "toJSON", |rt, _args| {
                Ok(rt.object_get(match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::String(Rc::new(String::new()))) }, "href"))
            });
            Ok(Value::Object(url_obj))
        });
        let url_id = self.alloc_object(url_ctor);
        let url_proto = self.alloc_object(Object::new_ordinary());
        self.object_set(url_id, "prototype".into(), Value::Object(url_proto));
        register_method(self, url_id, "canParse", |_rt, args| {
            let s = match args.first() { Some(Value::String(s)) => s.as_str().to_string(), _ => return Ok(Value::Boolean(false)) };
            Ok(Value::Boolean(s.contains("://") || s.starts_with("file:") || s.starts_with("data:")))
        });
        self.globals.insert("URL".into(), Value::Object(url_id));

        // Tier-Ω.5.ll: BigInt as callable global. zod uses `BigInt(x)`.
        let bi_obj = make_native("BigInt", |_rt, args| {
            let v = args.first().cloned().unwrap_or(Value::Undefined);
            match v {
                Value::BigInt(s) => Ok(Value::BigInt(s)),
                Value::Number(n) => Ok(Value::BigInt(std::rc::Rc::new(format!("{}", n as i64)))),
                Value::String(s) => {
                    let trimmed = s.trim();
                    Ok(Value::BigInt(std::rc::Rc::new(trimmed.to_string())))
                }
                Value::Boolean(b) => Ok(Value::BigInt(std::rc::Rc::new(if b { "1".into() } else { "0".into() }))),
                _ => Err(RuntimeError::Thrown(Value::String(std::rc::Rc::new(
                    "TypeError: Cannot convert to BigInt".into()
                )))),
            }
        });
        let bi_id = self.alloc_object(bi_obj);
        register_method(self, bi_id, "asIntN", |_rt, args| Ok(args.get(1).cloned().unwrap_or(Value::Undefined)));
        register_method(self, bi_id, "asUintN", |_rt, args| Ok(args.get(1).cloned().unwrap_or(Value::Undefined)));
        self.globals.insert("BigInt".into(), Value::Object(bi_id));
        self.install_error_globals();
        self.install_reflect();
        self.install_map_set_globals();
        self.install_date_global();
        self.install_typed_array_stubs();
        self.install_weak_ref_globals();
    }

    /// Tier-Ω.5.dd: Map / Set / WeakMap / WeakSet as real implementations.
    /// Storage uses the underlying Object's properties map for v1 — keys
    /// are stringified via ToString. This is a v1 deviation: real Map keys
    /// are by SameValueZero, so object keys would each be distinct identity-
    /// wise. Our string-keyed storage collides object keys via their
    /// stringified form. Most parity packages don't depend on object-keyed
    /// Maps; documented for future substrate.
    fn install_map_set_globals(&mut self) {
        for collection in &["Map", "WeakMap"] {
            let proto = self.alloc_object(Object::new_ordinary());
            register_method(self, proto, "get", |rt, args| {
                let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
                let key = args.first().cloned().unwrap_or(Value::Undefined);
                let key_s = abstract_ops::to_string(&key).as_str().to_string();
                let storage = match rt.object_get(this, "__map_data") {
                    Value::Object(id) => id,
                    _ => return Ok(Value::Undefined),
                };
                Ok(rt.object_get(storage, &key_s))
            });
            register_method(self, proto, "set", |rt, args| {
                let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
                let key = args.first().cloned().unwrap_or(Value::Undefined);
                let val = args.get(1).cloned().unwrap_or(Value::Undefined);
                let key_s = abstract_ops::to_string(&key).as_str().to_string();
                let storage = match rt.object_get(this, "__map_data") {
                    Value::Object(id) => id,
                    _ => {
                        let s = rt.alloc_object(Object::new_ordinary());
                        rt.object_set(this, "__map_data".into(), Value::Object(s));
                        s
                    }
                };
                let existed = !matches!(rt.object_get(storage, &key_s), Value::Undefined);
                rt.object_set(storage, key_s, val);
                if !existed {
                    let prev = match rt.object_get(this, "size") {
                        Value::Number(n) => n,
                        _ => 0.0,
                    };
                    rt.object_set(this, "size".into(), Value::Number(prev + 1.0));
                }
                Ok(Value::Object(this))
            });
            register_method(self, proto, "has", |rt, args| {
                let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Boolean(false)) };
                let key = args.first().cloned().unwrap_or(Value::Undefined);
                let key_s = abstract_ops::to_string(&key).as_str().to_string();
                let storage = match rt.object_get(this, "__map_data") {
                    Value::Object(id) => id,
                    _ => return Ok(Value::Boolean(false)),
                };
                Ok(Value::Boolean(rt.obj(storage).properties.contains_key(&key_s)))
            });
            register_method(self, proto, "delete", |rt, args| {
                let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Boolean(false)) };
                let key = args.first().cloned().unwrap_or(Value::Undefined);
                let key_s = abstract_ops::to_string(&key).as_str().to_string();
                let storage = match rt.object_get(this, "__map_data") {
                    Value::Object(id) => id,
                    _ => return Ok(Value::Boolean(false)),
                };
                let existed = rt.obj_mut(storage).properties.remove(&key_s).is_some();
                if existed {
                    let prev = match rt.object_get(this, "size") {
                        Value::Number(n) => n,
                        _ => 0.0,
                    };
                    rt.object_set(this, "size".into(), Value::Number((prev - 1.0).max(0.0)));
                }
                Ok(Value::Boolean(existed))
            });
            register_method(self, proto, "clear", |rt, _args| {
                let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
                let fresh = rt.alloc_object(Object::new_ordinary());
                rt.object_set(this, "__map_data".into(), Value::Object(fresh));
                rt.object_set(this, "size".into(), Value::Number(0.0));
                Ok(Value::Undefined)
            });
            register_method(self, proto, "forEach", |rt, args| {
                let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
                let cb = args.first().cloned().unwrap_or(Value::Undefined);
                let storage = match rt.object_get(this, "__map_data") {
                    Value::Object(id) => id,
                    _ => return Ok(Value::Undefined),
                };
                let pairs: Vec<(String, Value)> = rt.obj(storage).properties.iter()
                    .map(|(k, d)| (k.clone(), d.value.clone()))
                    .collect();
                for (k, v) in pairs {
                    let key_v = Value::String(Rc::new(k));
                    rt.call_function(cb.clone(), Value::Undefined, vec![v, key_v, Value::Object(this)])?;
                }
                Ok(Value::Undefined)
            });
            let proto_for_ctor = proto;
            let name = (*collection).to_string();
            let ctor_obj = make_native(&name, move |rt, args| {
                let mut o = Object::new_ordinary();
                o.proto = Some(proto_for_ctor);
                let id = rt.alloc_object(o);
                let storage = rt.alloc_object(Object::new_ordinary());
                rt.object_set(id, "__map_data".into(), Value::Object(storage));
                rt.object_set(id, "size".into(), Value::Number(0.0));
                // Optional initial iterable.
                if let Some(_init) = args.first() {
                    // Skipped for v1; documented deviation.
                }
                Ok(Value::Object(id))
            });
            let ctor = self.alloc_object(ctor_obj);
            self.object_set(ctor, "prototype".into(), Value::Object(proto));
            self.object_set(proto, "constructor".into(), Value::Object(ctor));
            self.globals.insert((*collection).to_string(), Value::Object(ctor));
        }
        for collection in &["Set", "WeakSet"] {
            let proto = self.alloc_object(Object::new_ordinary());
            register_method(self, proto, "add", |rt, args| {
                let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
                let v = args.first().cloned().unwrap_or(Value::Undefined);
                let key_s = abstract_ops::to_string(&v).as_str().to_string();
                let storage = match rt.object_get(this, "__set_data") {
                    Value::Object(id) => id,
                    _ => {
                        let s = rt.alloc_object(Object::new_ordinary());
                        rt.object_set(this, "__set_data".into(), Value::Object(s));
                        s
                    }
                };
                let existed = !matches!(rt.object_get(storage, &key_s), Value::Undefined);
                rt.object_set(storage, key_s, v);
                if !existed {
                    let prev = match rt.object_get(this, "size") {
                        Value::Number(n) => n,
                        _ => 0.0,
                    };
                    rt.object_set(this, "size".into(), Value::Number(prev + 1.0));
                }
                Ok(Value::Object(this))
            });
            register_method(self, proto, "has", |rt, args| {
                let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Boolean(false)) };
                let v = args.first().cloned().unwrap_or(Value::Undefined);
                let key_s = abstract_ops::to_string(&v).as_str().to_string();
                let storage = match rt.object_get(this, "__set_data") {
                    Value::Object(id) => id,
                    _ => return Ok(Value::Boolean(false)),
                };
                Ok(Value::Boolean(rt.obj(storage).properties.contains_key(&key_s)))
            });
            register_method(self, proto, "delete", |rt, args| {
                let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Boolean(false)) };
                let v = args.first().cloned().unwrap_or(Value::Undefined);
                let key_s = abstract_ops::to_string(&v).as_str().to_string();
                let storage = match rt.object_get(this, "__set_data") {
                    Value::Object(id) => id,
                    _ => return Ok(Value::Boolean(false)),
                };
                let existed = rt.obj_mut(storage).properties.remove(&key_s).is_some();
                if existed {
                    let prev = match rt.object_get(this, "size") {
                        Value::Number(n) => n,
                        _ => 0.0,
                    };
                    rt.object_set(this, "size".into(), Value::Number((prev - 1.0).max(0.0)));
                }
                Ok(Value::Boolean(existed))
            });
            register_method(self, proto, "clear", |rt, _args| {
                let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
                let fresh = rt.alloc_object(Object::new_ordinary());
                rt.object_set(this, "__set_data".into(), Value::Object(fresh));
                rt.object_set(this, "size".into(), Value::Number(0.0));
                Ok(Value::Undefined)
            });
            register_method(self, proto, "forEach", |rt, args| {
                let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
                let cb = args.first().cloned().unwrap_or(Value::Undefined);
                let storage = match rt.object_get(this, "__set_data") {
                    Value::Object(id) => id,
                    _ => return Ok(Value::Undefined),
                };
                let vals: Vec<Value> = rt.obj(storage).properties.values()
                    .map(|d| d.value.clone())
                    .collect();
                for v in vals {
                    rt.call_function(cb.clone(), Value::Undefined, vec![v.clone(), v, Value::Object(this)])?;
                }
                Ok(Value::Undefined)
            });
            // Tier-Ω.5.rrr: @@iterator returns a values-iterator. Per
            // spec Set.prototype[Symbol.iterator] === Set.prototype.values.
            // Required for `[...new Set(arr)]` to spread.
            register_method(self, proto, "@@iterator", |rt, _args| {
                let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
                make_set_values_iterator(rt, this)
            });
            register_method(self, proto, "values", |rt, _args| {
                let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
                make_set_values_iterator(rt, this)
            });
            let proto_for_ctor = proto;
            let name = (*collection).to_string();
            let ctor_obj = make_native(&name, move |rt, args| {
                let mut o = Object::new_ordinary();
                o.proto = Some(proto_for_ctor);
                let id = rt.alloc_object(o);
                let storage = rt.alloc_object(Object::new_ordinary());
                rt.object_set(id, "__set_data".into(), Value::Object(storage));
                rt.object_set(id, "size".into(), Value::Number(0.0));
                // Tier-Ω.5.rrr: populate from iterable arg. Per spec
                // `new Set(iterable)` calls .add for each yielded value.
                if let Some(arg) = args.first() {
                    if let Ok(values) = collect_iterable(rt, arg.clone()) {
                        let mut size = 0.0_f64;
                        for v in values {
                            let key_s = abstract_ops::to_string(&v).as_str().to_string();
                            if matches!(rt.object_get(storage, &key_s), Value::Undefined) {
                                rt.object_set(storage, key_s, v);
                                size += 1.0;
                            }
                        }
                        rt.object_set(id, "size".into(), Value::Number(size));
                    }
                }
                Ok(Value::Object(id))
            });
            let ctor = self.alloc_object(ctor_obj);
            self.object_set(ctor, "prototype".into(), Value::Object(proto));
            self.object_set(proto, "constructor".into(), Value::Object(ctor));
            self.globals.insert((*collection).to_string(), Value::Object(ctor));
        }
    }

    /// Tier-Ω.5.aaaa: Date global. Real Gregorian arithmetic for year/
    /// month/day extraction; ISO-string parsing in the constructor;
    /// per-spec getter methods.
    fn install_date_global(&mut self) {
        let proto = self.alloc_object(Object::new_ordinary());
        register_method(self, proto, "getTime", |rt, _args| {
            let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Number(0.0)) };
            Ok(rt.object_get(this, "__date_ms"))
        });
        register_method(self, proto, "valueOf", |rt, _args| {
            let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Number(0.0)) };
            Ok(rt.object_get(this, "__date_ms"))
        });
        register_method(self, proto, "getFullYear", |rt, _args| {
            let this_id = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Number(f64::NAN)) };
            let ms = match rt.object_get(this_id, "__date_ms") { Value::Number(n) => n, _ => return Ok(Value::Number(f64::NAN)) };
            Ok(Value::Number(date_components(ms).0 as f64))
        });
        register_method(self, proto, "getMonth", |rt, _args| {
            let this_id = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Number(f64::NAN)) };
            let ms = match rt.object_get(this_id, "__date_ms") { Value::Number(n) => n, _ => return Ok(Value::Number(f64::NAN)) };
            Ok(Value::Number(date_components(ms).1 as f64))
        });
        register_method(self, proto, "getDate", |rt, _args| {
            let this_id = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Number(f64::NAN)) };
            let ms = match rt.object_get(this_id, "__date_ms") { Value::Number(n) => n, _ => return Ok(Value::Number(f64::NAN)) };
            Ok(Value::Number(date_components(ms).2 as f64))
        });
        register_method(self, proto, "getDay", |rt, _args| {
            let this_id = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Number(f64::NAN)) };
            let ms = match rt.object_get(this_id, "__date_ms") { Value::Number(n) => n, _ => return Ok(Value::Number(f64::NAN)) };
            // Jan 1 1970 was a Thursday (day 4).
            let days = (ms / 86_400_000.0).floor() as i64;
            let dow = ((days % 7) + 7 + 4) % 7;
            Ok(Value::Number(dow as f64))
        });
        register_method(self, proto, "getHours", |rt, _args| {
            let this_id = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Number(f64::NAN)) };
            let ms = match rt.object_get(this_id, "__date_ms") { Value::Number(n) => n, _ => return Ok(Value::Number(f64::NAN)) };
            Ok(Value::Number(((ms / 3_600_000.0).floor() as i64 % 24) as f64))
        });
        register_method(self, proto, "getMinutes", |rt, _args| {
            let this_id = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Number(f64::NAN)) };
            let ms = match rt.object_get(this_id, "__date_ms") { Value::Number(n) => n, _ => return Ok(Value::Number(f64::NAN)) };
            Ok(Value::Number(((ms / 60_000.0).floor() as i64 % 60) as f64))
        });
        register_method(self, proto, "getSeconds", |rt, _args| {
            let this_id = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Number(f64::NAN)) };
            let ms = match rt.object_get(this_id, "__date_ms") { Value::Number(n) => n, _ => return Ok(Value::Number(f64::NAN)) };
            Ok(Value::Number(((ms / 1000.0).floor() as i64 % 60) as f64))
        });
        register_method(self, proto, "getMilliseconds", |rt, _args| {
            let this_id = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Number(f64::NAN)) };
            let ms = match rt.object_get(this_id, "__date_ms") { Value::Number(n) => n, _ => return Ok(Value::Number(f64::NAN)) };
            Ok(Value::Number((ms as i64 % 1000) as f64))
        });
        register_method(self, proto, "getTimezoneOffset", |_rt, _args| Ok(Value::Number(0.0)));
        register_method(self, proto, "toISOString", |rt, _args| {
            let this_id = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::String(Rc::new("".into()))) };
            let ms = match rt.object_get(this_id, "__date_ms") { Value::Number(n) => n, _ => return Ok(Value::String(Rc::new("".into()))) };
            let (y, mo, d) = date_components(ms);
            let h = (ms / 3_600_000.0).floor() as i64 % 24;
            let mi = (ms / 60_000.0).floor() as i64 % 60;
            let se = (ms / 1000.0).floor() as i64 % 60;
            let mss = ms as i64 % 1000;
            Ok(Value::String(Rc::new(format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                y, mo + 1, d, h, mi, se, mss))))
        });
        register_method(self, proto, "toJSON", |rt, _args| {
            let this_id = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::String(Rc::new("".into()))) };
            let ms = match rt.object_get(this_id, "__date_ms") { Value::Number(n) => n, _ => return Ok(Value::String(Rc::new("".into()))) };
            let (y, mo, d) = date_components(ms);
            Ok(Value::String(Rc::new(format!("{:04}-{:02}-{:02}T00:00:00.000Z", y, mo + 1, d))))
        });
        register_method(self, proto, "toString", |rt, _args| {
            let this_id = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::String(Rc::new("Invalid Date".into()))) };
            let ms = match rt.object_get(this_id, "__date_ms") { Value::Number(n) => n, _ => return Ok(Value::String(Rc::new("Invalid Date".into()))) };
            let (y, mo, d) = date_components(ms);
            Ok(Value::String(Rc::new(format!("{:04}-{:02}-{:02}T00:00:00Z", y, mo + 1, d))))
        });
        let proto_for_ctor = proto;
        let ctor_obj = make_native("Date", move |rt, args| {
            // Tier-Ω.5.iiiii: Date(y, mo, d, h, m, s, ms) multi-arg ctor
            // must be checked FIRST per ECMA-262 §21.4.2.1 step 2 — when
            // NewTarget supplies ≥ 2 args, treat them as date components.
            // The prior order let Date(2026,4,15) fall through to the
            // single-Number arm and treat 2026 as a unix-ms timestamp.
            // Tier-Ω.5.qqqqq: when single arg is a Date / object, coerce
            // via valueOf per ECMA-262 §21.4.2.1. `new Date(otherDate)`
            // should copy the time, not yield epoch zero.
            let ms = if args.len() == 1 {
                if let Some(Value::Object(id)) = args.first() {
                    let v = rt.object_get(*id, "valueOf");
                    if matches!(v, Value::Object(_)) {
                        let r = rt.call_function(v, Value::Object(*id), Vec::new())?;
                        if let Value::Number(n) = r {
                            let mut o = Object::new_ordinary();
                            o.proto = Some(proto_for_ctor);
                            let new_id = rt.alloc_object(o);
                            rt.object_set(new_id, "__date_ms".into(), Value::Number(n));
                            return Ok(Value::Object(new_id));
                        }
                    }
                }
                match args.first() {
                    Some(Value::Number(n)) => *n,
                    Some(Value::String(s)) => parse_date_string(s.as_str()),
                    _ => 0.0,
                }
            } else if args.len() >= 2 {
                let y = match &args[0] { Value::Number(n) => *n as i64, _ => 0 };
                let mo = match &args[1] { Value::Number(n) => *n as i64, _ => 0 };
                let d = args.get(2).map(|v| match v { Value::Number(n) => *n as i64, _ => 1 }).unwrap_or(1);
                let h = args.get(3).map(|v| match v { Value::Number(n) => *n as i64, _ => 0 }).unwrap_or(0);
                let mi = args.get(4).map(|v| match v { Value::Number(n) => *n as i64, _ => 0 }).unwrap_or(0);
                let se = args.get(5).map(|v| match v { Value::Number(n) => *n as i64, _ => 0 }).unwrap_or(0);
                let mss = args.get(6).map(|v| match v { Value::Number(n) => *n as i64, _ => 0 }).unwrap_or(0);
                (ymd_to_ms(y, mo, d) + h * 3_600_000 + mi * 60_000 + se * 1000 + mss) as f64
            } else {
                use std::time::{SystemTime, UNIX_EPOCH};
                SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as f64).unwrap_or(0.0)
            };
            let mut o = Object::new_ordinary();
            o.proto = Some(proto_for_ctor);
            let id = rt.alloc_object(o);
            rt.object_set(id, "__date_ms".into(), Value::Number(ms));
            Ok(Value::Object(id))
        });
        let ctor = self.alloc_object(ctor_obj);
        register_method(self, ctor, "now", |_rt, _args| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as f64).unwrap_or(0.0);
            Ok(Value::Number(ms))
        });
        register_method(self, ctor, "parse", |_rt, _args| Ok(Value::Number(0.0)));
        register_method(self, ctor, "UTC", |_rt, _args| Ok(Value::Number(0.0)));
        self.object_set(ctor, "prototype".into(), Value::Object(proto));
        self.object_set(proto, "constructor".into(), Value::Object(ctor));
        self.globals.insert("Date".into(), Value::Object(ctor));
    }

    /// Tier-Ω.5.dd: Uint8Array / ArrayBuffer / DataView / Int8Array etc.
    /// All as minimal stub constructors that succeed with `new X(n)` and
    /// expose `.length` / `.byteLength` / `.buffer`. Real binary semantics
    /// deferred to a substrate round.
    fn install_typed_array_stubs(&mut self) {
        // Tier-Ω.5.xxxx: shared TypedArray prototype with subarray / set /
        // slice / fill. tweetnacl, hash libs, and the crypto cluster reach
        // these methods at every step. Prior stub instances had no .subarray
        // so `keyPair()` failed at first byte op.
        let ta_proto = self.alloc_object(Object::new_ordinary());
        register_method(self, ta_proto, "subarray", |rt, args| {
            let this_id = match rt.current_this() {
                Value::Object(o) => o,
                _ => return Err(RuntimeError::TypeError("subarray: this must be a TypedArray".into())),
            };
            let len = match rt.object_get(this_id, "length") {
                Value::Number(n) => n as usize, _ => 0,
            };
            let start = args.first().and_then(|v| if let Value::Number(n) = v { Some(*n as i64) } else { None }).unwrap_or(0);
            let end = args.get(1).and_then(|v| if let Value::Number(n) = v { Some(*n as i64) } else { None }).unwrap_or(len as i64);
            let start = (if start < 0 { (len as i64 + start).max(0) } else { start }).min(len as i64) as usize;
            let end = (if end < 0 { (len as i64 + end).max(0) } else { end }).min(len as i64) as usize;
            let slice_len = end.saturating_sub(start);
            let kind = match rt.object_get(this_id, "__kind") { Value::String(s) => (*s).clone(), _ => "Uint8Array".into() };
            let mut o = Object::new_ordinary();
            o.set_own("length".into(), Value::Number(slice_len as f64));
            o.set_own("__kind".into(), Value::String(Rc::new(kind)));
            let new_id = rt.alloc_object(o);
            for i in 0..slice_len {
                let v = rt.object_get(this_id, &(start + i).to_string());
                rt.object_set(new_id, i.to_string(), v);
            }
            // Inherit prototype from the source so subarray methods chain.
            let src_proto = rt.obj(this_id).proto;
            rt.obj_mut(new_id).proto = src_proto;
            Ok(Value::Object(new_id))
        });
        register_method(self, ta_proto, "set", |rt, args| {
            let this_id = match rt.current_this() {
                Value::Object(o) => o,
                _ => return Err(RuntimeError::TypeError("set: this must be a TypedArray".into())),
            };
            let src = match args.first() {
                Some(Value::Object(id)) => *id,
                _ => return Ok(Value::Undefined),
            };
            let offset = args.get(1).and_then(|v| if let Value::Number(n) = v { Some(*n as usize) } else { None }).unwrap_or(0);
            let src_len = match rt.object_get(src, "length") { Value::Number(n) => n as usize, _ => 0 };
            for i in 0..src_len {
                let v = rt.object_get(src, &i.to_string());
                rt.object_set(this_id, (offset + i).to_string(), v);
            }
            Ok(Value::Undefined)
        });
        register_method(self, ta_proto, "fill", |rt, args| {
            let this_id = match rt.current_this() {
                Value::Object(o) => o,
                _ => return Err(RuntimeError::TypeError("fill: this must be a TypedArray".into())),
            };
            let v = args.first().cloned().unwrap_or(Value::Number(0.0));
            let len = match rt.object_get(this_id, "length") { Value::Number(n) => n as usize, _ => 0 };
            for i in 0..len { rt.object_set(this_id, i.to_string(), v.clone()); }
            Ok(Value::Object(this_id))
        });
        register_method(self, ta_proto, "slice", |rt, args| {
            let this_id = match rt.current_this() {
                Value::Object(o) => o,
                _ => return Err(RuntimeError::TypeError("slice: this must be a TypedArray".into())),
            };
            let len = match rt.object_get(this_id, "length") { Value::Number(n) => n as usize, _ => 0 };
            let start = args.first().and_then(|v| if let Value::Number(n) = v { Some(*n as i64) } else { None }).unwrap_or(0);
            let end = args.get(1).and_then(|v| if let Value::Number(n) = v { Some(*n as i64) } else { None }).unwrap_or(len as i64);
            let start = (if start < 0 { (len as i64 + start).max(0) } else { start }).min(len as i64) as usize;
            let end = (if end < 0 { (len as i64 + end).max(0) } else { end }).min(len as i64) as usize;
            let slice_len = end.saturating_sub(start);
            let mut o = Object::new_ordinary();
            o.set_own("length".into(), Value::Number(slice_len as f64));
            let new_id = rt.alloc_object(o);
            for i in 0..slice_len {
                let v = rt.object_get(this_id, &(start + i).to_string());
                rt.object_set(new_id, i.to_string(), v);
            }
            let src_proto = rt.obj(this_id).proto;
            rt.obj_mut(new_id).proto = src_proto;
            Ok(Value::Object(new_id))
        });

        for name in &[
            "ArrayBuffer", "SharedArrayBuffer", "DataView",
            "Uint8Array", "Uint8ClampedArray", "Int8Array",
            "Uint16Array", "Int16Array", "Uint32Array", "Int32Array",
            "Float32Array", "Float64Array", "BigInt64Array", "BigUint64Array",
        ] {
            let n = (*name).to_string();
            let proto_id = ta_proto;
            let ctor_obj = make_native(name, move |rt, args| {
                let len = match args.first() {
                    Some(Value::Number(n)) => *n,
                    Some(Value::Object(arr)) => {
                        // new Uint8Array(arrayLike) — copy length+contents.
                        match rt.object_get(*arr, "length") {
                            Value::Number(n) => n,
                            _ => 0.0,
                        }
                    }
                    _ => 0.0,
                };
                let mut o = Object::new_ordinary();
                o.set_own("length".into(), Value::Number(len));
                o.set_own("byteLength".into(), Value::Number(len * 4.0));
                o.set_own("__kind".into(), Value::String(Rc::new(n.clone())));
                o.proto = Some(proto_id);
                let id = rt.alloc_object(o);
                // Copy from source if first arg was an object.
                if let Some(Value::Object(src)) = args.first() {
                    let src_len = len as usize;
                    for i in 0..src_len {
                        let v = rt.object_get(*src, &i.to_string());
                        rt.object_set(id, i.to_string(), v);
                    }
                } else {
                    // Zero-initialize for new Uint8Array(N).
                    let cap = (len as usize).min(65536);
                    for i in 0..cap {
                        rt.object_set(id, i.to_string(), Value::Number(0.0));
                    }
                }
                Ok(Value::Object(id))
            });
            let id = self.alloc_object(ctor_obj);
            register_method(self, id, "isView", |_rt, _args| Ok(Value::Boolean(false)));
            let from_proto = ta_proto;
            register_method(self, id, "from", move |rt, args| {
                let src = args.first().cloned().unwrap_or(Value::Undefined);
                let len: usize = match &src {
                    Value::Object(id) => rt.array_length(*id) as usize,
                    Value::String(s) => s.chars().count(),
                    _ => 0,
                };
                let mut o = Object::new_ordinary();
                o.set_own("length".into(), Value::Number(len as f64));
                o.proto = Some(from_proto);
                let new_id = rt.alloc_object(o);
                if let Value::Object(sid) = &src {
                    for i in 0..len {
                        let v = rt.object_get(*sid, &i.to_string());
                        rt.object_set(new_id, i.to_string(), v);
                    }
                }
                Ok(Value::Object(new_id))
            });
            self.object_set(id, "prototype".into(), Value::Object(ta_proto));
            self.globals.insert((*name).to_string(), Value::Object(id));
        }
    }

    /// Tier-Ω.5.dd: WeakRef + FinalizationRegistry minimal stubs. Real
    /// weak-reference semantics need GC integration (deferred). Stubs hold
    /// strong references for v1; `.deref()` always returns the held value.
    fn install_weak_ref_globals(&mut self) {
        let weakref_proto = self.alloc_object(Object::new_ordinary());
        register_method(self, weakref_proto, "deref", |rt, _args| {
            let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
            Ok(rt.object_get(this, "__ref"))
        });
        let proto_for_ctor = weakref_proto;
        let weakref_ctor = make_native("WeakRef", move |rt, args| {
            let target = args.first().cloned().unwrap_or(Value::Undefined);
            let mut o = Object::new_ordinary();
            o.proto = Some(proto_for_ctor);
            let id = rt.alloc_object(o);
            rt.object_set(id, "__ref".into(), target);
            Ok(Value::Object(id))
        });
        let wr = self.alloc_object(weakref_ctor);
        self.object_set(wr, "prototype".into(), Value::Object(weakref_proto));
        self.object_set(weakref_proto, "constructor".into(), Value::Object(wr));
        self.globals.insert("WeakRef".into(), Value::Object(wr));

        let fr_proto = self.alloc_object(Object::new_ordinary());
        register_method(self, fr_proto, "register", |_rt, _args| Ok(Value::Undefined));
        register_method(self, fr_proto, "unregister", |_rt, _args| Ok(Value::Boolean(true)));
        let fr_proto_for_ctor = fr_proto;
        let fr_ctor = make_native("FinalizationRegistry", move |rt, _args| {
            let mut o = Object::new_ordinary();
            o.proto = Some(fr_proto_for_ctor);
            Ok(Value::Object(rt.alloc_object(o)))
        });
        let fr = self.alloc_object(fr_ctor);
        self.object_set(fr, "prototype".into(), Value::Object(fr_proto));
        self.object_set(fr_proto, "constructor".into(), Value::Object(fr));
        self.globals.insert("FinalizationRegistry".into(), Value::Object(fr));
    }

    /// Tier-Ω.5.cc: Reflect global — most methods route to existing Object
    /// statics. has/get/set/deleteProperty/ownKeys/getPrototypeOf used by
    /// many packages doing duck-type checks.
    fn install_reflect(&mut self) {
        let r = self.alloc_object(Object::new_ordinary());
        register_method(self, r, "has", |rt, args| {
            let obj = args.first().cloned().unwrap_or(Value::Undefined);
            let key = args.get(1).cloned().unwrap_or(Value::Undefined);
            let key_s = abstract_ops::to_string(&key).as_str().to_string();
            let id = match obj {
                Value::Object(id) => id,
                _ => return Err(RuntimeError::TypeError("Reflect.has: target must be object".into())),
            };
            let mut cur = Some(id);
            let mut found = false;
            while let Some(c) = cur {
                if rt.obj(c).properties.contains_key(&key_s) { found = true; break; }
                cur = rt.obj(c).proto;
            }
            Ok(Value::Boolean(found))
        });
        register_method(self, r, "get", |rt, args| {
            let obj = args.first().cloned().unwrap_or(Value::Undefined);
            let key = args.get(1).cloned().unwrap_or(Value::Undefined);
            let key_s = abstract_ops::to_string(&key).as_str().to_string();
            match obj {
                Value::Object(id) => Ok(rt.object_get(id, &key_s)),
                _ => Err(RuntimeError::TypeError("Reflect.get: target must be object".into())),
            }
        });
        register_method(self, r, "set", |rt, args| {
            let obj = args.first().cloned().unwrap_or(Value::Undefined);
            let key = args.get(1).cloned().unwrap_or(Value::Undefined);
            let val = args.get(2).cloned().unwrap_or(Value::Undefined);
            let key_s = abstract_ops::to_string(&key).as_str().to_string();
            match obj {
                Value::Object(id) => { rt.object_set(id, key_s, val); Ok(Value::Boolean(true)) }
                _ => Err(RuntimeError::TypeError("Reflect.set: target must be object".into())),
            }
        });
        register_method(self, r, "deleteProperty", |rt, args| {
            let obj = args.first().cloned().unwrap_or(Value::Undefined);
            let key = args.get(1).cloned().unwrap_or(Value::Undefined);
            let key_s = abstract_ops::to_string(&key).as_str().to_string();
            match obj {
                Value::Object(id) => {
                    rt.obj_mut(id).properties.remove(&key_s);
                    Ok(Value::Boolean(true))
                }
                _ => Err(RuntimeError::TypeError("Reflect.deleteProperty: target must be object".into())),
            }
        });
        register_method(self, r, "ownKeys", |rt, args| {
            let obj = args.first().cloned().unwrap_or(Value::Undefined);
            match obj {
                Value::Object(id) => {
                    let keys: Vec<String> = rt.obj(id).properties.keys().cloned().collect();
                    let arr = rt.alloc_object(Object::new_array());
                    for (i, k) in keys.iter().enumerate() {
                        rt.object_set(arr, i.to_string(), Value::String(Rc::new(k.clone())));
                    }
                    rt.object_set(arr, "length".into(), Value::Number(keys.len() as f64));
                    Ok(Value::Object(arr))
                }
                _ => Err(RuntimeError::TypeError("Reflect.ownKeys: target must be object".into())),
            }
        });
        register_method(self, r, "getPrototypeOf", |rt, args| {
            let obj = args.first().cloned().unwrap_or(Value::Undefined);
            match obj {
                Value::Object(id) => match rt.obj(id).proto {
                    Some(p) => Ok(Value::Object(p)),
                    None => Ok(Value::Null),
                },
                _ => Ok(Value::Null),
            }
        });
        // defineProperty / construct / apply — alias existing logic.
        if let Some(v) = self.globals.get("Object").cloned() {
            if let Value::Object(oid) = v {
                let dp = self.object_get(oid, "defineProperty");
                if !matches!(dp, Value::Undefined) { self.object_set(r, "defineProperty".into(), dp); }
            }
        }
        self.globals.insert("Reflect".into(), Value::Object(r));
    }

    /// Tier-Ω.5.z: Error + TypeError + RangeError + SyntaxError + ReferenceError
    /// + URIError + EvalError constructors. Each is callable; carrying a
    /// .prototype so `class X extends Error {}` works (the dense pattern
    /// in real packages: ulid, joi, commander, luxon all use it).
    /// The Error.prototype object exposes .name and .message so duck-type
    /// checks pass; instance shape is `{name, message, stack:""}`.
    fn install_error_globals(&mut self) {
        for (name, default_name) in &[
            ("Error", "Error"),
            ("TypeError", "TypeError"),
            ("RangeError", "RangeError"),
            ("SyntaxError", "SyntaxError"),
            ("ReferenceError", "ReferenceError"),
            ("URIError", "URIError"),
            ("EvalError", "EvalError"),
            ("AggregateError", "AggregateError"),
        ] {
            let proto_id = self.alloc_object(Object::new_ordinary());
            self.object_set(proto_id, "name".into(), Value::String(Rc::new((*default_name).to_string())));
            self.object_set(proto_id, "message".into(), Value::String(Rc::new("".to_string())));
            register_method(self, proto_id, "toString", |rt, _args| {
                let this = rt.current_this();
                let (name, message) = match &this {
                    Value::Object(id) => {
                        let n = rt.object_get(*id, "name");
                        let m = rt.object_get(*id, "message");
                        (abstract_ops::to_string(&n).as_str().to_string(),
                         abstract_ops::to_string(&m).as_str().to_string())
                    }
                    _ => ("Error".into(), "".into()),
                };
                let out = if message.is_empty() { name } else { format!("{}: {}", name, message) };
                Ok(Value::String(Rc::new(out)))
            });

            let default_name = (*default_name).to_string();
            let proto_for_ctor = proto_id;
            let ctor_obj = make_native(name, move |rt, args| {
                // Tier-Ω.5.ffff: when invoked via super(...) from a
                // derived class, the receiver is the already-allocated
                // derived-instance. Mutate it in place rather than
                // allocating a fresh one — otherwise `class E extends
                // Error { constructor(m) { super(m); } }; new E('hi')`
                // produces an E with empty .message because the Error
                // native allocates a sibling Object and discards it
                // (Op::CallMethod takes call_function's return Object
                // as the result, overwriting the synthesized this).
                let receiver_id = match rt.current_this() {
                    Value::Object(id) => {
                        // Use receiver iff it's an ordinary (not
                        // already an Error-shaped) object. The derived
                        // class's Op::New synthesized this with proto
                        // wired to the derived ctor's prototype, which
                        // already inherits from Error.prototype.
                        Some(id)
                    }
                    _ => None,
                };
                let id = match receiver_id {
                    Some(id) => id,
                    None => {
                        let mut o = Object::new_ordinary();
                        o.proto = Some(proto_for_ctor);
                        rt.alloc_object(o)
                    }
                };
                if let Some(msg) = args.first() {
                    let m = abstract_ops::to_string(msg).as_str().to_string();
                    rt.object_set(id, "message".into(), Value::String(Rc::new(m)));
                }
                rt.object_set(id, "name".into(), Value::String(Rc::new(default_name.clone())));
                rt.object_set(id, "stack".into(), Value::String(Rc::new("".into())));
                Ok(Value::Object(id))
            });
            let ctor_id = self.alloc_object(ctor_obj);
            self.object_set(ctor_id, "prototype".into(), Value::Object(proto_id));
            // proto.constructor = ctor (per spec).
            self.object_set(proto_id, "constructor".into(), Value::Object(ctor_id));
            self.globals.insert((*name).to_string(), Value::Object(ctor_id));
        }
    }

    fn install_symbol_static(&mut self) {
        // Tier-Ω.5.w: Symbol is now callable as `Symbol(desc?)`. Returns a
        // fresh Value::String of the form "@@sym:<counter>:<desc>" — the
        // counter is appended via a thread_local AtomicUsize so two calls
        // with the same description produce distinct strings (sufficient
        // for the spec's identity-distinct expectation under v1's
        // string-shaped Symbol representation).
        let sym_obj = make_native("Symbol", |_rt, args| {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let desc = args.first()
                .map(|v| crate::abstract_ops::to_string(v).as_str().to_string())
                .unwrap_or_default();
            Ok(Value::String(Rc::new(format!("@@sym:{}:{}", n, desc))))
        });
        let sym = self.alloc_object(sym_obj);
        // Well-known Symbol.iterator is, in v1, the string key "@@iterator".
        self.object_set(sym, "iterator".into(), Value::String(Rc::new("@@iterator".into())));
        self.object_set(sym, "asyncIterator".into(), Value::String(Rc::new("@@asyncIterator".into())));
        self.object_set(sym, "hasInstance".into(), Value::String(Rc::new("@@hasInstance".into())));
        self.object_set(sym, "toPrimitive".into(), Value::String(Rc::new("@@toPrimitive".into())));
        register_method(self, sym, "for", |_rt, args| {
            let s = args.first().map(|v| crate::abstract_ops::to_string(v).as_str().to_string()).unwrap_or_default();
            Ok(Value::String(Rc::new(format!("@@sym:{}", s))))
        });
        register_method(self, sym, "keyFor", |_rt, args| {
            // v1: returns the description portion of a Symbol.for()-produced
            // string, undefined otherwise. Matches Ω.5.c's string-shaped
            // Symbol representation.
            let s = args.first().and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None });
            match s {
                Some(s) if s.starts_with("@@sym:") && !s.contains(':') => Ok(Value::Undefined),
                Some(s) => {
                    let body = s.strip_prefix("@@sym:").unwrap_or(&s);
                    Ok(Value::String(Rc::new(body.split_once(':').map(|(_, d)| d.to_string()).unwrap_or_else(|| body.to_string()))))
                }
                _ => Ok(Value::Undefined),
            }
        });
        // Tier-Ω.5.wwww: Symbol.prototype with a toString that returns the
        // description. yup captures Symbol.prototype.toString at module init.
        let sym_proto = self.alloc_object(Object::new_ordinary());
        register_method(self, sym_proto, "toString", |rt, _args| {
            match rt.current_this() {
                Value::String(s) if s.starts_with("@@sym:") => {
                    let body = s.strip_prefix("@@sym:").unwrap_or(&s);
                    let desc = body.split_once(':').map(|(_, d)| d).unwrap_or(body);
                    Ok(Value::String(Rc::new(format!("Symbol({})", desc))))
                }
                v => Ok(Value::String(Rc::new(crate::abstract_ops::to_string(&v).as_str().to_string()))),
            }
        });
        self.object_set(sym, "prototype".into(), Value::Object(sym_proto));
        self.globals.insert("Symbol".into(), Value::Object(sym));
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

/// Drain an iterable's @@iterator into a Vec<Value>. Used by
/// Object.fromEntries / Array.from.
/// Tier-Ω.5.rrr: build a values-iterator for a Set. The iterator object
/// snapshots the Set's current values into a private array and exposes a
/// next() that yields each in turn. Sufficient for `[...new Set(arr)]`
/// spread.
pub(crate) fn make_set_values_iterator(rt: &mut Runtime, set_id: crate::value::ObjectRef) -> Result<Value, RuntimeError> {
    let values: Vec<Value> = match rt.object_get(set_id, "__set_data") {
        Value::Object(storage) => {
            rt.obj(storage).properties.values().map(|d| d.value.clone()).collect()
        }
        _ => Vec::new(),
    };
    // Build an iterator object: { __idx: 0, __vals: [v0,v1,...], next() }
    let iter = rt.alloc_object(Object::new_ordinary());
    let vals_arr = rt.alloc_object(Object::new_array());
    for (i, v) in values.iter().enumerate() {
        rt.object_set(vals_arr, i.to_string(), v.clone());
    }
    rt.object_set(vals_arr, "length".into(), Value::Number(values.len() as f64));
    rt.object_set(iter, "__vals".into(), Value::Object(vals_arr));
    rt.object_set(iter, "__idx".into(), Value::Number(0.0));
    register_method(rt, iter, "next", |rt, _args| {
        let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::Undefined) };
        let idx = match rt.object_get(this, "__idx") {
            Value::Number(n) => n as usize,
            _ => 0,
        };
        let vals = match rt.object_get(this, "__vals") {
            Value::Object(id) => id,
            _ => return Ok(Value::Undefined),
        };
        let len = rt.array_length(vals);
        let result = rt.alloc_object(Object::new_ordinary());
        if idx >= len {
            rt.object_set(result, "done".into(), Value::Boolean(true));
            rt.object_set(result, "value".into(), Value::Undefined);
        } else {
            let v = rt.object_get(vals, &idx.to_string());
            rt.object_set(result, "done".into(), Value::Boolean(false));
            rt.object_set(result, "value".into(), v);
            rt.object_set(this, "__idx".into(), Value::Number((idx + 1) as f64));
        }
        Ok(Value::Object(result))
    });
    Ok(Value::Object(iter))
}

pub(crate) fn collect_iterable(rt: &mut Runtime, src: Value) -> Result<Vec<Value>, RuntimeError> {
    let id = match src {
        Value::Object(id) => id,
        _ => return Ok(Vec::new()),
    };
    let method = rt.object_get(id, "@@iterator");
    let iter = rt.call_function(method, Value::Object(id), Vec::new())?;
    let iter_id = match iter {
        Value::Object(id) => id,
        _ => return Err(RuntimeError::TypeError("iterator is not an object".into())),
    };
    let next = rt.object_get(iter_id, "next");
    let mut out = Vec::new();
    loop {
        let result = rt.call_function(next.clone(), Value::Object(iter_id), Vec::new())?;
        let rid = match result {
            Value::Object(id) => id,
            _ => return Err(RuntimeError::TypeError("iterator next did not return an object".into())),
        };
        let done = abstract_ops::to_boolean(&rt.object_get(rid, "done"));
        if done { break; }
        out.push(rt.object_get(rid, "value"));
    }
    Ok(out)
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

pub fn json_parse(rt: &mut Runtime, s: &str) -> Result<Value, RuntimeError> {
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

// Tier-Ω.5.eee: minimal base64 codec for atob/btoa. Standard alphabet,
// padding required on decode (entities-generated data is well-formed).
const B64_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= input.len() {
        let n = ((input[i] as u32) << 16) | ((input[i+1] as u32) << 8) | (input[i+2] as u32);
        out.push(B64_ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[(n & 0x3F) as usize] as char);
        i += 3;
    }
    let rem = input.len() - i;
    if rem == 1 {
        let n = (input[i] as u32) << 16;
        out.push(B64_ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let n = ((input[i] as u32) << 16) | ((input[i+1] as u32) << 8);
        out.push(B64_ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        out.push('=');
    }
    out
}
fn base64_decode(s: &str) -> Result<Vec<u8>, &'static str> {
    let mut lut = [255u8; 256];
    for (i, &c) in B64_ALPHABET.iter().enumerate() { lut[c as usize] = i as u8; }
    let bytes: Vec<u8> = s.bytes().filter(|&b| b != b'=').collect();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    let mut i = 0;
    while i + 4 <= bytes.len() {
        let (a, b, c, d) = (lut[bytes[i] as usize], lut[bytes[i+1] as usize], lut[bytes[i+2] as usize], lut[bytes[i+3] as usize]);
        if (a | b | c | d) == 255 { return Err("invalid base64 character"); }
        let n = ((a as u32) << 18) | ((b as u32) << 12) | ((c as u32) << 6) | (d as u32);
        out.push(((n >> 16) & 0xFF) as u8);
        out.push(((n >> 8) & 0xFF) as u8);
        out.push((n & 0xFF) as u8);
        i += 4;
    }
    let rem = bytes.len() - i;
    if rem == 2 {
        let (a, b) = (lut[bytes[i] as usize], lut[bytes[i+1] as usize]);
        if (a | b) == 255 { return Err("invalid base64 character"); }
        let n = ((a as u32) << 18) | ((b as u32) << 12);
        out.push(((n >> 16) & 0xFF) as u8);
    } else if rem == 3 {
        let (a, b, c) = (lut[bytes[i] as usize], lut[bytes[i+1] as usize], lut[bytes[i+2] as usize]);
        if (a | b | c) == 255 { return Err("invalid base64 character"); }
        let n = ((a as u32) << 18) | ((b as u32) << 12) | ((c as u32) << 6);
        out.push(((n >> 16) & 0xFF) as u8);
        out.push(((n >> 8) & 0xFF) as u8);
    } else if rem == 1 {
        return Err("invalid base64 length");
    }
    Ok(out)
}

// Tier-Ω.5.aaaa: Gregorian date arithmetic helpers for Date intrinsics.
//
// All functions operate on milliseconds since Unix epoch (UTC, no
// timezone). Sufficient for moment / dayjs / date-fns module-load and
// basic API exercise; not full IANA-timezone-aware.

/// Compute (year, month-0-based, day-1-based) from epoch-ms.
fn date_components(ms: f64) -> (i64, i64, i64) {
    let days = (ms / 86_400_000.0).floor() as i64;
    // Days since 1970-01-01.
    // Convert to year, month, day via Gregorian algorithm.
    let mut z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe/1460 + doe/36524 - doe/146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe/4 - yoe/100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    z = m - 1; // month 0-based
    let _ = z;
    (year, m - 1, d)
}

/// Build epoch-ms from (year, month-0-based, day-1-based).
fn ymd_to_ms(year: i64, month: i64, day: i64) -> i64 {
    let y = if month < 2 { year - 1 } else { year };
    let m = if month < 2 { (month + 9) as i64 } else { (month - 2) as i64 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days_since_epoch = era * 146097 + doe - 719468;
    days_since_epoch * 86_400_000
}

/// Parse a Date string. Supports:
/// - "YYYY-MM-DD"
/// - "YYYY-MM-DDTHH:MM:SS"
/// - "YYYY-MM-DDTHH:MM:SS.sssZ"
/// Returns f64 ms-since-epoch, or NaN on parse failure.
fn parse_date_string(s: &str) -> f64 {
    let s = s.trim();
    if s.len() < 10 { return f64::NAN; }
    let y: i64 = match s[0..4].parse() { Ok(v) => v, Err(_) => return f64::NAN };
    if s.as_bytes()[4] != b'-' { return f64::NAN; }
    let mo: i64 = match s[5..7].parse() { Ok(v) => v, Err(_) => return f64::NAN };
    if s.as_bytes()[7] != b'-' { return f64::NAN; }
    let d: i64 = match s[8..10].parse() { Ok(v) => v, Err(_) => return f64::NAN };
    let mut ms = ymd_to_ms(y, mo - 1, d);
    if s.len() >= 19 && s.as_bytes()[10] == b'T' {
        let h: i64 = s[11..13].parse().unwrap_or(0);
        let mi: i64 = s[14..16].parse().unwrap_or(0);
        let se: i64 = s[17..19].parse().unwrap_or(0);
        ms += h * 3_600_000 + mi * 60_000 + se * 1000;
        if s.len() >= 23 && s.as_bytes()[19] == b'.' {
            let mss: i64 = s[20..23].parse().unwrap_or(0);
            ms += mss;
        }
    }
    ms as f64
}
