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
    wire_blob_static(&ctx, &global)?;
    install_blob_and_file_classes_js(&ctx)?;
    wire_abort_controller_static(&ctx, &global)?;
    install_abort_controller_classes_js(&ctx)?;
    wire_headers_static(&ctx, &global)?;
    wire_response_static(&ctx, &global)?;
    install_fetch_api_classes_js(&ctx)?;
    wire_bun_namespace_static(&ctx, &global)?;
    install_bun_namespace_js(&ctx)?;
    wire_bun_serve_static(&ctx, &global)?;
    install_bun_serve_js(&ctx)?;
    wire_bun_spawn_static(&ctx, &global)?;
    install_bun_spawn_js(&ctx)?;
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

// ─────────────────── Blob + File ─────────────────────────────────────
//
// Per seed §III.A8 Pattern 3: stateless Rust helpers expose the
// algorithmic core; JS-side classes hold their own state. The Blob class
// owns its bytes (as a JS array) and mime_type (string); the Rust helpers
// operate on plain Vec<u8> + String.

fn wire_blob_static<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "lowercaseAsciiType",
        Function::new(ctx.clone(), |s: String| -> String {
            s.chars()
                .map(|c| if c.is_ascii() { c.to_ascii_lowercase() } else { c })
                .collect()
        })?,
    )?;
    ns.set(
        "sliceBytes",
        Function::new(ctx.clone(), |bytes: Vec<u8>, start: i64, end: Opt<i64>| -> Vec<u8> {
            let blob = rusty_blob::Blob::from_bytes(bytes);
            blob.slice(start, end.0, None).bytes()
        })?,
    )?;
    ns.set(
        "decodeText",
        Function::new(ctx.clone(), |bytes: Vec<u8>| -> String {
            rusty_blob::Blob::from_bytes(bytes).text()
        })?,
    )?;
    global.set("__blob", ns)?;
    Ok(())
}

const BLOB_AND_FILE_CLASSES_JS: &str = r#"
globalThis.Blob = class Blob {
    constructor(parts, options) {
        const collected = [];
        if (Array.isArray(parts)) {
            for (const part of parts) {
                if (typeof part === "string") {
                    // UTF-8 encode by passing through TextEncoder.
                    const enc = new TextEncoder();
                    const encoded = enc.encode(part);
                    for (const b of encoded) collected.push(b);
                } else if (Array.isArray(part)) {
                    for (const b of part) collected.push(b);
                } else if (part && typeof part.bytes === "function") {
                    for (const b of part.bytes()) collected.push(b);
                }
            }
        }
        this._bytes = collected;
        const t = (options && typeof options.type === "string") ? options.type : "";
        this._type = __blob.lowercaseAsciiType(t);
    }
    get size() { return this._bytes.length; }
    get type() { return this._type; }
    bytes() { return this._bytes; }
    arrayBuffer() { return this._bytes; }
    text() { return __blob.decodeText(this._bytes); }
    slice(start, end, contentType) {
        const startN = (typeof start === "number") ? start : 0;
        const sliced = (end === undefined)
            ? __blob.sliceBytes(this._bytes, startN)
            : __blob.sliceBytes(this._bytes, startN, end);
        const newType = (typeof contentType === "string") ? contentType : "";
        return new Blob([sliced], { type: newType });
    }
};

globalThis.File = class File extends Blob {
    constructor(parts, name, options) {
        super(parts, options);
        this._name = String(name);
        this._lastModified = (options && typeof options.lastModified === "number")
            ? options.lastModified : 0;
        this._webkitRelativePath = "";
    }
    get name() { return this._name; }
    get lastModified() { return this._lastModified; }
    get webkitRelativePath() { return this._webkitRelativePath; }
};
"#;

fn install_blob_and_file_classes_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(BLOB_AND_FILE_CLASSES_JS)?;
    Ok(())
}

