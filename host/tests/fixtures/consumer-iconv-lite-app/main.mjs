import iconv from "iconv-lite";
const b = iconv.encode("hello", "utf8");
const s = iconv.decode(b, "utf8");
process.stdout.write(JSON.stringify({ s, len: b.length }) + "\n");
