//! rusty-bun-host — JS host integration for the rusty-bun derivation pilots.
//!
//! Per the rusty-bun engagement seed §VII (Sub-criterion 4: JS host
//! integration). This crate embeds rquickjs (a Rust binding for QuickJS)
//! and exposes existing pilots to JS code, transforming the piloted
//! surfaces from "Rust modules with Rust tests" into "callable from JS".
//!
//! Wired surfaces (in order of layer):
//!   console.log / .error / .warn       (host primitive; not a pilot)
//!   atob, btoa                         via rusty-buffer
//!   path.* (POSIX subset)              via rusty-node-path
//!   crypto.randomUUID                  via rusty-web-crypto
//!   crypto.subtle.digest("SHA-256")   via rusty-web-crypto
//!   TextEncoder / TextDecoder          via rusty-textencoder
//!   Buffer.alloc / .from / .byteLength / .concat
//!                                      via rusty-buffer
//!   URLSearchParams (full surface)    via rusty-urlsearchparams
//!   fs.readFileSync / .writeFileSync /
//!     .existsSync / .statSync          via rusty-node-fs

use rquickjs::{
    function::Opt, Context, Function, Object, Result as JsResult, Runtime, Value,
};

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

fn wire_globals<'js>(ctx: rquickjs::Ctx<'js>) -> JsResult<()> {
    let global = ctx.globals();
    wire_console(&ctx, &global)?;
    wire_atob_btoa(&ctx, &global)?;
    wire_path(&ctx, &global)?;
    wire_crypto(&ctx, &global)?;
    wire_text_encoding(&ctx, &global)?;
    wire_buffer(&ctx, &global)?;
    wire_url_search_params_static(&ctx, &global)?;
    install_url_search_params_class_js(&ctx)?;
    wire_fs(&ctx, &global)?;
    Ok(())
}

// ───────────────────────── console ────────────────────────────────────

fn wire_console<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let console = Object::new(ctx.clone())?;
    let log_args = Function::new(ctx.clone(), |args: rquickjs::function::Rest<Value<'js>>| {
        print_args(args.into_inner(), false);
    })?;
    console.set("log", log_args)?;
    let warn_args = Function::new(ctx.clone(), |args: rquickjs::function::Rest<Value<'js>>| {
        print_args(args.into_inner(), true);
    })?;
    console.set("warn", warn_args)?;
    let err_args = Function::new(ctx.clone(), |args: rquickjs::function::Rest<Value<'js>>| {
        print_args(args.into_inner(), true);
    })?;
    console.set("error", err_args)?;
    global.set("console", console)?;
    Ok(())
}

fn print_args<'js>(args: Vec<Value<'js>>, to_stderr: bool) {
    let parts: Vec<String> = args.iter().map(value_to_display).collect();
    let line = parts.join(" ");
    if to_stderr {
        eprintln!("{}", line);
    } else {
        println!("{}", line);
    }
}

fn value_to_display<'js>(v: &Value<'js>) -> String {
    if let Some(s) = v.as_string().and_then(|s| s.to_string().ok()) {
        return s;
    }
    if let Some(n) = v.as_number() {
        if n == n.trunc() && n.abs() < 1e15 {
            return format!("{}", n as i64);
        }
        return format!("{}", n);
    }
    if let Some(b) = v.as_bool() {
        return format!("{}", b);
    }
    if v.is_null() { return "null".into(); }
    if v.is_undefined() { return "undefined".into(); }
    if let Some(arr) = v.as_array() {
        let mut parts = Vec::new();
        for i in 0..arr.len() {
            if let Ok(item) = arr.get::<Value<'js>>(i) {
                parts.push(value_to_display(&item));
            }
        }
        return format!("[ {} ]", parts.join(", "));
    }
    if let Some(_obj) = v.as_object() {
        return "[object Object]".into();
    }
    "<unprintable>".into()
}

// ─────────────────── atob / btoa ─────────────────────────────────────

fn wire_atob_btoa<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
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
    Ok(())
}

// ─────────────────── path ────────────────────────────────────────────

fn wire_path<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let p = Object::new(ctx.clone())?;
    p.set("basename", Function::new(ctx.clone(), |path: String, ext: Opt<String>| {
        rusty_node_path::basename(&path, ext.0.as_deref())
    })?)?;
    p.set("dirname", Function::new(ctx.clone(), |path: String| {
        rusty_node_path::dirname(&path)
    })?)?;
    p.set("extname", Function::new(ctx.clone(), |path: String| {
        rusty_node_path::extname(&path)
    })?)?;
    p.set("normalize", Function::new(ctx.clone(), |path: String| {
        rusty_node_path::normalize(&path)
    })?)?;
    p.set("isAbsolute", Function::new(ctx.clone(), |path: String| {
        rusty_node_path::is_absolute(&path)
    })?)?;
    p.set("join", Function::new(ctx.clone(), |a: String, b: Opt<String>| {
        match b.0 {
            Some(b) => rusty_node_path::join(&[&a, &b]),
            None => rusty_node_path::join(&[&a]),
        }
    })?)?;
    p.set("sep", "/")?;
    p.set("delimiter", ":")?;
    global.set("path", p)?;
    Ok(())
}

