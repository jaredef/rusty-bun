/**
 * @template T
 * @param {ReadableStream<Uint8Array>} stream
 * @returns {Promise<T[]>}
 */
export async function parseStream(stream) {
    const reader = stream.getReader();
    /** @type {T[]} */const dataArray = [];
    const decoder = new TextDecoder();
    let buffer = "";

    while (true) {
        const { value, done } = await reader.read();
        if (done) {
            if (buffer.trim()) {
                try {
                    dataArray.push(JSON.parse(buffer));
                } catch (e) { }
            }
            break;
        }

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split('\n');
        buffer = lines.pop() ?? "";
        for (const line of lines) {
            const trimmed = line.trim();
            if (trimmed) {
                dataArray.push(JSON.parse(trimmed));
            }
        }
    }
    return dataArray;
}