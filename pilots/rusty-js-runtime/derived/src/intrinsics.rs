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
        self.globals.insert("globalThis".into(), Value::Object(gt));
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
                let entries: Vec<(String, Value)> = rt.obj(*sid).properties.iter()
                    .filter(|(_, d)| d.enumerable)
                    .map(|(k, d)| (k.clone(), d.value.clone()))
                    .collect();
                for (k, v) in entries { rt.object_set(target, k, v); }
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
        register_global_fn(self, "Function", |_rt, _args| {
            // Throw a JS-catchable TypeError-shaped value so user try/catch
            // observes it. Until Error intrinsics land, a string suffices.
            Err(RuntimeError::Thrown(Value::String(Rc::new(
                "TypeError: Function constructor not yet supported in v1".into()))))
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
            let kvs: Vec<(String, Value)> = {
                let o = rt.obj(id);
                let is_array = matches!(o.internal_kind, InternalKind::Array);
                let mut entries: Vec<(String, Value)> = o.properties.iter()
                    .filter(|(k, d)| d.enumerable && !(is_array && *k == "length"))
                    .map(|(k, d)| (k.clone(), d.value.clone()))
                    .collect();
                if is_array {
                    entries.sort_by_key(|(k, _)| k.parse::<u64>().unwrap_or(u64::MAX));
                }
                entries
            };
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
            let kvs: Vec<(String, Value)> = {
                let o = rt.obj(id);
                let is_array = matches!(o.internal_kind, InternalKind::Array);
                let mut entries: Vec<(String, Value)> = o.properties.iter()
                    .filter(|(k, d)| d.enumerable && !(is_array && *k == "length"))
                    .map(|(k, d)| (k.clone(), d.value.clone()))
                    .collect();
                if is_array {
                    entries.sort_by_key(|(k, _)| k.parse::<u64>().unwrap_or(u64::MAX));
                }
                entries
            };
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
                _ => return Err(RuntimeError::TypeError("Object.assign: target must be an object".into())),
            };
            for src in args.iter().skip(1) {
                if let Value::Object(sid) = src {
                    let entries: Vec<(String, Value)> = rt.obj(*sid).properties.iter()
                        .filter(|(_, d)| d.enumerable)
                        .map(|(k, d)| (k.clone(), d.value.clone()))
                        .collect();
                    for (k, v) in entries { rt.object_set(target, k, v); }
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
            let value = rt.object_get(desc_id, "value");
            rt.object_set(target, key, value);
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
                    let value = rt.object_get(did, "value");
                    rt.object_set(target, k, value);
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
        }
        self.globals.insert("Object".into(), Value::Object(obj_ctor));
    }

    fn install_array_static(&mut self) {
        let arr_ctor = self.alloc_object(Object::new_ordinary());
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
            let proto_for_ctor = proto;
            let name = (*collection).to_string();
            let ctor_obj = make_native(&name, move |rt, _args| {
                let mut o = Object::new_ordinary();
                o.proto = Some(proto_for_ctor);
                let id = rt.alloc_object(o);
                let storage = rt.alloc_object(Object::new_ordinary());
                rt.object_set(id, "__set_data".into(), Value::Object(storage));
                rt.object_set(id, "size".into(), Value::Number(0.0));
                Ok(Value::Object(id))
            });
            let ctor = self.alloc_object(ctor_obj);
            self.object_set(ctor, "prototype".into(), Value::Object(proto));
            self.object_set(proto, "constructor".into(), Value::Object(ctor));
            self.globals.insert((*collection).to_string(), Value::Object(ctor));
        }
    }

    /// Tier-Ω.5.dd: Date global. Real Date.now() + minimal instance shape.
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
        register_method(self, proto, "toISOString", |rt, _args| {
            let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::String(Rc::new("".into()))) };
            let _ms = rt.object_get(this, "__date_ms");
            Ok(Value::String(Rc::new("1970-01-01T00:00:00.000Z".into())))
        });
        register_method(self, proto, "toString", |rt, _args| {
            let this = match rt.current_this() { Value::Object(id) => id, _ => return Ok(Value::String(Rc::new("Invalid Date".into()))) };
            let _ms = rt.object_get(this, "__date_ms");
            Ok(Value::String(Rc::new("Thu Jan 01 1970 00:00:00 GMT+0000".into())))
        });
        let proto_for_ctor = proto;
        let ctor_obj = make_native("Date", move |rt, args| {
            let ms = match args.first() {
                Some(Value::Number(n)) => *n,
                _ => {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as f64).unwrap_or(0.0)
                }
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
        for name in &[
            "ArrayBuffer", "SharedArrayBuffer", "DataView",
            "Uint8Array", "Uint8ClampedArray", "Int8Array",
            "Uint16Array", "Int16Array", "Uint32Array", "Int32Array",
            "Float32Array", "Float64Array", "BigInt64Array", "BigUint64Array",
        ] {
            let n = (*name).to_string();
            let ctor_obj = make_native(name, move |rt, args| {
                let len = match args.first() {
                    Some(Value::Number(n)) => *n,
                    _ => 0.0,
                };
                let mut o = Object::new_ordinary();
                o.set_own("length".into(), Value::Number(len));
                o.set_own("byteLength".into(), Value::Number(len * 4.0));
                o.set_own("__kind".into(), Value::String(Rc::new(n.clone())));
                Ok(Value::Object(rt.alloc_object(o)))
            });
            let id = self.alloc_object(ctor_obj);
            register_method(self, id, "isView", |_rt, _args| Ok(Value::Boolean(false)));
            register_method(self, id, "from", |rt, args| {
                let src = args.first().cloned().unwrap_or(Value::Undefined);
                let len: usize = match &src {
                    Value::Object(id) => rt.array_length(*id) as usize,
                    Value::String(s) => s.chars().count(),
                    _ => 0,
                };
                let mut o = Object::new_ordinary();
                o.set_own("length".into(), Value::Number(len as f64));
                Ok(Value::Object(rt.alloc_object(o)))
            });
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
                // Allocate a fresh Error instance with proto = Error.prototype.
                let mut o = Object::new_ordinary();
                o.proto = Some(proto_for_ctor);
                let id = rt.alloc_object(o);
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
