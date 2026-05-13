import { Temporal } from "temporal-polyfill";
const d = Temporal.PlainDate.from("2026-05-13");
process.stdout.write(JSON.stringify({ year: d.year, month: d.month, day: d.day }) + "\n");
