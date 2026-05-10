// Stream-source module. CJS shape with module.exports.
// Produces a ReadableStream of "records" — small objects representing
// the kind of payload a real consumer's pipeline would process.

function makeRecordStream(records, signal) {
    return new ReadableStream({
        start(controller) {
            for (const r of records) {
                if (signal && signal.aborted) {
                    controller.error(new Error("aborted"));
                    return;
                }
                controller.enqueue(r);
            }
            controller.close();
        }
    });
}

module.exports = { makeRecordStream };
