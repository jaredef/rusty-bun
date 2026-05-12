import md5Hex from "md5-hex";
const lines = [];
lines.push("1 " + md5Hex("hello"));
lines.push("2 " + md5Hex(""));
lines.push("3 " + md5Hex("abc"));
lines.push("4 " + md5Hex(Buffer.from([0xde, 0xad, 0xbe, 0xef])));
lines.push("5 " + md5Hex("The quick brown fox jumps over the lazy dog"));
lines.push("6 " + md5Hex(["a", "b", "c"]));
process.stdout.write(lines.join("\n") + "\n");
