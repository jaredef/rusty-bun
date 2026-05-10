// streams pilot — WHATWG Streams Standard (ReadableStream + WritableStream
// + TransformStream).
//
// Inputs:
//   AUDIT — pilots/streams/AUDIT.md
//   SPEC  — https://streams.spec.whatwg.org/
//   CD    — runs/2026-05-10-bun-v0.13b-spec-batch/constraints/{readable,
//           writable,transform}stream.constraints.md (sparse: 3 clauses)
//   SPEC EXTRACT — specs/streams.spec.md (44 clauses, primary input)
//
// First pilot where spec-extract layer dominates over test-corpus layer.
// Validates Doc 707's claim that spec is the constraint ceiling.
//
// Pure-Rust analog: synchronous poll-based read/write. Async/Promise model
// out of scope per AUDIT — `read()` returns `ReadResult` (Chunk|Done|Pending
// |Error), not Future<Result<...>>. The apparatus' data-layer claim suffices.

pub mod readable;
pub mod transform;
pub mod writable;

pub use readable::{
    Controller, ReadResult, ReadableStream, Reader, StreamError, UnderlyingSource,
};
pub use transform::{TransformStream, Transformer};
pub use writable::{UnderlyingSink, WritableStream, Writer};
