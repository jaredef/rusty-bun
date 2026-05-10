// Bun.serve pilot — flagship Bun API at data-layer scope.
//
// Inputs:
//   AUDIT — pilots/bun-serve/AUDIT.md
//   CD    — runs/2026-05-10-bun-v0.13b-spec-batch/constraints/bun.constraints.md
//   REF   — Bun docs at https://bun.sh/docs/api/http
//
// Tier-2 ecosystem-only. No spec exists; Bun's tests + Bun docs are the
// authoritative reference. Pilot models the data-layer of Bun.serve:
// given a Request, what Response does the server produce?
//
// No actual socket binding, no HTTP wire format. The transport-layer
// derivation is a separate pilot (deferred).

pub use rusty_fetch_api::{Body, Headers, Request, RequestInit, Response, ResponseInit, ResponseType};

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerState {
    Listening,
    Stopped,
}

/// A handler is anything that takes a Request and returns a Response.
/// Wrapped in a Box<dyn Fn> so we can store heterogeneous handler types
/// in route tables.
pub type Handler = Box<dyn Fn(&Request, &RouteParams) -> Response>;

/// Captured `:param` values from a route pattern match.
#[derive(Debug, Clone, Default)]
pub struct RouteParams {
    pub captures: HashMap<String, String>,
}

impl RouteParams {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.captures.get(key).map(|s| s.as_str())
    }
}

/// One route entry: a pattern + per-method handlers (or a catch-all).
pub struct Route {
    pattern: String,
    /// Method → handler. Empty key "" means catch-all (any method).
    methods: HashMap<String, Handler>,
}

impl Route {
    pub fn any<F>(pattern: impl Into<String>, handler: F) -> Self
    where F: Fn(&Request, &RouteParams) -> Response + 'static
    {
        let mut methods = HashMap::new();
        methods.insert(String::new(), Box::new(handler) as Handler);
        Self { pattern: pattern.into(), methods }
    }

    pub fn methods(pattern: impl Into<String>) -> RouteBuilder {
        RouteBuilder {
            pattern: pattern.into(),
            methods: HashMap::new(),
        }
    }
}

pub struct RouteBuilder {
    pattern: String,
    methods: HashMap<String, Handler>,
}

impl RouteBuilder {
    pub fn on<F>(mut self, method: impl Into<String>, handler: F) -> Self
    where F: Fn(&Request, &RouteParams) -> Response + 'static
    {
        self.methods.insert(method.into(), Box::new(handler) as Handler);
        self
    }
    pub fn build(self) -> Route {
        Route { pattern: self.pattern, methods: self.methods }
    }
}

/// Server-construction options. Per Bun docs.
pub struct ServeOptions {
    pub port: u16,
    pub hostname: String,
    pub development: bool,
    pub routes: Vec<Route>,
    /// Catch-all: invoked when no route matches.
    pub fetch: Option<Handler>,
    /// Error handler: invoked when a handler returns an error response,
    /// or in the pilot model, when a handler is missing.
    pub error: Option<Box<dyn Fn(&str) -> Response>>,
}

impl Default for ServeOptions {
    fn default() -> Self {
        Self {
            port: 3000,
            hostname: "localhost".into(),
            development: false,
            routes: Vec::new(),
            fetch: None,
            error: None,
        }
    }
}

pub struct Server {
    options: ServeOptions,
    state: ServerState,
    pending_requests: u64,
}

impl Server {
    /// `Bun.serve(options)` — construct + transition to Listening.
    pub fn new(options: ServeOptions) -> Self {
        Self { options, state: ServerState::Listening, pending_requests: 0 }
    }

    pub fn port(&self) -> u16 { self.options.port }
    pub fn hostname(&self) -> &str { &self.options.hostname }
    pub fn url(&self) -> String {
        format!("http://{}:{}/", self.options.hostname, self.options.port)
    }
    pub fn pending_requests(&self) -> u64 { self.pending_requests }
    pub fn state(&self) -> &ServerState { &self.state }
    pub fn is_listening(&self) -> bool { matches!(self.state, ServerState::Listening) }

