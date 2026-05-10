// Response — WHATWG Fetch §6.4.
//
// Pilot scope: the data-structure half of Response. Static methods .json(),
// .redirect(), .error() included. Body extraction same shape as Request.

use crate::body::{Body, BodyHolder, BodyError};
use crate::headers::{HeaderError, Headers};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseError {
    StatusOutOfRange(u16),
    InvalidRedirectStatus(u16),
    HeadersError(HeaderError),
    BodyError(BodyError),
}

impl From<HeaderError> for ResponseError {
    fn from(e: HeaderError) -> Self { ResponseError::HeadersError(e) }
}

impl From<BodyError> for ResponseError {
    fn from(e: BodyError) -> Self { ResponseError::BodyError(e) }
}

#[derive(Debug, Default, Clone)]
pub struct ResponseInit {
    pub status: Option<u16>,
    pub status_text: Option<String>,
    pub headers: Option<Headers>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseType {
    Basic,
    Cors,
    Default,
    Error,
    Opaque,
    OpaqueRedirect,
}

#[derive(Debug, Clone)]
pub struct Response {
    body: BodyHolder,
    headers: Headers,
    status: u16,
    status_text: String,
    response_type: ResponseType,
    url: String,
    redirected: bool,
}

impl Response {
    /// SPEC §6.4.constructor.
    /// CD: `expect(response.status).toBe(200)` for default-constructed.
    pub fn new(body: Option<Body>, init: ResponseInit) -> Result<Self, ResponseError> {
        let status = init.status.unwrap_or(200);
        if !(200..=599).contains(&status) {
            return Err(ResponseError::StatusOutOfRange(status));
        }
        Ok(Self {
            body: match body {
                Some(b) => BodyHolder::new(b),
                None => BodyHolder::empty(),
            },
            headers: init.headers.unwrap_or_default(),
            status,
            status_text: init.status_text.unwrap_or_default(),
            response_type: ResponseType::Default,
            url: String::new(),
            redirected: false,
        })
    }

    /// SPEC §6.4.error: network-error response, type "error", status 0,
    /// empty body.
    pub fn error() -> Self {
        Self {
            body: BodyHolder::empty(),
            headers: Headers::new(),
            status: 0,
            status_text: String::new(),
            response_type: ResponseType::Error,
            url: String::new(),
            redirected: false,
        }
    }

    /// SPEC §6.4.json: Response containing the JSON serialization of `data`,
    /// with Content-Type: application/json. Pilot accepts pre-serialized
    /// JSON text since we don't implement a JSON serializer; the cited
    /// consumer behavior (Content-Type header is set automatically) is
    /// what's tested.
    pub fn json(data: &str, init: ResponseInit) -> Result<Self, ResponseError> {
        let mut headers = init.headers.unwrap_or_default();
        headers.set("Content-Type", "application/json")?;
        let init = ResponseInit { headers: Some(headers), ..init };
        Self::new(Some(Body::Text(data.to_string())), init)
    }

    /// SPEC §6.4.redirect: only 301, 302, 303, 307, 308 valid.
    pub fn redirect(url: &str, status: u16) -> Result<Self, ResponseError> {
        if !matches!(status, 301 | 302 | 303 | 307 | 308) {
            return Err(ResponseError::InvalidRedirectStatus(status));
        }
        let mut headers = Headers::new();
        headers.set("Location", url)?;
        Ok(Self {
            body: BodyHolder::empty(),
            headers,
            status,
            status_text: String::new(),
            response_type: ResponseType::Default,
            url: String::new(),
            redirected: false,
        })
    }

    pub fn status(&self) -> u16 { self.status }
    pub fn status_text(&self) -> &str { &self.status_text }
    pub fn headers(&self) -> &Headers { &self.headers }
    pub fn ok(&self) -> bool { (200..=299).contains(&self.status) }
    pub fn response_type(&self) -> ResponseType { self.response_type }
    pub fn url(&self) -> &str { &self.url }
    pub fn redirected(&self) -> bool { self.redirected }
    pub fn body_used(&self) -> bool { self.body.used() }
    pub fn body_is_null(&self) -> bool { self.body.is_null() }

    pub fn text(&self) -> Result<String, ResponseError> {
        Ok(self.body.consume_text()?)
    }
    pub fn array_buffer(&self) -> Result<Vec<u8>, ResponseError> {
        Ok(self.body.consume_array_buffer()?)
    }
    pub fn bytes(&self) -> Result<Vec<u8>, ResponseError> {
        Ok(self.body.consume_bytes()?)
    }
    pub fn json_text(&self) -> Result<String, ResponseError> {
        Ok(self.body.consume_json()?)
    }

    pub fn clone_response(&self) -> Result<Self, ResponseError> {
        let body = self.body.tee()?;
        Ok(Self {
            body,
            headers: self.headers.clone(),
            status: self.status,
            status_text: self.status_text.clone(),
            response_type: self.response_type,
            url: self.url.clone(),
            redirected: self.redirected,
        })
    }
}
