const r = [];
try {
  const { DateTime, Duration } = await import("luxon");
  r.push("import=ok");
  const dt = DateTime.fromISO("2024-03-15T10:30:00Z").toUTC();
  r.push("1 iso=" + dt.toISO());
  r.push("2 fmt=" + dt.toFormat("yyyy-LL-dd HH:mm"));
  r.push("3 plus=" + dt.plus({ days: 5 }).toISO());
  r.push("4 dur=" + Duration.fromObject({ hours: 2, minutes: 30 }).toFormat("hh:mm"));
  r.push("5 diff=" + DateTime.fromISO("2024-03-15").diff(DateTime.fromISO("2024-03-10"), "days").days);
  r.push("6 wkday=" + dt.weekdayLong);
} catch (e) { r.push("ERR " + e.constructor.name + ": " + e.message); }
process.stdout.write(r.join("\n") + "\n");
