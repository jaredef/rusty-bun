import dayjs from "dayjs";
import utc from "dayjs/plugin/utc.js";
import timezone from "dayjs/plugin/timezone.js";
dayjs.extend(utc);
dayjs.extend(timezone);

const d = new Date("2026-05-13T12:00:00Z");
const r1 = dayjs(d).tz("America/New_York").format("YYYY-MM-DD HH:mm");
const r2 = dayjs(d).tz("Asia/Tokyo").format("YYYY-MM-DD HH:mm");
const r3 = dayjs(d).tz("Europe/London").format("YYYY-MM-DD HH:mm");
process.stdout.write(JSON.stringify({ r1, r2, r3 }) + "\n");
