// querystringify ^2 — simple URL querystring (used by url-parse).
import * as qs from "querystringify";

const lines = [];
lines.push("1 " + qs.stringify({ a: 1, b: "hello" }));
lines.push("2 " + qs.stringify({ a: 1, b: 2 }, true));
lines.push("3 " + qs.stringify({ x: "a b", y: "c+d" }));
lines.push("4 " + JSON.stringify(qs.parse("a=1&b=two")));
lines.push("5 " + JSON.stringify(qs.parse("?foo=bar&baz=qux")));
lines.push("6 " + JSON.stringify(qs.parse("a=hello%20world")));

process.stdout.write(lines.join("\n") + "\n");
