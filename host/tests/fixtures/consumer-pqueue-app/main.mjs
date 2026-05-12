// p-queue ^9 — priority-aware promise queue with concurrency.
import PQueue from "p-queue";

const lines = [];

async function main() {
  const q = new PQueue({ concurrency: 2 });

  const out = [];
  const tasks = [1, 2, 3, 4, 5].map(n =>
    q.add(async () => {
      await new Promise(r => setTimeout(r, 5));
      out.push(n);
      return n * 10;
    })
  );
  const results = await Promise.all(tasks);
  lines.push("1 results=" + JSON.stringify(results));
  lines.push("2 all=" + (out.length === 5));

  const q2 = new PQueue({ concurrency: 1 });
  const order = [];
  q2.add(async () => { order.push("low"); }, { priority: 1 });
  q2.add(async () => { order.push("high"); }, { priority: 10 });
  q2.add(async () => { order.push("mid"); }, { priority: 5 });
  await q2.onIdle();
  lines.push("3 " + JSON.stringify(order));

  const q3 = new PQueue({ concurrency: 1 });
  q3.pause();
  const p = q3.add(async () => "done");
  lines.push("4 paused=" + q3.isPaused + " size=" + q3.size);
  q3.start();
  const r = await p;
  lines.push("5 resumed=" + r);

  const q4 = new PQueue({ concurrency: 2 });
  await q4.addAll([async () => 1, async () => 2, async () => 3]);
  lines.push("6 idle=" + (q4.size === 0 && q4.pending === 0));
}

await main();
process.stdout.write(lines.join("\n") + "\n");
