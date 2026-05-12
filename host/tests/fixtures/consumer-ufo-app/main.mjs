// ufo ^1 — URL composition utilities (unjs ecosystem).
import { joinURL, withQuery, parseURL, normalizeURL, withTrailingSlash } from "ufo";

const lines = [];
lines.push("1 " + joinURL("https://example.com", "a", "b"));
lines.push("2 " + withQuery("https://example.com/", { q: "hi", n: 42 }));
lines.push("3 " + JSON.stringify(parseURL("https://example.com:8080/a/b?q=1#frag")));
lines.push("4 " + normalizeURL("https://example.com/a/./b/../c"));
lines.push("5 " + withTrailingSlash("https://example.com/path"));
lines.push("6 " + joinURL("foo", "bar?q=1"));

process.stdout.write(lines.join("\n") + "\n");
