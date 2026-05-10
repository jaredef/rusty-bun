# URL — WHATWG URL §6

[surface] URL
[spec] https://url.spec.whatwg.org/#url-class

## URL is exposed as a global constructor
- URL is defined as a global constructor in any execution context with [Exposed=*]
- new URL(url) parses url against the default base
- new URL(url, base) parses url relative to base
- new URL constructor throws TypeError on invalid input

## URL.canParse static method
- URL.canParse(url) returns true if URL parsing succeeds
- URL.canParse(url, base) returns true if URL parsing with base succeeds
- URL.canParse returns false for malformed input

## URL.parse static method
- URL.parse(url) returns a URL instance or null
- URL.parse never throws
- URL.parse(url, base) returns null when parsing fails

## URL.prototype.href
- URL.prototype.href returns the serialized URL string
- URL.prototype.href setter reparses the URL on assignment

## URL.prototype.protocol
- URL.prototype.protocol returns the scheme followed by a colon
- URL.prototype.protocol always lowercases the scheme

## URL.prototype.hostname
- URL.prototype.hostname returns the host without the port
- URL.prototype.hostname returns the empty string when host is null

## URL.prototype.host
- URL.prototype.host returns hostname concatenated with optional :port

## URL.prototype.port
- URL.prototype.port returns the URL's port as a string or empty string for default port

## URL.prototype.pathname
- URL.prototype.pathname returns the URL path including a leading slash for special schemes
- URL.prototype.pathname returns the opaque path for non-special schemes

## URL.prototype.search
- URL.prototype.search returns the query starting with "?" or empty string when query is null

## URL.prototype.searchParams
- URL.prototype.searchParams returns a URLSearchParams associated with the URL
- URL.prototype.searchParams updates the URL's query when modified

## URL.prototype.hash
- URL.prototype.hash returns the fragment starting with "#" or empty string when fragment is null

## URL.prototype.origin
- URL.prototype.origin returns the URL's origin as a serialized string
- URL.prototype.origin returns "null" for opaque origins

## URL.prototype.toString and toJSON
- URL.prototype.toString returns the same string as URL.prototype.href
- URL.prototype.toJSON returns the same string as URL.prototype.href
