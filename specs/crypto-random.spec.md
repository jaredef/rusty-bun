# crypto.randomUUID and crypto.getRandomValues — Web Crypto §10

[surface] crypto
[spec] https://w3c.github.io/webcrypto/#Crypto-interface

## crypto is exposed as a global object
- crypto is defined as a global object in any execution context with [Exposed=*]

## crypto.randomUUID method
- crypto.randomUUID returns a v4 UUID as a USVString
- crypto.randomUUID returns a string of length 36
- crypto.randomUUID matches the pattern xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
- crypto.randomUUID returns a different value on each call with extremely high probability

## crypto.getRandomValues method
- crypto.getRandomValues(typedArray) fills typedArray with cryptographically random bytes
- crypto.getRandomValues returns the same typedArray reference passed in
- crypto.getRandomValues throws QuotaExceededError when typedArray byte length exceeds 65536
- crypto.getRandomValues throws TypeMismatchError on non-integer typed arrays
- crypto.getRandomValues accepts Uint8Array Int8Array Uint16Array Int16Array Uint32Array Int32Array Uint8ClampedArray BigUint64Array BigInt64Array
