// Bun.file pilot — Bun's filesystem-backed Blob abstraction.
//
// Inputs:
//   AUDIT — pilots/bun-file/AUDIT.md
//   CD    — runs/2026-05-10-bun-v0.13b-spec-batch/constraints/bun.constraints.md
//           (Bun.file: 470+ cross-corroborated clauses)
//   SPEC  — none. Tier-2 ecosystem-only. Bun's tests are the spec.
//   REF   — Bun docs at https://bun.sh/docs/api/file-io
//
// First pilot with real filesystem I/O. Composes with the rusty-blob
// substrate from Pilot 4: BunFile is a path + (lazy bytes + metadata) +
// inferred-mime-type, with the same Blob-shaped methods.
//
// Async/Promise semantics deferred per AUDIT — pilot uses synchronous
// std::fs reads. ReadableStream stream(), writer(), unlink(), fd, S3-
// backed files all out of pilot scope.

pub use rusty_blob::Blob;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct BunFile {
    path: PathBuf,
    explicit_mime_type: Option<String>,
}

impl BunFile {
    /// `Bun.file(path)` — construct a BunFile lazily. No I/O happens here.
    pub fn open(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into(), explicit_mime_type: None }
    }

    /// `Bun.file(path, { type })` — construct with an explicit MIME type
    /// override.
    pub fn open_with_type(path: impl Into<PathBuf>, mime_type: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            explicit_mime_type: Some(mime_type.into()),
        }
    }

    /// CD `expect(file.name).toEqual(import.meta.filename)` — `.name` is the path.
    pub fn name(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }

    /// CD `expect(bunStat.size).toBe(Buffer.byteLength(content))` —
    /// size reflects the file's byte length on disk. Pilot queries fs each
    /// call; production caches with invalidation.
    pub fn size(&self) -> io::Result<u64> {
        Ok(fs::metadata(&self.path)?.len())
    }

    /// `.type` getter. Inferred from extension when not explicit; empty
    /// string when no mapping. Per Bun docs.
    pub fn mime_type(&self) -> String {
        if let Some(ref t) = self.explicit_mime_type { return t.clone(); }
        let ext = self.path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "html" | "htm" => "text/html;charset=utf-8",
            "css" => "text/css;charset=utf-8",
            "js" | "mjs" | "cjs" => "text/javascript;charset=utf-8",
            "json" => "application/json;charset=utf-8",
            "txt" | "md" => "text/plain;charset=utf-8",
            "wasm" => "application/wasm",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            "pdf" => "application/pdf",
            "" => "",
            _ => "application/octet-stream",
        }.to_string()
    }

    /// `.lastModified` getter — milliseconds since Unix epoch.
    pub fn last_modified(&self) -> io::Result<i64> {
        let m = fs::metadata(&self.path)?;
        let mtime = m.modified()?;
        let dur = mtime.duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        Ok((dur.as_secs() as i64) * 1000 + (dur.subsec_millis() as i64))
    }

    /// `Bun.file(...).exists()` — sync boolean predicate. (Real Bun returns
    /// a Promise; pure-Rust analog is sync.)
    pub fn exists(&self) -> bool { self.path.exists() }

    /// CD: `await Bun.file(...).text()` — read full file as UTF-8.
    pub fn text(&self) -> io::Result<String> {
        Ok(fs::read_to_string(&self.path)?)
    }

    /// `.arrayBuffer()` / `.bytes()` — read full file as bytes.
    pub fn bytes(&self) -> io::Result<Vec<u8>> {
        Ok(fs::read(&self.path)?)
    }

    pub fn array_buffer(&self) -> io::Result<Vec<u8>> { self.bytes() }

    /// `.slice(start, end?, contentType?)` — read a byte range, return as
    /// Blob (NOT BunFile, per File-API spec). Type-system enforces the
    /// "slice strips File metadata" invariant.
    pub fn slice(&self, start: i64, end: Option<i64>, content_type: Option<&str>)
        -> io::Result<Blob>
    {
        let bytes = self.bytes()?;
        let blob = Blob::from_bytes(bytes);
        Ok(blob.slice(start, end, content_type))
    }

    /// Coerce to a Blob view. JS's `instanceof Blob` is satisfied for any
    /// BunFile (Bun.file extends Blob); Rust's analog is explicit
    /// materialization. Reads the file content into memory.
    pub fn as_blob(&self) -> io::Result<Blob> {
        let bytes = self.bytes()?;
        let blob = if let Some(ref t) = self.explicit_mime_type {
            // Use the override directly when set.
            use rusty_blob::{BlobPart, BlobPropertyBag};
            Blob::from_parts(
                &[BlobPart::Bytes(&bytes)],
                BlobPropertyBag {
                    mime_type: t.clone(),
                    ..Default::default()
                },
            )
        } else {
            // Otherwise use the extension-inferred type.
            use rusty_blob::{BlobPart, BlobPropertyBag};
            Blob::from_parts(
                &[BlobPart::Bytes(&bytes)],
                BlobPropertyBag {
                    mime_type: self.mime_type(),
                    ..Default::default()
                },
            )
        };
        Ok(blob)
    }

    /// Reference to the underlying path; not part of Bun.file's JS API,
    /// exposed in pilot for verifier convenience.
    pub fn path(&self) -> &Path { &self.path }
}

/// Top-level `Bun::file(path)` convenience function matching JS shape.
pub fn file(path: impl Into<PathBuf>) -> BunFile {
    BunFile::open(path)
}
