/**
 * @template T
 * @returns {TransformStream<Uint8Array, T>}
 */
export function createTransformer() {
    let buffer = "";
    const decoder = new TextDecoder();

    return new TransformStream({
        /**
         * @param {Uint8Array} chunk 
         * @param {TransformStreamDefaultController<T>} controller 
         */
        transform(chunk, controller) {
            buffer += decoder.decode(chunk, { stream: true });

            const lines = buffer.split("\n");
            buffer = lines.pop() || "";

            for (const line of lines) {
                const trimmed = line.trim();
                if (trimmed) {
                    try {
                        controller.enqueue(JSON.parse(trimmed));
                    } catch (e) {
                        console.error("JSON 파싱 에러:", e);
                    }
                }
            }
        },

        /**
         * @param {TransformStreamDefaultController<T>} controller
         */
        flush(controller) {
            if (buffer.trim()) {
                try {
                    controller.enqueue(JSON.parse(buffer));
                } catch (e) {
                    // 마지막 파편 무시
                }
            }
        }
    });
}