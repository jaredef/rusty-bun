import prettyBytes from "pretty-bytes";
const lines = [];
lines.push("1 " + prettyBytes(0));
lines.push("2 " + prettyBytes(1024));
lines.push("3 " + prettyBytes(1500000));
lines.push("4 " + prettyBytes(1500000, { binary: true }));
lines.push("5 " + prettyBytes(-1234));
lines.push("6 " + prettyBytes(123456789, { maximumFractionDigits: 1 }));
process.stdout.write(lines.join("\n") + "\n");
