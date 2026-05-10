// Body — shared substrate for Request and Response.
//
// SPEC Fetch §6.2.body / §6.4.body: body is a ReadableStream-or-null in the
// full spec. Pilot scope omits ReadableStream; instead body is one of
// {Empty, Bytes, Text}. This is enough to exercise text/json/arrayBuffer
// extraction paths.

use std::cell::Cell;

#[derive(Debug, Clone, PartialEq)]
pub enum Body {
    Empty,
    Bytes(Vec<u8>),
    Text(String),
}

#[derive(Debug)]
pub struct BodyHolder {
    body: Body,
    used: Cell<bool>,
}

impl BodyHolder {
    pub fn new(body: Body) -> Self {
        Self { body, used: Cell::new(false) }
    }

    pub fn empty() -> Self {
        Self::new(Body::Empty)
    }

    pub fn used(&self) -> bool { self.used.get() }
    pub fn is_null(&self) -> bool { matches!(self.body, Body::Empty) }
    pub fn body(&self) -> &Body { &self.body }

    /// SPEC §6.2/§6.4 body-consuming methods reject when bodyUsed is already
    /// true. Pilot's Result analog returns Err.
    pub fn consume_text(&self) -> Result<String, BodyError> {
        if self.used.get() { return Err(BodyError::AlreadyUsed); }
        self.used.set(true);
        match &self.body {
            Body::Empty => Ok(String::new()),
            Body::Text(s) => Ok(s.clone()),
            Body::Bytes(b) => Ok(String::from_utf8_lossy(b).into_owned()),
        }
    }

    pub fn consume_bytes(&self) -> Result<Vec<u8>, BodyError> {
        if self.used.get() { return Err(BodyError::AlreadyUsed); }
        self.used.set(true);
        match &self.body {
            Body::Empty => Ok(Vec::new()),
            Body::Text(s) => Ok(s.as_bytes().to_vec()),
            Body::Bytes(b) => Ok(b.clone()),
        }
    }

    pub fn consume_array_buffer(&self) -> Result<Vec<u8>, BodyError> {
        self.consume_bytes()
    }

    /// SPEC: body.json() consumes body as text, then JSON-parses. Pilot
    /// returns the raw text since we don't implement a JSON parser; the
    /// text-side test suffices to verify the consumption + Content-Type
    /// pathway.
    pub fn consume_json(&self) -> Result<String, BodyError> {
        self.consume_text()
    }

    /// Tee for clone(): produces a fresh holder with the same body, neither
    /// marked used. SPEC: clone() throws TypeError when bodyUsed; pilot
    /// returns Err.
    pub fn tee(&self) -> Result<Self, BodyError> {
        if self.used.get() { return Err(BodyError::AlreadyUsed); }
        Ok(Self::new(self.body.clone()))
    }
}

impl Default for BodyHolder {
    fn default() -> Self { Self::empty() }
}

impl Clone for BodyHolder {
    fn clone(&self) -> Self {
        // Used to be Clone-derived; explicit impl preserves the used flag
        // and ensures Cell semantics.
        Self {
            body: self.body.clone(),
            used: Cell::new(self.used.get()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodyError {
    AlreadyUsed,
}