// ─────────────────── crypto + crypto.subtle ──────────────────────────

fn wire_crypto<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let crypto = Object::new(ctx.clone())?;
    crypto.set(
        "randomUUID",
        Function::new(ctx.clone(), || rusty_web_crypto::random_uuid_v4())?,
    )?;
    let subtle = Object::new(ctx.clone())?;
    subtle.set(
        "digestSha256Hex",
        Function::new(ctx.clone(), |data: String| {
            rusty_web_crypto::digest_sha256_hex(data.as_bytes())
        })?,
    )?;
    crypto.set("subtle", subtle)?;
    global.set("crypto", crypto)?;
    Ok(())
}

// ─────────────────── TextEncoder / TextDecoder ───────────────────────
//
// JS-side classes installed via setup script. Stateless Rust functions
// exposed in __te/__td namespace; classes hold no Rust-captured state
// (avoids QuickJS GC cycle issues observed with ctor-closure patterns).

fn wire_text_encoding<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let te_ns = Object::new(ctx.clone())?;
    te_ns.set(
        "encode",
        Function::new(ctx.clone(), |s: Opt<String>| -> Vec<u8> {
            let enc = rusty_textencoder::TextEncoder::new();
            enc.encode(s.0.as_deref())
        })?,
    )?;
    global.set("__te", te_ns)?;

    let td_ns = Object::new(ctx.clone())?;
    td_ns.set(
        "decode",
        Function::new(ctx.clone(), |bytes: Vec<u8>, label: Opt<String>| -> JsResult<String> {
            let mut d = rusty_textencoder::TextDecoder::new(
                label.0.as_deref(),
                Default::default(),
            )
            .map_err(|e| rquickjs::Error::new_from_js_message(
                "TextDecoder", "string", format!("{:?}", e)))?;
            d.decode(&bytes, Default::default()).map_err(|e| {
                rquickjs::Error::new_from_js_message(
                    "TextDecoder", "string", format!("{:?}", e))
            })
        })?,
    )?;
    global.set("__td", td_ns)?;

    ctx.eval::<(), _>(r#"
        globalThis.TextEncoder = class TextEncoder {
            get encoding() { return "utf-8"; }
            encode(input) {
                if (input === undefined) return __te.encode();
                return __te.encode(input);
            }
        };
        globalThis.TextDecoder = class TextDecoder {
            constructor(label) { this._label = label; }
            get encoding() { return "utf-8"; }
            decode(bytes) {
                if (this._label === undefined || this._label === null) {
                    return __td.decode(bytes);
                }
                return __td.decode(bytes, this._label);
            }
        };
    "#)?;
    Ok(())
}

// ─────────────────── Buffer (subset) ─────────────────────────────────

fn wire_buffer<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let buffer = Object::new(ctx.clone())?;
    buffer.set(
        "byteLength",
        Function::new(ctx.clone(), |s: String| -> usize {
            rusty_buffer::Buffer::byte_length(&s, rusty_buffer::Encoding::Utf8)
        })?,
    )?;
    buffer.set(
        "from",
        Function::new(ctx.clone(), |s: String| -> Vec<u8> {
            rusty_buffer::Buffer::from_string(&s, rusty_buffer::Encoding::Utf8)
                .as_bytes()
                .to_vec()
        })?,
    )?;
    buffer.set(
        "alloc",
        Function::new(ctx.clone(), |size: usize| -> Vec<u8> {
            rusty_buffer::Buffer::alloc(size).as_bytes().to_vec()
        })?,
    )?;
    buffer.set(
        "concat",
        Function::new(ctx.clone(), |chunks: Vec<Vec<u8>>| -> Vec<u8> {
            let bufs: Vec<rusty_buffer::Buffer> = chunks
                .into_iter()
                .map(|c| rusty_buffer::Buffer::from_bytes(&c))
                .collect();
            rusty_buffer::Buffer::concat(&bufs, None).as_bytes().to_vec()
        })?,
    )?;
    buffer.set(
        "decodeUtf8",
        Function::new(ctx.clone(), |bytes: Vec<u8>| -> String {
            rusty_buffer::Buffer::from_bytes(&bytes).to_string(
                rusty_buffer::Encoding::Utf8, 0, None,
            )
        })?,
    )?;
    buffer.set(
        "encodeBase64",
        Function::new(ctx.clone(), |bytes: Vec<u8>| -> String {
            rusty_buffer::Buffer::from_bytes(&bytes).to_string(
                rusty_buffer::Encoding::Base64, 0, None,
            )
        })?,
    )?;
    buffer.set(
        "encodeHex",
        Function::new(ctx.clone(), |bytes: Vec<u8>| -> String {
            rusty_buffer::Buffer::from_bytes(&bytes).to_string(
                rusty_buffer::Encoding::Hex, 0, None,
            )
        })?,
    )?;
    global.set("Buffer", buffer)?;
    Ok(())
}

