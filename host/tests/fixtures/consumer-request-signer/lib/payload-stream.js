// Source of payloads as a ReadableStream. Demonstrates async iteration
// of streams in consumer code (for-await-of).
export function makePayloadStream(payloads) {
    return new ReadableStream({
        start(controller) {
            for (const p of payloads) controller.enqueue(p);
            controller.close();
        }
    });
}
