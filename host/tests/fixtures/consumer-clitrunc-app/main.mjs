import ct from "cli-truncate";
const lines = [];
lines.push("1 " + ct("hello world", 8));
lines.push("2 " + ct("hi", 10));
lines.push("3 " + ct("a very long string for truncation", 12));
lines.push("4 " + ct("ends here.", 10));
lines.push("5 " + ct("[31mred[0m text and more", 8));
lines.push("6 " + ct("中文长字符串", 4));
process.stdout.write(lines.join("\n") + "\n");
