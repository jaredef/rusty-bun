// fetch-api pilot — multi-surface composition (Headers + Request + Response).
//
// Inputs:
//   AUDIT — pilots/fetch-api/AUDIT.md
//   SPEC  — WHATWG Fetch §§5.2 (Headers), §6.2 (Request), §6.4 (Response)
//   CD    — runs/2026-05-10-bun-v0.13b-spec-batch/constraints/{headers,
//           request,response}.constraints.md
//
// Pilot scope per AUDIT: no transport, no ReadableStream body, no
// CORS/credentials enforcement.

pub mod body;
pub mod headers;
pub mod request;
pub mod response;

pub use body::Body;
pub use headers::{HeaderError, Headers};
pub use request::{Request, RequestError, RequestInit};
pub use response::{Response, ResponseError, ResponseInit, ResponseType};
