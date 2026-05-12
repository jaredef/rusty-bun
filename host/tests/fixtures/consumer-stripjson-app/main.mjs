// strip-json-comments ^5 — remove // and /* */ from JSON-with-comments.
import stripJsonComments from "strip-json-comments";

const lines = [];
lines.push("1 " + JSON.stringify(stripJsonComments('{"a":1 /* c */, "b":2}')));
lines.push("2 " + JSON.stringify(stripJsonComments('{\n  // line\n  "x": 1\n}')));
lines.push("3 " + JSON.stringify(stripJsonComments('"no comments"')));
lines.push("4 " + JSON.stringify(stripJsonComments('"// not a comment"')));
lines.push("5 " + JSON.stringify(stripJsonComments('{"a":1, "b":2}', { whitespace: false })));
lines.push("6 " + JSON.stringify(stripJsonComments('{"a":1 /* multi\nline */, "b":2}')));

process.stdout.write(lines.join("\n") + "\n");
