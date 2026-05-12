// fast-sort ^3 — fluent multi-key array sort.
import { sort } from "fast-sort";

const lines = [];
const data = [
  { name: "alice", age: 30 },
  { name: "bob", age: 25 },
  { name: "carol", age: 30 },
];

lines.push("1 " + JSON.stringify(sort([3, 1, 4, 1, 5, 9, 2, 6]).asc()));
lines.push("2 " + JSON.stringify(sort([3, 1, 4, 1, 5]).desc()));
lines.push("3 " + sort(data).asc(u => u.age).map(u => u.name).join(","));
lines.push("4 " + sort(data).desc(u => u.name).map(u => u.name).join(","));
lines.push("5 " + sort(data).by([{ asc: u => u.age }, { asc: u => u.name }]).map(u => u.name).join(","));
lines.push("6 " + JSON.stringify(sort(["b", "a", "c"]).asc()));

process.stdout.write(lines.join("\n") + "\n");
