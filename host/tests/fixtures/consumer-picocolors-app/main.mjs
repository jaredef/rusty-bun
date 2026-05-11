// picocolors ^1 — terminal styling, CJS-only. Tests the E.13 CJS-in-ESM
// bridge: import pc from "picocolors" must work against a CJS module.
import pc, { createColors, isColorSupported } from "picocolors";

const lines = [];

// 1: default (non-TTY) — no color codes emitted
lines.push("1 isSupported=" + isColorSupported + " red=" + pc.red("hi"));

// 2: forced-on formatter — ANSI codes byte-identical
{
  const c = createColors(true);
  lines.push("2 red=" + c.red("hi") + " bold=" + c.bold("hi") + " green=" + c.green("ok"));
}

// 3: explicitly disabled
{
  const c = createColors(false);
  lines.push("3 red=" + c.red("hi") + " bold=" + c.bold("hi"));
}

// 4: nested colors
{
  const c = createColors(true);
  lines.push("4 " + c.red("err: " + c.bold("BAD") + " happened"));
}

process.stdout.write(lines.join("\n") + "\n");
