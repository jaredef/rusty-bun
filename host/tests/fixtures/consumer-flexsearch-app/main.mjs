// fuse.js ^7 — fuzzy search library.
import Fuse from "fuse.js";

const lines = [];
const list = ["apple", "application", "banana", "appliance", "orange", "approach"];
const fuse = new Fuse(list, { threshold: 0.4 });

const r1 = fuse.search("aple");
lines.push("1 best=" + r1[0].item + " count=" + r1.length);

const r2 = fuse.search("ban");
lines.push("2 " + r2[0].item);

const fuse2 = new Fuse([
  { title: "Old Man and the Sea", author: "Hemingway" },
  { title: "The Old Curiosity Shop", author: "Dickens" },
], { keys: ["title"], threshold: 0.3 });
const r3 = fuse2.search("old");
lines.push("3 found=" + r3.length);

const r4 = fuse.search("xyzpdq");
lines.push("4 empty=" + (r4.length === 0));

const fuse3 = new Fuse(list, { threshold: 0.6, includeScore: true });
const r5 = fuse3.search("appl");
lines.push("5 hasScore=" + (typeof r5[0].score === "number"));

const fuse4 = new Fuse([], { threshold: 0.3 });
lines.push("6 emptyList=" + fuse4.search("a").length);

process.stdout.write(lines.join("\n") + "\n");
