// node-http pilot — Node's `http`/`https` module data-layer.
//
// Inputs:
//   AUDIT — pilots/node-http/AUDIT.md
//   REF   — Node.js docs §http (https://nodejs.org/api/http.html)
//   Bun reference: js/node/http.ts + _http_server/outgoing/common/incoming.ts
//
// Tier-2 ecosystem-compat. No transport, no socket binding, no actual wire
// format. The pilot models the data structures and state transitions; a
// transport-layer pilot is deferred.

use std::collections::HashMap;

// ───────────────────────── Headers (Node-style) ────────────────────────
//
// Node represents headers as a plain object with lowercased keys. Pilot
// stores in a Vec for ordered iteration + a parallel HashMap for O(1) get.
// Case-insensitivity is by lowercasing on insert.

#[derive(Debug, Clone, Default)]
pub struct NodeHeaders {
    entries: Vec<(String, String)>,
}

impl NodeHeaders {
    pub fn new() -> Self { Self::default() }

    pub fn set(&mut self, name: &str, value: impl Into<String>) {
        let lower = name.to_ascii_lowercase();
        self.entries.retain(|(n, _)| n != &lower);
        self.entries.push((lower, value.into()));
    }

    pub fn append(&mut self, name: &str, value: impl Into<String>) {
        self.entries.push((name.to_ascii_lowercase(), value.into()));
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        let lower = name.to_ascii_lowercase();
        self.entries.iter().find(|(n, _)| n == &lower).map(|(_, v)| v.as_str())
    }

    pub fn has(&self, name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        self.entries.iter().any(|(n, _)| n == &lower)
    }

    pub fn remove(&mut self, name: &str) {
        let lower = name.to_ascii_lowercase();
        self.entries.retain(|(n, _)| n != &lower);
    }

    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries.iter().map(|(n, v)| (n.as_str(), v.as_str()))
    }

    pub fn count(&self) -> usize { self.entries.len() }

    /// Node's flat-object representation: HashMap<lowercased_name, value>.
    pub fn as_object(&self) -> HashMap<String, String> {
        let mut o = HashMap::new();
        for (n, v) in &self.entries { o.insert(n.clone(), v.clone()); }
        o
    }
}

// ─────────────────────── IncomingMessage ───────────────────────────────
//
// Incoming HTTP message — used on the server side as the request, on the
// client side as the response. Same shape per Node API.

#[derive(Debug, Clone, Default)]
pub struct IncomingMessage {
    pub method: String,
    pub url: String,
    pub http_version: String,
    pub headers: NodeHeaders,
    pub status_code: u16,
    pub status_message: String,
    pub body: Vec<u8>,
    pub complete: bool,
}

impl IncomingMessage {
    pub fn new() -> Self { Self::default() }
}

// ─────────────────────── ServerResponse ────────────────────────────────

#[derive(Debug)]
pub struct ServerResponse {
    pub status_code: u16,
    pub status_message: String,
    headers: NodeHeaders,
    body: Vec<u8>,
    headers_sent: bool,
    ended: bool,
}

impl ServerResponse {
    pub fn new() -> Self {
        Self {
            status_code: 200,
            status_message: "OK".into(),
            headers: NodeHeaders::new(),
            body: Vec::new(),
            headers_sent: false,
            ended: false,
        }
    }

    /// `res.writeHead(statusCode, statusMessage?, headers?)`.
    pub fn write_head(
        &mut self, status_code: u16,
        status_message: Option<&str>,
        headers: Option<&[(&str, &str)]>,
    ) {
        if self.headers_sent { return; }
        self.status_code = status_code;
        if let Some(msg) = status_message { self.status_message = msg.into(); }
        if let Some(hs) = headers {
            for (n, v) in hs { self.headers.set(n, *v); }
        }
        self.headers_sent = true;
    }

