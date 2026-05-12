const r = [];
try {
  const ndjsonParser = (await import("ndjson-parser")).default;
  const ndjson = '{"a":1}\n{"a":2}\n{"a":3}\n';
  const stream = new ReadableStream({
    start(c) { c.enqueue(new TextEncoder().encode(ndjson)); c.close(); }
  });
  const reader = stream.pipeThrough(ndjsonParser.createTransformer()).getReader();
  const out = [];
  while (true) {
    const { value, done } = await reader.read();
    if (done) break;
    out.push(value);
  }
  r.push("parsed=" + JSON.stringify(out));
} catch (e) { r.push("ERR " + e.constructor.name + ": " + e.message); }
process.stdout.write(r.join("\n") + "\n");