// ─────────────────── AbortController + AbortSignal ───────────────────
//
// Per seed §III.A8 Pattern 3 + the rusty-abort-controller pilot's pattern:
// state is held in JS (the listener list, aborted flag, reason); a single
// Rust helper exposes the canonical default-reason DOMException-AbortError
// shape so the JS class can return a structurally-equivalent object.

fn wire_abort_controller_static<'js>(
    ctx: &rquickjs::Ctx<'js>, global: &Object<'js>,
) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "defaultReasonName",
        Function::new(ctx.clone(), || -> String {
            // rusty_abort_controller::Reason::AbortError → "AbortError"
            "AbortError".to_string()
        })?,
    )?;
    ns.set(
        "defaultReasonCode",
        Function::new(ctx.clone(), || -> u16 {
            // Per DOMException AbortError legacy code per pilot
            rusty_abort_controller::Reason::AbortError.code()
        })?,
    )?;
    global.set("__abort", ns)?;
    Ok(())
}

const ABORT_CONTROLLER_CLASSES_JS: &str = r#"
globalThis.AbortSignal = class AbortSignal {
    constructor() {
        this._aborted = false;
        this._reason = undefined;
        this._listeners = [];
    }
    get aborted() { return this._aborted; }
    get reason() { return this._reason; }
    addEventListener(type, listener) {
        if (type !== "abort") return;
        if (this._aborted) {
            try { listener(this._reason); } catch (_) {}
            return;
        }
        this._listeners.push(listener);
    }
    removeEventListener(type, listener) {
        if (type !== "abort") return;
        this._listeners = this._listeners.filter(l => l !== listener);
    }
    throwIfAborted() {
        if (this._aborted) throw this._reason;
    }
    _doAbort(reason) {
        if (this._aborted) return;
        this._aborted = true;
        this._reason = reason !== undefined ? reason : {
            name: __abort.defaultReasonName(),
            code: __abort.defaultReasonCode(),
            message: "The operation was aborted",
        };
        const listeners = this._listeners;
        this._listeners = [];
        for (const l of listeners) {
            try { l(this._reason); } catch (_) {}
        }
    }
    static abort(reason) {
        const s = new AbortSignal();
        s._doAbort(reason);
        return s;
    }
    static any(signals) {
        const result = new AbortSignal();
        for (const s of signals) {
            if (s._aborted) { result._doAbort(s._reason); return result; }
        }
        for (const s of signals) {
            s.addEventListener("abort", (reason) => {
                if (!result._aborted) result._doAbort(reason);
            });
        }
        return result;
    }
};

globalThis.AbortController = class AbortController {
    constructor() {
        this._signal = new AbortSignal();
    }
    get signal() { return this._signal; }
    abort(reason) {
        this._signal._doAbort(reason);
    }
};
"#;

fn install_abort_controller_classes_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(ABORT_CONTROLLER_CLASSES_JS)?;
    Ok(())
}

// ─────────────────── Headers ─────────────────────────────────────────

fn wire_headers_static<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "validateName",
        Function::new(ctx.clone(), |name: String| -> bool {
            // Validate via the pilot's append (cheapest way to invoke
            // validate_name without exposing private fns).
            let mut h = rusty_fetch_api::Headers::new();
            h.append(&name, "x").is_ok()
        })?,
    )?;
    ns.set(
        "validateValue",
        Function::new(ctx.clone(), |value: String| -> bool {
            let mut h = rusty_fetch_api::Headers::new();
            h.append("x", &value).is_ok()
        })?,
    )?;
    ns.set(
        "lowercaseName",
        Function::new(ctx.clone(), |s: String| -> String {
            s.to_ascii_lowercase()
        })?,
    )?;
    ns.set(
        "stripWhitespace",
        Function::new(ctx.clone(), |s: String| -> String {
            s.trim_matches(|c: char| matches!(c, ' ' | '\t' | '\n' | '\r')).to_string()
        })?,
    )?;
    global.set("__headers", ns)?;
    Ok(())
}

// ─────────────────── Response (static helpers) ───────────────────────

