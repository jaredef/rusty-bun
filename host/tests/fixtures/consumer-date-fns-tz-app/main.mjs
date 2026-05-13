import { formatInTimeZone } from "date-fns-tz";
const r = formatInTimeZone(new Date("2026-05-13T12:00:00Z"), "America/New_York", "yyyy-MM-dd HH:mm zzz");
process.stdout.write(JSON.stringify({ result: r }) + "\n");
