# Streams — WHATWG Streams Standard

[surface] ReadableStream
[spec] https://streams.spec.whatwg.org/

## ReadableStream is exposed as a global constructor
- ReadableStream is defined as a global constructor in any execution context with [Exposed=*]
- new ReadableStream() returns a default-constructed ReadableStream
- new ReadableStream(underlyingSource) accepts an UnderlyingSource with start, pull, cancel callbacks
- new ReadableStream(underlyingSource, queuingStrategy) accepts a strategy with highWaterMark and size

## ReadableStream.prototype.locked getter
- ReadableStream.prototype.locked returns true when a reader has been acquired
- ReadableStream.prototype.locked returns false when no reader is active

## ReadableStream.prototype.cancel method
- ReadableStream.prototype.cancel returns a Promise resolving to undefined when the stream cancels successfully
- ReadableStream.prototype.cancel(reason) propagates the reason to the underlying source's cancel callback
- ReadableStream.prototype.cancel rejects with TypeError when the stream is locked

## ReadableStream.prototype.getReader method
- ReadableStream.prototype.getReader returns a ReadableStreamDefaultReader by default
- ReadableStream.prototype.getReader throws TypeError when the stream is already locked

## ReadableStream.prototype.tee method
- ReadableStream.prototype.tee returns a two-element array of two new ReadableStreams
- ReadableStream.prototype.tee locks the original stream
- The tee'd streams emit the same chunks in the same order as the original

## ReadableStreamDefaultReader.prototype.read method
- ReadableStreamDefaultReader.prototype.read returns a Promise resolving to {value, done}
- ReadableStreamDefaultReader.prototype.read returns {value: undefined, done: true} when the stream has ended
- ReadableStreamDefaultReader.prototype.read returns {value: chunk, done: false} for each enqueued chunk in order

## ReadableStreamDefaultReader.prototype.cancel method
- ReadableStreamDefaultReader.prototype.cancel cancels the underlying stream
- ReadableStreamDefaultReader.prototype.cancel releases the lock

## ReadableStreamDefaultReader.prototype.releaseLock method
- ReadableStreamDefaultReader.prototype.releaseLock unlocks the underlying stream
- After releaseLock, subsequent read calls on the released reader reject with TypeError

## ReadableStreamDefaultController.prototype.enqueue method
- ReadableStreamDefaultController.prototype.enqueue adds a chunk to the stream's internal queue
- ReadableStreamDefaultController.prototype.enqueue throws TypeError when the stream is closed or errored

## ReadableStreamDefaultController.prototype.close method
- ReadableStreamDefaultController.prototype.close transitions the stream to the closed state
- ReadableStreamDefaultController.prototype.close throws TypeError when called on an already-closed stream

## ReadableStreamDefaultController.prototype.error method
- ReadableStreamDefaultController.prototype.error transitions the stream to the errored state
- ReadableStreamDefaultController.prototype.error sets the stream's stored error

## WritableStream is exposed as a global constructor
[surface] WritableStream
- WritableStream is defined as a global constructor in any execution context with [Exposed=*]
- new WritableStream() accepts an UnderlyingSink with start, write, close, abort callbacks
- new WritableStream(sink, queuingStrategy) accepts a strategy with highWaterMark and size

## WritableStream.prototype.locked getter
- WritableStream.prototype.locked returns true when a writer has been acquired
- WritableStream.prototype.locked returns false when no writer is active

## WritableStream.prototype.abort method
- WritableStream.prototype.abort transitions the stream to the errored state with the provided reason
- WritableStream.prototype.abort calls the underlying sink's abort callback

## WritableStream.prototype.close method
- WritableStream.prototype.close requests the stream close after pending writes complete
- WritableStream.prototype.close throws TypeError when the stream is locked

## WritableStream.prototype.getWriter method
- WritableStream.prototype.getWriter returns a WritableStreamDefaultWriter
- WritableStream.prototype.getWriter throws TypeError when the stream is already locked

## WritableStreamDefaultWriter.prototype.write method
- WritableStreamDefaultWriter.prototype.write enqueues a chunk to the underlying sink
- WritableStreamDefaultWriter.prototype.write returns a Promise that resolves when the chunk is processed

## WritableStreamDefaultWriter.prototype.close method
- WritableStreamDefaultWriter.prototype.close requests the stream close
- WritableStreamDefaultWriter.prototype.close releases the writer lock when complete

## TransformStream is exposed as a global constructor
[surface] TransformStream
- TransformStream is defined as a global constructor in any execution context with [Exposed=*]
- new TransformStream() accepts a Transformer with start, transform, flush callbacks
- new TransformStream(transformer, writableStrategy, readableStrategy) accepts both queuing strategies

## TransformStream.prototype.readable getter
- TransformStream.prototype.readable returns the readable side of the transform
- The readable side emits chunks the transformer's transform callback produces

## TransformStream.prototype.writable getter
- TransformStream.prototype.writable returns the writable side of the transform
- Chunks written to the writable side feed the transformer's transform callback
