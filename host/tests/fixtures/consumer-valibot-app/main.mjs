// valibot ^1 — functional schema validator (alternative to zod). Tests
// the canonical safeParse / parse surface in a different style: schemas
// are values (object({...})) rather than methods on a builder class.
import * as v from "valibot";

const lines = [];

async function main() {
  // 1: primitives
  {
    const Str = v.string();
    lines.push("1 ok=" + v.safeParse(Str, "hello").success +
               " fail=" + v.safeParse(Str, 42).success);
  }

  // 2: numeric range via pipe
  {
    const N = v.pipe(v.number(), v.integer(), v.minValue(0), v.maxValue(100));
    lines.push("2 " + v.safeParse(N, 50).success + "," +
                       v.safeParse(N, -1).success + "," +
                       v.safeParse(N, 101).success + "," +
                       v.safeParse(N, 1.5).success);
  }

  // 3: object schema with optional + defaults
  {
    const User = v.object({
      name: v.pipe(v.string(), v.minLength(1)),
      age: v.pipe(v.number(), v.integer(), v.minValue(0)),
      tags: v.optional(v.array(v.string()), []),
    });
    const r1 = v.safeParse(User, { name: "alice", age: 30 });
    lines.push("3a ok=" + r1.success + " tags=" + JSON.stringify(r1.output.tags));
    const r2 = v.safeParse(User, { name: "", age: -1 });
    lines.push("3b ok=" + r2.success + " issueCount=" + (r2.issues ? r2.issues.length : 0));
  }

  // 4: discriminated union via variant
  {
    const Event = v.variant("type", [
      v.object({ type: v.literal("click"), x: v.number(), y: v.number() }),
      v.object({ type: v.literal("key"), code: v.string() }),
    ]);
    const a = v.safeParse(Event, { type: "click", x: 10, y: 20 }).success;
    const b = v.safeParse(Event, { type: "key", code: "Enter" }).success;
    const c = v.safeParse(Event, { type: "scroll", delta: 5 }).success;
    lines.push("4 click=" + a + " key=" + b + " scroll=" + c);
  }

  // 5: transform + check
  {
    const Trimmed = v.pipe(
      v.string(),
      v.transform((s) => s.trim()),
      v.check((s) => s.length > 0, "empty after trim"),
    );
    const r1 = v.safeParse(Trimmed, "  hi  ");
    const r2 = v.safeParse(Trimmed, "   ");
    lines.push("5 r1=" + r1.success + ":" + r1.output + " r2=" + r2.success);
  }

  // 6: nested + JSON round-trip
  {
    const Order = v.object({
      id: v.number(),
      items: v.array(v.object({
        sku: v.string(),
        qty: v.pipe(v.number(), v.integer(), v.minValue(1)),
      })),
    });
    const data = { id: 1, items: [{ sku: "A", qty: 2 }, { sku: "B", qty: 1 }] };
    const parsed = v.parse(Order, data);
    lines.push("6 " + JSON.stringify(parsed));
  }
}

try {
  await main();
  process.stdout.write(lines.join("\n") + "\n");
} catch (e) {
  process.stdout.write("FATAL " + e.constructor.name + ": " + e.message + "\n");
}
