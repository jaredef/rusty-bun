try {
  const m = await import("base64url");
  const b64u = m.default;
  const lines = [];
  lines.push("1 " + b64u("hello world"));
  lines.push("2 " + b64u.decode("aGVsbG8gd29ybGQ"));
  lines.push("3 " + b64u.fromBase64("YWJj+/8="));
  lines.push("4 " + b64u.toBase64("YWJj-_8"));
  lines.push("5 " + b64u.encode(Buffer.from([0xff, 0xfe, 0xfd])));
  lines.push("6 " + b64u(Buffer.from("subject=test+plus")));
  process.stdout.write(lines.join("\n") + "\n");
} catch (e) { process.stdout.write("ERR " + e.message + " stack=" + (e.stack||"").split("\n").slice(0,4).join("|") + "\n"); }
