//! rusty-bun-host — JS host integration spike for the rusty-bun derivation
//! pilots.
//!
//! Per the rusty-bun engagement seed §VII (Sub-criterion 4: JS host
//! integration). This crate embeds rquickjs (a Rust binding for QuickJS)
//! and exposes existing pilots to JS code, transforming the piloted
//! surfaces from "Rust modules with Rust tests" into "callable from JS".
//!
//! Spike scope (proves the integration model works):
//!   atob, btoa
//!   path.basename, path.dirname, path.extname, path.join, path.normalize
//!   crypto.randomUUID
//!
//! Future sessions extend the surface coverage. Each new pilot becomes a
//! global wired up at host-runtime construction.

use rquickjs::{function::Opt, Context, Function, Object, Result as JsResult, Runtime};

/// Build a fresh rquickjs Runtime + Context with all rusty-bun pilots wired
/// into globalThis.
pub fn new_runtime() -> JsResult<(Runtime, Context)> {
    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;
    context.with(|ctx| -> JsResult<()> {
        wire_globals(ctx)?;
        Ok(())
    })?;
    Ok((runtime, context))
}

/// Install all wired surfaces into the context's globalThis.
fn wire_globals<'js>(ctx: rquickjs::Ctx<'js>) -> JsResult<()> {
    let global = ctx.globals();

    // atob / btoa — both wired via the rusty-buffer base64 codec.
    global.set(
        "atob",
        Function::new(ctx.clone(), |s: String| -> String {
            let bytes = rusty_buffer::Buffer::from_string(&s, rusty_buffer::Encoding::Base64);
            bytes.as_bytes().iter().map(|&b| b as char).collect()
        })?,
    )?;
    global.set(
        "btoa",
        Function::new(ctx.clone(), |s: String| -> String {
            let bytes: Vec<u8> = s.chars().map(|c| c as u8).collect();
            let buf = rusty_buffer::Buffer::from_bytes(&bytes);
            buf.to_string(rusty_buffer::Encoding::Base64, 0, None)
        })?,
    )?;

    // path.* — POSIX subset wired from rusty-node-path.
    let path_obj = Object::new(ctx.clone())?;
    path_obj.set(
        "basename",
        Function::new(ctx.clone(), |path: String, ext: Opt<String>| -> String {
            rusty_node_path::basename(&path, ext.0.as_deref())
        })?,
    )?;
    path_obj.set(
        "dirname",
        Function::new(ctx.clone(), |path: String| -> String {
            rusty_node_path::dirname(&path)
        })?,
    )?;
    path_obj.set(
        "extname",
        Function::new(ctx.clone(), |path: String| -> String {
            rusty_node_path::extname(&path)
        })?,
    )?;
    path_obj.set(
        "normalize",
        Function::new(ctx.clone(), |path: String| -> String {
            rusty_node_path::normalize(&path)
        })?,
    )?;
    path_obj.set(
        "isAbsolute",
        Function::new(ctx.clone(), |path: String| -> bool {
            rusty_node_path::is_absolute(&path)
        })?,
    )?;
    path_obj.set(
        "join",
        Function::new(ctx.clone(), |a: String, b: Opt<String>| -> String {
            // Variable-arity is awkward in rquickjs; pilot accepts up to two
            // arguments here. Real JS path.join takes spread args.
            match b.0 {
                Some(b) => rusty_node_path::join(&[&a, &b]),
                None => rusty_node_path::join(&[&a]),
            }
        })?,
    )?;
    path_obj.set("sep", "/")?;
    path_obj.set("delimiter", ":")?;
    global.set("path", path_obj)?;

    // crypto.randomUUID — Web Crypto subset.
    let crypto_obj = Object::new(ctx.clone())?;
    crypto_obj.set(
        "randomUUID",
        Function::new(ctx.clone(), || -> String {
            rusty_web_crypto::random_uuid_v4()
        })?,
    )?;
    global.set("crypto", crypto_obj)?;

    Ok(())
}

// ─────────────────── Eval helpers ────────────────────────────────────

/// Eval JS source returning a `String`.
pub fn eval_string(source: &str) -> Result<String, String> {
    let (_runtime, context) = new_runtime().map_err(|e| format!("init: {:?}", e))?;
    context.with(|ctx| {
        ctx.eval::<String, _>(source).map_err(|e| format!("eval: {:?}", e))
    })
}

/// Eval JS source returning a `bool`.
pub fn eval_bool(source: &str) -> Result<bool, String> {
    let (_runtime, context) = new_runtime().map_err(|e| format!("init: {:?}", e))?;
    context.with(|ctx| {
        ctx.eval::<bool, _>(source).map_err(|e| format!("eval: {:?}", e))
    })
}

/// Eval JS source returning an `i64`.
pub fn eval_i64(source: &str) -> Result<i64, String> {
    let (_runtime, context) = new_runtime().map_err(|e| format!("init: {:?}", e))?;
    context.with(|ctx| {
        ctx.eval::<i64, _>(source).map_err(|e| format!("eval: {:?}", e))
    })
}
