// base32.js ^0.1 — RFC 4648 base32 encoder.
import base32 from "base32.js";

const lines = [];
const enc = new base32.Encoder();
const dec = new base32.Decoder();

lines.push("1 " + enc.write("hi").finalize());
lines.push("2 " + enc.write("hello world").finalize());
lines.push("3 " + enc.write("").finalize());

const round = enc.write(new Uint8Array([1, 2, 3, 4])).finalize();
lines.push("4 " + round);

const back = dec.write(round).finalize();
lines.push("5 backLen=" + back.length + " [0]=" + back[0]);

const e2 = new base32.Encoder({ type: "crockford" });
lines.push("6 " + e2.write("xyz").finalize());

process.stdout.write(lines.join("\n") + "\n");
