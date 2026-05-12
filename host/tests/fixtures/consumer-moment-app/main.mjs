import moment from "moment";

const d = moment("2026-05-12T10:30:00.000Z");
const out = {
  iso: d.toISOString(),
  year: d.year(),
  month: d.month(),
  date: d.date(),
  format1: d.format("YYYY-MM-DD"),
  format2: d.format("MMM D, YYYY"),
  format3: d.format("dddd"),
  addDay: d.clone().add(1, "day").toISOString(),
  subWeek: d.clone().subtract(1, "week").toISOString(),
  startOfMonth: d.clone().startOf("month").toISOString(),
  endOfYear: d.clone().endOf("year").toISOString(),
  diffDays: moment("2026-12-31T00:00:00Z").diff(moment("2026-01-01T00:00:00Z"), "days"),
  isValid: d.isValid(),
  isInvalid: moment("not-a-date", "YYYY-MM-DD", true).isValid(),
  isBefore: moment("2025-01-01").isBefore("2026-01-01"),
  isAfter: moment("2026-01-01").isAfter("2025-12-31"),
  // Edge: leap year
  isLeap2024: moment("2024-02-29").isValid(),
  feb29_2025: moment("2025-02-29", "YYYY-MM-DD", true).isValid(),
};

process.stdout.write(JSON.stringify(out) + "\n");
