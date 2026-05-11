// ulid ^2 — sortable IDs. Tests structural properties (length, character
// set, monotonicity over time, decode-of-time-component).
import { ulid, decodeTime, monotonicFactory } from "ulid";

const lines = [];

async function main() {
  // 1: structural — length 26, uppercase alphanumeric (Crockford base32)
  {
    const id = ulid();
    const okLen = id.length === 26;
    const okSet = /^[0-9A-HJKMNP-TV-Z]{26}$/.test(id);
    lines.push("1 len=" + id.length + " okLen=" + okLen + " okSet=" + okSet);
  }

  // 2: deterministic time component — given a fixed timestamp,
  //    decodeTime(ulid(ts)) === ts
  {
    const t = 1700000000000;
    const id = ulid(t);
    lines.push("2 decoded=" + (decodeTime(id) === t) + " idTimeOk=" + (id.slice(0, 10).length === 10));
  }

  // 3: monotonic factory — under fixed seed time, IDs strictly increase
  {
    const mono = monotonicFactory();
    const t = 1700000000000;
    const a = mono(t);
    const b = mono(t);
    const c = mono(t);
    lines.push("3 ordered=" + (a < b && b < c));
  }

  // 4: uniqueness across many calls
  {
    const set = new Set();
    for (let i = 0; i < 200; i++) set.add(ulid());
    lines.push("4 unique=" + (set.size === 200));
  }

  // 5: decodeTime monotonic with real time
  {
    const id1 = ulid();
    // small synchronous busy wait to ensure ms tick
    const start = Date.now();
    while (Date.now() - start < 5) { /* busy */ }
    const id2 = ulid();
    lines.push("5 later=" + (decodeTime(id2) >= decodeTime(id1)));
  }
}

try {
  await main();
  process.stdout.write(lines.join("\n") + "\n");
} catch (e) {
  process.stdout.write("FATAL " + e.constructor.name + ": " + e.message + "\n");
}
