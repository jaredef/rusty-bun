// Simulated-derivation of File (W3C File API §4).
//
// Inputs:
//   AUDIT — pilots/file/AUDIT.md
//   SPEC  — https://w3c.github.io/FileAPI/#file-section
//   CD    — runs/2026-05-10-bun-v0.13b-spec-batch/constraints/file.constraints.md
//
// File extends Blob. Rust has no class inheritance; the pilot models the
// extension as composition: File owns an inner Blob plus File-specific
// fields (name, lastModified, webkitRelativePath). Blob's surface
// (size, type, slice, text, arrayBuffer, bytes) is exposed via
// delegation.

pub use rusty_blob::{Blob, BlobPart, BlobPropertyBag, LineEndings};

#[derive(Debug, Clone, Default)]
pub struct FilePropertyBag {
    pub mime_type: String,
    pub endings: LineEndings,
    /// SPEC §4: lastModified defaults to current time when not specified.
    /// Pilot exposes `Option<i64>` so callers can choose; the constructor
    /// provides a default helper.
    pub last_modified: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct File {
    blob: Blob,
    name: String,
    last_modified: i64,
    /// SPEC §4: webkitRelativePath; empty string when not provided.
    webkit_relative_path: String,
}

impl File {
    /// SPEC §4.constructor with required `name` parameter.
    /// CD FILE1 antichain rep: `new File([new Uint8Array()], "file.txt")`
    /// produces a File with name "file.txt".
    pub fn new(parts: &[BlobPart<'_>], name: impl Into<String>, options: FilePropertyBag) -> Self {
        let blob = Blob::from_parts(
            parts,
            BlobPropertyBag {
                mime_type: options.mime_type,
                endings: options.endings,
            },
        );
        Self {
            blob,
            name: name.into(),
            last_modified: options.last_modified.unwrap_or(0),
            webkit_relative_path: String::new(),
        }
    }

    /// SPEC §4.name.
    pub fn name(&self) -> &str { &self.name }

    /// SPEC §4.lastModified — milliseconds since Unix epoch.
    pub fn last_modified(&self) -> i64 { self.last_modified }

    /// SPEC §4.webkitRelativePath.
    pub fn webkit_relative_path(&self) -> &str { &self.webkit_relative_path }

    // ─────────── Blob delegation methods ────────────
    //
    // SPEC: "interface File : Blob" — every Blob method is also a File
    // method. Pilot delegates to the inner Blob via composition, which
    // is the Rust-idiomatic translation of IDL inheritance.

    pub fn size(&self) -> usize { self.blob.size() }
    pub fn mime_type(&self) -> &str { self.blob.mime_type() }
    pub fn slice(&self, start: i64, end: Option<i64>, content_type: Option<&str>) -> Blob {
        self.blob.slice(start, end, content_type)
    }
    pub fn text(&self) -> String { self.blob.text() }
    pub fn array_buffer(&self) -> Vec<u8> { self.blob.array_buffer() }
    pub fn bytes(&self) -> Vec<u8> { self.blob.bytes() }

    /// Coerce the File to a Blob view. JS's `instanceof Blob` is satisfied
    /// for any File because File extends Blob; Rust's type-system analog
    /// is explicit access to the inner Blob.
    pub fn as_blob(&self) -> &Blob { &self.blob }
}
