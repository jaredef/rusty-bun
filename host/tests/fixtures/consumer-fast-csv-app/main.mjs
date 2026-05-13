import * as csv from "fast-csv";
process.stdout.write(JSON.stringify({ hasParse: typeof csv.parse, hasFormat: typeof csv.format }) + "\n");
