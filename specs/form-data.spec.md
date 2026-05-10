# FormData — WHATWG XHR §5

[surface] FormData
[spec] https://xhr.spec.whatwg.org/#interface-formdata

## FormData is exposed as a global constructor
- FormData is defined as a global constructor in any execution context with [Exposed=*]
- new FormData() returns an empty FormData instance
- new FormData(form) constructs entries from an HTMLFormElement
- new FormData(form, submitter) marks submitter inputs as activating

## FormData.prototype.append
- FormData.prototype.append(name, value) adds a string entry
- FormData.prototype.append(name, blobValue, filename) adds a Blob entry with optional filename
- FormData.prototype.append never replaces an existing entry with the same name

## FormData.prototype.delete
- FormData.prototype.delete(name) removes all entries with the given name

## FormData.prototype.get
- FormData.prototype.get(name) returns the first matching entry's value
- FormData.prototype.get returns null when no entry matches

## FormData.prototype.getAll
- FormData.prototype.getAll(name) returns an array of all matching entry values
- FormData.prototype.getAll returns an empty array when no entry matches

## FormData.prototype.has
- FormData.prototype.has(name) returns true when an entry with the name exists

## FormData.prototype.set
- FormData.prototype.set(name, value) replaces all existing entries with one
- FormData.prototype.set(name, blobValue, filename) replaces with a Blob entry

## FormData iteration
- FormData.prototype.entries returns name-value pairs in insertion order
- FormData.prototype.keys returns names
- FormData.prototype.values returns values
- FormData.prototype.forEach invokes a callback for each entry in insertion order
