// url-parse ^1 — drop-in URL parser used widely in Node libs (distinct
// axis from URL class and uri-js).
import URLParse from "url-parse";

const lines = [];

const u = new URLParse("https://user:pw@example.com:8080/a/b?x=1&y=2#frag");
lines.push("1 " + u.protocol + "|" + u.host + "|" + u.pathname + "|" + u.hash);
lines.push("2 user=" + u.username + " pw=" + u.password);
lines.push("3 origin=" + u.origin);

const u2 = new URLParse("/relative/path", "https://example.com");
lines.push("4 " + u2.href);

const u3 = new URLParse("https://example.com/?a=1&b=2", true);
lines.push("5 " + JSON.stringify(u3.query));

const u4 = new URLParse("https://example.com");
u4.set("query", { foo: "bar" });
lines.push("6 " + u4.href);

process.stdout.write(lines.join("\n") + "\n");
