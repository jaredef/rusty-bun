import mitt from "mitt";

const lines = [];

// 1: basic on + emit
{
  const e = mitt();
  const out = [];
  e.on("foo", v => out.push(v));
  e.emit("foo", 1);
  e.emit("foo", 2);
  lines.push("1 " + JSON.stringify(out));
}

// 2: off
{
  const e = mitt();
  const out = [];
  const h = v => out.push(v);
  e.on("x", h);
  e.emit("x", 1);
  e.off("x", h);
  e.emit("x", 2);
  lines.push("2 " + JSON.stringify(out));
}

// 3: wildcard *
{
  const e = mitt();
  const out = [];
  e.on("*", (type, value) => out.push(type + ":" + value));
  e.emit("a", 1);
  e.emit("b", 2);
  lines.push("3 " + JSON.stringify(out));
}

// 4: all map directly accessible
{
  const e = mitt();
  e.on("x", () => {});
  e.on("x", () => {});
  e.on("y", () => {});
  lines.push("4 xCount=" + e.all.get("x").length + " yCount=" + e.all.get("y").length);
}

// 5: off without handler clears all for type
{
  const e = mitt();
  e.on("z", () => {});
  e.on("z", () => {});
  e.off("z");
  lines.push("5 zCount=" + (e.all.get("z") ? e.all.get("z").length : 0));
}

process.stdout.write(lines.join("\n") + "\n");
