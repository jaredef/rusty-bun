# AbortController and AbortSignal — DOM §3.3

[surface] AbortController
[spec] https://dom.spec.whatwg.org/#interface-abortcontroller

## AbortController is exposed as a global constructor
- AbortController is defined as a global constructor in any execution context with [Exposed=*]
- new AbortController() returns an AbortController with a non-aborted signal

## AbortController.prototype.signal
- AbortController.prototype.signal returns the associated AbortSignal

## AbortController.prototype.abort
- AbortController.prototype.abort() aborts the signal with the default reason
- AbortController.prototype.abort(reason) aborts the signal with the given reason
- AbortController.prototype.abort is idempotent after the first call

## AbortSignal is exposed as a global constructor
- AbortSignal is defined as a global constructor in any execution context with [Exposed=*]
- AbortSignal cannot be constructed directly; new AbortSignal() throws TypeError

## AbortSignal.abort static method
- AbortSignal.abort() returns an already-aborted AbortSignal with default reason
- AbortSignal.abort(reason) returns an already-aborted AbortSignal with the reason

## AbortSignal.timeout static method
- AbortSignal.timeout(ms) returns an AbortSignal that aborts after ms milliseconds
- AbortSignal.timeout aborts with a TimeoutError DOMException

## AbortSignal.any static method
- AbortSignal.any(signals) returns an AbortSignal aborted when any signal aborts
- AbortSignal.any returns an already-aborted signal when any input is already aborted

## AbortSignal.prototype.aborted
- AbortSignal.prototype.aborted returns whether the signal has been aborted

## AbortSignal.prototype.reason
- AbortSignal.prototype.reason returns the abort reason or undefined when not aborted

## AbortSignal.prototype.throwIfAborted
- AbortSignal.prototype.throwIfAborted throws the abort reason when aborted
- AbortSignal.prototype.throwIfAborted does nothing when not aborted

## AbortSignal events
- AbortSignal dispatches an "abort" event when transitioning to aborted
