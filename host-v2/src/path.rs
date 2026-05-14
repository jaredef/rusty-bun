//! node:path intrinsic — minimal v1 surface. Migrates wire_path from
//! the pre-Ω host's lib.rs (~80 LOC), now installed as a globalThis.path
//! object via register_method.

use crate::register::{arg_string, new_object, register_method, set_constant};
use rusty_js_runtime::{Runtime, Value};
use std::rc::Rc;

pub fn install(rt: &mut Runtime) {
    let path = new_object();

    register_method(&path, "basename", |_rt, args| {
        let p = arg_string(args, 0);
        let ext = args.get(1).map(|v| {
            rusty_js_runtime::abstract_ops::to_string(v).as_str().to_string()
        });
        let base = p.rsplit('/').next().unwrap_or(&p).to_string();
        let result = if let Some(ext) = ext {
            if !ext.is_empty() && base.ends_with(&ext) {
                base[..base.len() - ext.len()].to_string()
            } else { base }
        } else { base };
        Ok(Value::String(Rc::new(result)))
    });

    register_method(&path, "dirname", |_rt, args| {
        let p = arg_string(args, 0);
        let result = if let Some(idx) = p.rfind('/') {
            if idx == 0 { "/".to_string() } else { p[..idx].to_string() }
        } else { ".".to_string() };
        Ok(Value::String(Rc::new(result)))
    });

    register_method(&path, "extname", |_rt, args| {
        let p = arg_string(args, 0);
        let base = p.rsplit('/').next().unwrap_or(&p);
        let result = if let Some(idx) = base.rfind('.') {
            if idx == 0 { String::new() } else { base[idx..].to_string() }
        } else { String::new() };
        Ok(Value::String(Rc::new(result)))
    });

    register_method(&path, "join", |_rt, args| {
        let mut out = String::new();
        for (i, v) in args.iter().enumerate() {
            let s = rusty_js_runtime::abstract_ops::to_string(v);
            let part = s.as_str();
            if part.is_empty() { continue; }
            if i > 0 && !out.ends_with('/') && !part.starts_with('/') {
                out.push('/');
            } else if i > 0 && out.ends_with('/') && part.starts_with('/') {
                out.push_str(&part[1..]);
                continue;
            }
            out.push_str(part);
        }
        if out.is_empty() { out = ".".to_string(); }
        Ok(Value::String(Rc::new(out)))
    });

    register_method(&path, "normalize", |_rt, args| {
        let p = arg_string(args, 0);
        if p.is_empty() { return Ok(Value::String(Rc::new(".".into()))); }
        let absolute = p.starts_with('/');
        let parts: Vec<&str> = p.split('/').filter(|s| !s.is_empty() && *s != ".").collect();
        let mut out: Vec<&str> = Vec::new();
        for part in parts {
            if part == ".." {
                if !out.is_empty() && out.last() != Some(&"..") { out.pop(); }
                else if !absolute { out.push(".."); }
            } else { out.push(part); }
        }
        let joined = out.join("/");
        let result = match (absolute, joined.as_str()) {
            (true, "") => "/".to_string(),
            (true, _) => format!("/{}", joined),
            (false, "") => ".".to_string(),
            (false, _) => joined,
        };
        Ok(Value::String(Rc::new(result)))
    });

    register_method(&path, "isAbsolute", |_rt, args| {
        let p = arg_string(args, 0);
        Ok(Value::Boolean(p.starts_with('/')))
    });

    register_method(&path, "resolve", |_rt, args| {
        // Right-to-left join until we hit an absolute path; if none, prepend cwd.
        let mut parts: Vec<String> = Vec::new();
        let mut hit_absolute = false;
        for v in args.iter().rev() {
            let s = rusty_js_runtime::abstract_ops::to_string(v);
            let part = s.as_str().to_string();
            if part.is_empty() { continue; }
            parts.insert(0, part.clone());
            if part.starts_with('/') { hit_absolute = true; break; }
        }
        if !hit_absolute {
            let cwd = std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "/".to_string());
            parts.insert(0, cwd);
        }
        let joined = parts.join("/");
        // Normalize the result.
        let absolute = joined.starts_with('/');
        let segs: Vec<&str> = joined.split('/').filter(|s| !s.is_empty() && *s != ".").collect();
        let mut out: Vec<&str> = Vec::new();
        for s in segs {
            if s == ".." { if !out.is_empty() { out.pop(); } }
            else { out.push(s); }
        }
        let result = if absolute {
            format!("/{}", out.join("/"))
        } else { out.join("/") };
        Ok(Value::String(Rc::new(result)))
    });

    set_constant(&path, "sep", Value::String(Rc::new("/".into())));
    set_constant(&path, "delimiter", Value::String(Rc::new(":".into())));

    rt.globals.insert("path".into(), Value::Object(path));
}
