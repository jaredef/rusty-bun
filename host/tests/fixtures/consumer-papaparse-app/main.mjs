// papaparse ^5 — CSV parser/unparser (distinct from csv-parse already
// tested; papaparse has its own parsing engine and quirks).
import Papa from "papaparse";

const lines = [];

const r1 = Papa.parse("a,b,c\n1,2,3\n4,5,6");
lines.push("1 " + JSON.stringify(r1.data));

const r2 = Papa.parse("name,age\nalice,30\nbob,25", { header: true });
lines.push("2 " + JSON.stringify(r2.data));

const r3 = Papa.parse('a,"b,c",d\n1,"two, three",4', { skipEmptyLines: true });
lines.push("3 " + JSON.stringify(r3.data));

lines.push("4 " + Papa.unparse([["a", "b"], [1, 2], [3, 4]]));
lines.push("5 " + Papa.unparse([{ x: 1, y: 2 }, { x: 3, y: 4 }]));

const r6 = Papa.parse("a;b;c\n1;2;3", { delimiter: ";" });
lines.push("6 " + JSON.stringify(r6.data));

process.stdout.write(lines.join("\n") + "\n");
