// Verifier for the Bun.serve pilot.
//
// CD = runs/2026-05-10-bun-v0.13b-spec-batch/constraints/bun.constraints.md
//      (Bun.serve cluster, sparse direct attribution due to local-binding
//       canonicalization)
// REF = Bun docs at https://bun.sh/docs/api/http

use rusty_bun_serve::*;

fn req(method: &str, url: &str) -> Request {
    Request::new(url, RequestInit {
        method: Some(method.into()),
        ..Default::default()
    }).unwrap()
}

// ════════════════════ CONSTRUCTION ════════════════════

#[test]
fn cd_serve_construction_default() {
    let s = serve(Default::default());
    assert_eq!(s.port(), 3000);
    assert_eq!(s.hostname(), "localhost");
    assert!(s.is_listening());
}

#[test]
fn cd_serve_url_includes_hostname_and_port() {
    let s = serve(ServeOptions {
        port: 8080,
        hostname: "0.0.0.0".into(),
        ..Default::default()
    });
    assert_eq!(s.url(), "http://0.0.0.0:8080/");
}

#[test]
fn spec_serve_pending_requests_zero_initially() {
    let s = serve(Default::default());
    assert_eq!(s.pending_requests(), 0);
}

// ════════════════════ FETCH HANDLER ════════════════════

#[test]
fn cd_fetch_catchall_handler() {
    let mut s = serve(ServeOptions {
        fetch: Some(Box::new(|_req, _params| {
            Response::new(
                Some(Body::Text("hello".into())),
                Default::default(),
            ).unwrap()
        })),
        ..Default::default()
    });
    let r = s.fetch(&req("GET", "http://localhost/"));
    assert_eq!(r.status(), 200);
    assert_eq!(r.text().unwrap(), "hello");
}

#[test]
fn cd_no_handler_returns_404() {
    let mut s = serve(Default::default());
    let r = s.fetch(&req("GET", "http://localhost/"));
    assert_eq!(r.status(), 404);
}

#[test]
fn spec_fetch_after_stop_returns_error() {
    let mut s = serve(Default::default());
    s.stop();
    let r = s.fetch(&req("GET", "http://localhost/"));
    assert_eq!(r.response_type(), ResponseType::Error);
}

// ════════════════════ ROUTE MATCHING ════════════════════

#[test]
fn cd_route_static_path_match() {
    let mut s = serve(ServeOptions {
        routes: vec![Route::any("/health", |_req, _params| {
            Response::new(
                Some(Body::Text("ok".into())),
                Default::default(),
            ).unwrap()
        })],
        ..Default::default()
    });
    let r = s.fetch(&req("GET", "http://localhost/health"));
    assert_eq!(r.status(), 200);
    assert_eq!(r.text().unwrap(), "ok");
}

#[test]
fn cd_route_param_capture() {
    let mut s = serve(ServeOptions {
        routes: vec![Route::any("/users/:id", |_req, params| {
            let id = params.get("id").unwrap_or("?");
            Response::new(
                Some(Body::Text(format!("user-{}", id))),
                Default::default(),
            ).unwrap()
        })],
        ..Default::default()
    });
    let r = s.fetch(&req("GET", "http://localhost/users/42"));
    assert_eq!(r.text().unwrap(), "user-42");
}

#[test]
fn cd_route_multi_param_capture() {
    let mut s = serve(ServeOptions {
        routes: vec![Route::any("/posts/:postId/comments/:commentId", |_req, params| {
            Response::new(
                Some(Body::Text(format!(
                    "{}-{}",
                    params.get("postId").unwrap_or(""),
                    params.get("commentId").unwrap_or(""),
                ))),
                Default::default(),
            ).unwrap()
        })],
        ..Default::default()
    });
    let r = s.fetch(&req("GET", "http://localhost/posts/abc/comments/xyz"));
    assert_eq!(r.text().unwrap(), "abc-xyz");
}

#[test]
fn spec_route_no_match_falls_through_to_fetch() {
    let mut s = serve(ServeOptions {
        routes: vec![Route::any("/users/:id", |_req, _params| {
            Response::new(
                Some(Body::Text("user-handler".into())),
                Default::default(),
            ).unwrap()
        })],
        fetch: Some(Box::new(|_req, _params| {
            Response::new(
                Some(Body::Text("catchall".into())),
                Default::default(),
            ).unwrap()
        })),
        ..Default::default()
    });
    let r = s.fetch(&req("GET", "http://localhost/other"));
    assert_eq!(r.text().unwrap(), "catchall");
}

#[test]
fn spec_route_segment_count_must_match() {
    // /users/:id should NOT match /users/42/extra
    let mut s = serve(ServeOptions {
        routes: vec![Route::any("/users/:id", |_req, _params| {
            Response::new(
                Some(Body::Text("matched".into())),
                Default::default(),
            ).unwrap()
        })],
        ..Default::default()
    });
    let r = s.fetch(&req("GET", "http://localhost/users/42/extra"));
    assert_eq!(r.status(), 404);
}

#[test]
fn spec_route_trailing_slash_normalized() {
    let mut s = serve(ServeOptions {
        routes: vec![Route::any("/health", |_req, _params| {
            Response::new(
                Some(Body::Text("ok".into())),
                Default::default(),
            ).unwrap()
        })],
        ..Default::default()
    });
    // Both /health and /health/ should match
    assert_eq!(s.fetch(&req("GET", "http://localhost/health")).status(), 200);
    assert_eq!(s.fetch(&req("GET", "http://localhost/health/")).status(), 200);
}

