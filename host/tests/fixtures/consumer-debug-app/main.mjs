// debug ^4 — env-based logger. Pure CJS, tests CJS bridge on a package
// that uses ms internally (transitive CJS-in-CJS) and reads process.env.
// Captures emitted log lines via debug.log override.
import debug from "debug";

const lines = [];

// Capture: override debug.log so output is deterministic.
const captured = [];
debug.log = (...args) => { captured.push(args.join(" ")); };

// 1: enable all namespaces, verify enabled() shape
debug.enable("test:*");
lines.push("1 enabledAB=" + debug.enabled("test:a") + " enabledOther=" + debug.enabled("other:x"));

// 2: create a logger + emit
{
  const log = debug("test:basic");
  log("hello");
  log("with arg %d", 42);
  log("with %s", "format");
  // captured[i] contains a colored ns prefix + the args. Just check counts
  // and substrings byte-identical between runtimes.
  lines.push("2 count=" + captured.length +
             " hasHello=" + (captured[0] || "").includes("hello") +
             " hasFmt=" + (captured[1] || "").includes("42") +
             " hasStr=" + (captured[2] || "").includes("format"));
}

// 3: disabled namespace doesn't emit
{
  const before = captured.length;
  const off = debug("other:silent");
  off("should not appear");
  lines.push("3 noEmit=" + (captured.length === before) + " enabled=" + debug.enabled("other:silent"));
}

// 4: extend
{
  const log = debug("test:basic");
  const sub = log.extend("sub");
  // sub namespace becomes "test:basic:sub"
  lines.push("4 subNs=" + sub.namespace + " enabled=" + debug.enabled(sub.namespace));
}

// 5: disable
{
  debug.disable();
  const log = debug("test:basic");
  const before = captured.length;
  log("after disable");
  lines.push("5 noEmitAfterDisable=" + (captured.length === before));
}

process.stdout.write(lines.join("\n") + "\n");
