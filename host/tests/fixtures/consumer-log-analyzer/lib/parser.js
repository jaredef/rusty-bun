// Log-line parser using regex with named capture groups.
// Pattern: "2024-01-15T10:23:45.123Z [LEVEL] component: message"

const LINE_RE =
    /^(?<timestamp>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z)\s+\[(?<level>[A-Z]+)\]\s+(?<component>[\w.-]+):\s+(?<message>.+)$/;

export function parseLine(line) {
    const m = LINE_RE.exec(line);
    if (!m || !m.groups) return null;
    return {
        timestamp: new Date(m.groups.timestamp),
        level: m.groups.level,
        component: m.groups.component,
        message: m.groups.message,
    };
}

// Extract all duration mentions of the form "took 123ms" from a message.
const DUR_RE = /took (?<ms>\d+)ms/g;

export function extractDurations(message) {
    const out = [];
    for (const m of message.matchAll(DUR_RE)) {
        out.push(Number(m.groups.ms));
    }
    return out;
}
