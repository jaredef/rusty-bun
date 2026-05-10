# queueMicrotask — HTML §8.1.3.4

[surface] queueMicrotask
[spec] https://html.spec.whatwg.org/multipage/webappapis.html#dom-queuemicrotask

## queueMicrotask is exposed as a global function
- queueMicrotask is defined as a global function in any execution context with [Exposed=*]
- queueMicrotask(callback) schedules callback for invocation in the microtask queue
- queueMicrotask returns undefined

## queueMicrotask scheduling semantics
- queueMicrotask runs callback before the next macrotask
- queueMicrotask runs callbacks in FIFO order within the same microtask checkpoint
- queueMicrotask preserves the calling realm for callback invocation

## queueMicrotask error handling
- queueMicrotask throws TypeError when callback is not a function
- queueMicrotask reports uncaught exceptions through the host's error reporting
