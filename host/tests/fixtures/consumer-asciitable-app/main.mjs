// cli-table3 ^0.6 — Unicode-aware ASCII tables.
import Table from "cli-table3";

const lines = [];
const t = new Table({ head: ["Name", "Age"], style: { head: [], border: [] } });
t.push(["alice", 30], ["bob", 25]);
lines.push("1\n" + t.toString());

const t2 = new Table({ head: ["A", "B"], style: { head: [], border: [] } });
t2.push(["1", "2"]);
lines.push("2\n" + t2.toString());

const v = new Table({ style: { head: [], border: [] } });
v.push({ "Name": ["alice"] }, { "Age": ["30"] });
lines.push("3\n" + v.toString());

process.stdout.write(lines.join("\n") + "\n");
