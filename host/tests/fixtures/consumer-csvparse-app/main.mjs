import { parse } from "csv-parse/sync";

const input = `name,age,city
Ada,32,London
Bob,45,"New, York"
"Carol ""Tre""",27,Paris
`;

const rows = parse(input, { columns: true, skip_empty_lines: true });
const noHeader = parse("1,2,3\n4,5,6", { skip_empty_lines: true });

process.stdout.write(JSON.stringify({
  rows,
  noHeader,
}) + "\n");
