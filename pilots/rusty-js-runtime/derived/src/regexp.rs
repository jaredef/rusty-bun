//! Tier-Ω.5.i — RegExp object, %RegExp.prototype%, the `__createRegExp`
//! lowering helper, the `RegExp` constructor, and regex-aware String
//! prototype extensions (.match / .search / .replace / .replaceAll /
//! .split with a RegExp argument).
//!
//! Pattern translation strategy: JS regex syntax overlaps the Rust `regex`
//! crate's syntax for the common case. Flag handling differs — JS uses a
//! trailing flag string, Rust uses an inline `(?flags)` prefix. We
//! translate `i`, `m`, `s` directly; `g` (global) and `y` (sticky) are
//! consumed at the API level (stateful exec); `u` (unicode) and `d`
//! (indices) are accepted as no-ops; anything else is an error.
//!
//! Compilation failures (lookbehind, backreferences, named groups in
//! incompatible syntax, etc.) are non-fatal: the RegExp object is still
//! constructed with `compiled = None`. Calling .test / .exec / etc. then
//! throws a TypeError. This is a v1 deviation from spec, documented in
//! the round's trajectory row.

use crate::abstract_ops;
use crate::interp::{Runtime, RuntimeError};
use crate::intrinsics::make_native;
use crate::value::{
    CompiledRegex, InternalKind, Object, ObjectRef, PropertyDescriptor, RegExpInternals, Value,
};
use std::collections::HashMap;
use std::rc::Rc;

impl Runtime {
    /// Install %RegExp.prototype%, the `RegExp` constructor, the
    /// `__createRegExp` helper that the compiler lowers literals into,
    /// and the regex-aware String.prototype extensions. Called from
    /// install_intrinsics.
    pub fn install_regexp(&mut self) {
        // Allocate the prototype first so subsequent alloc_object calls
        // for RegExp instances auto-wire through the InternalKind seam.
        let proto = self.alloc_object(Object::new_ordinary());
        self.regexp_prototype = Some(proto);
        install_regexp_proto(self, proto);

        // Hidden global helper the compiler emits LoadGlobal+Call into.
        register_global_native(self, "__createRegExp", |rt, args| {
            let pattern = abstract_ops::to_string(
                &args.first().cloned().unwrap_or(Value::Undefined)
            ).as_str().to_string();
            let flags = abstract_ops::to_string(
                &args.get(1).cloned().unwrap_or(Value::Undefined)
            ).as_str().to_string();
            Ok(Value::Object(new_regexp(rt, &pattern, &flags)?))
        });

        // RegExp constructor — `new RegExp(p, f)` and `RegExp(p, f)` are
        // both routed through this. If `p` is itself a RegExp the spec
        // says to return a fresh copy; v1 just rebuilds from its source.
        register_global_native(self, "RegExp", |rt, args| {
            let (pattern, flags) = match args.first() {
                Some(Value::Object(id)) => {
                    if let InternalKind::RegExp(re) = &rt.obj(*id).internal_kind {
                        let src = (*re.source).clone();
                        let f = match args.get(1) {
                            Some(Value::Undefined) | None => (*re.flags).clone(),
                            Some(v) => abstract_ops::to_string(v).as_str().to_string(),
                        };
                        (src, f)
                    } else {
                        let p = abstract_ops::to_string(&args[0]).as_str().to_string();
                        let f = abstract_ops::to_string(
                            &args.get(1).cloned().unwrap_or(Value::Undefined)
                        ).as_str().to_string();
                        (p, f)
                    }
                }
                Some(v) => {
                    let p = abstract_ops::to_string(v).as_str().to_string();
                    let f = abstract_ops::to_string(
                        &args.get(1).cloned().unwrap_or(Value::Undefined)
                    ).as_str().to_string();
                    (p, f)
                }
                None => (String::new(), String::new()),
            };
            Ok(Value::Object(new_regexp(rt, &pattern, &flags)?))
        });

        install_string_regex_methods(self);
    }
}

