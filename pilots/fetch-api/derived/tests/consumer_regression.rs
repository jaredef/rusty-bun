// Consumer-regression suite for the fetch-api system pilot
// (Headers + Request + Response). Each test cites a real consumer.

use rusty_fetch_api::*;

// ────────── undici / node-fetch — Response constructor ──────────
//
// Source: https://github.com/nodejs/undici/blob/main/lib/web/fetch/response.js
//   `Response` constructor validates status range and constructs body
//   per spec. Many node-fetch tests exercise this directly.

#[test]
fn consumer_undici_response_default_status_200() {
    let r = Response::new(None, Default::default()).unwrap();
    assert_eq!(r.status(), 200);
    assert!(r.ok());
}

#[test]
fn consumer_undici_response_status_text_preserved() {
    let r = Response::new(
        None,
        ResponseInit { status: Some(204), status_text: Some("No Content".into()), ..Default::default() },
    ).unwrap();
    assert_eq!(r.status_text(), "No Content");
}

// ────────── ky — Response.json typed extraction ──────────
//
// Source: https://github.com/sindresorhus/ky/blob/main/source/core/Ky.ts
//   `_decorateResponse` calls `response.json()` after checking
//   Content-Type. Consumer expectation: Response.json static returns a
//   Response whose Content-Type is application/json and whose text() yields
//   the JSON serialization.

