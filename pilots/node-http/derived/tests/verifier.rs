// Verifier for the node-http pilot.

use rusty_node_http::*;

// ════════════════════ HEADERS ════════════════════

#[test]
fn spec_headers_set_lowercases_name() {
    let mut h = NodeHeaders::new();
    h.set("Content-Type", "text/html");
    assert_eq!(h.get("content-type"), Some("text/html"));
    assert_eq!(h.get("CONTENT-TYPE"), Some("text/html"));
}

#[test]
fn spec_headers_set_replaces_existing() {
    let mut h = NodeHeaders::new();
    h.set("X", "1");
    h.set("X", "2");
    assert_eq!(h.get("X"), Some("2"));
    assert_eq!(h.count(), 1);
}

#[test]
fn spec_headers_remove() {
    let mut h = NodeHeaders::new();
    h.set("X", "1");
    h.remove("x");
    assert!(!h.has("X"));
}

#[test]
fn spec_headers_as_object_lowercased() {
    let mut h = NodeHeaders::new();
    h.set("Accept", "application/json");
    h.set("X-Custom", "v");
    let o = h.as_object();
    assert_eq!(o.get("accept"), Some(&"application/json".to_string()));
    assert_eq!(o.get("x-custom"), Some(&"v".to_string()));
}

// ════════════════════ INCOMING MESSAGE ════════════════════

#[test]
fn spec_incoming_construction() {
    let mut m = IncomingMessage::new();
    m.method = "GET".into();
    m.url = "/path".into();
    m.headers.set("host", "example.com");
    assert_eq!(m.method, "GET");
    assert_eq!(m.url, "/path");
    assert_eq!(m.headers.get("Host"), Some("example.com"));
}

// ════════════════════ SERVER RESPONSE ════════════════════

#[test]
fn spec_server_response_default_status_200() {
    let r = ServerResponse::new();
    assert_eq!(r.status_code, 200);
    assert_eq!(r.status_message, "OK");
}

#[test]
fn spec_server_response_write_head_sets_status_and_headers() {
    let mut r = ServerResponse::new();
    r.write_head(404, Some("Not Found"), Some(&[("Content-Type", "text/plain")]));
    assert_eq!(r.status_code, 404);
    assert_eq!(r.status_message, "Not Found");
    assert_eq!(r.get_header("content-type"), Some("text/plain"));
    assert!(r.headers_sent());
}

#[test]
fn spec_server_response_write_appends_to_body() {
    let mut r = ServerResponse::new();
    r.write_str("hello ");
    r.write_str("world");
    assert_eq!(r.body(), b"hello world");
}

#[test]
fn spec_server_response_end_finalizes() {
    let mut r = ServerResponse::new();
    r.write_str("partial");
    r.end_str(" end");
    assert_eq!(r.body(), b"partial end");
    assert!(r.ended());
}

#[test]
fn spec_server_response_end_with_no_chunk() {
    let mut r = ServerResponse::new();
    r.write_str("body");
    r.end(None);
    assert_eq!(r.body(), b"body");
    assert!(r.ended());
}

#[test]
fn spec_server_response_writes_after_end_ignored() {
    let mut r = ServerResponse::new();
    r.end_str("done");
    r.write_str("ignored");
    assert_eq!(r.body(), b"done");
}

#[test]
fn spec_server_response_set_get_header() {
    let mut r = ServerResponse::new();
    r.set_header("Cache-Control", "no-cache");
    assert_eq!(r.get_header("cache-control"), Some("no-cache"));
}

// ════════════════════ CLIENT REQUEST ════════════════════

#[test]
fn spec_client_request_construction() {
    let req = ClientRequest::new("POST", "https://api.example.com/users");
    assert_eq!(req.method, "POST");
    assert_eq!(req.url, "https://api.example.com/users");
}

#[test]
fn spec_client_request_set_headers() {
    let mut req = ClientRequest::new("GET", "/");
    req.set_header("Authorization", "Bearer token");
    assert_eq!(req.headers().get("authorization"), Some("Bearer token"));
}

#[test]
fn spec_client_request_write_then_end_assembles_body() {
    let mut req = ClientRequest::new("POST", "/");
    req.write(b"payload ");
    req.end(Some(b"bytes"));
    assert_eq!(req.body(), b"payload bytes");
    assert!(req.ended());
}

#[test]
fn spec_client_request_abort() {
    let mut req = ClientRequest::new("GET", "/");
    req.write(b"x");
    req.abort();
    assert!(req.aborted());
    // Subsequent writes are ignored
    req.write(b"y");
    assert_eq!(req.body(), b"x");
}

// ════════════════════ SERVER ════════════════════

#[test]
fn spec_create_server_with_handler() {
    let server = create_server(|req, res| {
        if req.url == "/health" {
            res.write_head(200, None, None);
            res.end_str("ok");
        } else {
            res.write_head(404, None, None);
        }
    });

    let mut req = IncomingMessage::new();
    req.method = "GET".into();
    req.url = "/health".into();
    let res = server.dispatch(&req);
    assert_eq!(res.status_code, 200);
    assert_eq!(res.body(), b"ok");
}

#[test]
fn spec_server_listen_records_state() {
    let mut s = create_server(|_req, _res| {});
    assert!(!s.listening());
    s.listen(8080);
    assert!(s.listening());
    assert_eq!(s.port(), 8080);
}

#[test]
fn spec_server_close_transitions() {
    let mut s = create_server(|_req, _res| {});
    s.listen(3000);
    s.close();
    assert!(!s.listening());
    assert!(s.closed());
}

#[test]
fn spec_server_dispatch_404_with_no_handler_match() {
    let server = create_server(|_req, res| {
        res.write_head(404, Some("Not Found"), None);
    });
    let mut req = IncomingMessage::new();
    req.method = "GET".into();
    req.url = "/missing".into();
    let res = server.dispatch(&req);
    assert_eq!(res.status_code, 404);
    assert_eq!(res.status_message, "Not Found");
}

// ════════════════════ TOP-LEVEL request() ════════════════════

#[test]
fn spec_request_factory_with_headers() {
    let req = request("POST", "/api", Some(&[
        ("Content-Type", "application/json"),
        ("Accept", "application/json"),
    ]));
    assert_eq!(req.method, "POST");
    assert_eq!(req.headers().get("content-type"), Some("application/json"));
}
