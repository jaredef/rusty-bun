// minilog ^3 — tiny logger lib (CJS-style, distinct from pino/winston).
import Minilog from "minilog";

const lines = [];
const log = Minilog("test");

lines.push("1 isFn=" + (typeof log.info === "function"));
lines.push("2 hasError=" + (typeof log.error === "function"));
lines.push("3 hasWarn=" + (typeof log.warn === "function"));
lines.push("4 hasDebug=" + (typeof log.debug === "function"));

const l2 = Minilog("other");
lines.push("5 separate=" + (l2 !== log));

Minilog.suggest.allow("test", "info");
lines.push("6 hasSuggest=" + (typeof Minilog.suggest === "object"));

process.stdout.write(lines.join("\n") + "\n");
