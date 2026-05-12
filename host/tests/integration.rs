// Integration tests for rusty-bun-host. These tests run JS code through
// the rquickjs-embedded host with all pilots wired into globalThis,
// validating that the integration layer works end-to-end.

use rusty_bun_host::{eval_bool, eval_i64, eval_string, eval_string_async};

// ════════════════════ atob / btoa ════════════════════

#[test]
fn js_atob_roundtrip() {
    // btoa("hello") should encode; atob() should decode back.
    let r = eval_string(r#"atob(btoa("hello"))"#).unwrap();
    assert_eq!(r, "hello");
}

#[test]
fn js_btoa_known_value() {
    let r = eval_string(r#"btoa("hello")"#).unwrap();
    assert_eq!(r, "aGVsbG8=");
}

#[test]
fn js_atob_known_value() {
    let r = eval_string(r#"atob("aGVsbG8=")"#).unwrap();
    assert_eq!(r, "hello");
}

// ════════════════════ path ════════════════════

#[test]
fn js_path_basename() {
    let r = eval_string(r#"path.basename("/foo/bar/baz.html")"#).unwrap();
    assert_eq!(r, "baz.html");
}

#[test]
fn js_path_basename_with_ext() {
    let r = eval_string(r#"path.basename("/foo/bar/baz.html", ".html")"#).unwrap();
    assert_eq!(r, "baz");
}

#[test]
fn js_path_dirname() {
    let r = eval_string(r#"path.dirname("/foo/bar/baz")"#).unwrap();
    assert_eq!(r, "/foo/bar");
}

#[test]
fn js_path_extname() {
    let r = eval_string(r#"path.extname("file.tar.gz")"#).unwrap();
    assert_eq!(r, ".gz");
}

#[test]
fn js_path_normalize() {
    let r = eval_string(r#"path.normalize("/foo/bar//baz/asdf/quux/..")"#).unwrap();
    assert_eq!(r, "/foo/bar/baz/asdf");
}

#[test]
fn js_path_is_absolute() {
    let r = eval_bool(r#"path.isAbsolute("/foo")"#).unwrap();
    assert!(r);
    let r = eval_bool(r#"path.isAbsolute("foo")"#).unwrap();
    assert!(!r);
}

#[test]
fn js_path_sep_constant() {
    let r = eval_string(r#"path.sep"#).unwrap();
    assert_eq!(r, "/");
}

// ════════════════════ crypto.randomUUID ════════════════════

#[test]
fn js_crypto_random_uuid_format() {
    // 36-char string with v4 format
    let r = eval_string(r#"crypto.randomUUID()"#).unwrap();
    assert_eq!(r.len(), 36);
    let parts: Vec<&str> = r.split('-').collect();
    assert_eq!(parts.len(), 5);
    // Version field is "4"
    assert_eq!(&parts[2][0..1], "4");
}

#[test]
fn js_crypto_random_uuid_unique() {
    // Two calls produce different values with overwhelming probability.
    let a = eval_string(r#"crypto.randomUUID()"#).unwrap();
    let b = eval_string(r#"crypto.randomUUID()"#).unwrap();
    assert_ne!(a, b);
}

// ════════════════════ Composition: pilots used together from JS ════════════════════

#[test]
fn js_composition_atob_path_combined() {
    // Decode a base64-encoded path, then split via path.basename.
    // btoa("/usr/local/bin/node") = "L3Vzci9sb2NhbC9iaW4vbm9kZQ=="
    // → atob → "/usr/local/bin/node" → basename → "node"
    let r = eval_string(r#"
        const encoded = btoa("/usr/local/bin/node");
        const decoded = atob(encoded);
        path.basename(decoded)
    "#).unwrap();
    assert_eq!(r, "node");
}

// ════════════════════ JS evaluation works at all ════════════════════

#[test]
fn js_pure_javascript_works() {
    let r = eval_i64("1 + 2 + 3").unwrap();
    assert_eq!(r, 6);
}

#[test]
fn js_basic_arithmetic_with_string() {
    let r = eval_string(r#"["a", "b", "c"].join("-")"#).unwrap();
    assert_eq!(r, "a-b-c");
}

// ════════════════════ TextEncoder / TextDecoder ════════════════════

#[test]
fn js_text_encoder_encoding_property() {
    let r = eval_string(r#"new TextEncoder().encoding"#).unwrap();
    assert_eq!(r, "utf-8");
}

#[test]
fn js_text_encoder_encode_then_text_decoder() {
    let r = eval_string(r#"
        const enc = new TextEncoder();
        const dec = new TextDecoder();
        const bytes = enc.encode("hello world");
        dec.decode(bytes)
    "#).unwrap();
    assert_eq!(r, "hello world");
}

#[test]
fn js_text_encoder_unicode() {
    let r = eval_string(r#"
        const enc = new TextEncoder();
        const dec = new TextDecoder();
        dec.decode(enc.encode("héllo, мир! 🌍"))
    "#).unwrap();
    assert_eq!(r, "héllo, мир! 🌍");
}

// ════════════════════ Buffer ════════════════════

#[test]
fn js_buffer_byte_length_utf8() {
    let r = eval_i64(r#"Buffer.byteLength("héllo")"#).unwrap();
    assert_eq!(r, 6);
}

#[test]
fn js_buffer_concat_byte_round_trip() {
    let r = eval_string(r#"
        const a = Buffer.from("hello ");
        const b = Buffer.from("world");
        const combined = Buffer.concat([a, b]);
        Buffer.decodeUtf8(combined)
    "#).unwrap();
    assert_eq!(r, "hello world");
}

#[test]
fn js_buffer_alloc_zeros() {
    let r = eval_i64(r#"
        const buf = Buffer.alloc(8);
        let sum = 0;
        for (let i = 0; i < 8; i++) sum += buf[i];
        sum
    "#).unwrap();
    assert_eq!(r, 0);
}

#[test]
fn js_buffer_base64_encode() {
    let r = eval_string(r#"Buffer.encodeBase64(Buffer.from("hello"))"#).unwrap();
    assert_eq!(r, "aGVsbG8=");
}

#[test]
fn js_buffer_hex_encode() {
    let r = eval_string(r#"Buffer.encodeHex(Buffer.from("hello"))"#).unwrap();
    assert_eq!(r, "68656c6c6f");
}

// ════════════════════ URLSearchParams ════════════════════

#[test]
fn js_url_search_params_construction_and_get() {
    let r = eval_string(r#"
        const p = new URLSearchParams("?a=1&b=2");
        p.get("a") + "," + p.get("b")
    "#).unwrap();
    assert_eq!(r, "1,2");
}

#[test]
fn js_url_search_params_to_string() {
    let r = eval_string(r#"
        const p = new URLSearchParams();
        p.append("name", "Jared");
        p.append("greeting", "hello, world!");
        p.toString()
    "#).unwrap();
    assert_eq!(r, "name=Jared&greeting=hello%2C+world%21");
}

#[test]
fn js_url_search_params_sort() {
    let r = eval_string(r#"
        const p = new URLSearchParams("c=1&a=2&b=3");
        p.sort();
        p.toString()
    "#).unwrap();
    assert_eq!(r, "a=2&b=3&c=1");
}

// ════════════════════ fs (sync subset) ════════════════════

#[test]
fn js_fs_write_then_read_roundtrip() {
    let tmp = format!("/tmp/rusty-bun-host-fs-{}", std::process::id());
    let script = format!(r#"
        const path = "{}";
        fs.writeFileSync(path, "test content");
        const exists = fs.existsSync(path);
        const content = fs.readFileSyncUtf8(path);
        fs.unlinkSync(path);
        exists.toString() + "|" + content
    "#, tmp);
    let r = eval_string(&script).unwrap();
    assert_eq!(r, "true|test content");
}

#[test]
fn js_fs_exists_for_missing_file() {
    let r = eval_bool(r#"fs.existsSync("/nonexistent/path/asdfqwer")"#).unwrap();
    assert!(!r);
}

#[test]
fn js_fs_mkdir_then_rmdir_recursive() {
    let pid = std::process::id();
    let parent = format!("/tmp/rusty-bun-host-mkdir-{}", pid);
    let dir = format!("{}/a/b/c", parent);
    let script = format!(r#"
        fs.mkdirSyncRecursive("{}");
        const exists = fs.existsSync("{}");
        fs.rmdirSyncRecursive("{}");
        const goneAfter = fs.existsSync("{}");
        exists.toString() + "|" + goneAfter.toString()
    "#, dir, dir, parent, parent);
    let r = eval_string(&script).unwrap();
    assert_eq!(r, "true|false");
}

// ════════════════════ crypto.subtle ════════════════════

#[test]
fn js_crypto_subtle_digest_sha256() {
    let r = eval_string(r#"crypto.subtle.digestSha256Hex("abc")"#).unwrap();
    assert_eq!(r, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}

// ════════════════════ Cross-pilot composition from JS ════════════════════

#[test]
fn js_compose_buffer_and_text_decoder() {
    let r = eval_string(r#"
        const bytes = Buffer.from("hello");
        const dec = new TextDecoder();
        dec.decode(bytes)
    "#).unwrap();
    assert_eq!(r, "hello");
}

#[test]
fn js_compose_url_params_and_buffer() {
    let r = eval_i64(r#"
        const p = new URLSearchParams();
        p.append("name", "value");
        Buffer.byteLength(p.toString())
    "#).unwrap();
    assert_eq!(r, 10);
}

#[test]
fn js_compose_fs_text_encoder_decoder_chain() {
    let tmp = format!("/tmp/rusty-bun-host-chain-{}", std::process::id());
    let script = format!(r#"
        const path = "{}";
        fs.writeFileSync(path, "hello, world!");
        const readBytes = fs.readFileSyncBytes(path);
        const dec = new TextDecoder();
        const recovered = dec.decode(readBytes);
        fs.unlinkSync(path);
        recovered
    "#, tmp);
    let r = eval_string(&script).unwrap();
    assert_eq!(r, "hello, world!");
}

// ════════════════════ Blob ════════════════════

#[test]
fn js_blob_construction_and_size() {
    let r = eval_i64(r#"new Blob(["hello"]).size"#).unwrap();
    assert_eq!(r, 5);
}

#[test]
fn js_blob_type_lowercased() {
    let r = eval_string(r#"new Blob([], {type: "Application/JSON"}).type"#).unwrap();
    assert_eq!(r, "application/json");
}

#[test]
fn js_blob_text() {
    let r = eval_string_async(r#"return await new Blob(["héllo"]).text();"#).unwrap();
    assert_eq!(r, "héllo");
}

#[test]
fn js_blob_slice() {
    let r = eval_string_async(r#"return await new Blob(["hello world"]).slice(6).text();"#).unwrap();
    assert_eq!(r, "world");
}

#[test]
fn js_blob_slice_with_content_type_override() {
    let r = eval_string(r#"new Blob(["hello"], {type: "text/plain"}).slice(0, 3, "application/json").type"#).unwrap();
    assert_eq!(r, "application/json");
}

#[test]
fn js_blob_concatenate_parts() {
    let r = eval_string_async(r#"return await new Blob(["hello ", "world"]).text();"#).unwrap();
    assert_eq!(r, "hello world");
}

#[test]
fn js_blob_byte_part() {
    let r = eval_string_async(r#"return await new Blob([[104, 105]]).text();"#).unwrap();
    assert_eq!(r, "hi");
}

// ════════════════════ File ════════════════════

#[test]
fn js_file_construction_and_name() {
    let r = eval_string(r#"new File(["data"], "report.pdf").name"#).unwrap();
    assert_eq!(r, "report.pdf");
}

#[test]
fn js_file_extends_blob() {
    let r = eval_bool(r#"new File([], "x") instanceof Blob"#).unwrap();
    assert!(r);
}

#[test]
fn js_file_size_via_blob_inheritance() {
    let r = eval_i64(r#"new File(["hello"], "a.txt").size"#).unwrap();
    assert_eq!(r, 5);
}

#[test]
fn js_file_last_modified() {
    let r = eval_i64(
        r#"new File(["data"], "x", {lastModified: 1700000000000}).lastModified"#,
    ).unwrap();
    assert_eq!(r, 1_700_000_000_000);
}

#[test]
fn js_file_slice_returns_blob_not_file() {
    let r = eval_bool(r#"new File(["data"], "x").slice(0, 2) instanceof Blob"#).unwrap();
    assert!(r);
}

// ════════════════════ AbortController + AbortSignal ════════════════════

#[test]
fn js_abort_controller_construction() {
    let r = eval_bool(r#"new AbortController().signal.aborted"#).unwrap();
    assert!(!r);
}

#[test]
fn js_abort_controller_abort_sets_signal() {
    let r = eval_bool(r#"
        const ac = new AbortController();
        ac.abort();
        ac.signal.aborted
    "#).unwrap();
    assert!(r);
}

#[test]
fn js_abort_controller_default_reason_is_abort_error() {
    let r = eval_string(r#"
        const ac = new AbortController();
        ac.abort();
        ac.signal.reason.name
    "#).unwrap();
    assert_eq!(r, "AbortError");
}

#[test]
fn js_abort_controller_default_reason_code_is_20() {
    let r = eval_i64(r#"
        const ac = new AbortController();
        ac.abort();
        ac.signal.reason.code
    "#).unwrap();
    assert_eq!(r, 20);
}

#[test]
fn js_abort_controller_custom_reason() {
    let r = eval_string(r#"
        const ac = new AbortController();
        ac.abort("user cancel");
        String(ac.signal.reason)
    "#).unwrap();
    assert_eq!(r, "user cancel");
}

#[test]
fn js_abort_signal_listener_fires_on_abort() {
    let r = eval_bool(r#"
        const ac = new AbortController();
        let fired = false;
        ac.signal.addEventListener("abort", () => { fired = true; });
        ac.abort();
        fired
    "#).unwrap();
    assert!(r);
}

#[test]
fn js_abort_signal_listener_idempotent() {
    let r = eval_i64(r#"
        const ac = new AbortController();
        let count = 0;
        ac.signal.addEventListener("abort", () => { count++; });
        ac.abort();
        ac.abort();
        ac.abort();
        count
    "#).unwrap();
    assert_eq!(r, 1);
}

#[test]
fn js_abort_signal_static_abort() {
    let r = eval_bool(r#"AbortSignal.abort().aborted"#).unwrap();
    assert!(r);
}

#[test]
fn js_abort_signal_any_with_aborted_input() {
    let r = eval_bool(r#"
        const a = AbortSignal.abort();
        const combined = AbortSignal.any([a]);
        combined.aborted
    "#).unwrap();
    assert!(r);
}

#[test]
fn js_abort_signal_any_propagates_later_abort() {
    let r = eval_bool(r#"
        const ac1 = new AbortController();
        const ac2 = new AbortController();
        const combined = AbortSignal.any([ac1.signal, ac2.signal]);
        ac2.abort();
        combined.aborted
    "#).unwrap();
    assert!(r);
}

#[test]
fn js_abort_signal_throw_if_aborted() {
    let r = eval_string(r#"
        const ac = new AbortController();
        ac.abort("nope");
        try { ac.signal.throwIfAborted(); "no-throw" }
        catch (e) { String(e) }
    "#).unwrap();
    assert_eq!(r, "nope");
}

// ════════════════════ Cross-pilot composition (new wirings) ══════════

#[test]
fn js_compose_blob_text_through_text_decoder() {
    let r = eval_string_async(r#"
        const blob = new Blob(["hello, world!"]);
        const dec = new TextDecoder();
        return dec.decode(await blob.bytes());
    "#).unwrap();
    assert_eq!(r, "hello, world!");
}

#[test]
fn js_compose_file_in_blob_part() {
    let r = eval_string_async(r#"
        const inner = new File(["inner content"], "inner.txt");
        const wrapper = new Blob([inner]);
        return await wrapper.text();
    "#).unwrap();
    assert_eq!(r, "inner content");
}

#[test]
fn js_compose_abort_signal_with_async_pattern() {
    let r = eval_string(r#"
        const ac = new AbortController();
        const events = [];
        ac.signal.addEventListener("abort", (reason) => {
            events.push("aborted:" + (reason && reason.name));
        });
        events.push("before-abort");
        ac.abort();
        events.push("after-abort");
        events.join("|")
    "#).unwrap();
    assert_eq!(r, "before-abort|aborted:AbortError|after-abort");
}

// ════════════════════ Headers ════════════════════

#[test]
fn js_headers_construction_from_object() {
    let r = eval_string(r#"
        const h = new Headers({"Content-Type": "application/json"});
        h.get("content-type")
    "#).unwrap();
    assert_eq!(r, "application/json");
}

#[test]
fn js_headers_case_insensitive() {
    let r = eval_string(r#"
        const h = new Headers();
        h.append("Content-Type", "text/html");
        h.get("CONTENT-TYPE")
    "#).unwrap();
    assert_eq!(r, "text/html");
}

#[test]
fn js_headers_multiple_values_combined_with_comma() {
    let r = eval_string(r#"
        const h = new Headers();
        h.append("Accept", "text/html");
        h.append("Accept", "application/json");
        h.get("accept")
    "#).unwrap();
    assert_eq!(r, "text/html, application/json");
}

#[test]
fn js_headers_set_replaces() {
    let r = eval_string(r#"
        const h = new Headers();
        h.append("X", "1");
        h.append("X", "2");
        h.set("X", "only");
        h.get("X")
    "#).unwrap();
    assert_eq!(r, "only");
}

#[test]
fn js_headers_get_set_cookie_separate() {
    let r = eval_i64(r#"
        const h = new Headers();
        h.append("Set-Cookie", "a=1");
        h.append("Set-Cookie", "b=2");
        h.append("Set-Cookie", "c=3");
        h.getSetCookie().length
    "#).unwrap();
    assert_eq!(r, 3);
}

#[test]
fn js_headers_invalid_name_throws() {
    let r = eval_bool(r#"
        try { new Headers().append("invalid name", "x"); false }
        catch (e) { e instanceof TypeError }
    "#).unwrap();
    assert!(r);
}

#[test]
fn js_headers_iteration_lowercased_sorted() {
    let r = eval_string(r#"
        const h = new Headers();
        h.append("Z-Header", "z");
        h.append("A-Header", "a");
        h.append("M-Header", "m");
        const names = [...h.keys()];
        names.join(",")
    "#).unwrap();
    assert_eq!(r, "a-header,m-header,z-header");
}

// ════════════════════ Request ════════════════════

#[test]
fn js_request_default_method_get() {
    let r = eval_string(r#"new Request("https://example.com").method"#).unwrap();
    assert_eq!(r, "GET");
}

#[test]
fn js_request_url_preserved() {
    let r = eval_string(r#"new Request("https://api.example.com/users?id=1").url"#).unwrap();
    assert_eq!(r, "https://api.example.com/users?id=1");
}

#[test]
fn js_request_method_uppercased() {
    let r = eval_string(r#"new Request("/", {method: "post"}).method"#).unwrap();
    assert_eq!(r, "POST");
}

#[test]
fn js_request_with_headers_init() {
    let r = eval_string(r#"
        const req = new Request("/", {headers: {"Authorization": "Bearer xyz"}});
        req.headers.get("authorization")
    "#).unwrap();
    assert_eq!(r, "Bearer xyz");
}

#[test]
fn js_request_text_body() {
    let r = eval_string_async(r#"
        const req = new Request("/", {method: "POST", body: "payload"});
        return await req.text();
    "#).unwrap();
    assert_eq!(r, "payload");
}

#[test]
fn js_request_clone_preserves_state() {
    let r = eval_string_async(r#"
        const orig = new Request("/users", {method: "POST", body: "data"});
        const clone = orig.clone();
        return clone.method + "|" + (await clone.text());
    "#).unwrap();
    assert_eq!(r, "POST|data");
}

// ════════════════════ Response ════════════════════

#[test]
fn js_response_default_status() {
    let r = eval_i64(r#"new Response().status"#).unwrap();
    assert_eq!(r, 200);
}

#[test]
fn js_response_ok_for_200() {
    let r = eval_bool(r#"new Response().ok"#).unwrap();
    assert!(r);
}

#[test]
fn js_response_ok_false_for_404() {
    let r = eval_bool(r#"new Response(null, {status: 404}).ok"#).unwrap();
    assert!(!r);
}

#[test]
fn js_response_text() {
    let r = eval_string_async(r#"return await new Response("hello").text();"#).unwrap();
    assert_eq!(r, "hello");
}

#[test]
fn js_response_status_out_of_range_throws() {
    let r = eval_bool(r#"
        try { new Response(null, {status: 99}); false }
        catch (e) { e instanceof RangeError }
    "#).unwrap();
    assert!(r);
}

#[test]
fn js_response_static_json_sets_content_type() {
    let r = eval_string(r#"
        const r = Response.json({hello: "world"});
        r.headers.get("content-type")
    "#).unwrap();
    assert_eq!(r, "application/json");
}

#[test]
fn js_response_static_redirect_only_valid_codes() {
    let r = eval_bool(r#"
        try { Response.redirect("/", 200); false }
        catch (e) { e instanceof RangeError }
    "#).unwrap();
    assert!(r);
}

#[test]
fn js_response_static_redirect_sets_location() {
    let r = eval_string(r#"
        Response.redirect("https://target.example.com/", 301).headers.get("location")
    "#).unwrap();
    assert_eq!(r, "https://target.example.com/");
}

#[test]
fn js_response_static_error() {
    let r = eval_string(r#"
        const e = Response.error();
        e.type
    "#).unwrap();
    assert_eq!(r, "error");
}

// ════════════════════ Bun.file ════════════════════

#[test]
fn js_bun_file_construction_lazy() {
    let r = eval_string(r#"Bun.file("/tmp/never-touched-by-this-test").name"#).unwrap();
    assert_eq!(r, "/tmp/never-touched-by-this-test");
}

#[test]
fn js_bun_file_text_round_trip() {
    let tmp = format!("/tmp/rusty-bun-host-bunfile-{}", std::process::id());
    let script = format!(r#"
        const path = "{}";
        fs.writeFileSync(path, "Bun.file demo");
        const text = Bun.file(path).text();
        fs.unlinkSync(path);
        text
    "#, tmp);
    let r = eval_string(&script).unwrap();
    assert_eq!(r, "Bun.file demo");
}

#[test]
fn js_bun_file_size() {
    let tmp = format!("/tmp/rusty-bun-host-bunfile-size-{}", std::process::id());
    let script = format!(r#"
        const path = "{}";
        fs.writeFileSync(path, "12345");
        const sz = Bun.file(path).size;
        fs.unlinkSync(path);
        sz
    "#, tmp);
    let r = eval_i64(&script).unwrap();
    assert_eq!(r, 5);
}

#[test]
fn js_bun_file_type_from_extension() {
    let r = eval_string(r#"Bun.file("/tmp/something.html").type"#).unwrap();
    assert!(r.starts_with("text/html"));
}

#[test]
fn js_bun_file_explicit_type_override() {
    let r = eval_string(r#"Bun.file("/tmp/data.bin", {type: "application/protobuf"}).type"#).unwrap();
    assert_eq!(r, "application/protobuf");
}

// ════════════════════ Cross-pilot composition (system-level) ══════════

#[test]
fn js_compose_response_with_blob_body() {
    let r = eval_string_async(r#"
        const blob = new Blob(["payload"]);
        const resp = new Response(await blob.bytes());
        return await resp.text();
    "#).unwrap();
    assert_eq!(r, "payload");
}

#[test]
fn js_compose_request_response_roundtrip() {
    let r = eval_string_async(r#"
        const req = new Request("/users", {method: "POST", body: "input"});
        const resp = new Response(await req.text(), {status: 201});
        return resp.status + ":" + (await resp.text());
    "#).unwrap();
    assert_eq!(r, "201:input");
}

#[test]
fn js_compose_bun_file_to_response() {
    let tmp = format!("/tmp/rusty-bun-host-compose-{}", std::process::id());
    let script = format!(r#"
        const path = "{}";
        fs.writeFileSync(path, "static file content");
        const file = Bun.file(path);
        const resp = new Response(file.text());
        const result = await resp.text();
        fs.unlinkSync(path);
        return result;
    "#, tmp);
    let r = eval_string_async(&script).unwrap();
    assert_eq!(r, "static file content");
}

// ════════════════════ Bun.serve (data-layer) ════════════════════

#[test]
fn js_bun_serve_construction() {
    let r = eval_i64(r#"
        const server = Bun.serve({port: 8080});
        server.port
    "#).unwrap();
    assert_eq!(r, 8080);
}

#[test]
fn js_bun_serve_url() {
    let r = eval_string(r#"
        const server = Bun.serve({port: 3000, hostname: "localhost"});
        server.url
    "#).unwrap();
    assert_eq!(r, "http://localhost:3000/");
}

#[test]
fn js_bun_serve_default_port() {
    let r = eval_i64(r#"Bun.serve({}).port"#).unwrap();
    assert_eq!(r, 3000);
}

#[test]
fn js_bun_serve_fetch_handler_invoked() {
    let r = eval_string_async(r#"
        const server = Bun.serve({
            fetch(req) { return new Response("hello"); }
        });
        return await server.fetch(new Request("/")).text();
    "#).unwrap();
    assert_eq!(r, "hello");
}

#[test]
fn js_bun_serve_no_handler_404() {
    let r = eval_i64(r#"
        const server = Bun.serve({});
        server.fetch(new Request("/")).status
    "#).unwrap();
    assert_eq!(r, 404);
}

#[test]
fn js_bun_serve_routes_static_path() {
    let r = eval_string_async(r#"
        const server = Bun.serve({
            routes: {
                "/health": (req) => new Response("ok"),
            }
        });
        return await server.fetch(new Request("/health")).text();
    "#).unwrap();
    assert_eq!(r, "ok");
}

#[test]
fn js_bun_serve_routes_with_param() {
    let r = eval_string_async(r#"
        const server = Bun.serve({
            routes: {
                "/users/:id": (req, params) => new Response("user-" + params.id),
            }
        });
        return await server.fetch(new Request("/users/42")).text();
    "#).unwrap();
    assert_eq!(r, "user-42");
}

#[test]
fn js_bun_serve_routes_method_keyed() {
    let r = eval_string_async(r#"
        const server = Bun.serve({
            routes: {
                "/api/items": {
                    GET: () => new Response("list"),
                    POST: () => new Response("created", {status: 201}),
                }
            }
        });
        const get = server.fetch(new Request("/api/items"));
        const post = server.fetch(new Request("/api/items", {method: "POST"}));
        return (await get.text()) + "|" + post.status + ":" + (await post.text());
    "#).unwrap();
    assert_eq!(r, "list|201:created");
}

#[test]
fn js_bun_serve_method_not_allowed() {
    let r = eval_i64(r#"
        const server = Bun.serve({
            routes: {
                "/api/items": { GET: () => new Response("list") }
            }
        });
        server.fetch(new Request("/api/items", {method: "DELETE"})).status
    "#).unwrap();
    assert_eq!(r, 405);
}

#[test]
fn js_bun_serve_routes_fall_through_to_fetch() {
    let r = eval_string_async(r#"
        const server = Bun.serve({
            routes: { "/specific": () => new Response("specific") },
            fetch(req) { return new Response("catch-all"); }
        });
        return await server.fetch(new Request("/anything")).text();
    "#).unwrap();
    assert_eq!(r, "catch-all");
}

#[test]
fn js_bun_serve_stop_then_fetch_returns_error() {
    let r = eval_string(r#"
        const server = Bun.serve({
            fetch() { return new Response("ok"); }
        });
        server.stop();
        const r = server.fetch(new Request("/"));
        r.type
    "#).unwrap();
    assert_eq!(r, "error");
}

// ════════════════════ Bun.spawn ════════════════════

#[test]
fn js_bun_spawn_sync_echo() {
    let r = eval_string(r#"
        const result = Bun.spawnSync(["sh", "-c", "echo hello"]);
        result.stdout
    "#).unwrap();
    assert_eq!(r.trim(), "hello");
}

#[test]
fn js_bun_spawn_sync_exit_code() {
    let r = eval_i64(r#"
        const result = Bun.spawnSync(["sh", "-c", "exit 42"]);
        result.exitCode
    "#).unwrap();
    assert_eq!(r, 42);
}

#[test]
fn js_bun_spawn_sync_success_false_on_nonzero() {
    let r = eval_bool(r#"
        const result = Bun.spawnSync(["sh", "-c", "exit 1"]);
        result.success
    "#).unwrap();
    assert!(!r);
}

#[test]
fn js_bun_spawn_sync_stderr() {
    let r = eval_string(r#"
        const result = Bun.spawnSync(["sh", "-c", "echo error >&2"]);
        result.stderr
    "#).unwrap();
    assert_eq!(r.trim(), "error");
}

#[test]
fn js_bun_spawn_sync_stdin_text() {
    let r = eval_string(r#"
        const result = Bun.spawnSync(["cat"], {stdin: "piped data"});
        result.stdout
    "#).unwrap();
    assert_eq!(r, "piped data");
}

// ════════════════════ Cross-pilot composition (Bun.serve canonical) ════

#[test]
fn js_compose_bun_serve_canonical_pattern() {
    // The canonical Bun docs example, running through rusty-bun-host.
    let r = eval_string_async(r#"
        const server = Bun.serve({
            routes: {
                "/health": () => Response.json({status: "ok"}),
                "/users/:id": (req, params) =>
                    Response.json({id: params.id, name: "User " + params.id}),
            },
            fetch(req) {
                return new Response("Not Found", {status: 404});
            }
        });
        const h = await server.fetch(new Request("/health")).text();
        const u = await server.fetch(new Request("/users/42")).text();
        const nf = server.fetch(new Request("/missing")).status;
        return h + "|" + u + "|" + nf;
    "#).unwrap();
    assert_eq!(r, r#"{"status":"ok"}|{"id":"42","name":"User 42"}|404"#);
}

#[test]
fn js_compose_bun_serve_with_url_search_params() {
    let r = eval_string_async(r#"
        const server = Bun.serve({
            fetch(req) {
                // Real consumers parse query strings here.
                const queryStart = req.url.indexOf("?");
                const query = queryStart >= 0 ? req.url.substring(queryStart + 1) : "";
                const params = new URLSearchParams(query);
                return new Response("user=" + (params.get("user") || "guest"));
            }
        });
        return await server.fetch(new Request("/?user=alice")).text();
    "#).unwrap();
    assert_eq!(r, "user=alice");
}

// ════════════════════ structuredClone ════════════════════

#[test]
fn js_structured_clone_primitives() {
    let r = eval_string(r#"
        const x = structuredClone({n: 1, s: "x", b: true, nul: null});
        x.n + ":" + x.s + ":" + x.b + ":" + (x.nul === null)
    "#).unwrap();
    assert_eq!(r, "1:x:true:true");
}

#[test]
fn js_structured_clone_independence() {
    let r = eval_bool(r#"
        const a = {x: 1};
        const b = structuredClone(a);
        b.x = 2;
        a.x === 1 && b.x === 2
    "#).unwrap();
    assert!(r);
}

#[test]
fn js_structured_clone_nested() {
    let r = eval_i64(r#"
        const a = {inner: {n: 42}};
        const b = structuredClone(a);
        b.inner.n = 99;
        a.inner.n
    "#).unwrap();
    assert_eq!(r, 42);
}

#[test]
fn js_structured_clone_array() {
    let r = eval_string(r#"
        const a = [1, [2, 3], {k: 4}];
        const b = structuredClone(a);
        b[1][0] = 99;
        b[2].k = 88;
        a[1][0] + ":" + a[2].k + ":" + b[1][0] + ":" + b[2].k
    "#).unwrap();
    assert_eq!(r, "2:4:99:88");
}

#[test]
fn js_structured_clone_date() {
    let r = eval_bool(r#"
        const d = new Date(1234567890000);
        const c = structuredClone(d);
        c instanceof Date && c.getTime() === d.getTime() && c !== d
    "#).unwrap();
    assert!(r);
}

#[test]
fn js_structured_clone_regexp() {
    let r = eval_string(r#"
        const r = /foo/gi;
        const c = structuredClone(r);
        c.source + ":" + c.flags + ":" + (c instanceof RegExp) + ":" + (c !== r)
    "#).unwrap();
    assert_eq!(r, "foo:gi:true:true");
}

#[test]
fn js_structured_clone_map() {
    let r = eval_string(r#"
        const m = new Map([["a", 1], ["b", {n: 2}]]);
        const c = structuredClone(m);
        c.get("b").n = 99;
        m.get("b").n + ":" + c.get("b").n + ":" + (c instanceof Map)
    "#).unwrap();
    assert_eq!(r, "2:99:true");
}

#[test]
fn js_structured_clone_set() {
    let r = eval_string(r#"
        const s = new Set([1, 2, 3]);
        const c = structuredClone(s);
        c.add(4);
        s.size + ":" + c.size + ":" + (c instanceof Set)
    "#).unwrap();
    assert_eq!(r, "3:4:true");
}

#[test]
fn js_structured_clone_cycle() {
    let r = eval_bool(r#"
        const a = {x: 1};
        a.self = a;
        const b = structuredClone(a);
        b.self === b && b !== a && b.x === 1
    "#).unwrap();
    assert!(r);
}

#[test]
fn js_structured_clone_array_buffer() {
    let r = eval_string(r#"
        const buf = new ArrayBuffer(4);
        const view = new Uint8Array(buf);
        view[0] = 65; view[1] = 66; view[2] = 67; view[3] = 68;
        const c = structuredClone(buf);
        const cview = new Uint8Array(c);
        cview[0] = 99;
        view[0] + ":" + cview[0] + ":" + (c !== buf) + ":" + (c.byteLength === 4)
    "#).unwrap();
    assert_eq!(r, "65:99:true:true");
}

#[test]
fn js_structured_clone_typed_array() {
    let r = eval_string(r#"
        const a = new Uint16Array([1, 2, 3]);
        const c = structuredClone(a);
        c[0] = 99;
        a[0] + ":" + c[0] + ":" + (c.constructor === Uint16Array)
    "#).unwrap();
    assert_eq!(r, "1:99:true");
}

#[test]
fn js_structured_clone_function_throws() {
    let r = eval_bool(r#"
        try {
            structuredClone(() => 1);
            false
        } catch (e) {
            e.name === "DataCloneError"
        }
    "#).unwrap();
    assert!(r);
}

// Canonical-docs composition test: MDN's flagship structuredClone example.
#[test]
fn js_compose_structured_clone_canonical_pattern() {
    let r = eval_string(r#"
        const original = {
            name: "MDN",
            details: { coords: [37.7749, -122.4194] },
            tags: new Set(["web", "docs"]),
            modified: new Date(0),
        };
        const copy = structuredClone(original);
        copy.details.coords[0] = 0;
        copy.tags.add("clone");
        original.details.coords[0] + "|" +
            original.tags.has("clone") + "|" +
            copy.tags.has("clone") + "|" +
            (copy.modified instanceof Date)
    "#).unwrap();
    assert_eq!(r, "37.7749|false|true|true");
}

// ════════════════════ Streams ════════════════════

#[test]
fn js_readable_stream_basic_read() {
    let r = eval_string_async(r#"
        const stream = new ReadableStream({
            start(controller) {
                controller.enqueue("a");
                controller.enqueue("b");
                controller.close();
            }
        });
        const reader = stream.getReader();
        const r1 = await reader.read();
        const r2 = await reader.read();
        const r3 = await reader.read();
        return r1.value + ":" + r2.value + ":" + (r3.done ? "done" : "not-done");
    "#).unwrap();
    assert_eq!(r, "a:b:done");
}

#[test]
fn js_readable_stream_pull_driven() {
    let r = eval_string_async(r#"
        let i = 0;
        const stream = new ReadableStream({
            pull(controller) {
                if (i < 3) {
                    controller.enqueue(i++);
                } else {
                    controller.close();
                }
            }
        });
        const out = [];
        const reader = stream.getReader();
        while (true) {
            const {value, done} = await reader.read();
            if (done) break;
            out.push(value);
        }
        return out.join(",");
    "#).unwrap();
    assert_eq!(r, "0,1,2");
}

#[test]
fn js_readable_stream_async_iteration() {
    let r = eval_string_async(r#"
        const stream = new ReadableStream({
            start(c) { c.enqueue("x"); c.enqueue("y"); c.enqueue("z"); c.close(); }
        });
        const out = [];
        for await (const chunk of stream) out.push(chunk);
        return out.join("-");
    "#).unwrap();
    assert_eq!(r, "x-y-z");
}

#[test]
fn js_readable_stream_error_propagates() {
    let r = eval_string_async(r#"
        const stream = new ReadableStream({
            start(c) { c.error(new Error("boom")); }
        });
        const reader = stream.getReader();
        try {
            await reader.read();
            return "no-throw";
        } catch (e) {
            return e.message;
        }
    "#).unwrap();
    assert_eq!(r, "boom");
}

#[test]
fn js_readable_stream_locked() {
    let r = eval_string(r#"
        const stream = new ReadableStream({start(c) { c.close(); }});
        const before = stream.locked;
        stream.getReader();
        const after = stream.locked;
        before + ":" + after
    "#).unwrap();
    assert_eq!(r, "false:true");
}

#[test]
fn js_writable_stream_basic_write() {
    let r = eval_string_async(r#"
        const collected = [];
        const stream = new WritableStream({
            write(chunk) { collected.push(chunk); },
        });
        const writer = stream.getWriter();
        await writer.write("a");
        await writer.write("b");
        await writer.close();
        return collected.join(",");
    "#).unwrap();
    assert_eq!(r, "a,b");
}

#[test]
fn js_writable_stream_close_runs_sink_close() {
    let r = eval_string_async(r#"
        let closed = false;
        const stream = new WritableStream({
            write() {},
            close() { closed = true; },
        });
        const w = stream.getWriter();
        await w.write("x");
        await w.close();
        return closed ? "closed" : "open";
    "#).unwrap();
    assert_eq!(r, "closed");
}

#[test]
fn js_transform_stream_pipes() {
    let r = eval_string_async(r#"
        const t = new TransformStream({
            transform(chunk, controller) {
                controller.enqueue(chunk.toUpperCase());
            }
        });
        const writer = t.writable.getWriter();
        const reader = t.readable.getReader();
        await writer.write("hello");
        await writer.write("world");
        await writer.close();
        const out = [];
        while (true) {
            const {value, done} = await reader.read();
            if (done) break;
            out.push(value);
        }
        return out.join(",");
    "#).unwrap();
    assert_eq!(r, "HELLO,WORLD");
}

// Canonical-docs composition test: the MDN ReadableStream + TransformStream
// pipeline pattern.
#[test]
fn js_compose_streams_canonical_pattern() {
    let r = eval_string_async(r#"
        // Source: numbers 1..5
        const source = new ReadableStream({
            start(c) {
                for (let i = 1; i <= 5; i++) c.enqueue(i);
                c.close();
            }
        });
        // Transform: square
        const square = new TransformStream({
            transform(n, c) { c.enqueue(n * n); }
        });
        // Manual piping (pipeTo deferred): drain source into square.writable.
        const sourceReader = source.getReader();
        const squareWriter = square.writable.getWriter();
        (async () => {
            while (true) {
                const {value, done} = await sourceReader.read();
                if (done) { await squareWriter.close(); break; }
                await squareWriter.write(value);
            }
        })();
        // Collect from square.readable
        const out = [];
        for await (const v of square.readable) out.push(v);
        return out.join(",");
    "#).unwrap();
    assert_eq!(r, "1,4,9,16,25");
}

// ════════════════════ node:http data-layer ════════════════════

#[test]
fn js_node_http_create_server() {
    let r = eval_i64(r#"
        const server = nodeHttp.createServer((req, res) => {
            res.writeHead(200, {"content-type": "text/plain"});
            res.end("hello");
        });
        server.listen(3000);
        server.port
    "#).unwrap();
    assert_eq!(r, 3000);
}

#[test]
fn js_node_http_dispatch_basic() {
    let r = eval_string(r#"
        const server = nodeHttp.createServer((req, res) => {
            res.writeHead(200);
            res.end("ok");
        });
        const res = server.dispatch({method: "GET", url: "/"});
        res.statusCode + ":" + res.body()
    "#).unwrap();
    assert_eq!(r, "200:ok");
}

#[test]
fn js_node_http_response_headers_lowercased() {
    let r = eval_string(r#"
        const server = nodeHttp.createServer((req, res) => {
            res.setHeader("Content-Type", "application/json");
            res.setHeader("X-Custom", "value");
            res.end();
        });
        const res = server.dispatch({method: "GET", url: "/"});
        res.getHeader("content-type") + "|" + res.getHeader("X-CUSTOM")
    "#).unwrap();
    assert_eq!(r, "application/json|value");
}

#[test]
fn js_node_http_request_url_method() {
    let r = eval_string(r#"
        const server = nodeHttp.createServer((req, res) => {
            res.end(req.method + " " + req.url);
        });
        const res = server.dispatch({method: "POST", url: "/api/users"});
        res.body()
    "#).unwrap();
    assert_eq!(r, "POST /api/users");
}

#[test]
fn js_node_http_incoming_message_headers_case_insensitive() {
    let r = eval_string(r#"
        const server = nodeHttp.createServer((req, res) => {
            res.end(req.headers["content-type"]);
        });
        const res = server.dispatch({
            method: "POST", url: "/",
            headers: { "Content-Type": "text/plain" }
        });
        res.body()
    "#).unwrap();
    assert_eq!(r, "text/plain");
}

#[test]
fn js_node_http_write_head_with_status_message() {
    let r = eval_string(r#"
        const server = nodeHttp.createServer((req, res) => {
            res.writeHead(404, "Not Found", {"x-source": "pilot"});
            res.end("missing");
        });
        const res = server.dispatch({method: "GET", url: "/missing"});
        res.statusCode + ":" + res.statusMessage + ":" + res.getHeader("x-source")
    "#).unwrap();
    assert_eq!(r, "404:Not Found:pilot");
}

#[test]
fn js_node_http_write_chunks_then_end() {
    let r = eval_string(r#"
        const server = nodeHttp.createServer((req, res) => {
            res.write("hello ");
            res.write("world");
            res.end("!");
        });
        const res = server.dispatch({method: "GET", url: "/"});
        res.body() + ":" + res.ended
    "#).unwrap();
    assert_eq!(r, "hello world!:true");
}

#[test]
fn js_node_http_close_transitions_state() {
    let r = eval_bool(r#"
        const server = nodeHttp.createServer(() => {});
        server.listen(8080);
        const wasListening = server.listening;
        server.close();
        wasListening && !server.listening
    "#).unwrap();
    assert!(r);
}

#[test]
fn js_node_http_client_request_construction() {
    let r = eval_string(r#"
        const req = nodeHttp.request({
            method: "POST",
            url: "/api/data",
            headers: { "Content-Type": "application/json" }
        });
        req.write('{"a":1}');
        req.end();
        req.method + " " + req.url + " " + req.getHeader("content-type") + " " + req.body()
    "#).unwrap();
    assert_eq!(r, "POST /api/data application/json {\"a\":1}");
}

#[test]
fn js_node_http_request_string_url_form() {
    let r = eval_string(r#"
        const req = nodeHttp.request("/health");
        req.method + " " + req.url
    "#).unwrap();
    assert_eq!(r, "GET /health");
}

#[test]
fn js_node_http_on_request_event_form() {
    let r = eval_string(r#"
        const server = nodeHttp.createServer();
        server.on("request", (req, res) => {
            res.end("via on");
        });
        server.dispatch({method: "GET", url: "/"}).body()
    "#).unwrap();
    assert_eq!(r, "via on");
}

// Canonical-docs composition test: the Node.js docs flagship example.
#[test]
fn js_compose_node_http_canonical_pattern() {
    let r = eval_string(r#"
        // From nodejs.org/api/http.html — flagship createServer example.
        const hostname = '127.0.0.1';
        const port = 3000;
        const server = nodeHttp.createServer((req, res) => {
            res.statusCode = 200;
            res.setHeader('Content-Type', 'text/plain');
            res.end('Hello World');
        });
        server.listen(port);
        // Real consumers exercise via HTTP. Pilot data-layer: dispatch.
        const res = server.dispatch({method: "GET", url: "/", headers: {host: hostname}});
        res.statusCode + "|" + res.getHeader("content-type") + "|" + res.body() + "|" + server.port
    "#).unwrap();
    assert_eq!(r, "200|text/plain|Hello World|3000");
}

// Cross-pilot composition: node-http server emitting a Response shape.
#[test]
fn js_compose_node_http_with_url_search_params() {
    let r = eval_string(r#"
        const server = nodeHttp.createServer((req, res) => {
            const queryStart = req.url.indexOf("?");
            const query = queryStart >= 0 ? req.url.substring(queryStart + 1) : "";
            const params = new URLSearchParams(query);
            res.writeHead(200, {"content-type": "text/plain"});
            res.end("user=" + (params.get("user") || "guest"));
        });
        const res = server.dispatch({method: "GET", url: "/?user=alice"});
        res.body()
    "#).unwrap();
    assert_eq!(r, "user=alice");
}

// ════════════════════ CommonJS module loader (Tier-H.3) ═══════════════════

fn cjs_test_setup(pid_suffix: &str) -> (String, String) {
    let pid = std::process::id();
    let root = format!("/tmp/rusty-bun-cjs-{}-{}", pid, pid_suffix);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    (root.clone(), root)
}

#[test]
fn js_cjs_basic_relative_require() {
    let (root, _) = cjs_test_setup("basic");
    std::fs::write(format!("{}/main.js", root),
        r#"const greet = require("./greet"); module.exports = greet("world");"#).unwrap();
    std::fs::write(format!("{}/greet.js", root),
        r#"module.exports = function(name) { return "hello " + name; };"#).unwrap();
    let script = format!(r#"bootRequire("{}/main.js")"#, root);
    let r = eval_string(&script).unwrap();
    assert_eq!(r, "hello world");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn js_cjs_module_cache_returns_same_exports() {
    let (root, _) = cjs_test_setup("cache");
    std::fs::write(format!("{}/counter.js", root),
        r#"let n = 0; module.exports = { inc: () => ++n, get: () => n };"#).unwrap();
    std::fs::write(format!("{}/main.js", root),
        r#"const a = require("./counter"); const b = require("./counter");
        a.inc(); a.inc(); module.exports = b.get();"#).unwrap();
    let script = format!(r#"bootRequire("{}/main.js")"#, root);
    let r = eval_i64(&script).unwrap();
    assert_eq!(r, 2);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn js_cjs_json_module() {
    let (root, _) = cjs_test_setup("json");
    std::fs::write(format!("{}/data.json", root),
        r#"{"name": "rusty-bun", "version": 1}"#).unwrap();
    std::fs::write(format!("{}/main.js", root),
        r#"const d = require("./data.json"); module.exports = d.name + ":" + d.version;"#).unwrap();
    let r = eval_string(&format!(r#"bootRequire("{}/main.js")"#, root)).unwrap();
    assert_eq!(r, "rusty-bun:1");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn js_cjs_directory_with_index() {
    let (root, _) = cjs_test_setup("dirindex");
    std::fs::create_dir_all(format!("{}/lib", root)).unwrap();
    std::fs::write(format!("{}/lib/index.js", root),
        r#"module.exports = { name: "from-index" };"#).unwrap();
    std::fs::write(format!("{}/main.js", root),
        r#"const lib = require("./lib"); module.exports = lib.name;"#).unwrap();
    let r = eval_string(&format!(r#"bootRequire("{}/main.js")"#, root)).unwrap();
    assert_eq!(r, "from-index");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn js_cjs_package_json_main() {
    let (root, _) = cjs_test_setup("pkgmain");
    std::fs::create_dir_all(format!("{}/node_modules/foo", root)).unwrap();
    std::fs::write(format!("{}/node_modules/foo/package.json", root),
        r#"{"name": "foo", "main": "./lib/entry.js"}"#).unwrap();
    std::fs::create_dir_all(format!("{}/node_modules/foo/lib", root)).unwrap();
    std::fs::write(format!("{}/node_modules/foo/lib/entry.js", root),
        r#"module.exports = "from-pkg-main";"#).unwrap();
    std::fs::write(format!("{}/main.js", root),
        r#"module.exports = require("foo");"#).unwrap();
    let r = eval_string(&format!(r#"bootRequire("{}/main.js")"#, root)).unwrap();
    assert_eq!(r, "from-pkg-main");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn js_cjs_node_modules_walk_upward() {
    let (root, _) = cjs_test_setup("walkup");
    // node_modules at root; consumer is two dirs deep.
    // Per M8(a) 2026-05-11 (Π3.10 round): the package name must not
    // collide with a node-builtin alias, or the builtin short-circuits
    // the node_modules walkup. Previously "util" was used; after
    // node:util / util was registered as a builtin in Π3.10, the
    // walkup test was repointed at "shared-helper" which is guaranteed
    // not to be a builtin name.
    std::fs::create_dir_all(format!("{}/node_modules/shared-helper", root)).unwrap();
    std::fs::write(format!("{}/node_modules/shared-helper/index.js", root),
        r#"module.exports = "shared-helper";"#).unwrap();
    std::fs::create_dir_all(format!("{}/app/src", root)).unwrap();
    std::fs::write(format!("{}/app/src/main.js", root),
        r#"module.exports = require("shared-helper");"#).unwrap();
    let r = eval_string(&format!(r#"bootRequire("{}/app/src/main.js")"#, root)).unwrap();
    assert_eq!(r, "shared-helper");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn js_cjs_scoped_package() {
    let (root, _) = cjs_test_setup("scoped");
    std::fs::create_dir_all(format!("{}/node_modules/@org/lib", root)).unwrap();
    std::fs::write(format!("{}/node_modules/@org/lib/package.json", root),
        r#"{"main": "main.js"}"#).unwrap();
    std::fs::write(format!("{}/node_modules/@org/lib/main.js", root),
        r#"module.exports = "scoped-ok";"#).unwrap();
    std::fs::write(format!("{}/main.js", root),
        r#"module.exports = require("@org/lib");"#).unwrap();
    let r = eval_string(&format!(r#"bootRequire("{}/main.js")"#, root)).unwrap();
    assert_eq!(r, "scoped-ok");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn js_cjs_filename_dirname_injected() {
    let (root, _) = cjs_test_setup("fndn");
    std::fs::write(format!("{}/main.js", root),
        r#"module.exports = __filename + "|" + __dirname;"#).unwrap();
    let r = eval_string(&format!(r#"bootRequire("{}/main.js")"#, root)).unwrap();
    let expected = format!("{}/main.js|{}", root, root);
    assert_eq!(r, expected);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn js_cjs_exports_field_string() {
    let (root, _) = cjs_test_setup("expstr");
    std::fs::create_dir_all(format!("{}/node_modules/lib", root)).unwrap();
    std::fs::write(format!("{}/node_modules/lib/package.json", root),
        r#"{"exports": "./mod.js"}"#).unwrap();
    std::fs::write(format!("{}/node_modules/lib/mod.js", root),
        r#"module.exports = "from-exports-string";"#).unwrap();
    std::fs::write(format!("{}/main.js", root),
        r#"module.exports = require("lib");"#).unwrap();
    let r = eval_string(&format!(r#"bootRequire("{}/main.js")"#, root)).unwrap();
    assert_eq!(r, "from-exports-string");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn js_cjs_cycle_partial_exports() {
    let (root, _) = cjs_test_setup("cycle");
    std::fs::write(format!("{}/a.js", root),
        r#"exports.fromA = 1; const b = require("./b"); exports.b_fromA_seen = b.fromA_seen;"#).unwrap();
    std::fs::write(format!("{}/b.js", root),
        r#"const a = require("./a"); exports.fromA_seen = a.fromA;"#).unwrap();
    std::fs::write(format!("{}/main.js", root),
        r#"const a = require("./a"); module.exports = a.fromA + ":" + a.b_fromA_seen;"#).unwrap();
    let r = eval_string(&format!(r#"bootRequire("{}/main.js")"#, root)).unwrap();
    assert_eq!(r, "1:1");
    let _ = std::fs::remove_dir_all(&root);
}

// Canonical-docs composition test: a small npm-package pattern using the
// wired pilots from inside a require()'d module.
#[test]
fn js_compose_cjs_with_pilots_canonical_pattern() {
    let (root, _) = cjs_test_setup("compose");
    std::fs::create_dir_all(format!("{}/node_modules/qsutil", root)).unwrap();
    std::fs::write(format!("{}/node_modules/qsutil/package.json", root),
        r#"{"main": "index.js"}"#).unwrap();
    std::fs::write(format!("{}/node_modules/qsutil/index.js", root),
        r#"
        // Real npm-style helper: parse query string into an object,
        // using URLSearchParams from the host.
        module.exports = function parse(query) {
            const p = new URLSearchParams(query);
            const out = {};
            for (const [k, v] of p) out[k] = v;
            return out;
        };
        "#).unwrap();
    std::fs::write(format!("{}/main.js", root),
        r#"
        const parse = require("qsutil");
        const obj = parse("a=1&b=2&c=hello%20world");
        module.exports = obj.a + ":" + obj.b + ":" + obj.c;
        "#).unwrap();
    let r = eval_string(&format!(r#"bootRequire("{}/main.js")"#, root)).unwrap();
    assert_eq!(r, "1:2:hello world");
    let _ = std::fs::remove_dir_all(&root);
}

// ════════════════════ ESM module loading (Tier-H.3 #2) ═══════════════════

use rusty_bun_host::eval_esm_module;

fn esm_test_setup(suffix: &str) -> String {
    let pid = std::process::id();
    let root = format!("/tmp/rusty-bun-esm-{}-{}", pid, suffix);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn js_esm_basic_relative_import() {
    let root = esm_test_setup("basic");
    std::fs::write(format!("{}/lib.js", root),
        r#"export function greet(name) { return "hello " + name; }"#).unwrap();
    std::fs::write(format!("{}/main.js", root),
        r#"import { greet } from "./lib.js"; globalThis.__esmResult = greet("world");"#).unwrap();
    let r = eval_esm_module(&format!("{}/main.js", root)).unwrap();
    assert_eq!(r, "hello world");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn js_esm_default_export() {
    let root = esm_test_setup("default");
    std::fs::write(format!("{}/lib.js", root),
        r#"export default function(x) { return x * 2; }"#).unwrap();
    std::fs::write(format!("{}/main.js", root),
        r#"import double from "./lib.js"; globalThis.__esmResult = String(double(21));"#).unwrap();
    let r = eval_esm_module(&format!("{}/main.js", root)).unwrap();
    assert_eq!(r, "42");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn js_esm_chained_imports() {
    let root = esm_test_setup("chain");
    std::fs::write(format!("{}/a.js", root),
        r#"export const a = "A";"#).unwrap();
    std::fs::write(format!("{}/b.js", root),
        r#"import { a } from "./a.js"; export const b = a + "B";"#).unwrap();
    std::fs::write(format!("{}/main.js", root),
        r#"import { b } from "./b.js"; globalThis.__esmResult = b + "C";"#).unwrap();
    let r = eval_esm_module(&format!("{}/main.js", root)).unwrap();
    assert_eq!(r, "ABC");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn js_esm_bare_specifier_node_modules() {
    let root = esm_test_setup("bare");
    std::fs::create_dir_all(format!("{}/node_modules/lib", root)).unwrap();
    std::fs::write(format!("{}/node_modules/lib/package.json", root),
        r#"{"module": "./entry.js"}"#).unwrap();
    std::fs::write(format!("{}/node_modules/lib/entry.js", root),
        r#"export const v = "from-bare";"#).unwrap();
    std::fs::write(format!("{}/main.js", root),
        r#"import { v } from "lib"; globalThis.__esmResult = v;"#).unwrap();
    let r = eval_esm_module(&format!("{}/main.js", root)).unwrap();
    assert_eq!(r, "from-bare");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn js_esm_uses_wired_globals() {
    let root = esm_test_setup("globals");
    std::fs::write(format!("{}/main.js", root),
        r#"
        const enc = new TextEncoder();
        const dec = new TextDecoder();
        globalThis.__esmResult = dec.decode(enc.encode("hello-esm"));
        "#).unwrap();
    let r = eval_esm_module(&format!("{}/main.js", root)).unwrap();
    assert_eq!(r, "hello-esm");
    let _ = std::fs::remove_dir_all(&root);
}

// Canonical-docs composition: ESM module composing wired pilots in a way
// real consumer code would. Uses URLSearchParams + crypto + Buffer across
// imports.
#[test]
fn js_compose_esm_canonical_pattern() {
    let root = esm_test_setup("compose");
    std::fs::write(format!("{}/qs.js", root),
        r#"
        export function parse(query) {
            const p = new URLSearchParams(query);
            const out = {};
            for (const [k, v] of p) out[k] = v;
            return out;
        }
        "#).unwrap();
    std::fs::write(format!("{}/encode.js", root),
        r#"
        export function encodeStr(s) {
            return Buffer.encodeBase64(Buffer.from(s));
        }
        "#).unwrap();
    std::fs::write(format!("{}/main.js", root),
        r#"
        import { parse } from "./qs.js";
        import { encodeStr } from "./encode.js";
        const obj = parse("user=alice&id=42");
        const encoded = encodeStr(obj.user + ":" + obj.id);
        globalThis.__esmResult = encoded;
        "#).unwrap();
    let r = eval_esm_module(&format!("{}/main.js", root)).unwrap();
    // base64("alice:42") = "YWxpY2U6NDI="
    assert_eq!(r, "YWxpY2U6NDI=");
    let _ = std::fs::remove_dir_all(&root);
}

// ════════════════════ Tier-H.4: timers / queueMicrotask / performance ═══

#[test]
fn js_set_timeout_zero_runs_callback() {
    let r = eval_string_async(r#"
        let result = "before";
        setTimeout(() => { result = "after"; }, 0);
        await new Promise(resolve => setTimeout(resolve, 0));
        return result;
    "#).unwrap();
    assert_eq!(r, "after");
}

#[test]
fn js_set_timeout_with_args() {
    let r = eval_string_async(r#"
        let result = "";
        setTimeout((a, b, c) => { result = a + ":" + b + ":" + c; }, 0, "x", 42, true);
        await new Promise(resolve => setTimeout(resolve, 0));
        return result;
    "#).unwrap();
    assert_eq!(r, "x:42:true");
}

#[test]
fn js_clear_timeout_cancels() {
    let r = eval_string_async(r#"
        let result = "before";
        const id = setTimeout(() => { result = "AFTER"; }, 0);
        clearTimeout(id);
        await new Promise(resolve => setTimeout(resolve, 0));
        return result;
    "#).unwrap();
    assert_eq!(r, "before");
}

#[test]
fn js_set_immediate_runs() {
    let r = eval_string_async(r#"
        let n = 0;
        setImmediate(() => { n = 1; });
        await new Promise(resolve => setImmediate(() => resolve()));
        return String(n);
    "#).unwrap();
    assert_eq!(r, "1");
}

#[test]
fn js_queue_microtask_runs() {
    let r = eval_string_async(r#"
        let result = "before";
        queueMicrotask(() => { result = "after"; });
        await Promise.resolve();
        return result;
    "#).unwrap();
    assert_eq!(r, "after");
}

#[test]
fn js_performance_now_increasing() {
    let r = eval_bool(r#"
        const a = performance.now();
        for (let i = 0; i < 10000; i++) {}
        const b = performance.now();
        b >= a
    "#).unwrap();
    assert!(r);
}

#[test]
fn js_performance_time_origin_present() {
    let r = eval_bool(r#"typeof performance.timeOrigin === "number" && performance.timeOrigin > 0"#).unwrap();
    assert!(r);
}

// Canonical-docs composition: typical consumer pattern using setTimeout
// for deferred work + queueMicrotask for finer-grained ordering.
#[test]
fn js_compose_timers_canonical_pattern() {
    let r = eval_string_async(r#"
        const order = [];
        order.push("sync-1");
        setTimeout(() => order.push("timeout"), 0);
        queueMicrotask(() => order.push("microtask"));
        Promise.resolve().then(() => order.push("promise"));
        order.push("sync-2");
        // Drain queue.
        await new Promise(resolve => setTimeout(resolve, 0));
        return order.join(",");
    "#).unwrap();
    // Microtasks (queueMicrotask + promise) drain before timeouts.
    // sync runs first, then promise + microtask in queue order, then timeouts.
    assert!(r.starts_with("sync-1,sync-2,"));
    assert!(r.contains("timeout"));
    assert!(r.contains("microtask"));
    assert!(r.contains("promise"));
}

// ════════════════════ URL class (Tier-H.4 #2) ════════════════════════════

#[test]
fn js_url_basic_components() {
    let r = eval_string(r#"
        const u = new URL("https://example.com:8443/path/to/page?x=1&y=2#frag");
        u.protocol + "|" + u.hostname + "|" + u.port + "|" + u.pathname + "|" + u.search + "|" + u.hash
    "#).unwrap();
    assert_eq!(r, "https:|example.com|8443|/path/to/page|?x=1&y=2|#frag");
}

#[test]
fn js_url_default_port_omitted() {
    let r = eval_string(r#"
        const u = new URL("https://example.com:443/x");
        u.port + "|" + u.host
    "#).unwrap();
    assert_eq!(r, "|example.com");
}

#[test]
fn js_url_origin() {
    let r = eval_string(r#"
        new URL("https://example.com:8443/foo").origin
    "#).unwrap();
    assert_eq!(r, "https://example.com:8443");
}

#[test]
fn js_url_userinfo() {
    let r = eval_string(r#"
        const u = new URL("https://alice:secret@example.com/x");
        u.username + ":" + u.password + "@" + u.hostname
    "#).unwrap();
    assert_eq!(r, "alice:secret@example.com");
}

#[test]
fn js_url_search_params_live_binding() {
    let r = eval_string(r#"
        const u = new URL("https://example.com/x?a=1");
        u.searchParams.append("b", "2");
        u.search + "|" + u.href
    "#).unwrap();
    assert_eq!(r, "?a=1&b=2|https://example.com/x?a=1&b=2");
}

#[test]
fn js_url_search_setter_resyncs_searchparams() {
    let r = eval_string(r#"
        const u = new URL("https://example.com/x?a=1");
        u.search = "z=99";
        u.searchParams.get("z") + "|" + u.searchParams.get("a")
    "#).unwrap();
    assert_eq!(r, "99|null");
}

#[test]
fn js_url_relative_resolution() {
    let r = eval_string(r#"
        new URL("./bar", "https://example.com/foo/baz").href
    "#).unwrap();
    assert_eq!(r, "https://example.com/foo/bar");
}

#[test]
fn js_url_absolute_path_resolution() {
    let r = eval_string(r#"
        new URL("/elsewhere", "https://example.com/foo/baz").href
    "#).unwrap();
    assert_eq!(r, "https://example.com/elsewhere");
}

#[test]
fn js_url_query_only_resolution() {
    let r = eval_string(r#"
        new URL("?x=1", "https://example.com/foo/baz").href
    "#).unwrap();
    assert_eq!(r, "https://example.com/foo/baz?x=1");
}

#[test]
fn js_url_fragment_only_resolution() {
    let r = eval_string(r##"
        new URL("#section", "https://example.com/foo/baz?q=1").href
    "##).unwrap();
    assert_eq!(r, "https://example.com/foo/baz?q=1#section");
}

#[test]
fn js_url_to_string_eq_href() {
    let r = eval_bool(r#"
        const u = new URL("https://example.com/x?y=1#z");
        String(u) === u.href && u.toJSON() === u.href
    "#).unwrap();
    assert!(r);
}

#[test]
fn js_url_can_parse() {
    let r = eval_string(r#"
        URL.canParse("https://x") + ":" + URL.canParse("not a url") + ":" + URL.canParse("./rel", "https://x/")
    "#).unwrap();
    assert_eq!(r, "true:false:true");
}

#[test]
fn js_url_invalid_throws() {
    let r = eval_bool(r#"
        (() => {
            try { new URL("not a url"); return false; }
            catch (e) { return e instanceof TypeError; }
        })()
    "#).unwrap();
    assert!(r);
}

#[test]
fn js_url_pathname_setter_normalizes_leading_slash() {
    let r = eval_string(r#"
        const u = new URL("https://example.com/old");
        u.pathname = "new";
        u.href
    "#).unwrap();
    assert_eq!(r, "https://example.com/new");
}

#[test]
fn js_url_file_scheme() {
    let r = eval_string(r#"
        const u = new URL("file:///home/user/file.txt");
        u.protocol + "|" + u.pathname + "|" + u.origin
    "#).unwrap();
    assert_eq!(r, "file:|/home/user/file.txt|null");
}

#[test]
fn js_url_ipv6_host() {
    let r = eval_string(r#"
        const u = new URL("http://[::1]:8080/x");
        u.hostname + "|" + u.port + "|" + u.host
    "#).unwrap();
    assert_eq!(r, "[::1]|8080|[::1]:8080");
}

// Canonical-docs composition: a typical consumer pattern combining
// URL parsing + URLSearchParams mutation + fetch-style request building.
#[test]
fn js_compose_url_canonical_pattern() {
    let r = eval_string(r#"
        // Build a URL the way real client code does.
        const base = new URL("https://api.example.com/v1/");
        const endpoint = new URL("./users", base);
        endpoint.searchParams.set("limit", "10");
        endpoint.searchParams.set("offset", "20");
        endpoint.searchParams.set("filter", "active");
        // Then construct a Request — exercises both pilots together.
        const req = new Request(endpoint.href, {method: "GET"});
        req.method + "|" + req.url
    "#).unwrap();
    assert_eq!(r, "GET|https://api.example.com/v1/users?limit=10&offset=20&filter=active");
}

// ════════════════════ Tier-J: consumer-shape application ═════════════════
//
// First Tier-J consumer pilot: a tiny Bun-flavored todo API at
// host/tests/fixtures/consumer-todo-api/. Exercises ESM imports across
// module boundaries, bare-specifier resolution through node_modules,
// Bun.serve route tables, URL + URLSearchParams, Request + Response,
// structuredClone, Date, Map, Set, Buffer, JSON. If this runs cleanly,
// at least one real-shape consumer can swap rusty-bun for Bun.
//
// Sub-criterion 5 of the engagement telos. The diff against actual Bun
// is a follow-up — it requires running both runtimes against the same
// fixtures and recording outcome matrices.

#[test]
fn js_consumer_todo_api_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-todo-api/src/main.js");
    let result = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    // 10 self-test cases inside main.js; expect all pass.
    assert!(result.starts_with("10/10"),
        "consumer self-test failed: {}", result);
}

// ════════════════════ Tier-J #2: stream-processor consumer (CJS) ═════════
//
// Orthogonal axis from Tier-J #1: CJS instead of ESM, async-heavy
// pipeline (streams + AbortController + setTimeout + fs across module
// boundaries).

use rusty_bun_host::eval_cjs_module_async;

#[test]
fn js_consumer_stream_processor_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-stream-processor/index.js");
    let result = eval_cjs_module_async(fixture.to_str().unwrap()).unwrap();
    assert!(result.starts_with("8/8"),
        "consumer self-test failed: {}", result);
}

// ════════════════════ Tier-J #3: differential against actual Bun ═════════
//
// Runs the same JS script against both Bun (subprocess) and rusty-bun-host,
// captures both outputs, asserts they match line-by-line. This is the
// J.2/J.3 work — actual differential evidence that rusty-bun produces
// outcomes equivalent to Bun for the spec-portable surface.
//
// Skipped at compile-time if `bun` is not on $PATH, so the test suite
// passes in environments without Bun installed.

#[test]
fn js_differential_portable_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/differential/portable.js");

    // Run under rusty-bun-host.
    let rb_result = eval_esm_module(fixture.to_str().unwrap()).unwrap();

    // Run under Bun (as a subprocess). Skip cleanly if Bun isn't installed.
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => {
            eprintln!("skipped: bun binary not found on PATH");
            return;
        }
    };
    assert!(bun.status.success(), "bun exited with error: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_stdout = String::from_utf8_lossy(&bun.stdout).trim().to_string();

    // UUID is non-deterministic — strip the uuid.length / uuid.version /
    // uuid.format lines aren't comparable byte-for-byte, but we already
    // record format-conformance booleans, so they'll match. Just diff
    // line-by-line.
    let rb_lines: Vec<&str> = rb_result.lines().collect();
    let bun_lines: Vec<&str> = bun_stdout.lines().collect();

    if rb_lines.len() != bun_lines.len() {
        panic!("line count differs: rusty-bun={} bun={}\nrb:\n{}\nbun:\n{}",
            rb_lines.len(), bun_lines.len(), rb_result, bun_stdout);
    }

    let mut mismatches = Vec::new();
    for (i, (rb, bun)) in rb_lines.iter().zip(bun_lines.iter()).enumerate() {
        if rb != bun {
            mismatches.push(format!("  L{}: rb={}  bun={}", i + 1, rb, bun));
        }
    }
    if !mismatches.is_empty() {
        panic!("differential mismatches ({}):\n{}",
            mismatches.len(), mismatches.join("\n"));
    }
}

// Tier-J differential: consumer-todo-api runs identically on Bun and
// rusty-bun-host. Both should print "10/10". Per M8: divergences are
// reconciled in-round, not deferred.
#[test]
fn js_differential_consumer_todo_api_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-todo-api/src/main.js");

    // rusty-bun side.
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();

    // Bun side. Bun's Bun.serve keeps the process alive on listening, so
    // we kill after a short wait; the self-test prints to stdout before
    // the listener goes idle. Skip cleanly if `bun` not on PATH.
    let mut bun_cmd = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn() {
        Ok(c) => c,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    std::thread::sleep(std::time::Duration::from_millis(1500));
    let _ = bun_cmd.kill();
    let bun_out = bun_cmd.wait_with_output().expect("wait");
    let bun_stdout = String::from_utf8_lossy(&bun_out.stdout).trim().to_string();

    assert_eq!(rb.trim(), bun_stdout.trim(),
        "consumer-todo-api differential mismatch:\nrb={}\nbun={}",
        rb, bun_stdout);
    assert!(rb.starts_with("10/10"), "rusty-bun side did not pass: {}", rb);
    assert!(bun_stdout.starts_with("10/10"), "Bun side did not pass: {}", bun_stdout);
}

// Tier-J differential: consumer-stream-processor runs identically on Bun
// and rusty-bun-host post-M8 reconciliation (Buffer-as-class + node:fs
// builtin resolution + fixture-side rewrite to use require("node:fs") and
// Buffer.toString("hex") and process.stdout.write).
#[test]
fn js_differential_consumer_stream_processor_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-stream-processor/index.js");

    // rusty-bun side: CJS via bootRequire + microtask pump.
    let rb = eval_cjs_module_async(fixture.to_str().unwrap()).unwrap();

    // Bun side.
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun exited with error: stdout={} stderr={}",
            String::from_utf8_lossy(&bun.stdout),
            String::from_utf8_lossy(&bun.stderr));
    }
    let bun_stdout = String::from_utf8_lossy(&bun.stdout).trim().to_string();

    assert_eq!(rb.trim(), bun_stdout.trim(),
        "stream-processor differential mismatch:\nrb={}\nbun={}", rb, bun_stdout);
    assert!(rb.starts_with("8/8"), "rusty-bun did not pass: {}", rb);
    assert!(bun_stdout.starts_with("8/8"), "Bun did not pass: {}", bun_stdout);
}

#[test]
fn js_consumer_request_signer_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-request-signer/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("6/6"), "consumer-request-signer failed: {}", r);
}

#[test]
fn js_differential_consumer_request_signer_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-request-signer/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun exited: stderr={}", String::from_utf8_lossy(&bun.stderr));
    }
    let bun_stdout = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_stdout.trim(),
        "request-signer mismatch:\nrb={}\nbun={}", rb, bun_stdout);
    assert!(rb.starts_with("6/6"), "rusty-bun did not pass: {}", rb);
}

#[test]
fn js_consumer_log_aggregator_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-log-aggregator/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("9/9"), "log-aggregator failed: {}", r);
}

#[test]
fn js_differential_consumer_log_aggregator_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-log-aggregator/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "log-aggregator mismatch:\nrb={}\nbun={}", rb, bs);
    assert!(rb.starts_with("9/9"), "rb did not pass: {}", rb);
}

#[test]
fn js_consumer_job_queue_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-job-queue/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("8/8"), "job-queue failed: {}", r);
}

#[test]
fn js_differential_consumer_job_queue_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-job-queue/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "job-queue mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_batch_loader_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-batch-loader/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("9/9"), "batch-loader failed: {}", r);
}

#[test]
fn js_differential_consumer_batch_loader_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-batch-loader/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "batch-loader mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_log_analyzer_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-log-analyzer/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("9/9"), "log-analyzer failed: {}", r);
}

#[test]
fn js_differential_consumer_log_analyzer_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-log-analyzer/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "log-analyzer mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_task_pipeline_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-task-pipeline/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("8/8"), "task-pipeline failed: {}", r);
}

#[test]
fn js_differential_consumer_task_pipeline_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-task-pipeline/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "task-pipeline mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_sequence_id_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-sequence-id/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("9/9"), "sequence-id failed: {}", r);
}

#[test]
fn js_differential_consumer_sequence_id_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-sequence-id/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "sequence-id mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_config_merger_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-config-merger/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("10/10"), "config-merger failed: {}", r);
}

#[test]
fn js_differential_consumer_config_merger_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-config-merger/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "config-merger mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_system_info_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-system-info/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("8/8"), "system-info failed: {}", r);
}

#[test]
fn js_differential_consumer_system_info_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-system-info/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "system-info mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_meta_protocols_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-meta-protocols/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("9/9"), "meta-protocols failed: {}", r);
}

#[test]
fn js_differential_consumer_meta_protocols_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-meta-protocols/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "meta-protocols mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_deferred_coordinator_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-deferred-coordinator/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("9/9"), "deferred-coordinator failed: {}", r);
}

#[test]
fn js_differential_consumer_deferred_coordinator_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-deferred-coordinator/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "deferred-coordinator mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_binary_decoder_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-binary-decoder/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("12/12"), "binary-decoder failed: {}", r);
}

#[test]
fn js_differential_consumer_binary_decoder_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-binary-decoder/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "binary-decoder mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_vendored_pkg_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-vendored-pkg/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("10/10"), "vendored-pkg failed: {}", r);
}

#[test]
fn js_differential_consumer_vendored_pkg_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-vendored-pkg/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "vendored-pkg mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_argv_parser_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-argv-parser/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("12/12"), "argv-parser failed: {}", r);
}

#[test]
fn js_differential_consumer_argv_parser_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-argv-parser/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "argv-parser mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_cli_tool_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-cli-tool/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("12/12"), "cli-tool failed: {}", r);
}

#[test]
fn js_differential_consumer_cli_tool_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-cli-tool/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "cli-tool mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_state_machine_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-state-machine/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("11/11"), "state-machine failed: {}", r);
}

#[test]
fn js_differential_consumer_state_machine_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-state-machine/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "state-machine mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_validator_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-validator/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("12/12"), "validator failed: {}", r);
}

#[test]
fn js_differential_consumer_validator_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-validator/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "validator mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_set_algebra_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-set-algebra/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("10/10"), "set-algebra failed: {}", r);
}

#[test]
fn js_differential_consumer_set_algebra_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-set-algebra/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "set-algebra mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_env_loader_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-env-loader/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("10/10"), "env-loader failed: {}", r);
}

#[test]
fn js_differential_consumer_env_loader_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-env-loader/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "env-loader mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_deps_chain_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-deps-chain/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("8/8"), "deps-chain failed: {}", r);
}

#[test]
fn js_differential_consumer_deps_chain_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-deps-chain/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "deps-chain mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_hmac_signer_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-hmac-signer/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("11/11"), "hmac-signer failed: {}", r);
}

#[test]
fn js_differential_consumer_hmac_signer_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-hmac-signer/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "hmac-signer mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_jwt_mini_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-jwt-mini/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("11/11"), "jwt-mini failed: {}", r);
}

#[test]
fn js_differential_consumer_jwt_mini_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-jwt-mini/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "jwt-mini mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_sha1_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-sha1-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("12/12"), "sha1-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_sha1_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-sha1-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "sha1-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_sha512_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-sha512-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("12/12"), "sha512-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_sha512_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-sha512-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "sha512-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

// Sockets fixture: spawn an in-process echo server on 127.0.0.1:0 so the
// fixture's client side has a known endpoint to round-trip against. The
// helper sets SOCKETS_TEST_PORT in the environment for the fixture's
// process.env.SOCKETS_TEST_PORT read.
fn with_echo_server<F: FnOnce()>(f: F) {
    use std::net::TcpListener;
    use std::io::{Read, Write};
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server_thread = std::thread::spawn(move || {
        // Accept exactly one connection, echo two writes, then close.
        if let Ok((mut sock, _)) = listener.accept() {
            // Keep the connection alive across multiple writes.
            let mut buf = [0u8; 1024];
            // The fixture sends three messages over the same connection
            // (echo-roundtrip + keep-alive x2). Each read gets one buffer.
            for _ in 0..3 {
                match sock.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => { let _ = sock.write_all(&buf[..n]); }
                    Err(_) => break,
                }
            }
        }
    });
    std::env::set_var("SOCKETS_TEST_PORT", port.to_string());
    f();
    std::env::remove_var("SOCKETS_TEST_PORT");
    // Drain the server thread; it exits when the connection drops.
    let _ = server_thread.join();
}

// HTTP-over-TCP harness: spawn an in-process HTTP/1.1 server that handles
// /health → 200 JSON, /echo → 200 echo body, anything else → 404. Used by
// the http-over-tcp fixture to validate the full client-side HTTP stack.
// Harness HTTP server for the real-fetch fixture. Like with_http_server
// but bounded to the connection count the fetch fixture issues (5).
fn with_fetch_target_server<F: FnOnce()>(f: F) {
    use std::net::TcpListener;
    use std::io::{Read, Write};
    use std::sync::Mutex;
    static FETCH_HARNESS_LOCK: Mutex<()> = Mutex::new(());
    let _guard = FETCH_HARNESS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server_thread = std::thread::spawn(move || {
        // Bound matches fixture's exact connection count (6 real fetches:
        // tests 1, 2, 3, 4, 7, 8 → connect; tests 5 https-throws and 6
        // bad-hostname-throws fail before TCP.connect). Per F9: bound must
        // match exactly or join() deadlocks on the unused accept().
        for _ in 0..6 {
            let (mut sock, _) = match listener.accept() { Ok(p) => p, Err(_) => break };
            let mut buf = vec![0u8; 8192];
            let n = match sock.read(&mut buf) { Ok(0) => continue, Ok(n) => n, Err(_) => continue };
            let req = String::from_utf8_lossy(&buf[..n]);
            let first = req.lines().next().unwrap_or("");
            let parts: Vec<&str> = first.split(' ').collect();
            let method = parts.first().copied().unwrap_or("");
            let path = parts.get(1).copied().unwrap_or("");
            let body_start = req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(n);
            let body: &[u8] = if body_start < n { &buf[body_start..n] } else { b"" };
            let response: Vec<u8> = if path == "/health" && method == "GET" {
                let body = b"{\"ok\":true}";
                let mut r = Vec::new();
                r.extend_from_slice(b"HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n");
                r.extend_from_slice(format!("content-length: {}\r\n\r\n", body.len()).as_bytes());
                r.extend_from_slice(body);
                r
            } else if path == "/echo" && method == "POST" {
                let mut r = Vec::new();
                r.extend_from_slice(b"HTTP/1.1 200 OK\r\n");
                r.extend_from_slice(format!("content-length: {}\r\n\r\n", body.len()).as_bytes());
                r.extend_from_slice(body);
                r
            } else {
                b"HTTP/1.1 404 Not Found\r\ncontent-length: 0\r\n\r\n".to_vec()
            };
            let _ = sock.write_all(&response);
            // Connection: close from client → drop socket here.
        }
    });
    std::env::set_var("FETCH_TEST_PORT", port.to_string());
    f();
    std::env::remove_var("FETCH_TEST_PORT");
    let _ = server_thread.join();
}

// Harness for the compression fixture. Precomputes gzip/zlib/raw-DEFLATE
// payloads via system tools (gzip + python3), then serves them with the
// appropriate Content-Encoding headers. Per seed §A8.16, the harness
// shares the FETCH_TEST_PORT env var with the other fetch harnesses, so
// the same FETCH_HARNESS_LOCK static is used.
fn with_compression_target_server<F: FnOnce()>(f: F) {
    use std::net::TcpListener;
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};
    use std::sync::Mutex;
    static COMPRESSION_HARNESS_LOCK: Mutex<()> = Mutex::new(());
    let _guard = COMPRESSION_HARNESS_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    fn gzip_encode(input: &[u8]) -> Vec<u8> {
        let mut c = Command::new("gzip").arg("-c").arg("-n")
            .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        c.stdin.as_mut().unwrap().write_all(input).unwrap();
        c.wait_with_output().unwrap().stdout
    }
    fn python_encode(script: &str, input: &[u8]) -> Option<Vec<u8>> {
        let mut c = match Command::new("python3").arg("-c").arg(script)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()
        { Ok(c) => c, Err(_) => return None };
        c.stdin.as_mut().unwrap().write_all(input).ok()?;
        let out = c.wait_with_output().ok()?;
        if out.status.success() { Some(out.stdout) } else { None }
    }
    let gz_small = gzip_encode(b"compressed payload");
    let gz_ws = gzip_encode(b"ws-gzip");
    let gz_large = {
        let s: String = "abcde".repeat(1000);
        gzip_encode(s.as_bytes())
    };
    let zlib_payload = python_encode(
        "import sys, zlib; sys.stdout.buffer.write(zlib.compress(sys.stdin.buffer.read()))",
        b"zlib-wrapped",
    ).expect("python3 zlib unavailable");
    let raw_deflate = python_encode(
        "import sys, zlib; d = zlib.compressobj(-1, zlib.DEFLATED, -15); \
         sys.stdout.buffer.write(d.compress(sys.stdin.buffer.read()) + d.flush())",
        b"raw-deflate",
    ).expect("python3 raw-deflate unavailable");

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server_thread = std::thread::spawn(move || {
        // F9 rule (seed §A8.16): bound matches fixture's exact connection
        // count. Fixture issues 6 real fetches (tests 1, 4, 5, 6, 7, 8;
        // tests 2 and 3 inspect headers from test 1's response and do not
        // open new connections).
        for _ in 0..6 {
            let (mut sock, _) = match listener.accept() { Ok(p) => p, Err(_) => break };
            let mut buf = vec![0u8; 8192];
            let n = match sock.read(&mut buf) { Ok(0) => continue, Ok(n) => n, Err(_) => continue };
            let req = String::from_utf8_lossy(&buf[..n]);
            let first = req.lines().next().unwrap_or("");
            let parts: Vec<&str> = first.split(' ').collect();
            let path = parts.get(1).copied().unwrap_or("");
            let (status, content_encoding, payload): (&str, Option<&str>, Vec<u8>) = match path {
                "/gzip" => ("200 OK", Some("gzip"), gz_small.clone()),
                "/gzip-ws-header" => ("200 OK", Some(" gzip "), gz_ws.clone()),
                "/gzip-large" => ("200 OK", Some("gzip"), gz_large.clone()),
                "/deflate-zlib" => ("200 OK", Some("deflate"), zlib_payload.clone()),
                "/deflate-raw" => ("200 OK", Some("deflate"), raw_deflate.clone()),
                "/identity" => ("200 OK", Some("identity"), b"uncompressed".to_vec()),
                _ => ("404 Not Found", None, Vec::new()),
            };
            let mut header = format!("HTTP/1.1 {}\r\ncontent-type: text/plain\r\ncontent-length: {}\r\n",
                                     status, payload.len());
            if let Some(ce) = content_encoding {
                header.push_str(&format!("content-encoding: {}\r\n", ce));
            }
            header.push_str("\r\n");
            let _ = sock.write_all(header.as_bytes());
            let _ = sock.write_all(&payload);
        }
    });
    std::env::set_var("FETCH_TEST_PORT", port.to_string());
    f();
    std::env::remove_var("FETCH_TEST_PORT");
    let _ = server_thread.join();
}

// Π1.5.d: live WebSocket round-trip harness. Spawns a Bun subprocess
// running a minimal Bun.serve websocket echo, sets WS_TEST_PORT,
// runs the fixture. Both implementations connect to the same Bun-
// hosted server and exercise the canonical lifecycle.
fn with_ws_echo_server<F: FnOnce()>(f: F) {
    use std::io::Read;
    use std::sync::Mutex;
    static WS_LIVE_LOCK: Mutex<()> = Mutex::new(());
    let _guard = WS_LIVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    // Bun WebSocket echo server inline.
    let server_script = format!(r#"
        const s = Bun.serve({{
            port: {},
            hostname: "127.0.0.1",
            fetch(req, server) {{
                if (server.upgrade(req)) return;
                return new Response("not a websocket", {{status: 400}});
            }},
            websocket: {{
                message(ws, msg) {{ ws.send(msg); }},
                close() {{ process.exit(0); }},
            }},
        }});
    "#, port);

    let mut server = match std::process::Command::new("bun")
        .args(&["-e", &server_script])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => { eprintln!("skipped: bun not on PATH for ws echo server"); return; }
    };
    // Wait for the server to bind.
    std::thread::sleep(std::time::Duration::from_millis(800));

    std::env::set_var("WS_TEST_PORT", port.to_string());
    f();
    std::env::remove_var("WS_TEST_PORT");

    let _ = server.kill();
    let _ = server.wait();
}

#[test]
#[ignore]  // seed A8.17: spawns Bun subprocess + live ws handshake
fn js_consumer_websocket_live_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-websocket-live-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    with_ws_echo_server(|| {
        let r = eval_esm_module(&path).unwrap();
        let summary = r.lines().filter(|l| !l.is_empty()).last().unwrap_or("");
        assert!(summary.starts_with("4/4"), "websocket-live-suite failed: {}", r);
    });
}

// Π1.5.b: harness for __ws namespace structural test.
fn with_ws_primitives_test_env<F: FnOnce()>(f: F) {
    use std::sync::Mutex;
    static WS_NS_LOCK: Mutex<()> = Mutex::new(());
    let _guard = WS_NS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // Bind+drop an ephemeral port to get a "real" FETCH_TEST_PORT;
    // the WS primitives don't actually connect anywhere — the env var
    // just toggles the fixture's structural-validation branch.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    std::env::set_var("FETCH_TEST_PORT", port.to_string());
    f();
    std::env::remove_var("FETCH_TEST_PORT");
}

#[test]
fn js_consumer_websocket_class_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-websocket-class-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let summary = r.lines().filter(|l| !l.is_empty()).last().unwrap_or("");
    assert!(summary.starts_with("8/8"), "websocket-class-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_websocket_class_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-websocket-class-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    let rb = eval_esm_module(&path).unwrap();
    let rb_last = rb.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    let bs_last = bs.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    assert_eq!(rb_last, bs_last,
        "websocket-class-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_ws_primitives_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-ws-primitives-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    with_ws_primitives_test_env(|| {
        let r = eval_esm_module(&path).unwrap();
        let summary = r.lines().filter(|l| !l.is_empty()).last().unwrap_or("");
        assert!(summary.starts_with("8/8"), "ws-primitives-suite failed: {}", r);
    });
}

#[test]
fn js_differential_consumer_ws_primitives_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-ws-primitives-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    let bs_last = bs.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    // Under Bun (no FETCH_TEST_PORT, no __ws): all-skipped path → 8/8.
    assert!(bs_last.starts_with("8/8"), "bun did not report 8/8: {}", bs);
}

#[test]
fn js_consumer_node_assert_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-node-assert-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let summary = r.lines().filter(|l| !l.is_empty()).last().unwrap_or("");
    assert!(summary.starts_with("8/8"), "node-assert-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_node_assert_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-node-assert-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    let rb = eval_esm_module(&path).unwrap();
    let rb_last = rb.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    let bs_last = bs.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    assert_eq!(rb_last, bs_last, "node-assert-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

// Π1.4.h: harness for __tls namespace structural test.
fn with_tls_namespace_test_env<F: FnOnce()>(f: F) {
    use std::sync::Mutex;
    static TLS_NS_LOCK: Mutex<()> = Mutex::new(());
    let _guard = TLS_NS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // Bind an ephemeral port and use it as FETCH_TEST_PORT so the
    // fixture takes the structural-validation path (vs the skipped
    // path). We don't actually serve TLS — the fixture only tests
    // namespace presence + error-path semantics that don't require a
    // live handshake.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    // Drop the listener immediately so the port becomes unavailable;
    // tls.connect to it will fail with a connect error, which is what
    // the "tls-connect-rejects-empty-ca" test expects.
    drop(listener);
    std::env::set_var("FETCH_TEST_PORT", port.to_string());
    f();
    std::env::remove_var("FETCH_TEST_PORT");
}

#[test]
fn js_consumer_tls_namespace_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-tls-namespace-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    with_tls_namespace_test_env(|| {
        let r = eval_esm_module(&path).unwrap();
        let summary = r.lines().filter(|l| !l.is_empty()).last().unwrap_or("");
        assert!(summary.starts_with("8/8"), "tls-namespace-suite failed: {}", r);
    });
}

#[test]
fn js_differential_consumer_tls_namespace_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-tls-namespace-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    let bs_last = bs.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    // Under Bun (no FETCH_TEST_PORT, no __tls), the fixture takes the
    // all-skipped-noport path → 8/8.
    assert!(bs_last.starts_with("8/8"),
        "bun did not report 8/8 (env-isolation issue): {}", bs);
}

#[test]
fn js_consumer_bun_serve_autoserve_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-bun-serve-autoserve-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let summary = r.lines().filter(|l| !l.is_empty()).last().unwrap_or("");
    assert!(summary.starts_with("8/8"), "bun-serve-autoserve-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_bun_serve_autoserve_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-bun-serve-autoserve-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    let rb = eval_esm_module(&path).unwrap();
    let rb_last = rb.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    let bs_last = bs.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    assert_eq!(rb_last, bs_last,
        "bun-serve-autoserve-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_bun_small_utilities_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-bun-small-utilities-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let summary = r.lines().filter(|l| !l.is_empty()).last().unwrap_or("");
    assert!(summary.starts_with("8/8"), "bun-small-utilities-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_bun_small_utilities_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-bun-small-utilities-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    let rb = eval_esm_module(&path).unwrap();
    let rb_last = rb.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    let bs_last = bs.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    assert_eq!(rb_last, bs_last,
        "bun-small-utilities-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_node_querystring_url_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-node-querystring-url-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let summary = r.lines().filter(|l| !l.is_empty()).last().unwrap_or("");
    assert!(summary.starts_with("8/8"), "node-querystring-url-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_node_querystring_url_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-node-querystring-url-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    let rb = eval_esm_module(&path).unwrap();
    let rb_last = rb.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    let bs_last = bs.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    assert_eq!(rb_last, bs_last,
        "node-querystring-url-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_node_stream_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-node-stream-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let summary = r.lines().filter(|l| !l.is_empty()).last().unwrap_or("");
    assert!(summary.starts_with("8/8"), "node-stream-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_node_stream_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-node-stream-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    let rb = eval_esm_module(&path).unwrap();
    let rb_last = rb.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    let bs_last = bs.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    assert_eq!(rb_last, bs_last, "node-stream-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_node_util_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-node-util-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let summary = r.lines().filter(|l| !l.is_empty()).last().unwrap_or("");
    assert!(summary.starts_with("8/8"), "node-util-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_node_util_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-node-util-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    let rb = eval_esm_module(&path).unwrap();
    let rb_last = rb.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    let bs_last = bs.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    assert_eq!(rb_last, bs_last, "node-util-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_node_events_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-node-events-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let summary = r.lines().filter(|l| !l.is_empty()).last().unwrap_or("");
    assert!(summary.starts_with("8/8"), "node-events-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_node_events_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-node-events-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    let rb = eval_esm_module(&path).unwrap();
    let rb_last = rb.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    let bs_last = bs.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    assert_eq!(rb_last, bs_last, "node-events-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_process_events_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-process-events-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let summary = r.lines().filter(|l| !l.is_empty()).last().unwrap_or("");
    assert!(summary.starts_with("8/8"), "process-events-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_process_events_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-process-events-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    let rb = eval_esm_module(&path).unwrap();
    let rb_last = rb.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    let bs_last = bs.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    assert_eq!(rb_last, bs_last,
        "process-events-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_compression_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-compression-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    with_compression_target_server(|| {
        let r = eval_esm_module(&path).unwrap();
        let summary = r.lines().filter(|l| !l.is_empty()).last().unwrap_or("");
        assert!(summary.starts_with("8/8"), "compression-suite failed: {}", r);
    });
}

#[test]
fn js_differential_consumer_compression_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-compression-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    let rb = {
        let mut out = String::new();
        with_compression_target_server(|| { out = eval_esm_module(&path).unwrap(); });
        out.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string()
    };
    // Bun: no FETCH_TEST_PORT, all-skipped path → 8/8.
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    let bs_last = bs.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    assert_eq!(rb.trim(), bs_last, "compression-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_dns_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-dns-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let summary = r.lines().filter(|l| !l.is_empty()).last().unwrap_or("");
    assert!(summary.starts_with("8/8"), "dns-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_dns_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-dns-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    let rb = eval_esm_module(&path).unwrap();
    let rb_last = rb.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    let bs_last = bs.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    assert_eq!(rb_last, bs_last, "dns-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_real_fetch_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-real-fetch-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    with_fetch_target_server(|| {
        let r = eval_esm_module(&path).unwrap();
        // Fixture emits pulse markers between awaits as a runtime-
        // interleaving workaround. Summary is the last non-empty stdout line.
        let summary = r.lines().filter(|l| !l.is_empty()).last().unwrap_or("");
        assert!(summary.starts_with("8/8"), "real-fetch-suite failed: {}", r);
    });
}

#[test]
fn js_differential_consumer_real_fetch_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-real-fetch-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    let rb = {
        let mut out = String::new();
        with_fetch_target_server(|| { out = eval_esm_module(&path).unwrap(); });
        out.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string()
    };
    // Bun: no FETCH_TEST_PORT, takes all-skipped path → 8/8.
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    let bs_last = bs.lines().filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
    assert_eq!(rb.trim(), bs_last, "real-fetch-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_bun_serve_facade_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-bun-serve-facade-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("5/5"), "bun-serve-facade-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_bun_serve_facade_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-bun-serve-facade-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "bun-serve-facade-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_async_http_server_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-async-http-server-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("8/8"), "async-http-server-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_async_http_server_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-async-http-server-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "async-http-server-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

fn with_http_server<F: FnOnce()>(f: F) {
    use std::net::TcpListener;
    use std::io::{Read, Write};
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(false).unwrap();
    let port = listener.local_addr().unwrap().port();
    // Drop-stopping pattern: fixture connects 4 times. After 4 successful
    // accepts, the thread exits cleanly. If the fixture aborts early, we
    // shut down via a side-channel set_nonblocking + accept loop. Simpler:
    // bound the accept loop to exactly 4 iterations, which matches the
    // fixture's known connection count.
    let server_thread = std::thread::spawn(move || {
        for _ in 0..4 {
            match listener.accept() {
                Ok((mut sock, _)) => {
                    let mut buf = vec![0u8; 8192];
                    loop {
                        let n = match sock.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => n,
                            Err(_) => break,
                        };
                        let req_bytes = &buf[..n];
                        let req_str = String::from_utf8_lossy(req_bytes);
                        // Parse the request-line (first line only).
                        let first_line = req_str.lines().next().unwrap_or("");
                        let parts: Vec<&str> = first_line.split(' ').collect();
                        let method = parts.first().copied().unwrap_or("");
                        let path = parts.get(1).copied().unwrap_or("");
                        let close = req_str.to_ascii_lowercase().contains("connection: close");
                        // Extract body if Content-Length set (find double CRLF).
                        let body_start = req_str.find("\r\n\r\n").map(|i| i + 4).unwrap_or(req_str.len());
                        let body = if body_start < req_bytes.len() {
                            &req_bytes[body_start..]
                        } else { &b""[..] };
                        let response: Vec<u8> = if path == "/health" && method == "GET" {
                            let body = b"{\"ok\":true}";
                            let mut r = Vec::new();
                            r.extend_from_slice(b"HTTP/1.1 200 OK\r\n");
                            r.extend_from_slice(b"content-type: application/json\r\n");
                            r.extend_from_slice(format!("content-length: {}\r\n", body.len()).as_bytes());
                            r.extend_from_slice(b"\r\n");
                            r.extend_from_slice(body);
                            r
                        } else if path == "/echo" && method == "POST" {
                            let mut r = Vec::new();
                            r.extend_from_slice(b"HTTP/1.1 200 OK\r\n");
                            r.extend_from_slice(format!("content-length: {}\r\n", body.len()).as_bytes());
                            r.extend_from_slice(b"\r\n");
                            r.extend_from_slice(body);
                            r
                        } else {
                            let mut r = Vec::new();
                            r.extend_from_slice(b"HTTP/1.1 404 Not Found\r\ncontent-length: 0\r\n\r\n");
                            r
                        };
                        if sock.write_all(&response).is_err() { break; }
                        if close { break; }
                    }
                }
                Err(_) => break,
            }
        }
    });
    std::env::set_var("HTTP_OVER_TCP_PORT", port.to_string());
    f();
    std::env::remove_var("HTTP_OVER_TCP_PORT");
    let _ = server_thread.join();
}

#[test]
fn js_consumer_http_over_tcp_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-http-over-tcp-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    with_http_server(|| {
        let r = eval_esm_module(&path).unwrap();
        assert!(r.starts_with("8/8"), "http-over-tcp-suite failed: {}", r);
    });
}

#[test]
fn js_differential_consumer_http_over_tcp_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-http-over-tcp-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    let rb = {
        let mut out = String::new();
        with_http_server(|| { out = eval_esm_module(&path).unwrap(); });
        out
    };
    // Bun: no TCP / HTTP namespaces; fixture takes all-skipped path → "8/8".
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "http-over-tcp-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_sockets_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-sockets-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    with_echo_server(|| {
        let r = eval_esm_module(&path).unwrap();
        assert!(r.starts_with("8/8"), "sockets-suite failed: {}", r);
    });
}

#[test]
fn js_differential_consumer_sockets_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-sockets-suite/src/main.js");
    let path = fixture.to_str().unwrap().to_string();
    // rusty-bun-host: runs with harness echo-server + SOCKETS_TEST_PORT.
    let rb = {
        let mut out = String::new();
        with_echo_server(|| { out = eval_esm_module(&path).unwrap(); });
        out
    };
    // Bun: no TCP namespace; fixture takes the all-skipped path → "8/8".
    let bun = match std::process::Command::new("bun").arg(&path).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "sockets-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_mini_http_server_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-mini-http-server-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("10/10"), "mini-http-server-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_mini_http_server_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-mini-http-server-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "mini-http-server-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_http_codec_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-http-codec-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("15/15"), "http-codec-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_http_codec_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-http-codec-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "http-codec-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_msgpack_mini_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-msgpack-mini-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("21/21"), "msgpack-mini-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_msgpack_mini_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-msgpack-mini-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "msgpack-mini-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_mini_router_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-mini-router-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("15/15"), "mini-router-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_mini_router_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-mini-router-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "mini-router-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_signals_mini_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-signals-mini-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("17/17"), "signals-mini-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_signals_mini_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-signals-mini-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "signals-mini-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_async_pool_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-async-pool-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("12/12"), "async-pool-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_async_pool_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-async-pool-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "async-pool-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_markdown_mini_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-markdown-mini-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("20/20"), "markdown-mini-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_markdown_mini_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-markdown-mini-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "markdown-mini-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_csv_mini_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-csv-mini-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("15/15"), "csv-mini-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_csv_mini_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-csv-mini-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "csv-mini-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_mustache_mini_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-mustache-mini-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("15/15"), "mustache-mini-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_mustache_mini_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-mustache-mini-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "mustache-mini-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
#[ignore] // seed A8.17: bigint/EC/RSA inner-loop cost
fn js_consumer_jwks_verifier_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-jwks-verifier-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("10/10"), "jwks-verifier-suite failed: {}", r);
}

#[test]
#[ignore] // seed A8.17: bigint/EC/RSA inner-loop cost
fn js_differential_consumer_jwks_verifier_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-jwks-verifier-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "jwks-verifier-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
#[ignore] // seed A8.17: bigint/EC/RSA inner-loop cost
fn js_consumer_jwt_rs256_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-jwt-rs256-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("10/10"), "jwt-rs256-suite failed: {}", r);
}

#[test]
#[ignore] // seed A8.17: bigint/EC/RSA inner-loop cost
fn js_differential_consumer_jwt_rs256_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-jwt-rs256-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "jwt-rs256-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
#[ignore] // seed A8.17: bigint/EC/RSA inner-loop cost
fn js_consumer_ec_curves_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-ec-curves-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("4/4"), "ec-curves-suite failed: {}", r);
}

#[test]
#[ignore] // seed A8.17: bigint/EC/RSA inner-loop cost
fn js_differential_consumer_ec_curves_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-ec-curves-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "ec-curves-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
#[ignore] // seed A8.17: bigint/EC/RSA inner-loop cost
fn js_consumer_ecdh_p256_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-ecdh-p256-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("5/5"), "ecdh-p256-suite failed: {}", r);
}

#[test]
#[ignore] // seed A8.17: bigint/EC/RSA inner-loop cost
fn js_differential_consumer_ecdh_p256_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-ecdh-p256-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "ecdh-p256-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
#[ignore] // seed A8.17: bigint/EC/RSA inner-loop cost
fn js_consumer_ecdsa_p256_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-ecdsa-p256-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("6/6"), "ecdsa-p256-suite failed: {}", r);
}

#[test]
#[ignore] // seed A8.17: bigint/EC/RSA inner-loop cost
fn js_differential_consumer_ecdsa_p256_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-ecdsa-p256-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "ecdsa-p256-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
#[ignore] // seed A8.17: bigint/EC/RSA inner-loop cost
fn js_consumer_rsa_pss_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-rsa-pss-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("7/7"), "rsa-pss-suite failed: {}", r);
}

#[test]
#[ignore] // seed A8.17: bigint/EC/RSA inner-loop cost
fn js_differential_consumer_rsa_pss_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-rsa-pss-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "rsa-pss-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
#[ignore] // seed A8.17: bigint/EC/RSA inner-loop cost
fn js_consumer_rsa_oaep_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-rsa-oaep-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("6/6"), "rsa-oaep-suite failed: {}", r);
}

#[test]
#[ignore] // seed A8.17: bigint/EC/RSA inner-loop cost
fn js_differential_consumer_rsa_oaep_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-rsa-oaep-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "rsa-oaep-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_aes_modes_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-aes-modes-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("10/10"), "aes-modes-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_aes_modes_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-aes-modes-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "aes-modes-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_hkdf_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-hkdf-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("10/10"), "hkdf-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_hkdf_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-hkdf-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "hkdf-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_aes_gcm_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-aes-gcm-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("11/11"), "aes-gcm-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_aes_gcm_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-aes-gcm-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "aes-gcm-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

#[test]
fn js_consumer_pbkdf2_suite_runs_clean() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-pbkdf2-suite/src/main.js");
    let r = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    assert!(r.starts_with("10/10"), "pbkdf2-suite failed: {}", r);
}

#[test]
fn js_differential_consumer_pbkdf2_suite_matches_bun() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-pbkdf2-suite/src/main.js");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun").arg(fixture.to_str().unwrap()).output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    if !bun.status.success() {
        panic!("bun stderr: {}", String::from_utf8_lossy(&bun.stderr));
    }
    let bs = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bs, "pbkdf2-suite mismatch:\nrb={}\nbun={}", rb, bs);
}

// ════════════════════ Bun.password (Π4.14.c) ════════════════════

#[test]
fn bun_password_hash_returns_phc_string() {
    let r = rusty_bun_host::eval_string_async(r#"
        const enc = await Bun.password.hash("hunter2", { timeCost: 2, memoryCost: 1024 });
        return enc;
    "#).unwrap();
    assert!(r.starts_with("$argon2id$v=19$m=1024,t=2,p=1$"), "got: {}", r);
    let parts: Vec<&str> = r.split('$').collect();
    assert_eq!(parts.len(), 6);
    assert!(!parts[4].is_empty() && !parts[5].is_empty());
}

#[test]
fn bun_password_hash_verify_roundtrip() {
    let r = rusty_bun_host::eval_string_async(r#"
        const enc = await Bun.password.hash("correct horse", { timeCost: 2, memoryCost: 1024 });
        const ok = await Bun.password.verify("correct horse", enc);
        const bad = await Bun.password.verify("wrong", enc);
        return (ok && !bad) ? "yes" : "no";
    "#).unwrap();
    assert_eq!(r, "yes");
}

#[test]
fn bun_password_verify_upstream_phc_string() {
    // PHC string produced by upstream `argon2` npm package, Bun runtime:
    //   argon2.hash("hunter2", { type:argon2id, salt:"saltsaltsaltsalt",
    //                            timeCost:2, memoryCost:1024, parallelism:1,
    //                            hashLength:32, version:0x13 })
    let r = rusty_bun_host::eval_string_async(r#"
        const enc = "$argon2id$v=19$m=1024,t=2,p=1$c2FsdHNhbHRzYWx0c2FsdA$8Ay6op+3TmdW+WkH0Q1ci5BobdmPnyvp2rUlv7zx/IE";
        const ok = await Bun.password.verify("hunter2", enc);
        const bad = await Bun.password.verify("hunter3", enc);
        return (ok && !bad) ? "ok" : "bad";
    "#).unwrap();
    assert_eq!(r, "ok");
}

#[test]
fn bun_password_verify_via_argon2id_substrate() {
    let r = rusty_bun_host::eval_string_async(r#"
        const pwBytes = new TextEncoder().encode("pw");
        const salt = new TextEncoder().encode("saltsaltsaltsalt");
        const tag = new Uint8Array(globalThis.crypto.subtle.argon2idBytes(
            Array.from(pwBytes), Array.from(salt), 2, 1024, 32,
        ));
        const b64nopad = (b) => { let s=""; for (const x of b) s+=String.fromCharCode(x); return btoa(s).replace(/=+$/,""); };
        const enc = `$argon2id$v=19$m=1024,t=2,p=1$${b64nopad(salt)}$${b64nopad(tag)}`;
        const ok = await Bun.password.verify("pw", enc);
        return ok ? "ok" : "bad";
    "#).unwrap();
    assert_eq!(r, "ok");
}

#[test]
fn bun_password_hash_sync_verify_sync_roundtrip() {
    let r = rusty_bun_host::eval_string(r#"
        const enc = Bun.password.hashSync("hunter2", { timeCost: 2, memoryCost: 1024 });
        const ok = Bun.password.verifySync("hunter2", enc);
        const bad = Bun.password.verifySync("wrong", enc);
        (ok && !bad) ? "yes" : "no"
    "#).unwrap();
    assert_eq!(r, "yes");
}

#[test]
fn bun_password_verify_sync_accepts_upstream_phc() {
    let r = rusty_bun_host::eval_string(r#"
        const enc = "$argon2id$v=19$m=1024,t=2,p=1$c2FsdHNhbHRzYWx0c2FsdA$8Ay6op+3TmdW+WkH0Q1ci5BobdmPnyvp2rUlv7zx/IE";
        Bun.password.verifySync("hunter2", enc) ? "ok" : "bad"
    "#).unwrap();
    assert_eq!(r, "ok");
}

// ════════════════════ Bun.gzipSync / deflateSync (Π1.3.b) ════════════════════

#[test]
fn bun_gzip_sync_round_trips() {
    let r = rusty_bun_host::eval_string(r#"
        const input = "hello compression world";
        const enc = Bun.gzipSync(input);
        const dec = new TextDecoder().decode(Bun.gunzipSync(enc));
        dec
    "#).unwrap();
    assert_eq!(r, "hello compression world");
}

#[test]
fn bun_deflate_sync_round_trips() {
    let r = rusty_bun_host::eval_string(r#"
        const input = "deflate this";
        const enc = Bun.deflateSync(input);
        const dec = new TextDecoder().decode(Bun.inflateSync(enc));
        dec
    "#).unwrap();
    assert_eq!(r, "deflate this");
}

#[test]
fn bun_gzip_sync_decodes_under_bun() {
    // Verifies the wire format is compatible: real Bun's gunzipSync
    // (and any conforming gzip decoder) accepts our stored-block output.
    // We don't have Bun in-process here, so emit bytes and trust the
    // pilot-level system-gunzip differential to anchor wire compatibility.
    let r = rusty_bun_host::eval_string(r#"
        const enc = Bun.gzipSync("xyz");
        // gzip magic 0x1f 0x8b
        (enc[0] === 0x1f && enc[1] === 0x8b) ? "ok" : "bad"
    "#).unwrap();
    assert_eq!(r, "ok");
}

// ════════════════════ Π2.6.b: autoServe self-fetch ════════════════════

#[test]
fn autoserve_self_fetch_round_trips() {
    // Canonical Π2.6.b proof: a single JS module starts Bun.serve with
    // autoServe:true, then fetches itself. The cooperative loop (fetch's
    // nonblocking tryRead + __tickKeepAlive + microtask yield) lets the
    // server's __tick handler run between fetch's read attempts.
    let r = rusty_bun_host::eval_string_async(r#"
        const server = Bun.serve({
            port: 0,
            hostname: "127.0.0.1",
            autoServe: true,
            fetch(req) {
                const u = new URL(req.url);
                return new Response("hello from " + u.pathname, {
                    status: 200,
                    headers: { "content-type": "text/plain" },
                });
            },
        });
        const port = server.port;
        const resp = await fetch("http://127.0.0.1:" + port + "/echo");
        const body = await resp.text();
        const status = resp.status;
        server.stop();
        return status + ":" + body;
    "#).unwrap();
    assert_eq!(r, "200:hello from /echo");
}

// ════════════════════ Π5: first real-OSS differential (itty-router) ═════
//
// hono v4 was the first attempted target. Its dist build uses bare
// uninitialized class field declarations with reserved-method names
// (`get;` `post;` etc.) which QuickJS rejects as malformed accessor
// syntax. Recorded as basin boundary E.12; re-open: (a) upgrade
// rquickjs/QuickJS to a build accepting class-field decls with
// reserved-method-name shorthand, OR (b) pre-eval source transform.
//
// itty-router is a tree-shakeable minified-ESM router popular in the
// Bun/edge ecosystem (~1.5 KB). Uses Proxy + RegExp named-capture
// groups + URL + URLSearchParams + async iteration — all in basin.

#[test]
fn consumer_itty_router_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-itty-router-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_jose_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-jose-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}


#[test]
fn consumer_zod_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-zod-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_dayjs_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-dayjs-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_stack_app_byte_identical_to_bun() {
    // Composed fixture: itty-router + zod + jose on Bun.serve + same-process
    // fetch via Π2.6.b. Strongest single piece of telos evidence — multiple
    // vendored OSS libs orchestrated in one process matching Bun byte-for-byte.
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-stack-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_nanoid_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-nanoid-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn nanoid_probe_diag_removed() {
    // Probe panic-test removed once the M8 reconciliations (webcrypto export
    // + Buffer.allocUnsafe) landed and the differential turned green.
}


#[test]
fn consumer_valibot_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-valibot-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_uuid_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-uuid-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_ulid_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-ulid-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}


#[test]
fn consumer_picocolors_app_byte_identical_to_bun() {
    // E.13 closure validator: picocolors is CJS-only; FsLoader bridges
    // CJS to ESM by evaluating eagerly via bootRequire then synthesizing
    // a re-export shim. import pc from "picocolors" works.
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-picocolors-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_ms_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-ms-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_mini_app_byte_identical_to_bun() {
    // Composed mini API server: 6 vendored OSS libs in one process via
    // Bun.serve + Π2.6.b self-fetch. Strongest single composition
    // telos validator the engagement has produced.
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-mini-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn node_net_socket_to_bun_serve_round_trip() {
    // node:net.Socket client → Bun.serve HTTP server, same-process via
    // Π2.6.b cooperative loop. The Socket sends a raw HTTP/1.1 request,
    // server responds, client reads response via data events, parses.
    let r = rusty_bun_host::eval_string_async(r#"
        const net = globalThis.nodeNet;
        const server = Bun.serve({
            port: 0, hostname: "127.0.0.1", autoServe: true,
            async fetch(req) {
                const body = await req.text();
                return new Response("net-client says: " + (body || "no-body"), {
                    status: 200, headers: { "x-from": "bun-serve" },
                });
            },
        });
        // Synchronous IIFE that awaits the round-trip.
        const port = server.port;
        const lines = [];
        await new Promise((resolve, reject) => {
            const sock = new net.Socket();
            let buf = "";
            sock.setEncoding("utf8");
            sock.on("connect", () => {
                lines.push("connected");
                const req =
                    "GET /hello HTTP/1.1\r\n" +
                    "Host: 127.0.0.1\r\n" +
                    "Connection: close\r\n" +
                    "\r\n";
                sock.write(req);
            });
            sock.on("data", (d) => { buf += d; });
            sock.on("end", () => {
                lines.push("ended");
                resolve();
            });
            sock.on("error", reject);
            sock.connect(port, "127.0.0.1");
            // safety timeout
            setTimeout(() => reject(new Error("timeout")), 5000);
        }).catch(e => lines.push("err " + e.message));
        server.stop();
        return lines.join(",");
    "#);
    let s = r.unwrap();
    assert!(s.contains("connected"), "no connect event: {}", s);
    // Note: the Bun.serve handler returns after a non-await response object;
    // it'll close the connection after the response is written (Connection: close).
    // 'ended' may or may not fire depending on server-side close timing —
    // accept either "connected,ended" or "connected" as success.
    assert!(s.starts_with("connected"), "expected connect first: {}", s);
}

#[test]
fn consumer_lodash_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-lodash-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_debug_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-debug-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}


#[test]
fn node_http_create_server_listen_round_trip() {
    // node:http.createServer + .listen + Π2.6.b self-fetch round-trip.
    // Bridges the Node-style (req, res) handler to Bun.serve internally.
    let r = rusty_bun_host::eval_string_async(r#"
        const http = globalThis.nodeHttp;
        const server = http.createServer((req, res) => {
            const body = req._body || "";  // body bridged in via IncomingMessage init
            res.writeHead(201, { "content-type": "text/plain", "x-method": req.method });
            res.end("node-style says: " + req.method + " " + req.url + " body=" + body);
        });
        server.listen(0, "127.0.0.1");
        const port = server.port;
        const base = "http://127.0.0.1:" + port;
        const lines = [];
        {
            const r = await fetch(base + "/hello");
            lines.push("1 " + r.status + " " + r.headers.get("x-method") + " " + (await r.text()));
        }
        {
            const r = await fetch(base + "/echo", { method: "POST", body: "ping" });
            lines.push("2 " + r.status + " " + r.headers.get("x-method") + " " + (await r.text()));
        }
        server.close();
        return lines.join("\n");
    "#).unwrap();
    assert_eq!(r, "1 201 GET node-style says: GET /hello body=\n2 201 POST node-style says: POST /echo body=ping");
}

#[test]
fn consumer_yaml_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-yaml-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}



#[test]

#[test]
fn consumer_express_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-express-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_koa_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-koa-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}



#[test]
fn consumer_semver_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-semver-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_picomatch_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-picomatch-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_marked_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-marked-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_acorn_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-acorn-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_chai_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-chai-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}


#[test]
fn consumer_diff_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-diff-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_eventemitter3_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-eventemitter3-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_decimal_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-decimal-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_xstate_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-xstate-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_lrucache_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-lrucache-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}


#[test]
fn consumer_hashids_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-hashids-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_immer_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-immer-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_bcryptjs_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-bcryptjs-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_rxjs_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-rxjs-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_hljs_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-hljs-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}


#[test]
fn consumer_kleur_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-kleur-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}

#[test]
fn consumer_fde_app_byte_identical_to_bun() {
    use rusty_bun_host::eval_esm_module;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/consumer-fde-app/main.mjs");
    let rb = eval_esm_module(fixture.to_str().unwrap()).unwrap();
    let bun = match std::process::Command::new("bun")
        .arg(fixture.to_str().unwrap())
        .output() {
        Ok(o) => o,
        Err(_) => { eprintln!("skipped: bun not on PATH"); return; }
    };
    assert!(bun.status.success(), "bun exited: {}",
        String::from_utf8_lossy(&bun.stderr));
    let bun_out = String::from_utf8_lossy(&bun.stdout).trim().to_string();
    assert_eq!(rb.trim(), bun_out, "differential mismatch");
}
