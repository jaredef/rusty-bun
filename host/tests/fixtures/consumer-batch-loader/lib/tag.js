// Tagged template literal: a small query-format helper. Real consumers
// use tagged templates for SQL escaping, GraphQL, CSS-in-JS, etc.

export function q(strings, ...values) {
    const escaped = values.map((v) => {
        if (typeof v === "bigint") return v.toString();
        if (typeof v === "string") return JSON.stringify(v);
        if (v === null || v === undefined) return "null";
        return String(v);
    });
    let out = strings[0];
    for (let i = 0; i < escaped.length; i++) {
        out += escaped[i] + strings[i + 1];
    }
    return out;
}
