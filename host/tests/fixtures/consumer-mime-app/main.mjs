// mime ^4 — content-type lookup by extension. Distinct axis from any
// existing fixture (no other MIME-DB consumer).
import mime from "mime";

const lines = [];

lines.push("1 " + mime.getType("html"));
lines.push("2 " + mime.getType("file.json"));
lines.push("3 " + mime.getType("archive.tar.gz"));
lines.push("4 " + mime.getType("unknownext"));
lines.push("5 " + mime.getExtension("application/json"));
lines.push("6 " + mime.getExtension("image/png"));

process.stdout.write(lines.join("\n") + "\n");
