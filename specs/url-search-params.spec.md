# URLSearchParams — WHATWG URL §5.2

[surface] URLSearchParams
[spec] https://url.spec.whatwg.org/#interface-urlsearchparams

## URLSearchParams is exposed as a global constructor
- URLSearchParams is defined as a global constructor in any execution context with [Exposed=*]
- new URLSearchParams() returns an empty URLSearchParams instance
- new URLSearchParams(init) accepts a USVString, sequence of pairs, or record

## URLSearchParams constructor input forms
- URLSearchParams constructor accepts a query-string starting with optional "?"
- URLSearchParams constructor accepts a sequence of name-value pairs
- URLSearchParams constructor accepts a record of name to value
- URLSearchParams constructor with another URLSearchParams copies its entries

## URLSearchParams.prototype.append method
- URLSearchParams.prototype.append(name, value) adds a new entry
- URLSearchParams.prototype.append never replaces an existing entry with the same name

## URLSearchParams.prototype.delete method
- URLSearchParams.prototype.delete(name) removes all entries with the given name
- URLSearchParams.prototype.delete(name, value) removes only entries matching both

## URLSearchParams.prototype.get method
- URLSearchParams.prototype.get(name) returns the value of the first matching entry
- URLSearchParams.prototype.get returns null when no entry matches

## URLSearchParams.prototype.getAll method
- URLSearchParams.prototype.getAll(name) returns an array of all values for the name
- URLSearchParams.prototype.getAll returns an empty array when no entry matches

## URLSearchParams.prototype.has method
- URLSearchParams.prototype.has(name) returns true if any entry matches the name
- URLSearchParams.prototype.has(name, value) returns true if an entry matches both

## URLSearchParams.prototype.set method
- URLSearchParams.prototype.set(name, value) replaces all existing entries with one
- URLSearchParams.prototype.set preserves the position of the first existing entry

## URLSearchParams.prototype.sort method
- URLSearchParams.prototype.sort orders entries by name using a stable sort over UTF-16 code units
- URLSearchParams.prototype.sort preserves the relative order of entries with equal names

## URLSearchParams.prototype.toString method
- URLSearchParams.prototype.toString returns the application/x-www-form-urlencoded serialization
- URLSearchParams.prototype.toString percent-encodes per the form-urlencoded character set
- URLSearchParams.prototype.toString joins entries with "&" and pairs with "="

## URLSearchParams.prototype.size getter
- URLSearchParams.prototype.size returns the count of entries

## URLSearchParams iteration
- URLSearchParams.prototype.entries returns an iterator of name-value pairs
- URLSearchParams.prototype.keys returns an iterator of names
- URLSearchParams.prototype.values returns an iterator of values
- URLSearchParams.prototype.forEach invokes a callback for each entry in insertion order