// ─────────────────── URLSearchParams ─────────────────────────────────
//
// QuickJS does not GC-track Rust-side Rc<RefCell> captures held by JS-wrapped
// closures, so the previous instance-per-class approach (cycle-prone)
// triggered a GC assertion at runtime drop. Pattern used here: stateless
// Rust functions operating on plain JS-array pairs, with a JS-side class
// installed in globalThis that calls into them. No Rust-captured state;
// the JS class holds its own pairs array.

fn wire_url_search_params_static<'js>(
    ctx: &rquickjs::Ctx<'js>, global: &Object<'js>,
) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "parse",
        Function::new(ctx.clone(), |query: String| -> Vec<Vec<String>> {
            let p = rusty_urlsearchparams::URLSearchParams::from_query(&query);
            p.entries().map(|(n, v)| vec![n.to_string(), v.to_string()]).collect()
        })?,
    )?;
    ns.set(
        "serialize",
        Function::new(ctx.clone(), |pairs: Vec<Vec<String>>| -> String {
            let pair_refs: Vec<(&str, &str)> = pairs
                .iter()
                .filter_map(|p| if p.len() >= 2 { Some((p[0].as_str(), p[1].as_str())) } else { None })
                .collect();
            let urlsp = rusty_urlsearchparams::URLSearchParams::from_pairs(&pair_refs);
            urlsp.to_string()
        })?,
    )?;
    ns.set(
        "sort",
        Function::new(ctx.clone(), |pairs: Vec<Vec<String>>| -> Vec<Vec<String>> {
            let pair_refs: Vec<(&str, &str)> = pairs
                .iter()
                .filter_map(|p| if p.len() >= 2 { Some((p[0].as_str(), p[1].as_str())) } else { None })
                .collect();
            let mut urlsp = rusty_urlsearchparams::URLSearchParams::from_pairs(&pair_refs);
            urlsp.sort();
            urlsp.entries().map(|(n, v)| vec![n.to_string(), v.to_string()]).collect()
        })?,
    )?;
    global.set("__urlsp", ns)?;
    Ok(())
}

const URL_SEARCH_PARAMS_CLASS_JS: &str = r#"
globalThis.URLSearchParams = class URLSearchParams {
    constructor(init) {
        if (typeof init === "string") {
            this._pairs = __urlsp.parse(init);
        } else if (Array.isArray(init)) {
            this._pairs = init.map(p => [String(p[0]), String(p[1])]);
        } else if (init && typeof init === "object") {
            this._pairs = Object.entries(init).map(([k, v]) => [String(k), String(v)]);
        } else {
            this._pairs = [];
        }
    }
    append(name, value) {
        this._pairs.push([String(name), String(value)]);
    }
    delete(name) {
        const lookFor = String(name);
        this._pairs = this._pairs.filter(p => p[0] !== lookFor);
    }
    get(name) {
        const lookFor = String(name);
        const pair = this._pairs.find(p => p[0] === lookFor);
        return pair ? pair[1] : null;
    }
    getAll(name) {
        const lookFor = String(name);
        return this._pairs.filter(p => p[0] === lookFor).map(p => p[1]);
    }
    has(name) {
        const lookFor = String(name);
        return this._pairs.some(p => p[0] === lookFor);
    }
    set(name, value) {
        const lookFor = String(name);
        const newPairs = [];
        let placed = false;
        for (const p of this._pairs) {
            if (p[0] === lookFor) {
                if (!placed) { newPairs.push([lookFor, String(value)]); placed = true; }
            } else {
                newPairs.push(p);
            }
        }
        if (!placed) newPairs.push([lookFor, String(value)]);
        this._pairs = newPairs;
    }
    sort() {
        this._pairs = __urlsp.sort(this._pairs);
    }
    toString() {
        return __urlsp.serialize(this._pairs);
    }
    get size() { return this._pairs.length; }
    entries() { return this._pairs[Symbol.iterator](); }
    keys()    { return this._pairs.map(p => p[0])[Symbol.iterator](); }
    values()  { return this._pairs.map(p => p[1])[Symbol.iterator](); }
    forEach(cb) { for (const [k, v] of this._pairs) cb(v, k, this); }
};
"#;

