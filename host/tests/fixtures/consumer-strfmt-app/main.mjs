// fast-printf ^1 — printf-style format strings.
import { printf } from "fast-printf";

const lines = [];

lines.push("1 " + printf("hello %s", "world"));
lines.push("2 " + printf("%d + %d = %d", 1, 2, 3));
lines.push("3 " + printf("%05d", 42));
lines.push("4 " + printf("%.3f", Math.PI));
lines.push("5 " + printf("%-10s|", "left"));
lines.push("6 " + printf("%x %X %o %b", 255, 255, 8, 5));

process.stdout.write(lines.join("\n") + "\n");