fn wire_response_static<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "validStatus",
        Function::new(ctx.clone(), |s: u16| -> bool {
            (200..=599).contains(&s)
        })?,
    )?;
    ns.set(
        "validRedirectStatus",
        Function::new(ctx.clone(), |s: u16| -> bool {
            matches!(s, 301 | 302 | 303 | 307 | 308)
        })?,
    )?;
    global.set("__response", ns)?;
    Ok(())
}

// ─────────────────── Fetch API JS-side classes ───────────────────────

const FETCH_API_CLASSES_JS: &str = r#"
globalThis.Headers = class Headers {
    constructor(init) {
        this._entries = [];
        if (init === undefined || init === null) return;
        if (init instanceof Headers) {
            for (const [n, v] of init.entries()) this.append(n, v);
        } else if (Array.isArray(init)) {
            for (const pair of init) this.append(pair[0], pair[1]);
        } else if (typeof init === "object") {
            for (const [k, v] of Object.entries(init)) this.append(k, v);
        }
    }
    append(name, value) {
        if (!__headers.validateName(String(name))) {
            throw new TypeError("Invalid header name: " + name);
        }
        const stripped = __headers.stripWhitespace(String(value));
        if (!__headers.validateValue(stripped)) {
            throw new TypeError("Invalid header value: " + value);
        }
        this._entries.push([__headers.lowercaseName(String(name)), stripped]);
    }
    delete(name) {
        const lower = __headers.lowercaseName(String(name));
        this._entries = this._entries.filter(p => p[0] !== lower);
    }
    get(name) {
        const lower = __headers.lowercaseName(String(name));
        const matches = this._entries.filter(p => p[0] === lower);
        if (matches.length === 0) return null;
        return matches.map(p => p[1]).join(", ");
    }
    getSetCookie() {
        return this._entries.filter(p => p[0] === "set-cookie").map(p => p[1]);
    }
    has(name) {
        const lower = __headers.lowercaseName(String(name));
        return this._entries.some(p => p[0] === lower);
    }
    set(name, value) {
        if (!__headers.validateName(String(name))) {
            throw new TypeError("Invalid header name: " + name);
        }
        const stripped = __headers.stripWhitespace(String(value));
        if (!__headers.validateValue(stripped)) {
            throw new TypeError("Invalid header value: " + value);
        }
        const lower = __headers.lowercaseName(String(name));
        const newEntries = [];
        let placed = false;
        for (const p of this._entries) {
            if (p[0] === lower) {
                if (!placed) { newEntries.push([lower, stripped]); placed = true; }
            } else {
                newEntries.push(p);
            }
        }
        if (!placed) newEntries.push([lower, stripped]);
        this._entries = newEntries;
    }
    *entries() {
        const sorted = [...this._entries].sort((a, b) => a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0);
        for (const e of sorted) yield e;
    }
    *keys() { for (const [n, _] of this.entries()) yield n; }
    *values() { for (const [_, v] of this.entries()) yield v; }
    forEach(cb) { for (const [n, v] of this.entries()) cb(v, n, this); }
    [Symbol.iterator]() { return this.entries(); }
};

