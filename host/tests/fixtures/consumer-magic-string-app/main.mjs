import MagicString from "magic-string";
const s = new MagicString("hello world");
s.overwrite(0, 5, "HI");
process.stdout.write(JSON.stringify({ result: s.toString() }) + "\n");