    /// `server.fetch(request)` — the data-layer core. Match routes first,
    /// then catch-all fetch handler, then error handler.
    pub fn fetch(&mut self, request: &Request) -> Response {
        if matches!(self.state, ServerState::Stopped) {
            return Response::error();
        }
        self.pending_requests += 1;
        let response = self.dispatch(request);
        // Decrement after the handler returns (data-layer model: no async).
        self.pending_requests -= 1;
        response
    }

    fn dispatch(&self, request: &Request) -> Response {
        // 1. Try routes in order.
        for route in &self.options.routes {
            if let Some(params) = match_pattern(&route.pattern, request.url()) {
                // Method dispatch: try exact method, then catch-all "".
                if let Some(handler) = route.methods.get(request.method()) {
                    return handler(request, &params);
                }
                if let Some(handler) = route.methods.get("") {
                    return handler(request, &params);
                }
                // Pattern matched but no handler for this method → 405.
                return Response::new(
                    None,
                    ResponseInit { status: Some(405), ..Default::default() },
                ).unwrap_or_else(|_| Response::error());
            }
        }
        // 2. Catch-all fetch handler.
        if let Some(handler) = &self.options.fetch {
            return handler(request, &RouteParams::default());
        }
        // 3. Error handler.
        if let Some(error_handler) = &self.options.error {
            return error_handler("no route matched and no fetch handler configured");
        }
        // 4. Default: 404.
        Response::new(
            None,
            ResponseInit { status: Some(404), ..Default::default() },
        ).unwrap_or_else(|_| Response::error())
    }

    /// `server.reload(newOptions)` — hot-reload handler/routes without
    /// changing port/hostname.
    pub fn reload(&mut self, new_options: ServeOptions) {
        // Per Bun docs: port/hostname can NOT change via reload.
        let port = self.options.port;
        let hostname = self.options.hostname.clone();
        self.options = new_options;
        self.options.port = port;
        self.options.hostname = hostname;
    }

    /// `server.stop()` — transition to Stopped state.
    pub fn stop(&mut self) {
        self.state = ServerState::Stopped;
    }
}

// ─────────────────────── Pattern matcher ────────────────────────────────
//
// Pattern syntax (subset):
//   /static        matches "/static" exactly
//   /users/:id     matches "/users/<anything>", captures id
//   /a/:x/b/:y     multiple captures
//
// Trailing slashes are normalized off both sides before matching.

pub fn match_pattern(pattern: &str, url: &str) -> Option<RouteParams> {
    // Extract path from URL (strip scheme + host + query).
    let path = extract_path(url);
    let pat_segs: Vec<&str> = strip_trailing_slash(pattern).trim_start_matches('/').split('/').collect();
    let path_segs: Vec<&str> = strip_trailing_slash(&path).trim_start_matches('/').split('/').collect();
    if pat_segs.len() != path_segs.len() {
        return None;
    }
    let mut params = RouteParams::default();
    for (p, u) in pat_segs.iter().zip(path_segs.iter()) {
        if let Some(name) = p.strip_prefix(':') {
            params.captures.insert(name.to_string(), u.to_string());
        } else if p != u {
            return None;
        }
    }
    Some(params)
}

fn extract_path(url: &str) -> String {
    // If it's an absolute URL, find scheme://host/ and take the rest.
    if let Some(after_scheme) = url.find("://") {
        let after = &url[after_scheme + 3..];
        let path_start = after.find('/').unwrap_or(after.len());
        let path_and_query = &after[path_start..];
        // Strip query string.
        match path_and_query.find('?') {
            Some(q) => path_and_query[..q].to_string(),
            None => path_and_query.to_string(),
        }
    } else {
        // Treat as path-only.
        match url.find('?') {
            Some(q) => url[..q].to_string(),
            None => url.to_string(),
        }
    }
}

fn strip_trailing_slash(s: &str) -> &str {
    if s.len() > 1 && s.ends_with('/') {
        &s[..s.len() - 1]
    } else {
        s
    }
}

/// Top-level `Bun::serve(options)` matching JS shape.
pub fn serve(options: ServeOptions) -> Server { Server::new(options) }
