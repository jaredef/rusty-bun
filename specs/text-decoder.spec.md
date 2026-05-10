# TextDecoder — WHATWG Encoding §10

[surface] TextDecoder
[spec] https://encoding.spec.whatwg.org/#textdecoder

## TextDecoder is exposed as a global constructor
- TextDecoder is defined as a global constructor in any execution context with [Exposed=*]
- new TextDecoder() returns a TextDecoder instance with default "utf-8" encoding
- new TextDecoder(label) returns a TextDecoder using the resolved encoding
- new TextDecoder(label, options) accepts a TextDecoderOptions dictionary

## TextDecoder constructor label resolution
- TextDecoder constructor with unknown label throws RangeError
- TextDecoder constructor accepts label "utf-8" and resolves to "utf-8"
- TextDecoder constructor accepts label "utf8" and resolves to "utf-8"
- TextDecoder constructor accepts label "UTF-8" case-insensitively and resolves to "utf-8"
- TextDecoder constructor strips ASCII whitespace from label before resolution

## TextDecoder.prototype.encoding getter
- TextDecoder.prototype.encoding returns the canonical name of the resolved encoding
- TextDecoder.prototype.encoding is the string "utf-8" by default

## TextDecoder.prototype.fatal getter
- TextDecoder.prototype.fatal returns the boolean fatal option
- TextDecoder.prototype.fatal is false by default

## TextDecoder.prototype.ignoreBOM getter
- TextDecoder.prototype.ignoreBOM returns the boolean ignoreBOM option
- TextDecoder.prototype.ignoreBOM is false by default

## TextDecoder.prototype.decode method
- TextDecoder.prototype.decode returns a string
- TextDecoder.prototype.decode(empty) returns ""
- TextDecoder.prototype.decode(bytes) returns the decoded string per the resolved encoding
- TextDecoder.prototype.decode with fatal true throws TypeError on invalid byte sequence
- TextDecoder.prototype.decode with fatal false replaces invalid byte sequences with U+FFFD
- TextDecoder.prototype.decode with ignoreBOM false consumes a leading UTF-8 BOM EF BB BF
- TextDecoder.prototype.decode with ignoreBOM true preserves a leading UTF-8 BOM as U+FEFF
- TextDecoder.prototype.decode with stream true retains incomplete trailing sequences for the next call
- TextDecoder.prototype.decode with stream false flushes pending state on end-of-input

## TextDecoder.prototype.toString
- TextDecoder.prototype.toString returns "[object TextDecoder]"