/// Allocate a RegExp instance and populate its accessor own-properties.
pub fn new_regexp(rt: &mut Runtime, pattern: &str, flags: &str) -> Result<ObjectRef, RuntimeError> {
    let compiled = compile_either(pattern, flags);
    let internals = RegExpInternals {
        source: Rc::new(pattern.to_string()),
        flags: Rc::new(flags.to_string()),
        compiled,
        last_index: 0,
    };
    let obj = Object {
        proto: None,
        extensible: true,
        properties: HashMap::new(),
        internal_kind: InternalKind::RegExp(internals),
    };
    let id = rt.alloc_object(obj);
    // Plain own-properties for the accessor surface — v1 stand-in for
    // real getter/setter accessor descriptors (deferred).
    rt.object_set(id, "source".into(), Value::String(Rc::new(pattern.to_string())));
    rt.object_set(id, "flags".into(), Value::String(Rc::new(flags.to_string())));
    rt.object_set(id, "global".into(),     Value::Boolean(flags.contains('g')));
    rt.object_set(id, "ignoreCase".into(), Value::Boolean(flags.contains('i')));
    rt.object_set(id, "multiline".into(),  Value::Boolean(flags.contains('m')));
    rt.object_set(id, "sticky".into(),     Value::Boolean(flags.contains('y')));
    rt.object_set(id, "unicode".into(),    Value::Boolean(flags.contains('u')));
    rt.object_set(id, "dotAll".into(),     Value::Boolean(flags.contains('s')));
    rt.object_set(id, "hasIndices".into(), Value::Boolean(flags.contains('d')));
    rt.object_set(id, "lastIndex".into(),  Value::Number(0.0));
    Ok(id)
}

/// Translate `pattern` + JS `flags` into a Rust `regex::Regex`. Returns
/// Err if the pattern uses features the Rust `regex` crate doesn't
/// support (lookbehind, backreferences) or if a flag is unsupported.
fn translate(pattern: &str, flags: &str) -> Result<regex::Regex, String> {
    let mut flag_set = String::new();
    for c in flags.chars() {
        match c {
            'i' => flag_set.push('i'),
            'm' => flag_set.push('m'),
            's' => flag_set.push('s'),
            // g (global) and y (sticky) are handled at the API level —
            // they govern stateful exec/replace, not the regex compile.
            // u (unicode) is the default for the Rust crate; d (indices)
            // is a no-op in v1.
            'g' | 'y' | 'u' | 'd' => {}
            _ => return Err(format!("unsupported regex flag '{}'", c)),
        }
    }
    let prefixed = if flag_set.is_empty() {
        pattern.to_string()
    } else {
        format!("(?{}){}", flag_set, pattern)
    };
    regex::Regex::new(&prefixed).map_err(|e| format!("{}", e))
}

/// Tier-Ω.5.ggg: dual-engine compile. Try the Rust `regex` crate first
/// (fast for the patterns it supports). On rejection, fall back to the
/// hand-rolled backtracking engine which supports lookaround.
pub fn compile_either(pattern: &str, flags: &str) -> Option<CompiledRegex> {
    if let Ok(r) = translate(pattern, flags) {
        return Some(CompiledRegex::Rust(r));
    }
    if let Ok(h) = crate::regex_hand::compile(pattern, flags) {
        return Some(CompiledRegex::Hand(h));
    }
    None
}

// ──────────────── %RegExp.prototype% ────────────────

fn install_regexp_proto(rt: &mut Runtime, host: ObjectRef) {
    register_method(rt, host, "test", |rt, args| {
        let this_id = current_regexp_this(rt, "RegExp.prototype.test")?;
        let input = abstract_ops::to_string(&args.first().cloned().unwrap_or(Value::Undefined))
            .as_str().to_string();
        let result = {
            let re = match &rt.obj(this_id).internal_kind {
                InternalKind::RegExp(r) => r,
                _ => unreachable!(),
            };
            match &re.compiled {
                Some(rx) => Ok(rx.is_match(&input)),
                None => Err(RuntimeError::TypeError(format!(
                    "RegExp pattern uses features unsupported by the v1 regex engine: /{}/{}",
                    re.source, re.flags))),
            }
        }?;
        Ok(Value::Boolean(result))
    });

    register_method(rt, host, "exec", |rt, args| {
        let this_id = current_regexp_this(rt, "RegExp.prototype.exec")?;
        let input = abstract_ops::to_string(&args.first().cloned().unwrap_or(Value::Undefined))
            .as_str().to_string();
        regexp_exec(rt, this_id, &input)
    });

    register_method(rt, host, "toString", |rt, _args| {
        let this_id = current_regexp_this(rt, "RegExp.prototype.toString")?;
        let s = match &rt.obj(this_id).internal_kind {
            InternalKind::RegExp(r) => format!("/{}/{}", r.source, r.flags),
            _ => unreachable!(),
        };
        Ok(Value::String(Rc::new(s)))
    });
}

