import { format, addDays, subWeeks, differenceInDays, startOfMonth, endOfYear, isValid, parseISO, isAfter, isBefore } from "date-fns";

const d = parseISO("2026-05-12T10:30:00.000Z");

process.stdout.write(JSON.stringify({
  iso: d.toISOString(),
  format1: format(d, "yyyy-MM-dd"),
  format2: format(d, "MMM d, yyyy"),
  addDay: addDays(d, 1).toISOString(),
  subWeek: subWeeks(d, 1).toISOString(),
  startMonth: startOfMonth(d).toISOString(),
  endYear: endOfYear(d).toISOString(),
  diffDays: differenceInDays(parseISO("2026-12-31"), parseISO("2026-01-01")),
  isValid: isValid(d),
  isInvalid: isValid(parseISO("not-a-date")),
  isBefore: isBefore(parseISO("2025-01-01"), parseISO("2026-01-01")),
  isAfter: isAfter(parseISO("2026-01-01"), parseISO("2025-12-31")),
}) + "\n");