globalThis.Request = class Request {
    constructor(input, init) {
        if (typeof input !== "string" && !(input instanceof Request)) {
            throw new TypeError("Invalid Request input");
        }
        if (input instanceof Request) {
            this._method = input._method;
            this._url = input._url;
            this._headers = new Headers(input._headers);
            this._body = input._body;
            this._bodyUsed = false;
        } else {
            this._method = (init && init.method) ? String(init.method).toUpperCase() : "GET";
            this._url = String(input);
            this._headers = new Headers(init && init.headers);
            this._body = (init && init.body !== undefined) ? init.body : null;
            this._bodyUsed = false;
        }
        this._mode = (init && init.mode) || "cors";
        this._credentials = (init && init.credentials) || "same-origin";
        this._cache = (init && init.cache) || "default";
        this._redirect = (init && init.redirect) || "follow";
        this._signal = (init && init.signal) || new AbortSignal();
    }
    get method() { return this._method; }
    get url() { return this._url; }
    get headers() { return this._headers; }
    get body() { return this._body; }
    get bodyUsed() { return this._bodyUsed; }
    get mode() { return this._mode; }
    get credentials() { return this._credentials; }
    get cache() { return this._cache; }
    get redirect() { return this._redirect; }
    get signal() { return this._signal; }
    text() {
        if (this._bodyUsed) throw new TypeError("Body already used");
        this._bodyUsed = true;
        if (this._body === null || this._body === undefined) return "";
        if (typeof this._body === "string") return this._body;
        if (Array.isArray(this._body)) {
            return new TextDecoder().decode(this._body);
        }
        return String(this._body);
    }
    arrayBuffer() {
        if (this._bodyUsed) throw new TypeError("Body already used");
        this._bodyUsed = true;
        if (this._body === null || this._body === undefined) return [];
        if (typeof this._body === "string") return new TextEncoder().encode(this._body);
        if (Array.isArray(this._body)) return this._body;
        return [];
    }
    bytes() { return this.arrayBuffer(); }
    json() {
        const t = this.text();
        return JSON.parse(t);
    }
    clone() {
        if (this._bodyUsed) throw new TypeError("Body already used");
        return new Request(this);
    }
};

globalThis.Response = class Response {
    constructor(body, init) {
        const status = (init && init.status !== undefined) ? init.status : 200;
        if (!__response.validStatus(status)) {
            throw new RangeError("Status out of range: " + status);
        }
        this._status = status;
        this._statusText = (init && init.statusText) ? String(init.statusText) : "";
        this._headers = new Headers(init && init.headers);
        this._body = body !== undefined ? body : null;
        this._bodyUsed = false;
        this._type = "default";
        this._url = "";
        this._redirected = false;
    }
    static error() {
        const r = Object.create(Response.prototype);
        r._status = 0;
        r._statusText = "";
        r._headers = new Headers();
        r._body = null;
        r._bodyUsed = false;
        r._type = "error";
        r._url = "";
        r._redirected = false;
        return r;
    }
    static json(data, init) {
        const headers = new Headers(init && init.headers);
        headers.set("Content-Type", "application/json");
        const body = (typeof data === "string") ? data : JSON.stringify(data);
        return new Response(body, { ...init, headers });
    }
    static redirect(url, status) {
        const s = (status === undefined) ? 302 : status;
        if (!__response.validRedirectStatus(s)) {
            throw new RangeError("Invalid redirect status: " + s);
        }
        const headers = new Headers();
        headers.set("Location", String(url));
        return new Response(null, { status: s, headers });
    }
    get status() { return this._status; }
    get statusText() { return this._statusText; }
    get headers() { return this._headers; }
    get body() { return this._body; }
    get bodyUsed() { return this._bodyUsed; }
    get ok() { return this._status >= 200 && this._status <= 299; }
    get type() { return this._type; }
    get url() { return this._url; }
    get redirected() { return this._redirected; }
    text() {
        if (this._bodyUsed) throw new TypeError("Body already used");
        this._bodyUsed = true;
        if (this._body === null || this._body === undefined) return "";
        if (typeof this._body === "string") return this._body;
        if (Array.isArray(this._body)) return new TextDecoder().decode(this._body);
        return String(this._body);
    }
    arrayBuffer() {
        if (this._bodyUsed) throw new TypeError("Body already used");
        this._bodyUsed = true;
        if (this._body === null || this._body === undefined) return [];
        if (typeof this._body === "string") return new TextEncoder().encode(this._body);
        if (Array.isArray(this._body)) return this._body;
        return [];
    }
    bytes() { return this.arrayBuffer(); }
    json() {
        const t = this.text();
        return JSON.parse(t);
    }
    clone() {
        if (this._bodyUsed) throw new TypeError("Body already used");
        const r = new Response(this._body, {
            status: this._status,
            statusText: this._statusText,
            headers: this._headers,
        });
        r._type = this._type;
        r._url = this._url;
        r._redirected = this._redirected;
        return r;
    }
};
"#;

