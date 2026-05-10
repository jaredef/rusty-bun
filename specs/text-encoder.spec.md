# TextEncoder — WHATWG Encoding §9

[surface] TextEncoder
[spec] https://encoding.spec.whatwg.org/#textencoder

## TextEncoder is exposed as a global constructor
- TextEncoder is defined as a global constructor in any execution context with [Exposed=*]
- new TextEncoder() returns a TextEncoder instance
- TextEncoder.prototype is the prototype object for TextEncoder instances

## TextEncoder.prototype.encoding getter
- TextEncoder.prototype.encoding is the string "utf-8"
- TextEncoder.prototype.encoding always equals "utf-8" regardless of constructor input

## TextEncoder.prototype.encode method
- TextEncoder.prototype.encode is a method that returns a Uint8Array
- TextEncoder.prototype.encode(input) returns the UTF-8 byte encoding of input
- TextEncoder.prototype.encode() with no argument returns Uint8Array of length 0
- TextEncoder.prototype.encode("") returns Uint8Array of length 0
- TextEncoder.prototype.encode("hello") returns the bytes 0x68 0x65 0x6c 0x6c 0x6f
- TextEncoder.prototype.encode replaces unpaired UTF-16 surrogates with U+FFFD before encoding

## TextEncoder.prototype.encodeInto method
- TextEncoder.prototype.encodeInto is a method that returns an object with read and written number fields
- TextEncoder.prototype.encodeInto(source, destination) writes UTF-8 bytes of source into destination
- TextEncoder.prototype.encodeInto never writes past destination.length
- TextEncoder.prototype.encodeInto never splits a multi-byte UTF-8 sequence across the destination boundary
- TextEncoder.prototype.encodeInto returns read equal to UTF-16 code units consumed from source
- TextEncoder.prototype.encodeInto returns written equal to UTF-8 bytes written to destination

## TextEncoder.prototype.toString
- TextEncoder.prototype.toString returns "[object TextEncoder]"
