// Verifier for the fetch-api pilot — Headers + Request + Response.
//
// CD-H = runs/2026-05-10-bun-v0.13b-spec-batch/constraints/headers.constraints.md
// CD-Q = ditto request.constraints.md
// CD-S = ditto response.constraints.md
// SPEC = WHATWG Fetch §§5.2, 6.2, 6.4

use rusty_fetch_api::*;

// ════════════════════ HEADERS ════════════════════

// CD-H HEAD1: typeof Headers !== "undefined"
#[test]
fn cd_h_class_exists() { let _ = Headers::new(); }

// CD-H HEAD1: empty headers count is 0
#[test]
fn cd_h_count_empty_is_zero() {
    let h = Headers::new();
    assert_eq!(h.count(), 0);
}

// CD-H HEAD2: append throws TypeError on invalid name
#[test]
fn cd_h_append_invalid_name_errors() {
    let mut h = Headers::new();
    let r = h.append("invalid name", "value");
    assert!(matches!(r, Err(HeaderError::InvalidName(_))));
}

// CD-H HEAD2: append throws TypeError on invalid value
#[test]
fn cd_h_append_invalid_value_errors() {
    let mut h = Headers::new();
    let r = h.append("X-Custom", "value\nwith\nnewlines");
    assert!(matches!(r, Err(HeaderError::InvalidValue(_))));
}

// CD-H HEAD5: set throws on invalid name or value
#[test]
fn cd_h_set_invalid_errors() {
    let mut h = Headers::new();
    assert!(h.set("bad name", "v").is_err());
    assert!(h.set("X", "v\rwith\rcr").is_err());
}

#[test]
fn spec_h_get_combines_repeated_with_comma_space() {
    let mut h = Headers::new();
    h.append("Accept", "text/html").unwrap();
    h.append("Accept", "application/json").unwrap();
    assert_eq!(h.get("accept"), Some("text/html, application/json".to_string()));
}

#[test]
fn spec_h_case_insensitive_get() {
    let mut h = Headers::new();
    h.append("Content-Type", "application/json").unwrap();
    assert_eq!(h.get("content-type"), Some("application/json".to_string()));
    assert_eq!(h.get("CONTENT-TYPE"), Some("application/json".to_string()));
    assert_eq!(h.get("Content-Type"), Some("application/json".to_string()));
}

#[test]
fn spec_h_delete_case_insensitive() {
    let mut h = Headers::new();
    h.append("X-Foo", "1").unwrap();
    h.delete("x-foo");
    assert!(!h.has("X-Foo"));
}

#[test]
fn spec_h_set_replaces_all_existing() {
    let mut h = Headers::new();
    h.append("X", "1").unwrap();
    h.append("X", "2").unwrap();
    h.set("X", "only").unwrap();
    assert_eq!(h.get("X"), Some("only".to_string()));
}

#[test]
fn spec_h_value_whitespace_stripped() {
    let mut h = Headers::new();
    h.append("X", "   value with spaces   \t").unwrap();
    assert_eq!(h.get("X"), Some("value with spaces".to_string()));
}

#[test]
fn spec_h_get_set_cookie_returns_separate_values() {
    let mut h = Headers::new();
    h.append("Set-Cookie", "a=1").unwrap();
    h.append("Set-Cookie", "b=2").unwrap();
    h.append("Set-Cookie", "c=3").unwrap();
    let cookies = h.get_set_cookie();
    assert_eq!(cookies, vec!["a=1", "b=2", "c=3"]);
}

#[test]
fn spec_h_iteration_lowercases_names() {
    let mut h = Headers::new();
    h.append("X-Custom-Header", "v").unwrap();
    let names: Vec<&str> = h.keys().collect();
    assert_eq!(names, vec!["x-custom-header"]);
}

// ════════════════════ REQUEST ════════════════════

// CD-Q REQU1: Request is a global constructor
#[test]
fn cd_q_class_exists() {
    let _ = Request::new("https://example.com", Default::default()).unwrap();
}

// CD-Q REQU1: req.body is null when no body provided
#[test]
fn cd_q_body_null_default() {
    let r = Request::new("https://example.com", Default::default()).unwrap();
    assert!(r.body_is_null());
}

#[test]
fn spec_q_default_method_is_get() {
    let r = Request::new("https://example.com", Default::default()).unwrap();
    assert_eq!(r.method(), "GET");
}

#[test]
fn spec_q_method_from_init() {
    let r = Request::new("https://example.com", RequestInit {
        method: Some("POST".into()),
        ..Default::default()
    }).unwrap();
    assert_eq!(r.method(), "POST");
}

#[test]
fn spec_q_url_preserved() {
    let r = Request::new("https://example.com/path?q=1", Default::default()).unwrap();
    assert_eq!(r.url(), "https://example.com/path?q=1");
}

#[test]
fn spec_q_default_mode_credentials_cache_redirect() {
    let r = Request::new("https://example.com", Default::default()).unwrap();
    assert_eq!(r.mode(), "cors");
    assert_eq!(r.credentials(), "same-origin");
    assert_eq!(r.cache(), "default");
    assert_eq!(r.redirect(), "follow");
}