// ════════════════════ METHOD-KEYED ROUTES ════════════════════

#[test]
fn cd_method_keyed_get_only() {
    let mut s = serve(ServeOptions {
        routes: vec![
            Route::methods("/api/items")
                .on("GET", |_req, _params| {
                    Response::new(
                        Some(Body::Text("list".into())),
                        Default::default(),
                    ).unwrap()
                })
                .on("POST", |_req, _params| {
                    Response::new(
                        Some(Body::Text("create".into())),
                        ResponseInit { status: Some(201), ..Default::default() },
                    ).unwrap()
                })
                .build(),
        ],
        ..Default::default()
    });
    let get = s.fetch(&req("GET", "http://localhost/api/items"));
    assert_eq!(get.text().unwrap(), "list");
    let post = s.fetch(&req("POST", "http://localhost/api/items"));
    assert_eq!(post.status(), 201);
    assert_eq!(post.text().unwrap(), "create");
}

#[test]
fn spec_method_keyed_unknown_method_returns_405() {
    let mut s = serve(ServeOptions {
        routes: vec![
            Route::methods("/api/items")
                .on("GET", |_req, _params| {
                    Response::new(
                        Some(Body::Text("list".into())),
                        Default::default(),
                    ).unwrap()
                })
                .build(),
        ],
        ..Default::default()
    });
    let delete = s.fetch(&req("DELETE", "http://localhost/api/items"));
    assert_eq!(delete.status(), 405);
}

#[test]
fn spec_route_priority_first_match_wins() {
    let mut s = serve(ServeOptions {
        routes: vec![
            Route::any("/a/:x", |_req, params| {
                Response::new(
                    Some(Body::Text(format!("first-{}", params.get("x").unwrap_or("")))),
                    Default::default(),
                ).unwrap()
            }),
            Route::any("/a/:y", |_req, params| {
                Response::new(
                    Some(Body::Text(format!("second-{}", params.get("y").unwrap_or("")))),
                    Default::default(),
                ).unwrap()
            }),
        ],
        ..Default::default()
    });
    let r = s.fetch(&req("GET", "http://localhost/a/value"));
    assert_eq!(r.text().unwrap(), "first-value");
}

// ════════════════════ ERROR HANDLER ════════════════════

#[test]
fn spec_error_handler_invoked_when_no_fetch_and_no_route() {
    let mut s = serve(ServeOptions {
        error: Some(Box::new(|_msg| {
            Response::new(
                Some(Body::Text("custom-error".into())),
                ResponseInit { status: Some(500), ..Default::default() },
            ).unwrap()
        })),
        ..Default::default()
    });
    let r = s.fetch(&req("GET", "http://localhost/"));
    assert_eq!(r.status(), 500);
    assert_eq!(r.text().unwrap(), "custom-error");
}

// ════════════════════ RELOAD ════════════════════

#[test]
fn spec_reload_swaps_handler() {
    let mut s = serve(ServeOptions {
        fetch: Some(Box::new(|_req, _params| {
            Response::new(
                Some(Body::Text("v1".into())),
                Default::default(),
            ).unwrap()
        })),
        ..Default::default()
    });
    assert_eq!(s.fetch(&req("GET", "http://localhost/")).text().unwrap(), "v1");

    s.reload(ServeOptions {
        fetch: Some(Box::new(|_req, _params| {
            Response::new(
                Some(Body::Text("v2".into())),
                Default::default(),
            ).unwrap()
        })),
        ..Default::default()
    });
    assert_eq!(s.fetch(&req("GET", "http://localhost/")).text().unwrap(), "v2");
}

#[test]
fn spec_reload_preserves_port_and_hostname() {
    let mut s = serve(ServeOptions {
        port: 9999,
        hostname: "0.0.0.0".into(),
        ..Default::default()
    });
    s.reload(ServeOptions {
        port: 1234,        // pilot ignores per spec
        hostname: "x".into(),
        ..Default::default()
    });
    assert_eq!(s.port(), 9999);
    assert_eq!(s.hostname(), "0.0.0.0");
}

// ════════════════════ STOP ════════════════════

#[test]
fn spec_stop_transitions_state() {
    let mut s = serve(Default::default());
    assert!(s.is_listening());
    s.stop();
    assert!(!s.is_listening());
    assert_eq!(s.state(), &ServerState::Stopped);
}

// ════════════════════ PATTERN MATCHER (UNIT) ════════════════════

#[test]
fn spec_match_static_path_exact() {
    let p = match_pattern("/health", "http://localhost/health");
    assert!(p.is_some());
}

#[test]
fn spec_match_param_captures() {
    let p = match_pattern("/users/:id", "http://localhost/users/42").unwrap();
    assert_eq!(p.get("id"), Some("42"));
}

#[test]
fn spec_match_with_query_string() {
    let p = match_pattern("/health", "http://localhost/health?v=1");
    assert!(p.is_some());
}

#[test]
fn spec_match_path_only() {
    let p = match_pattern("/users/:id", "/users/abc").unwrap();
    assert_eq!(p.get("id"), Some("abc"));
}

#[test]
fn spec_match_no_match_returns_none() {
    assert!(match_pattern("/users/:id", "http://localhost/posts/42").is_none());
    assert!(match_pattern("/users/:id", "http://localhost/users").is_none());
}