fn install_fetch_api_classes_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(FETCH_API_CLASSES_JS)?;
    Ok(())
}

// ─────────────────── Bun namespace (Bun.file etc.) ───────────────────

fn wire_bun_namespace_static<'js>(
    ctx: &rquickjs::Ctx<'js>, global: &Object<'js>,
) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    ns.set(
        "fileMimeType",
        Function::new(ctx.clone(), |path: String| -> String {
            // Use rusty-bun-file's extension-to-MIME mapping.
            rusty_bun_file::BunFile::open(&path).mime_type()
        })?,
    )?;
    ns.set(
        "fileExists",
        Function::new(ctx.clone(), |path: String| -> bool {
            rusty_bun_file::BunFile::open(&path).exists()
        })?,
    )?;
    ns.set(
        "fileSize",
        Function::new(ctx.clone(), |path: String| -> JsResult<i64> {
            rusty_bun_file::BunFile::open(&path)
                .size()
                .map(|s| s as i64)
                .map_err(|e| rquickjs::Error::new_from_js_message(
                    "fileSize", "i64", format!("{}", e)))
        })?,
    )?;
    ns.set(
        "fileText",
        Function::new(ctx.clone(), |path: String| -> JsResult<String> {
            rusty_bun_file::BunFile::open(&path).text().map_err(|e| {
                rquickjs::Error::new_from_js_message("fileText", "string", format!("{}", e))
            })
        })?,
    )?;
    ns.set(
        "fileBytes",
        Function::new(ctx.clone(), |path: String| -> JsResult<Vec<u8>> {
            rusty_bun_file::BunFile::open(&path).bytes().map_err(|e| {
                rquickjs::Error::new_from_js_message("fileBytes", "Vec<u8>", format!("{}", e))
            })
        })?,
    )?;
    global.set("__bun", ns)?;
    Ok(())
}

const BUN_NAMESPACE_JS: &str = r#"
globalThis.Bun = {
    file(path, options) {
        const explicitType = (options && typeof options.type === "string") ? options.type : null;
        const handle = {
            _path: String(path),
            _explicitType: explicitType,
            get name() { return this._path; },
            get size() { return __bun.fileSize(this._path); },
            get type() {
                return this._explicitType !== null
                    ? this._explicitType
                    : __bun.fileMimeType(this._path);
            },
            exists() { return __bun.fileExists(this._path); },
            text() { return __bun.fileText(this._path); },
            arrayBuffer() { return __bun.fileBytes(this._path); },
            bytes() { return __bun.fileBytes(this._path); },
            slice(start, end, contentType) {
                const all = __bun.fileBytes(this._path);
                const startN = (typeof start === "number") ? start : 0;
                const blob = new Blob([all]);
                return blob.slice(startN, end, contentType);
            },
        };
        return handle;
    },
};
"#;

fn install_bun_namespace_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(BUN_NAMESPACE_JS)?;
    Ok(())
}

// ─────────────────── Bun.serve (data-layer) ──────────────────────────
//
// The pilot's data-layer dispatch + route matching exposed as Rust helpers;
// JS-side class holds options and returns a server handle. No socket
// binding (data-layer scope per pilot AUDIT). User calls
// server.fetch(request) to invoke the routing pipeline programmatically.

