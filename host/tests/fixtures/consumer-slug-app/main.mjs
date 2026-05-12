import slugify from "slugify";
const lines = [];

lines.push("1 " + slugify("Hello World"));
lines.push("2 " + slugify("Café & Restaurant"));
lines.push("3 " + slugify("Hello World", { lower: true }));
lines.push("4 " + slugify("foo--bar  baz"));
lines.push("5 " + slugify("hello world", { replacement: "_" }));
lines.push("6 " + slugify("Hello!@#World$"));
lines.push("7 " + slugify("über große Köpfe"));

process.stdout.write(lines.join("\n") + "\n");
