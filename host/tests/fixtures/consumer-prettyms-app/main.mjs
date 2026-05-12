// pretty-ms ^9 — humanize milliseconds.
import prettyMs from "pretty-ms";

const lines = [];
lines.push("1 " + prettyMs(1337));
lines.push("2 " + prettyMs(60000));
lines.push("3 " + prettyMs(3601000));
lines.push("4 " + prettyMs(86400000));
lines.push("5 " + prettyMs(1500, { secondsDecimalDigits: 0 }));
lines.push("6 " + prettyMs(1234567, { compact: true }));

process.stdout.write(lines.join("\n") + "\n");