/// Per §22.2.5.2 RegExpBuiltinExec. v1 surface: returns null on no match,
/// else an Array with [match, ...groups] plus .index / .input properties.
/// Honors the 'g' flag via lastIndex.
pub fn regexp_exec(rt: &mut Runtime, this_id: ObjectRef, input: &str) -> Result<Value, RuntimeError> {
    let (is_global, start, has_compiled) = {
        let o = rt.obj(this_id);
        let re = match &o.internal_kind {
            InternalKind::RegExp(r) => r,
            _ => return Err(RuntimeError::TypeError("RegExp.prototype.exec: this is not a RegExp".into())),
        };
        let is_global = re.flags.contains('g') || re.flags.contains('y');
        let start = if is_global { re.last_index } else { 0 };
        (is_global, start, re.compiled.is_some())
    };
    if !has_compiled {
        let (src, flags) = match &rt.obj(this_id).internal_kind {
            InternalKind::RegExp(r) => ((*r.source).clone(), (*r.flags).clone()),
            _ => unreachable!(),
        };
        return Err(RuntimeError::TypeError(format!(
            "RegExp pattern uses features unsupported by the v1 regex engine: /{}/{}",
            src, flags)));
    }
    if start > input.len() {
        if is_global {
            if let InternalKind::RegExp(r) = &mut rt.obj_mut(this_id).internal_kind {
                r.last_index = 0;
            }
            rt.object_set(this_id, "lastIndex".into(), Value::Number(0.0));
        }
        return Ok(Value::Null);
    }

    // Snapshot of the captures we need. We borrow the regex immutably,
    // collect everything into owned strings, then release.
    let captures_opt: Option<(usize, usize, Vec<Option<String>>)> = {
        let re = match &rt.obj(this_id).internal_kind {
            InternalKind::RegExp(r) => r,
            _ => unreachable!(),
        };
        let rx = re.compiled.as_ref().unwrap();
        rx.captures_at(input, start)
    };

    match captures_opt {
        None => {
            if is_global {
                if let InternalKind::RegExp(r) = &mut rt.obj_mut(this_id).internal_kind {
                    r.last_index = 0;
                }
                rt.object_set(this_id, "lastIndex".into(), Value::Number(0.0));
            }
            Ok(Value::Null)
        }
        Some((mstart, mend, groups)) => {
            if is_global {
                if let InternalKind::RegExp(r) = &mut rt.obj_mut(this_id).internal_kind {
                    r.last_index = mend;
                }
                rt.object_set(this_id, "lastIndex".into(), Value::Number(mend as f64));
            }
            let arr = rt.alloc_object(Object::new_array());
            for (i, g) in groups.iter().enumerate() {
                let v = match g {
                    Some(s) => Value::String(Rc::new(s.clone())),
                    None => Value::Undefined,
                };
                rt.object_set(arr, i.to_string(), v);
            }
            rt.object_set(arr, "length".into(), Value::Number(groups.len() as f64));
            rt.object_set(arr, "index".into(), Value::Number(byte_to_char_index(input, mstart) as f64));
            rt.object_set(arr, "input".into(), Value::String(Rc::new(input.to_string())));
            Ok(Value::Object(arr))
        }
    }
}

fn byte_to_char_index(s: &str, byte_off: usize) -> usize {
    s[..byte_off.min(s.len())].chars().count()
}

fn current_regexp_this(rt: &Runtime, label: &str) -> Result<ObjectRef, RuntimeError> {
    match rt.current_this() {
        Value::Object(id) if matches!(rt.obj(id).internal_kind, InternalKind::RegExp(_)) => Ok(id),
        _ => Err(RuntimeError::TypeError(format!("{}: this is not a RegExp", label))),
    }
}

// ──────────────── String.prototype regex-aware methods ────────────────
//
// These mount onto the existing string_prototype object after install_prototypes
// runs. They shadow/replace the existing .replace and .split, which handle
// only string arguments today.