#[test]
fn spec_q_text_body_consumed_once() {
    let r = Request::new("https://example.com", RequestInit {
        method: Some("POST".into()),
        body: Some(Body::Text("payload".into())),
        ..Default::default()
    }).unwrap();
    assert_eq!(r.text().unwrap(), "payload");
    assert!(r.body_used());
    let r2 = r.text();
    assert!(matches!(r2, Err(RequestError::BodyError(_))));
}

#[test]
fn spec_q_clone_throws_when_body_used() {
    let r = Request::new("https://example.com", RequestInit {
        body: Some(Body::Text("x".into())),
        ..Default::default()
    }).unwrap();
    let _ = r.text().unwrap();
    let r2 = r.clone_request();
    assert!(r2.is_err());
}

#[test]
fn spec_q_clone_preserves_state_when_body_unused() {
    let r = Request::new("https://example.com", RequestInit {
        method: Some("POST".into()),
        body: Some(Body::Text("x".into())),
        ..Default::default()
    }).unwrap();
    let r2 = r.clone_request().unwrap();
    assert_eq!(r2.method(), "POST");
    assert_eq!(r2.url(), "https://example.com");
    assert_eq!(r2.text().unwrap(), "x");
}

// ════════════════════ RESPONSE ════════════════════

// CD-S: typeof Response !== "undefined"
#[test]
fn cd_s_class_exists() {
    let _ = Response::new(None, Default::default()).unwrap();
}

// CD-S: response.status default is 200
#[test]
fn cd_s_default_status_is_200() {
    let r = Response::new(None, Default::default()).unwrap();
    assert_eq!(r.status(), 200);
}

// CD-S: response.ok is true for 200
#[test]
fn cd_s_ok_for_200() {
    let r = Response::new(None, Default::default()).unwrap();
    assert!(r.ok());
}

#[test]
fn cd_s_ok_false_for_404() {
    let r = Response::new(None, ResponseInit { status: Some(404), ..Default::default() }).unwrap();
    assert!(!r.ok());
}

// CD-S: text() returns body as string
#[test]
fn cd_s_text_returns_body() {
    let r = Response::new(
        Some(Body::Text("hello world".into())),
        Default::default(),
    ).unwrap();
    assert_eq!(r.text().unwrap(), "hello world");
}

// CD-S: response.headers.get("content-type") works
#[test]
fn cd_s_headers_accessible() {
    let mut headers = Headers::new();
    headers.set("Content-Type", "text/plain").unwrap();
    let r = Response::new(
        Some(Body::Text("x".into())),
        ResponseInit { headers: Some(headers), ..Default::default() },
    ).unwrap();
    assert_eq!(
        r.headers().get("content-type"),
        Some("text/plain".to_string())
    );
}

// CD-S: Response.json() sets Content-Type
#[test]
fn cd_s_json_sets_content_type() {
    let r = Response::json(r#"{"ok":true}"#, Default::default()).unwrap();
    assert_eq!(
        r.headers().get("content-type"),
        Some("application/json".to_string())
    );
}

#[test]
fn spec_s_status_out_of_range_errors() {
    let r = Response::new(None, ResponseInit { status: Some(99), ..Default::default() });
    assert!(matches!(r, Err(ResponseError::StatusOutOfRange(_))));
    let r = Response::new(None, ResponseInit { status: Some(600), ..Default::default() });
    assert!(matches!(r, Err(ResponseError::StatusOutOfRange(_))));
}

#[test]
fn spec_s_redirect_only_valid_codes() {
    for code in [301, 302, 303, 307, 308] {
        assert!(Response::redirect("https://example.com", code).is_ok());
    }
    for bad in [200, 304, 305, 306, 309, 400] {
        let r = Response::redirect("https://example.com", bad);
        assert!(matches!(r, Err(ResponseError::InvalidRedirectStatus(_))),
            "code {} should be invalid redirect", bad);
    }
}

#[test]
fn spec_s_redirect_sets_location_header() {
    let r = Response::redirect("https://target.example.com/path", 301).unwrap();
    assert_eq!(
        r.headers().get("location"),
        Some("https://target.example.com/path".to_string())
    );
    assert_eq!(r.status(), 301);
}

#[test]
fn spec_s_error_response_has_type_error_status_0() {
    let r = Response::error();
    assert_eq!(r.status(), 0);
    assert_eq!(r.response_type(), ResponseType::Error);
}

#[test]
fn spec_s_clone_throws_when_body_used() {
    let r = Response::new(Some(Body::Text("x".into())), Default::default()).unwrap();
    let _ = r.text().unwrap();
    let r2 = r.clone_response();
    assert!(r2.is_err());
}

#[test]
fn spec_s_clone_preserves_status_and_body() {
    let r = Response::new(
        Some(Body::Text("payload".into())),
        ResponseInit {
            status: Some(201),
            status_text: Some("Created".into()),
            ..Default::default()
        },
    ).unwrap();
    let r2 = r.clone_response().unwrap();
    assert_eq!(r2.status(), 201);
    assert_eq!(r2.status_text(), "Created");
    assert_eq!(r2.text().unwrap(), "payload");
}
