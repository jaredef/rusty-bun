// Integration tests for rusty-bun-host. These tests run JS code through
// the rquickjs-embedded host with all pilots wired into globalThis,
// validating that the integration layer works end-to-end.

use rusty_bun_host::{eval_bool, eval_i64, eval_string};

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
    let r = eval_string(r#"new Blob(["héllo"]).text()"#).unwrap();
    assert_eq!(r, "héllo");
}

#[test]
fn js_blob_slice() {
    let r = eval_string(r#"new Blob(["hello world"]).slice(6).text()"#).unwrap();
    assert_eq!(r, "world");
}

#[test]
fn js_blob_slice_with_content_type_override() {
    let r = eval_string(r#"new Blob(["hello"], {type: "text/plain"}).slice(0, 3, "application/json").type"#).unwrap();
    assert_eq!(r, "application/json");
}

#[test]
fn js_blob_concatenate_parts() {
    let r = eval_string(r#"new Blob(["hello ", "world"]).text()"#).unwrap();
    assert_eq!(r, "hello world");
}

#[test]
fn js_blob_byte_part() {
    let r = eval_string(r#"new Blob([[104, 105]]).text()"#).unwrap();
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
    let r = eval_string(r#"
        const blob = new Blob(["hello, world!"]);
        const dec = new TextDecoder();
        dec.decode(blob.bytes())
    "#).unwrap();
    assert_eq!(r, "hello, world!");
}

#[test]
fn js_compose_file_in_blob_part() {
    let r = eval_string(r#"
        const inner = new File(["inner content"], "inner.txt");
        const wrapper = new Blob([inner]);
        wrapper.text()
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
    let r = eval_string(r#"
        const req = new Request("/", {method: "POST", body: "payload"});
        req.text()
    "#).unwrap();
    assert_eq!(r, "payload");
}

#[test]
fn js_request_clone_preserves_state() {
    let r = eval_string(r#"
        const orig = new Request("/users", {method: "POST", body: "data"});
        const clone = orig.clone();
        clone.method + "|" + clone.text()
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
    let r = eval_string(r#"new Response("hello").text()"#).unwrap();
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
    let r = eval_string(r#"
        const blob = new Blob(["payload"]);
        const resp = new Response(blob.bytes());
        resp.text()
    "#).unwrap();
    assert_eq!(r, "payload");
}

#[test]
fn js_compose_request_response_roundtrip() {
    let r = eval_string(r#"
        const req = new Request("/users", {method: "POST", body: "input"});
        const resp = new Response(req.text(), {status: 201});
        resp.status + ":" + resp.text()
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
        const result = resp.text();
        fs.unlinkSync(path);
        result
    "#, tmp);
    let r = eval_string(&script).unwrap();
    assert_eq!(r, "static file content");
}
