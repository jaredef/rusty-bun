# structuredClone — HTML §2.10

[surface] structuredClone
[spec] https://html.spec.whatwg.org/multipage/structured-data.html#dom-structuredclone

## structuredClone is exposed as a global function
- structuredClone is defined as a global function in any execution context with [Exposed=*]
- structuredClone(value) returns a deep clone of value
- structuredClone(value, options) accepts a StructuredSerializeOptions with transfer

## structuredClone supported types
- structuredClone clones primitives by value
- structuredClone clones Date with the same time value
- structuredClone clones RegExp with the same source and flags
- structuredClone clones Map preserving entry order
- structuredClone clones Set preserving entry order
- structuredClone clones plain objects recursively
- structuredClone clones arrays recursively
- structuredClone clones ArrayBuffer including its byte content
- structuredClone clones typed array views attached to the cloned ArrayBuffer

## structuredClone unsupported types
- structuredClone throws DataCloneError on functions
- structuredClone throws DataCloneError on DOM nodes outside the supported list
- structuredClone throws DataCloneError on values containing non-cloneable references

## structuredClone identity preservation
- structuredClone preserves shared-reference identity within a single call
- structuredClone preserves circular references

## structuredClone transfer
- structuredClone with transfer detaches transferred ArrayBuffers
- structuredClone with transfer transfers a MessagePort
