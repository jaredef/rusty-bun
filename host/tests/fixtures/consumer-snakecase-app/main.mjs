// snake-case ^4 — convert string to snake_case (distinct from camelcase
// which had QuickJS regex flag edge; this is its sister lib by same author).
import { snakeCase } from "snake-case";

const lines = [];
lines.push("1 " + snakeCase("helloWorld"));
lines.push("2 " + snakeCase("Hello World"));
lines.push("3 " + snakeCase("foo-bar-baz"));
lines.push("4 " + snakeCase("HTTPServer"));
lines.push("5 " + snakeCase("getURLPath"));
lines.push("6 " + snakeCase("foo123Bar456"));

process.stdout.write(lines.join("\n") + "\n");
