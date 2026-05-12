// uniqid ^5 — timestamp-based id generator (distinct from hyperid/ulid/nanoid).
import uniqid from "uniqid";

const lines = [];
const a = uniqid();
const b = uniqid();
lines.push("1 both=" + (a !== b));
lines.push("2 allHex=" + /^[a-z0-9]+$/.test(a));
lines.push("3 withPrefix=" + uniqid("p-").startsWith("p-"));
lines.push("4 withSuffix=" + uniqid("", "-s").endsWith("-s"));
lines.push("5 processVariant=" + (typeof uniqid.process === "function"));
lines.push("6 timeVariant=" + (typeof uniqid.time === "function"));

process.stdout.write(lines.join("\n") + "\n");
