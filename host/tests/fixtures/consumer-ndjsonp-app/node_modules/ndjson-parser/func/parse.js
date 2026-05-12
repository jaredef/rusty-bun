/**
 * @template T
 * @param {string} ndJson
 * @return {T[]}
 */
export function parse(ndJson) {
    const lines = ndJson.split('\n');
    const dataArray = [];
    for (const line of lines) {
        const trimmed = line.trim();
        if (trimmed) {
            dataArray.push(JSON.parse(trimmed));
        }
    }
    return dataArray;
}