fn install_string_regex_methods(rt: &mut Runtime) {
    let host = match rt.string_prototype {
        Some(id) => id,
        None => return, // install_prototypes hasn't run — defensive.
    };

    register_method(rt, host, "match", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let re_id = coerce_regexp(rt, args.first().cloned().unwrap_or(Value::Undefined))?;
        // If the regex is global, return an array of all match strings;
        // otherwise behave like exec.
        let is_global = match &rt.obj(re_id).internal_kind {
            InternalKind::RegExp(r) => r.flags.contains('g'),
            _ => false,
        };
        if !is_global {
            return regexp_exec(rt, re_id, &s);
        }
        let rx = match &rt.obj(re_id).internal_kind {
            InternalKind::RegExp(r) => r.compiled.clone(),
            _ => None,
        };
        let rx = match rx {
            Some(r) => r,
            None => return Err(RuntimeError::TypeError(
                "String.prototype.match: regex pattern unsupported".into())),
        };
        let matches: Vec<String> = rx.find_iter_owned(&s).into_iter().map(|(_,_,s)| s).collect();
        if matches.is_empty() { return Ok(Value::Null); }
        let arr = rt.alloc_object(Object::new_array());
        for (i, m) in matches.iter().enumerate() {
            rt.object_set(arr, i.to_string(), Value::String(Rc::new(m.clone())));
        }
        rt.object_set(arr, "length".into(), Value::Number(matches.len() as f64));
        Ok(Value::Object(arr))
    });

    register_method(rt, host, "search", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let re_id = coerce_regexp(rt, args.first().cloned().unwrap_or(Value::Undefined))?;
        let rx = match &rt.obj(re_id).internal_kind {
            InternalKind::RegExp(r) => r.compiled.clone(),
            _ => None,
        };
        let rx = match rx {
            Some(r) => r,
            None => return Err(RuntimeError::TypeError(
                "String.prototype.search: regex pattern unsupported".into())),
        };
        match rx.find_first(&s) {
            Some((start, _)) => Ok(Value::Number(byte_to_char_index(&s, start) as f64)),
            None => Ok(Value::Number(-1.0)),
        }
    });

    register_method(rt, host, "replace", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let pat_arg = args.first().cloned().unwrap_or(Value::Undefined);
        let repl = args.get(1).cloned().unwrap_or(Value::Undefined);
        string_replace_impl(rt, &s, pat_arg, repl, false)
    });

    register_method(rt, host, "replaceAll", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let pat_arg = args.first().cloned().unwrap_or(Value::Undefined);
        // Spec: if pat is a RegExp without 'g', throw.
        if let Value::Object(id) = &pat_arg {
            if let InternalKind::RegExp(r) = &rt.obj(*id).internal_kind {
                if !r.flags.contains('g') {
                    return Err(RuntimeError::TypeError(
                        "String.prototype.replaceAll: non-global RegExp".into()));
                }
            }
        }
        let repl = args.get(1).cloned().unwrap_or(Value::Undefined);
        string_replace_impl(rt, &s, pat_arg, repl, true)
    });

    register_method(rt, host, "split", |rt, args| {
        let s = abstract_ops::to_string(&rt.current_this()).as_str().to_string();
        let limit = args.get(1).map(|v| {
            let n = abstract_ops::to_number(v);
            if n.is_finite() && n >= 0.0 { Some(n as usize) } else { None }
        }).flatten();
        let parts: Vec<String> = match args.first() {
            None | Some(Value::Undefined) => vec![s.clone()],
            Some(Value::Object(id)) if matches!(rt.obj(*id).internal_kind, InternalKind::RegExp(_)) => {
                let rx = match &rt.obj(*id).internal_kind {
                    InternalKind::RegExp(r) => r.compiled.clone(),
                    _ => None,
                };
                let rx = match rx {
                    Some(r) => r,
                    None => return Err(RuntimeError::TypeError(
                        "String.prototype.split: regex pattern unsupported".into())),
                };
                rx.split_str(&s)
            }
            Some(sep_v) => {
                let sep = abstract_ops::to_string(sep_v).as_str().to_string();
                if sep.is_empty() {
                    s.chars().map(|c| c.to_string()).collect()
                } else {
                    s.split(&sep).map(|p| p.to_string()).collect()
                }
            }
        };
        let truncated: Vec<String> = match limit {
            Some(l) => parts.into_iter().take(l).collect(),
            None => parts,
        };
        let out = rt.alloc_object(Object::new_array());
        for (i, p) in truncated.iter().enumerate() {
            rt.object_set(out, i.to_string(), Value::String(Rc::new(p.clone())));
        }
        rt.object_set(out, "length".into(), Value::Number(truncated.len() as f64));
        Ok(Value::Object(out))
    });
}

