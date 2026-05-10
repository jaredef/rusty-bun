// Request — WHATWG Fetch §6.2.
//
// Pilot scope: no actual HTTP transport. Request is modeled as an immutable
// data structure: method, URL, headers, body, mode, credentials, cache,
// redirect. Body extraction methods consume the body once.

use crate::body::{Body, BodyHolder, BodyError};
use crate::headers::{HeaderError, Headers};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestError {
    InvalidUrl(String),
    HeadersError(HeaderError),
    BodyError(BodyError),
    BodyUsed,
}

impl From<HeaderError> for RequestError {
    fn from(e: HeaderError) -> Self { RequestError::HeadersError(e) }
}

impl From<BodyError> for RequestError {
    fn from(e: BodyError) -> Self { RequestError::BodyError(e) }
}

#[derive(Debug, Default, Clone)]
pub struct RequestInit {
    pub method: Option<String>,
    pub headers: Option<Headers>,
    pub body: Option<Body>,
    pub mode: Option<String>,
    pub credentials: Option<String>,
    pub cache: Option<String>,
    pub redirect: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Request {
    method: String,
    url: String,
    headers: Headers,
    body: BodyHolder,
    mode: String,
    credentials: String,
    cache: String,
    redirect: String,
}

impl Request {
    pub fn new(input: &str, init: RequestInit) -> Result<Self, RequestError> {
        if input.is_empty() {
            return Err(RequestError::InvalidUrl(input.to_string()));
        }
        Ok(Self {
            method: init.method.unwrap_or_else(|| "GET".to_string()),
            url: input.to_string(),
            headers: init.headers.unwrap_or_default(),
            body: match init.body {
                Some(b) => BodyHolder::new(b),
                None => BodyHolder::empty(),
            },
            mode: init.mode.unwrap_or_else(|| "cors".to_string()),
            credentials: init.credentials.unwrap_or_else(|| "same-origin".to_string()),
            cache: init.cache.unwrap_or_else(|| "default".to_string()),
            redirect: init.redirect.unwrap_or_else(|| "follow".to_string()),
        })
    }

    pub fn method(&self) -> &str { &self.method }
    pub fn url(&self) -> &str { &self.url }
    pub fn headers(&self) -> &Headers { &self.headers }
    pub fn body_used(&self) -> bool { self.body.used() }
    pub fn body_is_null(&self) -> bool { self.body.is_null() }
    pub fn mode(&self) -> &str { &self.mode }
    pub fn credentials(&self) -> &str { &self.credentials }
    pub fn cache(&self) -> &str { &self.cache }
    pub fn redirect(&self) -> &str { &self.redirect }

    pub fn text(&self) -> Result<String, RequestError> {
        Ok(self.body.consume_text()?)
    }
    pub fn array_buffer(&self) -> Result<Vec<u8>, RequestError> {
        Ok(self.body.consume_array_buffer()?)
    }
    pub fn bytes(&self) -> Result<Vec<u8>, RequestError> {
        Ok(self.body.consume_bytes()?)
    }
    pub fn json(&self) -> Result<String, RequestError> {
        Ok(self.body.consume_json()?)
    }

    /// SPEC §6.2.clone: returns a Request with the same state and a tee'd
    /// body. Throws TypeError when bodyUsed.
    pub fn clone_request(&self) -> Result<Self, RequestError> {
        let body = self.body.tee()?;
        Ok(Self {
            method: self.method.clone(),
            url: self.url.clone(),
            headers: self.headers.clone(),
            body,
            mode: self.mode.clone(),
            credentials: self.credentials.clone(),
            cache: self.cache.clone(),
            redirect: self.redirect.clone(),
        })
    }
}
