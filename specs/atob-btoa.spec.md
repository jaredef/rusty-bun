# atob and btoa — HTML §8.3

[surface] atob
[spec] https://html.spec.whatwg.org/multipage/webappapis.html#dom-atob

## atob is exposed as a global function
- atob is defined as a global function in any execution context with [Exposed=*]
- atob(data) returns the Base64 decoded byte string

## atob input validation
- atob throws DOMException InvalidCharacterError on non-Base64 characters
- atob ignores ASCII whitespace in the input
- atob accepts an optional trailing "=" or "==" padding

## btoa is exposed as a global function
- btoa is defined as a global function in any execution context with [Exposed=*]
- btoa(data) returns the Base64 encoding of the input byte string

## btoa input validation
- btoa throws DOMException InvalidCharacterError on input with code points above 0xFF
