# Headers — WHATWG Fetch §5.2

[surface] Headers
[spec] https://fetch.spec.whatwg.org/#headers-class

## Headers is exposed as a global constructor
- Headers is defined as a global constructor in any execution context with [Exposed=*]
- new Headers() returns an empty Headers instance
- new Headers(init) accepts a Headers, sequence of pairs, or record

## Headers.prototype.append
- Headers.prototype.append(name, value) adds a header value
- Headers.prototype.append throws TypeError on invalid header name
- Headers.prototype.append throws TypeError on invalid header value
- Headers.prototype.append combines multiple values for the same name with ", "

## Headers.prototype.delete
- Headers.prototype.delete(name) removes all values for the name
- Headers.prototype.delete is case-insensitive on the name

## Headers.prototype.get
- Headers.prototype.get(name) returns the combined value or null
- Headers.prototype.get is case-insensitive on the name
- Headers.prototype.get returns null when the header is absent

## Headers.prototype.getSetCookie
- Headers.prototype.getSetCookie returns an array of all Set-Cookie values
- Headers.prototype.getSetCookie returns an empty array when none are present

## Headers.prototype.has
- Headers.prototype.has(name) returns true when the name is present
- Headers.prototype.has is case-insensitive on the name

## Headers.prototype.set
- Headers.prototype.set(name, value) replaces existing values for the name
- Headers.prototype.set throws TypeError on invalid header name or value

## Headers.prototype iteration
- Headers.prototype.entries returns name-value pairs in lexicographic order on lowercased names
- Headers.prototype.keys returns lowercased header names
- Headers.prototype.values returns combined values
- Headers.prototype.forEach invokes a callback for each header in iteration order

## Headers normalization
- Headers stores header names case-insensitively but iterates lowercased
- Headers strips leading and trailing HTTP whitespace from values
