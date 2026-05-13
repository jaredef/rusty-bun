import moment from "moment-timezone";
const m = moment.tz("2026-05-13 12:00", "America/New_York");
process.stdout.write(JSON.stringify({ iso: m.toISOString() }) + "\n");
