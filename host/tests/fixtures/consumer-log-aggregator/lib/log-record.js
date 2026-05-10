// Log record shape. Demonstrates structured payloads with complex
// fields (Date, RegExp via tag-matcher, Map of metadata).

import path from "node:path";

export function makeLogRecord(sourcePath, level, message, tags) {
    return {
        source: {
            full: sourcePath,
            dir: path.dirname(sourcePath),
            base: path.basename(sourcePath),
            ext: path.extname(sourcePath),
        },
        level,
        message,
        // Date and Map exercise structuredClone's complex-type handling.
        when: new Date(0),
        tags: new Map(tags.map((t) => [t, true])),
    };
}

// Build a query-string for an upstream log API from a filter.
export function buildQuery(filter) {
    const p = new URLSearchParams();
    if (filter.level) p.set("level", filter.level);
    if (filter.tag) p.set("tag", filter.tag);
    if (filter.dir) p.set("dir", filter.dir);
    return p.toString();
}
