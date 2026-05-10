// Sink module. Writes records to a file via node:fs, returning a
// WritableStream and a count getter.
const fs = require("node:fs");

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
