import dayjs from "dayjs";

const lines = [];

async function main() {
  // 1: parse + format ISO date deterministically
  {
    const d = dayjs("2026-05-11T15:30:45.123Z");
    lines.push("1 " + d.toISOString());
  }

  // 2: arithmetic
  {
    const d = dayjs("2026-01-15T00:00:00Z").add(30, "day").subtract(5, "hour");
    lines.push("2 " + d.toISOString());
  }

  // 3: difference in days
  {
    const a = dayjs("2026-05-11T00:00:00Z");
    const b = dayjs("2026-01-01T00:00:00Z");
    lines.push("3 days=" + a.diff(b, "day"));
  }

  // 4: getters
  {
    const d = dayjs("2026-07-04T12:34:56Z");
    lines.push("4 y=" + d.year() + " m=" + d.month() + " d=" + d.date() + " hh=" + d.hour() + " mm=" + d.minute() + " ss=" + d.second());
  }

  // 5: isValid + bad input
  {
    const ok = dayjs("2026-02-29T00:00:00Z").isValid();
    const bad = dayjs("not-a-date").isValid();
    lines.push("5 leap2026=" + ok + " bogus=" + bad);
  }

  // 6: start/end of unit
  {
    const d = dayjs("2026-05-11T15:30:45.123Z");
    lines.push("6 startDay=" + d.startOf("day").toISOString() + " endDay=" + d.endOf("day").toISOString());
  }
}

try {
  await main();
  process.stdout.write(lines.join("\n") + "\n");
} catch (e) {
  process.stdout.write("FATAL " + e.constructor.name + ": " + e.message + "\n");
}