/// Common backend for .replace and .replaceAll. `force_global` is true
/// for .replaceAll. Replacement may be a string (no $1 backref handling
/// in v1) or a function (called with (match) — extended args deferred).
fn string_replace_impl(
    rt: &mut Runtime,
    s: &str,
    pat: Value,
    repl: Value,
    force_global: bool,
) -> Result<Value, RuntimeError> {
    // Pattern path.
    let (rx, is_global) = match &pat {
        Value::Object(id) if matches!(rt.obj(*id).internal_kind, InternalKind::RegExp(_)) => {
            let (rx, flags) = match &rt.obj(*id).internal_kind {
                InternalKind::RegExp(r) => (r.compiled.clone(), (*r.flags).clone()),
                _ => unreachable!(),
            };
            let rx = match rx {
                Some(r) => r,
                None => return Err(RuntimeError::TypeError(
                    "String.prototype.replace: regex pattern unsupported".into())),
            };
            (rx, force_global || flags.contains('g'))
        }
        _ => {
            // String needle — escape to a literal regex so we share the
            // same replacement plumbing. Cheaper than maintaining a
            // separate code path.
            let needle = abstract_ops::to_string(&pat).as_str().to_string();
            let escaped = regex::escape(&needle);
            let rx = regex::Regex::new(&escaped).map_err(|e| RuntimeError::TypeError(format!("{}", e)))?;
            (CompiledRegex::Rust(rx), force_global)
        }
    };

    // Replacement is either a function (callable) or coerced to string.
    let is_callable = matches!(&repl, Value::Object(id) if {
        matches!(rt.obj(*id).internal_kind,
            InternalKind::Function(_) | InternalKind::Closure(_) | InternalKind::BoundFunction(_))
    });

    if !is_callable {
        let repl_s = abstract_ops::to_string(&repl).as_str().to_string();
        let out = if is_global {
            rx.replace_all_lit(s, repl_s.as_str())
        } else {
            rx.replacen_lit(s, 1, repl_s.as_str())
        };
        return Ok(Value::String(Rc::new(out)));
    }

    // Function replacer — collect match ranges, then invoke the function
    // for each match and stitch the output. We split the borrow so we
    // can call back into the runtime.
    let matches: Vec<(usize, usize, String)> = rx.find_iter_owned(s);
    let take_n = if is_global { matches.len() } else { matches.len().min(1) };
    let mut out = String::new();
    let mut cursor = 0usize;
    for (mstart, mend, mstr) in matches.into_iter().take(take_n) {
        out.push_str(&s[cursor..mstart]);
        let call_args = vec![Value::String(Rc::new(mstr))];
        let r = rt.call_function(repl.clone(), Value::Undefined, call_args)?;
        let r_s = abstract_ops::to_string(&r).as_str().to_string();
        out.push_str(&r_s);
        cursor = mend;
    }
    out.push_str(&s[cursor..]);
    Ok(Value::String(Rc::new(out)))
}

/// If the value is already a RegExp, return its id. Otherwise treat it
/// as a string pattern (no flags) and construct a fresh RegExp.
fn coerce_regexp(rt: &mut Runtime, v: Value) -> Result<ObjectRef, RuntimeError> {
    if let Value::Object(id) = &v {
        if matches!(rt.obj(*id).internal_kind, InternalKind::RegExp(_)) {
            return Ok(*id);
        }
    }
    let pattern = abstract_ops::to_string(&v).as_str().to_string();
    new_regexp(rt, &pattern, "")
}

// ──────────────── local helpers ────────────────

fn register_method<F>(rt: &mut Runtime, host: ObjectRef, name: &str, f: F)
where F: Fn(&mut Runtime, &[Value]) -> Result<Value, RuntimeError> + 'static {
    let fn_obj = make_native(name, f);
    let fn_id = rt.alloc_object(fn_obj);
    rt.obj_mut(host).properties.insert(name.into(), PropertyDescriptor {
        value: Value::Object(fn_id),
        writable: true,
        enumerable: false,
        configurable: true, getter: None, setter: None,
    });
}

fn register_global_native<F>(rt: &mut Runtime, name: &str, f: F)
where F: Fn(&mut Runtime, &[Value]) -> Result<Value, RuntimeError> + 'static {
    let fn_obj = make_native(name, f);
    let fn_id = rt.alloc_object(fn_obj);
    rt.globals.insert(name.into(), Value::Object(fn_id));
}