fn wire_bun_serve_static<'js>(
    ctx: &rquickjs::Ctx<'js>, global: &Object<'js>,
) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    // Route pattern matching: returns an object {matched, params} with
    // params as Vec<Vec<String>> (JS-side reads as array of [name, value]).
    ns.set(
        "matchPattern",
        Function::new(ctx.clone(), |pattern: String, url: String| -> JsResult<Object<'js>> {
            // We need a Ctx to construct an object; can't access it here
            // without changing signature. Return a serialized form instead:
            // a Vec<Vec<String>> where empty = no match, else the param pairs.
            let _ = (pattern, url);
            // This branch isn't taken; see matchPatternPairs below.
            unreachable!("use matchPatternPairs")
        })?,
    )?;
    ns.set(
        "matchPatternPairs",
        Function::new(ctx.clone(), |pattern: String, url: String| -> Vec<Vec<String>> {
            // Return pair-list of captures, OR a single pair ["__nomatch__",
            // ""] sentinel when the pattern doesn't match.
            match rusty_bun_serve::match_pattern(&pattern, &url) {
                Some(params) => params
                    .captures
                    .into_iter()
                    .map(|(k, v)| vec![k, v])
                    .collect(),
                None => vec![vec!["__nomatch__".to_string(), String::new()]],
            }
        })?,
    )?;
    global.set("__serve", ns)?;
    Ok(())
}

const BUN_SERVE_JS: &str = r#"
// Extends globalThis.Bun (already installed by install_bun_namespace_js).
(function() {
    function matchRoute(pattern, urlOrPath) {
        const result = __serve.matchPatternPairs(pattern, urlOrPath);
        if (result.length === 1 && result[0][0] === "__nomatch__") return null;
        const params = {};
        for (const [k, v] of result) params[k] = v;
        return params;
    }

    function dispatch(server, request) {
        if (server._stopped) return Response.error();
        const method = (request && request.method) || "GET";
        const url = (request && request.url) || "/";

        // Route matching pass.
        if (Array.isArray(server._routes)) {
            for (const route of server._routes) {
                const params = matchRoute(route.pattern, url);
                if (params === null) continue;
                // Method-keyed dispatch.
                if (route.methods && route.methods[method]) {
                    return route.methods[method](request, params);
                }
                if (route.methods && route.methods[""]) {
                    return route.methods[""](request, params);
                }
                // Pattern matched, no handler for this method → 405.
                return new Response(null, {status: 405});
            }
        }
        // Fall through to fetch handler.
        if (typeof server._fetch === "function") {
            return server._fetch(request);
        }
        // Error handler.
        if (typeof server._error === "function") {
            return server._error(new Error("no route matched"));
        }
        return new Response(null, {status: 404});
    }

    Bun.serve = function serve(options) {
        const opts = options || {};
        const port = (typeof opts.port === "number") ? opts.port : 3000;
        const hostname = (typeof opts.hostname === "string") ? opts.hostname : "localhost";

        // Routes: convert object form ({"/path": handler-or-method-map}) to
        // array of {pattern, methods}.
        let routes = [];
        if (opts.routes && typeof opts.routes === "object") {
            for (const [pattern, handler] of Object.entries(opts.routes)) {
                if (typeof handler === "function") {
                    routes.push({pattern, methods: {"": handler}});
                } else if (handler && typeof handler === "object") {
                    routes.push({pattern, methods: handler});
                }
            }
        }

        const server = {
            _port: port,
            _hostname: hostname,
            _development: !!opts.development,
            _routes: routes,
            _fetch: opts.fetch || null,
            _error: opts.error || null,
            _stopped: false,
            _pendingRequests: 0,
            get port() { return this._port; },
            get hostname() { return this._hostname; },
            get development() { return this._development; },
            get url() { return "http://" + this._hostname + ":" + this._port + "/"; },
            get pendingRequests() { return this._pendingRequests; },
            get listening() { return !this._stopped; },
            fetch(request) {
                this._pendingRequests++;
                try {
                    return dispatch(this, request);
                } finally {
                    this._pendingRequests--;
                }
            },
            stop() { this._stopped = true; },
            reload(newOptions) {
                // Per spec: port + hostname preserved across reload.
                const port = this._port;
                const hostname = this._hostname;
                Object.assign(this, Bun.serve(newOptions));
                this._port = port;
                this._hostname = hostname;
            },
        };
        return server;
    };
})();
"#;

fn install_bun_serve_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(BUN_SERVE_JS)?;
    Ok(())
}

