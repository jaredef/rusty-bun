// encodeurl ^2 — RFC 3986 percent-encoding (idempotent on encoded chars).
import encodeUrl from "encodeurl";

const lines = [];
lines.push("1 " + encodeUrl("https://example.com/foo bar"));
lines.push("2 " + encodeUrl("https://example.com/foo%20bar"));
lines.push("3 " + encodeUrl("/path?q=hello world&n=2"));
lines.push("4 " + encodeUrl("/café"));
lines.push("5 " + encodeUrl("/<x>"));
lines.push("6 " + encodeUrl("/a%5Bb%5Dc"));

process.stdout.write(lines.join("\n") + "\n");
