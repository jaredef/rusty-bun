// Consumer-regression suite for node-http.

use rusty_node_http::*;

// ────────── express — case-insensitive req.headers lookup ──────────
//
// Source: https://github.com/expressjs/express/blob/master/lib/request.js
//   `req.get(name)` lowercases the lookup; downstream middleware reads
//   `req.headers["content-type"]` directly.

#[test]
fn consumer_express_headers_case_insensitive_get() {
    let mut req = IncomingMessage::new();
    req.headers.set("Content-Type", "application/json");
    assert_eq!(req.headers.get("content-type"), Some("application/json"));
    assert_eq!(req.headers.get("CONTENT-TYPE"), Some("application/json"));
}

// ────────── koa — res.setHeader before writeHead ──────────
//
// Source: https://github.com/koajs/koa/blob/master/lib/response.js
//   Koa's response wraps `res.setHeader` and expects setHeader before
//   write to apply to the eventual response.

#[test]
fn consumer_koa_set_header_before_write_head_applied() {
    let mut res = ServerResponse::new();
    res.set_header("X-Powered-By", "Koa");
    res.write_head(200, None, None);
    assert_eq!(res.get_header("x-powered-by"), Some("Koa"));
}

// ────────── supertest — full request/response roundtrip via dispatch ──
//
// Source: https://github.com/ladjs/supertest
//   supertest exercises Express handlers without binding a real socket;
//   the data-layer dispatch is what supertest tests against.

#[test]
fn consumer_supertest_dispatch_roundtrip() {
    let server = create_server(|req, res| {
        if req.url == "/users" && req.method == "GET" {
            res.write_head(200, None, Some(&[("Content-Type", "application/json")]));
            res.end_str(r#"[{"id":1}]"#);
        } else {
            res.write_head(404, None, None);
        }
    });
    let mut req = IncomingMessage::new();
    req.method = "GET".into();
    req.url = "/users".into();
    let res = server.dispatch(&req);
    assert_eq!(res.status_code, 200);
    assert_eq!(res.get_header("content-type"), Some("application/json"));
    assert_eq!(res.body(), br#"[{"id":1}]"#);
}

// ────────── axios — body assembly via write+end ──────────
//
// Source: axios server-side adapter writes JSON body via `req.write(body); req.end()`.
//   consumer expects body bytes preserved.

#[test]
fn consumer_axios_request_body_assembly() {
    let mut req = ClientRequest::new("POST", "https://api.example.com/users");
    req.set_header("Content-Type", "application/json");
    req.write(b"{\"name\":");
    req.write(b"\"Alice\"}");
    req.end(None);
    assert_eq!(req.body(), b"{\"name\":\"Alice\"}");
}

// ────────── http-proxy — header forwarding preserves order ──────────
//
// Source: https://github.com/http-party/node-http-proxy
//   forwards request headers in original order. Pilot's NodeHeaders
//   preserves insertion order in iteration.

#[test]
fn consumer_http_proxy_header_iteration_order() {
    let mut h = NodeHeaders::new();
    h.append("X-Forwarded-For", "1.2.3.4");
    h.append("X-Forwarded-Proto", "https");
    h.append("X-Real-IP", "5.6.7.8");
    let names: Vec<&str> = h.entries().map(|(n, _)| n).collect();
    assert_eq!(names, vec!["x-forwarded-for", "x-forwarded-proto", "x-real-ip"]);
}

// ────────── helmet — security headers via setHeader ──────────
//
// Source: https://github.com/helmetjs/helmet
//   helmet middleware sets multiple security headers via res.setHeader;
//   consumers expect lookup case-insensitivity.

#[test]
fn consumer_helmet_security_headers_set() {
    let mut res = ServerResponse::new();
    res.set_header("X-Frame-Options", "DENY");
    res.set_header("X-Content-Type-Options", "nosniff");
    res.set_header("Strict-Transport-Security", "max-age=63072000");
    assert_eq!(res.get_header("x-frame-options"), Some("DENY"));
    assert_eq!(res.get_header("X-CONTENT-TYPE-OPTIONS"), Some("nosniff"));
}

// ────────── compression — Content-Length removal after compress ──────
//
// Source: https://github.com/expressjs/compression
//   compression middleware removes Content-Length and adds Transfer-Encoding
//   after compressing. Consumer expects res.removeHeader to work.

#[test]
fn consumer_compression_remove_content_length() {
    let mut res = ServerResponse::new();
    res.set_header("Content-Length", "100");
    res.set_header("Content-Type", "text/html");
    res.remove_header("Content-Length");
    res.set_header("Transfer-Encoding", "chunked");
    assert!(res.get_header("content-length").is_none());
    assert_eq!(res.get_header("transfer-encoding"), Some("chunked"));
}

// ────────── client abort — fetch-timeout pattern ──────────
//
// Source: many AbortController-based timeout patterns where a request is
// aborted on timeout. Consumer expects req.aborted to flag the state.

#[test]
fn consumer_timeout_abort_pattern() {
    let mut req = ClientRequest::new("GET", "/slow-endpoint");
    req.write(b"data");
    req.abort();
    assert!(req.aborted());
    // Body remains as-is; no further writes accepted.
    req.write(b"never");
    assert_eq!(req.body(), b"data");
}
