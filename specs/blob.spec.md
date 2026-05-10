# Blob — W3C File API §3

[surface] Blob
[spec] https://w3c.github.io/FileAPI/#blob-section

## Blob is exposed as a global constructor
- Blob is defined as a global constructor in any execution context with [Exposed=*]
- new Blob() returns a zero-byte Blob with empty type
- new Blob(parts) accepts a sequence of BufferSource, USVString, or Blob
- new Blob(parts, options) accepts a BlobPropertyBag with type and endings

## Blob.prototype.size
- Blob.prototype.size returns the byte length of the blob

## Blob.prototype.type
- Blob.prototype.type returns the MIME type or empty string when none was provided
- Blob.prototype.type lowercases ASCII characters in the type

## Blob.prototype.slice
- Blob.prototype.slice returns a new Blob containing a byte range
- Blob.prototype.slice(start) defaults end to size
- Blob.prototype.slice(start, end, contentType) sets the new blob's type
- Blob.prototype.slice clamps negative offsets to be relative to size

## Blob.prototype.text
- Blob.prototype.text returns a Promise resolving to a UTF-8 decoded USVString

## Blob.prototype.arrayBuffer
- Blob.prototype.arrayBuffer returns a Promise resolving to an ArrayBuffer

## Blob.prototype.bytes
- Blob.prototype.bytes returns a Promise resolving to a Uint8Array

## Blob.prototype.stream
- Blob.prototype.stream returns a ReadableStream of Uint8Array chunks

## Blob endings normalization
- Blob constructor with endings "transparent" preserves line endings
- Blob constructor with endings "native" converts line endings to platform native form