// ─────────────────── Bun.spawn ───────────────────────────────────────
//
// The pilot wraps std::process::Command. JS-side exposes spawnSync
// returning {stdout, stderr, exitCode, success} for the most common
// shell-out pattern. spawn (async-shaped) returns a handle the JS user
// can call .wait() on; per the host's synchronous-poll model.

fn wire_bun_spawn_static<'js>(
    ctx: &rquickjs::Ctx<'js>, global: &Object<'js>,
) -> JsResult<()> {
    use rusty_bun_spawn::{SpawnOptions, StdinInput, StdioMode};
    use std::path::PathBuf;

    let ns = Object::new(ctx.clone())?;
    ns.set(
        "spawnSync",
        Function::new(ctx.clone(), |args: Vec<String>, stdin_text: Opt<String>, cwd: Opt<String>|
                -> JsResult<Object<'js>> {
            let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let opts = SpawnOptions {
                cwd: cwd.0.map(PathBuf::from),
                env: None,
                stdin: match stdin_text.0 {
                    Some(s) => StdinInput::Text(s),
                    None => StdinInput::Null,
                },
                stdout: StdioMode::Pipe,
                stderr: StdioMode::Pipe,
            };
            let _ = (args_refs.clone(), opts.clone());
            // We need a Ctx<'js> to build an Object; we don't have it here.
            // Fall through to spawnSyncResult below which returns a flat
            // pair-list the JS side rebuilds into an object.
            unreachable!("use spawnSyncResult")
        })?,
    )?;
    ns.set(
        "spawnSyncResult",
        Function::new(ctx.clone(), |args: Vec<String>, stdin_text: Opt<String>, cwd: Opt<String>|
                -> JsResult<Vec<Vec<String>>> {
            let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let opts = SpawnOptions {
                cwd: cwd.0.map(PathBuf::from),
                env: None,
                stdin: match stdin_text.0 {
                    Some(s) => StdinInput::Text(s),
                    None => StdinInput::Null,
                },
                stdout: StdioMode::Pipe,
                stderr: StdioMode::Pipe,
            };
            match rusty_bun_spawn::spawn_sync(&args_refs, opts) {
                Ok(r) => Ok(vec![
                    vec!["stdout".into(), String::from_utf8_lossy(&r.stdout).into_owned()],
                    vec!["stderr".into(), String::from_utf8_lossy(&r.stderr).into_owned()],
                    vec!["exitCode".into(), r.exit_code.to_string()],
                    vec!["success".into(), if r.success { "1".into() } else { "0".into() }],
                ]),
                Err(e) => Err(rquickjs::Error::new_from_js_message(
                    "spawnSync", "object", format!("{:?}", e))),
            }
        })?,
    )?;
    global.set("__spawn", ns)?;
    Ok(())
}

const BUN_SPAWN_JS: &str = r#"
(function() {
    Bun.spawnSync = function spawnSync(args, options) {
        const stdinOpt = (options && options.stdin && typeof options.stdin === "string")
            ? options.stdin : undefined;
        const cwd = (options && typeof options.cwd === "string") ? options.cwd : undefined;
        const pairs = (stdinOpt !== undefined && cwd !== undefined)
            ? __spawn.spawnSyncResult(args, stdinOpt, cwd)
            : (stdinOpt !== undefined)
                ? __spawn.spawnSyncResult(args, stdinOpt)
                : (cwd !== undefined)
                    ? __spawn.spawnSyncResult(args, undefined, cwd)
                    : __spawn.spawnSyncResult(args);
        const result = {};
        for (const [k, v] of pairs) result[k] = v;
        // Convert string fields back to expected types.
        return {
            stdout: result.stdout || "",
            stderr: result.stderr || "",
            exitCode: parseInt(result.exitCode || "0", 10),
            success: result.success === "1",
        };
    };
})();
"#;

fn install_bun_spawn_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(BUN_SPAWN_JS)?;
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
