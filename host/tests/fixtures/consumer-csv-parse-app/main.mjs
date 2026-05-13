import { parse } from "csv-parse/sync";
const r = parse("a,b\n1,2\n3,4", { columns: true });
process.stdout.write(JSON.stringify({ rows: r.length, first: r[0] }) + "\n");