    pub fn set_header(&mut self, name: &str, value: impl Into<String>) {
        self.headers.set(name, value);
    }

    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.headers.get(name)
    }

    pub fn remove_header(&mut self, name: &str) {
        self.headers.remove(name);
    }

    pub fn write(&mut self, chunk: &[u8]) {
        if self.ended { return; }
        self.headers_sent = true;
        self.body.extend_from_slice(chunk);
    }

    pub fn write_str(&mut self, chunk: &str) {
        self.write(chunk.as_bytes());
    }

    pub fn end(&mut self, chunk: Option<&[u8]>) {
        if self.ended { return; }
        if let Some(c) = chunk { self.body.extend_from_slice(c); }
        self.headers_sent = true;
        self.ended = true;
    }

    pub fn end_str(&mut self, chunk: &str) {
        self.end(Some(chunk.as_bytes()));
    }

    pub fn headers_sent(&self) -> bool { self.headers_sent }
    pub fn ended(&self) -> bool { self.ended }
    pub fn body(&self) -> &[u8] { &self.body }
    pub fn headers(&self) -> &NodeHeaders { &self.headers }
}

impl Default for ServerResponse {
    fn default() -> Self { Self::new() }
}

// ─────────────────────── ClientRequest ────────────────────────────────

#[derive(Debug)]
pub struct ClientRequest {
    pub method: String,
    pub url: String,
    headers: NodeHeaders,
    body: Vec<u8>,
    aborted: bool,
    ended: bool,
}

impl ClientRequest {
    pub fn new(method: &str, url: &str) -> Self {
        Self {
            method: method.into(),
            url: url.into(),
            headers: NodeHeaders::new(),
            body: Vec::new(),
            aborted: false,
            ended: false,
        }
    }

    pub fn set_header(&mut self, name: &str, value: impl Into<String>) {
        self.headers.set(name, value);
    }

    pub fn write(&mut self, chunk: &[u8]) {
        if self.aborted || self.ended { return; }
        self.body.extend_from_slice(chunk);
    }

    pub fn end(&mut self, chunk: Option<&[u8]>) {
        if self.aborted || self.ended { return; }
        if let Some(c) = chunk { self.body.extend_from_slice(c); }
        self.ended = true;
    }

    pub fn abort(&mut self) {
        self.aborted = true;
    }

    pub fn aborted(&self) -> bool { self.aborted }
    pub fn ended(&self) -> bool { self.ended }
    pub fn body(&self) -> &[u8] { &self.body }
    pub fn headers(&self) -> &NodeHeaders { &self.headers }
}

// ───────────────────────────── Server ──────────────────────────────────

pub type RequestHandler = Box<dyn Fn(&IncomingMessage, &mut ServerResponse)>;

pub struct Server {
    handler: Option<RequestHandler>,
    port: u16,
    listening: bool,
    closed: bool,
}

impl Server {
    pub fn new() -> Self {
        Self { handler: None, port: 0, listening: false, closed: false }
    }

    pub fn on_request<F>(&mut self, handler: F)
    where F: Fn(&IncomingMessage, &mut ServerResponse) + 'static
    {
        self.handler = Some(Box::new(handler));
    }

    /// `server.listen(port)` — pilot data-layer: records port + state, no bind.
    pub fn listen(&mut self, port: u16) {
        self.port = port;
        self.listening = true;
    }

    pub fn close(&mut self) {
        self.listening = false;
        self.closed = true;
    }

    pub fn port(&self) -> u16 { self.port }
    pub fn listening(&self) -> bool { self.listening }
    pub fn closed(&self) -> bool { self.closed }

    /// Pilot-only invocation: route a request through the handler and return
    /// the populated ServerResponse. Real Node would deliver via socket.
    pub fn dispatch(&self, request: &IncomingMessage) -> ServerResponse {
        let mut response = ServerResponse::new();
        if let Some(h) = &self.handler { h(request, &mut response); }
        response
    }
}

impl Default for Server {
    fn default() -> Self { Self::new() }
}

/// `http.createServer(handler)` — top-level factory.
pub fn create_server<F>(handler: F) -> Server
where F: Fn(&IncomingMessage, &mut ServerResponse) + 'static
{
    let mut s = Server::new();
    s.on_request(handler);
    s
}

/// `http.request(method, url, headers?)` — pilot data-layer: returns a
/// populated ClientRequest for the consumer to write/end.
pub fn request(method: &str, url: &str, headers: Option<&[(&str, &str)]>) -> ClientRequest {
    let mut req = ClientRequest::new(method, url);
    if let Some(hs) = headers {
        for (n, v) in hs { req.set_header(n, *v); }
    }
    req
}
