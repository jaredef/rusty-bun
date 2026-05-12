import camelCase from "camelcase";
const lines = [];
lines.push("1 " + camelCase("foo-bar"));
lines.push("2 " + camelCase("foo_bar_baz"));
lines.push("3 " + camelCase("Foo Bar"));
lines.push("4 " + camelCase(["foo", "bar", "baz"]));
lines.push("5 " + camelCase("foo-bar", { pascalCase: true }));
lines.push("6 " + camelCase("API_KEY_TOKEN", { preserveConsecutiveUppercase: true }));
process.stdout.write(lines.join("\n") + "\n");
