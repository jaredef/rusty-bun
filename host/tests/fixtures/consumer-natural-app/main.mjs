// natural-orderby ^5 — natural sort (numbers, alphanumeric, dates, etc).
import { orderBy, compare } from "natural-orderby";

const lines = [];
lines.push("1 " + JSON.stringify(orderBy(["A10", "A2", "A1", "A11"])));
lines.push("2 " + JSON.stringify(orderBy([{ v: "z2" }, { v: "z10" }, { v: "z1" }], [u => u.v])));
lines.push("3 " + JSON.stringify(orderBy(["3.5", "20.0", "1.0"], [], ["desc"])));
lines.push("4 " + ["b1", "a2", "a10", "a1"].sort(compare()).join(","));
lines.push("5 " + JSON.stringify(orderBy(["item1", "Item10", "item2"], [], ["asc"])));
lines.push("6 " + JSON.stringify(orderBy([{ a: 1, b: "x10" }, { a: 1, b: "x2" }, { a: 2, b: "x1" }], ["a", "b"])));

process.stdout.write(lines.join("\n") + "\n");
