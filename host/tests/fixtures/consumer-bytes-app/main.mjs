import bytes from "bytes";
const lines = [];

// parse string -> number
lines.push("1 " + bytes("1KB") + " " + bytes("1MB") + " " + bytes("1.5GB") + " " + bytes("500B"));

// format number -> string
lines.push("2 " + bytes(1024) + " " + bytes(1024 * 1024) + " " + bytes(1500));

// format with options
lines.push("3 " + bytes(1500, { unitSeparator: " " }) + " | " + bytes(1500, { decimalPlaces: 0 }));

// invalid
lines.push("4 " + bytes("not-a-byte") + " " + bytes("garbage"));

// negative numbers
lines.push("5 " + bytes("-1KB") + " " + bytes(-2048));

process.stdout.write(lines.join("\n") + "\n");