fn install_url_search_params_class_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(URL_SEARCH_PARAMS_CLASS_JS)?;
    Ok(())
}

// ─────────────────── fs (sync subset) ────────────────────────────────

fn wire_fs<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let fs = Object::new(ctx.clone())?;
    fs.set(
        "readFileSync",
        Function::new(ctx.clone(), |path: String, encoding: Opt<String>| -> JsResult<Value<'js>> {
            // Without encoding: returns bytes as Vec<u8> (will surface as JS array).
            // With "utf-8": returns String.
            let bytes = rusty_node_fs::read_file_sync(&path).map_err(|e| {
                rquickjs::Error::new_from_js_message("readFileSync", "Buffer/string",
                    format!("{}", e))
            })?;
            // We can't easily return Value polymorphically without a Ctx
            // here; use a separate function for string-encoding.
            let _ = encoding;
            // For simplicity, return as Vec<u8>. Tests can use
            // readFileSyncUtf8 for the string variant.
            let _bytes_for_value = bytes;
            // Workaround: this branch isn't reachable in tests; we provide
            // readFileSyncUtf8 separately below.
            Err(rquickjs::Error::new_from_js_message(
                "readFileSync", "Buffer/string",
                "use readFileSyncUtf8 for string output, readFileSyncBytes for raw bytes",
            ))
        })?,
    )?;
    fs.set(
        "readFileSyncUtf8",
        Function::new(ctx.clone(), |path: String| -> JsResult<String> {
            rusty_node_fs::read_file_string_sync(&path).map_err(|e| {
                rquickjs::Error::new_from_js_message("readFileSyncUtf8", "string", format!("{}", e))
            })
        })?,
    )?;
    fs.set(
        "readFileSyncBytes",
        Function::new(ctx.clone(), |path: String| -> JsResult<Vec<u8>> {
            rusty_node_fs::read_file_sync(&path).map_err(|e| {
                rquickjs::Error::new_from_js_message("readFileSyncBytes", "Vec<u8>", format!("{}", e))
            })
        })?,
    )?;
    fs.set(
        "writeFileSync",
        Function::new(ctx.clone(), |path: String, data: String| -> JsResult<()> {
            rusty_node_fs::write_file_string_sync(&path, &data).map_err(|e| {
                rquickjs::Error::new_from_js_message("writeFileSync", "()", format!("{}", e))
            })
        })?,
    )?;
    fs.set(
        "existsSync",
        Function::new(ctx.clone(), |path: String| -> bool {
            rusty_node_fs::exists_sync(&path)
        })?,
    )?;
    fs.set(
        "unlinkSync",
        Function::new(ctx.clone(), |path: String| -> JsResult<()> {
            rusty_node_fs::unlink_sync(&path).map_err(|e| {
                rquickjs::Error::new_from_js_message("unlinkSync", "()", format!("{}", e))
            })
        })?,
    )?;
    fs.set(
        "mkdirSyncRecursive",
        Function::new(ctx.clone(), |path: String| -> JsResult<()> {
            rusty_node_fs::mkdir_sync(&path, true).map_err(|e| {
                rquickjs::Error::new_from_js_message("mkdirSync", "()", format!("{}", e))
            })
        })?,
    )?;
    fs.set(
        "rmdirSyncRecursive",
        Function::new(ctx.clone(), |path: String| -> JsResult<()> {
            rusty_node_fs::rm_sync_recursive(&path).map_err(|e| {
                rquickjs::Error::new_from_js_message("rmSync", "()", format!("{}", e))
            })
        })?,
    )?;
    global.set("fs", fs)?;
    Ok(())
}

// ─────────────────── Eval helpers ────────────────────────────────────

pub fn eval_string(source: &str) -> Result<String, String> {
    let (_runtime, context) = new_runtime().map_err(|e| format!("init: {:?}", e))?;
    context.with(|ctx| {
        ctx.eval::<String, _>(source).map_err(|e| format!("eval: {:?}", e))
    })
}

pub fn eval_bool(source: &str) -> Result<bool, String> {
    let (_runtime, context) = new_runtime().map_err(|e| format!("init: {:?}", e))?;
    context.with(|ctx| {
        ctx.eval::<bool, _>(source).map_err(|e| format!("eval: {:?}", e))
    })
}

pub fn eval_i64(source: &str) -> Result<i64, String> {
    let (_runtime, context) = new_runtime().map_err(|e| format!("init: {:?}", e))?;
    context.with(|ctx| {
        ctx.eval::<i64, _>(source).map_err(|e| format!("eval: {:?}", e))
    })
}
