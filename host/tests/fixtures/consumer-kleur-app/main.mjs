// kleur ^4 — small terminal coloring, distinct from picocolors (chainable
// .red().bold() API rather than single-call function).
import kleur from "kleur";

const lines = [];

// Force colors so output is deterministic across TTY/non-TTY.
kleur.enabled = true;

// 1: simple
lines.push("1 red=" + kleur.red("hi") + " green=" + kleur.green("ok"));

// 2: chained
lines.push("2 " + kleur.red().bold("BAD"));

// 3: multi-attr
lines.push("3 " + kleur.bgYellow().black("warn") + " " + kleur.underline().green("ok"));

// 4: nested string interp
{
  const name = "alice";
  lines.push("4 hello, " + kleur.cyan(name) + "!");
}

// 5: disabled
kleur.enabled = false;
lines.push("5 " + kleur.red("plain") + " " + kleur.bold().green("again"));

// 6: re-enabled
kleur.enabled = true;
lines.push("6 " + kleur.magenta("colored"));

process.stdout.write(lines.join("\n") + "\n");
