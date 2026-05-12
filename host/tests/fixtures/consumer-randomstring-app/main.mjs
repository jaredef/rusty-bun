// randomstring ^1 — generate random alphanumeric strings.
import randomstring from "randomstring";

const lines = [];

const s1 = randomstring.generate();
lines.push("1 defaultLen=" + s1.length);

const s2 = randomstring.generate(16);
lines.push("2 customLen=" + s2.length);

const s3 = randomstring.generate({ length: 8, charset: "numeric" });
lines.push("3 numeric=" + /^[0-9]{8}$/.test(s3));

const s4 = randomstring.generate({ length: 10, charset: "hex" });
lines.push("4 hex=" + /^[0-9a-f]{10}$/.test(s4));

const s5 = randomstring.generate({ length: 12, charset: "alphabetic", capitalization: "uppercase" });
lines.push("5 upper=" + /^[A-Z]{12}$/.test(s5));

const ids = new Set();
for (let i = 0; i < 50; i++) ids.add(randomstring.generate(16));
lines.push("6 unique=" + (ids.size === 50));

process.stdout.write(lines.join("\n") + "\n");
