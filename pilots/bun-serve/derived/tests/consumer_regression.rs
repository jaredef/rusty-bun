// Consumer-regression suite for Bun.serve.

use rusty_bun_serve::*;

fn req(method: &str, url: &str) -> Request {
    Request::new(url, RequestInit {
        method: Some(method.into()),
        ..Default::default()
    }).unwrap()
}

// ────────── Bun docs canonical example — fetch handler ──────────
//
// Source: https://bun.sh/docs/api/http
//   `Bun.serve({ fetch(req) { return new Response("hello"); } })`
// Consumer expectation: catch-all `fetch` handler is invoked for any request.

#[test]
fn consumer_bun_docs_fetch_canonical() {
    let mut s = serve(ServeOptions {
        fetch: Some(Box::new(|_req, _params| {
            Response::new(
                Some(Body::Text("hello".into())),
                Default::default(),
            ).unwrap()
        })),
        ..Default::default()
    });
    let r = s.fetch(&req("GET", "http://localhost/anything"));
    assert_eq!(r.text().unwrap(), "hello");
}

// ────────── Hono framework — Hono adapter for Bun.serve ──────────
//
// Source: https://github.com/honojs/hono/blob/main/src/adapter/bun/serve.ts
//   Hono dispatches via `Bun.serve({ fetch: app.fetch })`. Hono's app.fetch
//   produces JSON responses with explicit Content-Type via Response.json.

#[test]
fn consumer_hono_json_response_pattern() {
    let mut s = serve(ServeOptions {
        fetch: Some(Box::new(|_req, _params| {
            Response::json(r#"{"message":"hello"}"#, Default::default()).unwrap()
        })),
        ..Default::default()
    });
    let r = s.fetch(&req("GET", "http://localhost/"));
    assert_eq!(r.headers().get("content-type"), Some("application/json".to_string()));
    assert_eq!(r.text().unwrap(), r#"{"message":"hello"}"#);
}

// ────────── ElysiaJS — high-throughput Bun framework ──────────
//
// Source: https://github.com/elysiajs/elysia
//   Elysia uses Bun.serve's routes object internally for static routes,
//   falling back to `fetch` for dynamic. Consumer relies on routes-first
//   priority order.

#[test]
fn consumer_elysia_routes_first_priority() {
    let mut s = serve(ServeOptions {
        routes: vec![
            Route::any("/healthz", |_req, _params| {
                Response::new(
                    Some(Body::Text("ok".into())),
                    Default::default(),
                ).unwrap()
            }),
        ],
        fetch: Some(Box::new(|_req, _params| {
            Response::new(
                Some(Body::Text("dynamic".into())),
                Default::default(),
            ).unwrap()
        })),
        ..Default::default()
    });
    // Route matches first, fetch never invoked
    assert_eq!(s.fetch(&req("GET", "http://localhost/healthz")).text().unwrap(), "ok");
    // No route matches → fetch invoked
    assert_eq!(s.fetch(&req("GET", "http://localhost/dynamic")).text().unwrap(), "dynamic");
}

// ────────── REST API — method-keyed routes ──────────
//
// Source: many third-party Bun-using REST frameworks. Pattern:
//   routes: { '/api/items': { GET: list, POST: create, DELETE: remove } }
// Consumer expectation: 405 Method Not Allowed for routes without that method.

#[test]
fn consumer_rest_api_method_dispatch() {
    let mut s = serve(ServeOptions {
        routes: vec![
            Route::methods("/api/items")
                .on("GET", |_req, _params| {
                    Response::new(
                        Some(Body::Text("[]".into())),
                        Default::default(),
                    ).unwrap()
                })
                .on("POST", |_req, _params| {
                    Response::new(
                        None,
                        ResponseInit { status: Some(201), ..Default::default() },
                    ).unwrap()
                })
                .build(),
        ],
        ..Default::default()
    });
    assert_eq!(s.fetch(&req("GET", "http://localhost/api/items")).status(), 200);
    assert_eq!(s.fetch(&req("POST", "http://localhost/api/items")).status(), 201);
    assert_eq!(s.fetch(&req("DELETE", "http://localhost/api/items")).status(), 405);
}

// ────────── Bun-native development workflow — server.reload ──────────
//
// Source: Bun docs at https://bun.sh/docs/api/http#hot-reload
//   `server.reload({...})` swaps the handler without binding a new port.
// Consumer expectation: reload preserves port + hostname; only the handler
// changes.

#[test]
fn consumer_bun_dev_hot_reload() {
    let mut s = serve(ServeOptions {
        port: 5555,
        hostname: "127.0.0.1".into(),
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
        port: 0, // ignored per spec
        hostname: "different".into(), // ignored per spec
        fetch: Some(Box::new(|_req, _params| {
            Response::new(
                Some(Body::Text("v2".into())),
                Default::default(),
            ).unwrap()
        })),
        ..Default::default()
    });
    assert_eq!(s.fetch(&req("GET", "http://localhost/")).text().unwrap(), "v2");
    assert_eq!(s.port(), 5555);
    assert_eq!(s.hostname(), "127.0.0.1");
}

// ────────── REST API — path parameters ──────────
//
// Source: standard REST API pattern across all Bun-using frameworks.
//   `/users/:id` → handler receives `params.id`.

#[test]
fn consumer_rest_path_parameter() {
    let mut s = serve(ServeOptions {
        routes: vec![Route::any("/users/:id", |_req, params| {
            let id = params.get("id").unwrap_or("?");
            Response::new(
                Some(Body::Text(format!(r#"{{"id":"{}"}}"#, id))),
                Default::default(),
            ).unwrap()
        })],
        ..Default::default()
    });
    let r = s.fetch(&req("GET", "http://localhost/users/abc-123"));
    assert_eq!(r.text().unwrap(), r#"{"id":"abc-123"}"#);
}

// ────────── Status checks — server.url, server.pendingRequests ──────────
//
// Source: ops/monitoring tools that introspect server state. Bun-using apps
// commonly log server.url at startup.

#[test]
fn consumer_ops_introspects_server_url() {
    let s = serve(ServeOptions {
        port: 8080,
        hostname: "0.0.0.0".into(),
        ..Default::default()
    });
    assert_eq!(s.url(), "http://0.0.0.0:8080/");
}

// ────────── Stop & restart — graceful shutdown ──────────
//
// Source: graceful-shutdown patterns (sending SIGTERM, server.stop). Consumer
// expectation: after stop, fetch returns an error response, not 200.

#[test]
fn consumer_graceful_shutdown_after_stop() {
    let mut s = serve(ServeOptions {
        fetch: Some(Box::new(|_req, _params| {
            Response::new(
                Some(Body::Text("ok".into())),
                Default::default(),
            ).unwrap()
        })),
        ..Default::default()
    });
    assert_eq!(s.fetch(&req("GET", "http://localhost/")).status(), 200);
    s.stop();
    assert_eq!(s.fetch(&req("GET", "http://localhost/")).response_type(), ResponseType::Error);
}
