// cookie ^1 — RFC 6265 cookie parse + serialize. Distinct axis: HTTP
// cookie header (de)serialization with attribute encoding.
import { parse, serialize } from "cookie";

const lines = [];

lines.push("1 " + JSON.stringify(parse("foo=bar; baz=qux")));
lines.push("2 " + JSON.stringify(parse("a=1; b=hello%20world")));
lines.push("3 " + serialize("name", "value"));
lines.push("4 " + serialize("sid", "abc", { httpOnly: true, secure: true, sameSite: "strict", path: "/" }));
lines.push("5 " + serialize("k", "needs encoding=yes"));
lines.push("6 " + serialize("exp", "x", { maxAge: 3600 }));

process.stdout.write(lines.join("\n") + "\n");
