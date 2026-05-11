// zod schema validation differential — pure-JS, no I/O. Tests the
// canonical safeParse / parse surface across primitives + objects +
// arrays + transforms + refinements + unions.
import { z } from "zod";

const lines = [];

async function main() {
  // 1: primitives
  {
    const ok = z.string().safeParse("hello").success;
    const fail = z.string().safeParse(42).success;
    lines.push("1 str-ok=" + ok + " num-as-str=" + fail);
  }

  // 2: number with min/max
  {
    const s = z.number().int().min(0).max(100);
    lines.push("2 " + s.safeParse(50).success + "," + s.safeParse(-1).success + "," + s.safeParse(101).success + "," + s.safeParse(1.5).success);
  }

  // 3: object schema
  {
    const User = z.object({
      id: z.string().uuid().optional(),
      name: z.string().min(1),
      age: z.number().int().nonnegative(),
      tags: z.array(z.string()).default([]),
    });
    const r1 = User.safeParse({ name: "alice", age: 30 });
    lines.push("3a ok=" + r1.success + " tags=" + JSON.stringify(r1.data.tags) + " hasUuid=" + (r1.data.id !== undefined));
    const r2 = User.safeParse({ name: "", age: -1 });
    lines.push("3b ok=" + r2.success + " errs=" + r2.error.issues.length);
  }

  // 4: union + literal
  {
    const Event = z.union([
      z.object({ type: z.literal("click"), x: z.number(), y: z.number() }),
      z.object({ type: z.literal("key"), code: z.string() }),
    ]);
    const a = Event.safeParse({ type: "click", x: 10, y: 20 }).success;
    const b = Event.safeParse({ type: "key", code: "Enter" }).success;
    const c = Event.safeParse({ type: "scroll", delta: 5 }).success;
    lines.push("4 click=" + a + " key=" + b + " scroll=" + c);
  }

  // 5: transform + refine
  {
    const Trimmed = z.string().transform((s) => s.trim()).refine((s) => s.length > 0, "empty after trim");
    const r1 = Trimmed.safeParse("  hi  ");
    const r2 = Trimmed.safeParse("   ");
    lines.push("5 r1=" + r1.success + ":" + r1.data + " r2=" + r2.success);
  }

  // 6: nested + JSON round-trip
  {
    const Order = z.object({
      id: z.number(),
      items: z.array(z.object({ sku: z.string(), qty: z.number().int().positive() })),
      meta: z.record(z.string()),
    });
    const data = { id: 1, items: [{ sku: "A", qty: 2 }, { sku: "B", qty: 1 }], meta: { source: "web" } };
    const parsed = Order.parse(data);
    lines.push("6 " + JSON.stringify(parsed));
  }
}

try {
  await main();
  process.stdout.write(lines.join("\n") + "\n");
} catch (e) {
  process.stdout.write("FATAL " + e.constructor.name + ": " + e.message + "\n");
}
