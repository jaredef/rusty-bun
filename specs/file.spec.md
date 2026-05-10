# File — W3C File API §4

[surface] File
[spec] https://w3c.github.io/FileAPI/#file-section

## File is exposed as a global constructor
- File is defined as a global constructor in any execution context with [Exposed=*]
- new File(parts, name) returns a File with the given name
- new File(parts, name, options) accepts a FilePropertyBag with type and lastModified
- File extends Blob

## File.prototype.name
- File.prototype.name returns the file name as a USVString

## File.prototype.lastModified
- File.prototype.lastModified returns the modification time in milliseconds since epoch
- File.prototype.lastModified defaults to current time when not specified

## File.prototype.webkitRelativePath
- File.prototype.webkitRelativePath returns the relative path or empty string
