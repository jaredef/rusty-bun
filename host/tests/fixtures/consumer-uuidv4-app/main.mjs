// uuid-random ^1 — fast UUIDv4 generator (alt to uuid package).
import uuid from "uuid-random";

const lines = [];
const ids = Array.from({ length: 50 }, () => uuid());
lines.push("1 unique=" + (new Set(ids).size === 50));
lines.push("2 v4=" + ids.every(s => /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(s)));
lines.push("3 isValid=" + uuid.test(ids[0]) + "/" + uuid.test("not-a-uuid"));
lines.push("4 len=" + ids[0].length);

const bin = uuid.bin();
lines.push("5 binLen=" + bin.length + " isBuf=" + (bin instanceof Uint8Array));

lines.push("6 isFn=" + (typeof uuid === "function"));

process.stdout.write(lines.join("\n") + "\n");
