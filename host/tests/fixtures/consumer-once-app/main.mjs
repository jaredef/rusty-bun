import once from "once";

const lines = [];

// 1: basic — fn called only first time, subsequent return cached value
{
  let n = 0;
  const f = once(() => ++n);
  const r1 = f();
  const r2 = f();
  const r3 = f();
  lines.push("1 r1=" + r1 + " r2=" + r2 + " r3=" + r3 + " n=" + n);
}

// 2: this + args
{
  const f = once(function (x, y) { return this.scale * (x + y); });
  const ctx = { scale: 10 };
  const r1 = f.call(ctx, 1, 2);
  const r2 = f.call(ctx, 99, 99);  // ignored, returns cached
  lines.push("2 r1=" + r1 + " r2=" + r2);
}

// 3: once.strict throws on re-call
{
  let n = 0;
  const f = once.strict(() => ++n);
  f();
  try { f(); lines.push("3 NOT_THROWN"); }
  catch (e) { lines.push("3 threw=" + (e instanceof Error)); }
}

// 4: called flag
{
  const f = once(() => 42);
  lines.push("4 beforeCall=" + f.called + " value=" + f.value);
  f();
  lines.push("4b afterCall=" + f.called + " value=" + f.value);
}

process.stdout.write(lines.join("\n") + "\n");
