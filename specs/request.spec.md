# Request — WHATWG Fetch §6.2

[surface] Request
[spec] https://fetch.spec.whatwg.org/#request-class

## Request is exposed as a global constructor
- Request is defined as a global constructor in any execution context with [Exposed=*]
- new Request(input) accepts a USVString URL or another Request
- new Request(input, init) accepts a RequestInit dictionary

## Request constructor validates init
- Request constructor throws TypeError on invalid URL
- Request constructor throws TypeError when init.method is a forbidden method
- Request constructor throws TypeError when init.mode is "navigate"

## Request.prototype.method
- Request.prototype.method returns the request's method
- Request.prototype.method defaults to "GET"

## Request.prototype.url
- Request.prototype.url returns the request's URL as a serialized string

## Request.prototype.headers
- Request.prototype.headers returns the Headers instance associated with the request

## Request.prototype.destination
- Request.prototype.destination returns the request's destination

## Request.prototype.referrer
- Request.prototype.referrer returns the URL of the referring document or "no-referrer" or "client"

## Request.prototype.mode
- Request.prototype.mode returns one of "cors" "no-cors" "same-origin" "navigate" or "websocket"

## Request.prototype.credentials
- Request.prototype.credentials returns one of "omit" "same-origin" or "include"

## Request.prototype.cache
- Request.prototype.cache returns a RequestCache enum value

## Request.prototype.redirect
- Request.prototype.redirect returns one of "follow" "error" or "manual"

## Request.prototype.signal
- Request.prototype.signal returns the AbortSignal associated with the request

## Request body extraction
- Request.prototype.body returns the request's body as a ReadableStream or null
- Request.prototype.bodyUsed returns whether the body has been consumed
- Request.prototype.arrayBuffer returns a Promise resolving to an ArrayBuffer
- Request.prototype.blob returns a Promise resolving to a Blob
- Request.prototype.formData returns a Promise resolving to a FormData
- Request.prototype.json returns a Promise resolving to the parsed JSON
- Request.prototype.text returns a Promise resolving to a USVString
- Request body methods reject with TypeError when bodyUsed is already true

## Request.prototype.clone
- Request.prototype.clone returns a new Request with the same state and a tee'd body
- Request.prototype.clone throws TypeError when bodyUsed is true
