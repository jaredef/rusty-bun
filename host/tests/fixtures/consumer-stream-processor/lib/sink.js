// Sink module. Writes records to a file via fs, returning a Promise
// that resolves with the count of records written.

function makeFileSink(path) {
    const lines = [];
    return {
        stream: new WritableStream({
            write(chunk) {
                lines.push(JSON.stringify(chunk));
            },
            close() {
                fs.writeFileSync(path, lines.join("\n") + "\n");
            }
        }),
        getCount: () => lines.length,
    };
}

module.exports = { makeFileSink };
