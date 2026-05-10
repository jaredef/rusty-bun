# Response — WHATWG Fetch §6.4

[surface] Response
[spec] https://fetch.spec.whatwg.org/#response-class

## Response is exposed as a global constructor
- Response is defined as a global constructor in any execution context with [Exposed=*]
- new Response() returns a 200 Response with empty body
- new Response(body) wraps body as the response body
- new Response(body, init) accepts a ResponseInit dictionary

## Response constructor validates init
- Response constructor throws RangeError when init.status is outside 200..=599
- Response constructor throws TypeError when init.statusText contains forbidden characters

## Response.error static method
- Response.error returns a network-error Response with type "error"
- Response.error has status 0 and empty body

## Response.json static method
- Response.json(data) returns a Response containing the JSON serialization of data
- Response.json sets Content-Type to "application/json"
- Response.json(data, init) accepts a ResponseInit dictionary

## Response.redirect static method
- Response.redirect(url) returns a Response with the Location header set to url
- Response.redirect(url, status) returns a Response with the given redirect status
- Response.redirect throws RangeError when status is not 301, 302, 303, 307, or 308

## Response.prototype.type
- Response.prototype.type returns one of "basic" "cors" "default" "error" "opaque" "opaqueredirect"

## Response.prototype.url
- Response.prototype.url returns the response URL as a serialized string

## Response.prototype.redirected
- Response.prototype.redirected returns whether the response is the result of a redirect

## Response.prototype.status
- Response.prototype.status returns the HTTP status code as a number

## Response.prototype.ok
- Response.prototype.ok returns true when status is in 200..=299

## Response.prototype.statusText
- Response.prototype.statusText returns the HTTP status reason phrase

## Response.prototype.headers
- Response.prototype.headers returns the Headers instance associated with the response

## Response body extraction
- Response.prototype.body returns the response's body as a ReadableStream or null
- Response.prototype.bodyUsed returns whether the body has been consumed
- Response.prototype.arrayBuffer returns a Promise resolving to an ArrayBuffer
- Response.prototype.blob returns a Promise resolving to a Blob
- Response.prototype.bytes returns a Promise resolving to a Uint8Array
- Response.prototype.formData returns a Promise resolving to a FormData
- Response.prototype.json returns a Promise resolving to the parsed JSON
- Response.prototype.text returns a Promise resolving to a USVString
- Response body methods reject with TypeError when bodyUsed is already true

## Response.prototype.clone
- Response.prototype.clone returns a new Response with the same state and a tee'd body
- Response.prototype.clone throws TypeError when bodyUsed is true