#[test]
fn consumer_ky_response_json_static_sets_content_type() {
    let r = Response::json(r#"{"k":1}"#, Default::default()).unwrap();
    assert_eq!(r.headers().get("content-type"), Some("application/json".to_string()));
    assert_eq!(r.text().unwrap(), r#"{"k":1}"#);
}

// ────────── express middleware — req.headers case-insensitive lookup ──
//
// Source: https://github.com/expressjs/express/blob/master/lib/request.js
//   `req.get(name)` is case-insensitive; many middleware paths read
//   `req.headers["content-type"]` directly. Consumer expectation: Headers'
//   get is case-insensitive regardless of insertion case.

#[test]
fn consumer_express_headers_case_insensitive() {
    let mut h = Headers::new();
    h.append("Content-Type", "application/json").unwrap();
    assert_eq!(h.get("CONTENT-TYPE"), Some("application/json".to_string()));
    assert_eq!(h.get("content-type"), Some("application/json".to_string()));
}

// ────────── Stripe SDK — multiple Set-Cookie values ──────────
//
// Source: https://github.com/stripe/stripe-node/blob/master/src/Webhooks.ts
//   reads multiple Set-Cookie headers via Headers.getSetCookie() (newer
//   Node versions). Consumer expectation: getSetCookie returns each value
//   separately, NOT joined.

#[test]
fn consumer_stripe_get_set_cookie_separate_values() {
    let mut h = Headers::new();
    h.append("Set-Cookie", "session=abc").unwrap();
    h.append("Set-Cookie", "csrf=xyz").unwrap();
    let cookies = h.get_set_cookie();
    assert_eq!(cookies.len(), 2);
    assert!(cookies.contains(&"session=abc".to_string()));
    assert!(cookies.contains(&"csrf=xyz".to_string()));
}

// ────────── @koa/router — request method dispatch ──────────
//
// Source: https://github.com/koajs/router/blob/master/lib/router.js
//   dispatches based on `req.method.toUpperCase()`. Consumer expectation:
//   default Request method is "GET" (case-preserving) so default routes hit.

#[test]
fn consumer_koa_default_method_is_get() {
    let r = Request::new("https://api.example.com", Default::default()).unwrap();
    assert_eq!(r.method(), "GET");
}

// ────────── axios fallback — Request body consumption ──────────
//
// Source: https://github.com/axios/axios/blob/v1.x/lib/adapters/fetch.js
//   reads response body once via `res.text()` or `res.arrayBuffer()`.
//   Consumer expectation: second consumption errors, not silent re-yield.

#[test]
fn consumer_axios_body_consumed_once_errors_on_second() {
    let r = Response::new(Some(Body::Text("payload".into())), Default::default()).unwrap();
    assert_eq!(r.text().unwrap(), "payload");
    let r2 = r.text();
    assert!(r2.is_err());
}

// ────────── Cloudflare Workers — Response.redirect status restriction ─
//
// Source: https://developers.cloudflare.com/workers/runtime-apis/response/
//   Cloudflare's docs explicitly note Response.redirect rejects non-redirect
//   status codes; production code relies on this rejection for safety.

#[test]
fn consumer_cloudflare_redirect_rejects_non_redirect_status() {
    assert!(Response::redirect("https://x.example", 200).is_err());
    assert!(Response::redirect("https://x.example", 304).is_err());
    assert!(Response::redirect("https://x.example", 500).is_err());
}

#[test]
fn consumer_cloudflare_redirect_accepts_canonical_redirect_codes() {
    for code in [301, 302, 303, 307, 308] {
        assert!(Response::redirect("https://x.example", code).is_ok(),
            "code {} should be a valid redirect", code);
    }
}

// ────────── HTTP/3 reverse proxies — body bytes round-trip ──────────
//
// Source: many reverse-proxy implementations (haproxy-rs, hyper-tower)
// pipe Response bodies through bytes() unchanged. Consumer expectation:
// arrayBuffer/bytes returns the body as raw bytes, no transcoding.

#[test]
fn consumer_proxy_body_bytes_passthrough() {
    let payload = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0xFF];
    let r = Response::new(
        Some(Body::Bytes(payload.clone())),
        Default::default(),
    ).unwrap();
    assert_eq!(r.bytes().unwrap(), payload);
}

// ────────── WPT Fetch test corpus ──────────
//
// Source: web-platform-tests/wpt/fetch/api/response/

#[test]
fn wpt_response_constructor_init_status_text() {
    let r = Response::new(
        None,
        ResponseInit { status: Some(418), status_text: Some("I'm a teapot".into()), ..Default::default() },
    ).unwrap();
    assert_eq!(r.status(), 418);
    assert_eq!(r.status_text(), "I'm a teapot");
}

#[test]
fn wpt_response_clone_independent_body() {
    let r = Response::new(Some(Body::Text("data".into())), Default::default()).unwrap();
    let r2 = r.clone_response().unwrap();
    assert_eq!(r2.text().unwrap(), "data");
    assert!(!r.body_used()); // original still unconsumed
    assert_eq!(r.text().unwrap(), "data");
}

#[test]
fn wpt_response_error_type_and_status() {
    let r = Response::error();
    assert_eq!(r.response_type(), ResponseType::Error);
    assert_eq!(r.status(), 0);
}

// ────────── Cross-surface integration ──────────
//
// These tests verify the apparatus' multi-surface composition rather
// than any single consumer.

#[test]
fn integration_request_headers_round_trip_through_clone() {
    let mut h = Headers::new();
    h.append("Authorization", "Bearer xyz").unwrap();
    h.append("X-Trace-Id", "abc-123").unwrap();
    let r = Request::new(
        "https://api.example.com",
        RequestInit {
            method: Some("POST".into()),
            headers: Some(h),
            body: Some(Body::Text("{}".into())),
            ..Default::default()
        },
    ).unwrap();
    let r2 = r.clone_request().unwrap();
    assert_eq!(
        r2.headers().get("authorization"),
        Some("Bearer xyz".to_string())
    );
    assert_eq!(r2.method(), "POST");
}

#[test]
fn integration_response_with_headers_constructed_externally() {
    let mut headers = Headers::new();
    headers.set("Cache-Control", "max-age=3600").unwrap();
    headers.set("Content-Type", "text/html").unwrap();
    let r = Response::new(
        Some(Body::Text("<html></html>".into())),
        ResponseInit {
            status: Some(200),
            headers: Some(headers),
            ..Default::default()
        },
    ).unwrap();
    assert_eq!(r.headers().get("cache-control"), Some("max-age=3600".to_string()));
    assert_eq!(r.headers().get("content-type"), Some("text/html".to_string()));
}

#[test]
fn integration_redirect_response_has_location_and_correct_status() {
    let r = Response::redirect("https://target.example.com/", 308).unwrap();
    assert_eq!(r.status(), 308);
    assert_eq!(
        r.headers().get("location"),
        Some("https://target.example.com/".to_string())
    );
    // Per SPEC, redirect responses have a location header but no body.
    assert!(r.body_is_null());
}
