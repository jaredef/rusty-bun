// p-cancelable ^4 — cancelable promises with onCancel callback.
import PCancelable from "p-cancelable";

const lines = [];

async function main() {
  const p1 = new PCancelable((resolve, reject, onCancel) => {
    setTimeout(() => resolve("done"), 5);
  });
  lines.push("1 " + (await p1));

  const p2 = new PCancelable((resolve, reject, onCancel) => {
    const id = setTimeout(() => resolve("late"), 100);
    onCancel(() => clearTimeout(id));
  });
  p2.cancel();
  let err = null;
  try { await p2; } catch (e) { err = e.name; }
  lines.push("2 cancelled=" + err);

  const p3 = new PCancelable((resolve) => resolve("immediate"));
  lines.push("3 " + (await p3));

  const p4 = new PCancelable((resolve, reject) => reject(new Error("nope")));
  let err4 = null;
  try { await p4; } catch (e) { err4 = e.message; }
  lines.push("4 err=" + err4);

  const p5 = PCancelable.fn(async (str) => str + "-done")("hi");
  lines.push("5 " + (await p5));

  lines.push("6 cls=" + (typeof PCancelable === "function"));
}

await main();
process.stdout.write(lines.join("\n") + "\n");
