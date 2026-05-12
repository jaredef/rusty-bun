import yargsParser from "yargs-parser";
const lines = [];
lines.push("1 " + JSON.stringify(yargsParser(["--foo", "1", "--bar", "two"])));
lines.push("2 " + JSON.stringify(yargsParser(["-abc", "-v"])));
lines.push("3 " + JSON.stringify(yargsParser(["--no-color", "--list", "a", "b", "c"])));
lines.push("4 " + JSON.stringify(yargsParser(["pos1", "pos2", "--flag"])));
lines.push("5 " + JSON.stringify(yargsParser(["--num", "42", "--str", "hi"], { number: ["num"], string: ["str"] })));
lines.push("6 " + JSON.stringify(yargsParser(["--a.b.c", "deep"])));
process.stdout.write(lines.join("\n") + "\n");
