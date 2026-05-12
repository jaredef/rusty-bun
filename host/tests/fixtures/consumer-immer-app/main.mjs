// immer ^10 — immutable state via Proxy-based draft objects.
// Distinct axis (structural sharing, copy-on-write).
import { produce, original, current, isDraft, freeze, enableMapSet } from "immer";
enableMapSet();

const lines = [];

// 1: basic produce — mutate draft, get new immutable
{
  const state = { count: 0, items: ["a", "b"] };
  const next = produce(state, (draft) => {
    draft.count = 1;
    draft.items.push("c");
  });
  lines.push("1 origCount=" + state.count + " nextCount=" + next.count +
             " origLen=" + state.items.length + " nextLen=" + next.items.length +
             " differ=" + (state !== next));
}

// 2: structural sharing — unchanged subtree referentially equal
{
  const state = { a: { x: 1 }, b: { y: 2 } };
  const next = produce(state, (draft) => { draft.a.x = 99; });
  lines.push("2 aShared=" + (state.a === next.a) + " bShared=" + (state.b === next.b));
}

// 3: no-op produce returns same reference
{
  const state = { x: 1 };
  const next = produce(state, () => { /* no change */ });
  lines.push("3 same=" + (state === next));
}

// 4: nested update
{
  const state = { user: { profile: { name: "alice", age: 30 } } };
  const next = produce(state, (draft) => { draft.user.profile.age = 31; });
  lines.push("4 origAge=" + state.user.profile.age + " nextAge=" + next.user.profile.age);
}

// 5: array operations
{
  const state = [1, 2, 3, 4, 5];
  const next = produce(state, (draft) => {
    draft.push(6);
    draft[0] = 100;
  });
  lines.push("5 orig=" + JSON.stringify(state) + " next=" + JSON.stringify(next));
}

// 6: Map support
{
  const state = new Map([["a", 1], ["b", 2]]);
  const next = produce(state, (draft) => { draft.set("c", 3); draft.delete("a"); });
  lines.push("6 origSize=" + state.size + " nextSize=" + next.size + " hasC=" + next.has("c") + " noA=" + !next.has("a"));
}

// 7: return value (replace state entirely)
{
  const state = { count: 5 };
  const next = produce(state, () => ({ count: 100, fresh: true }));
  lines.push("7 " + JSON.stringify(next));
}

// 8: original() inside producer
{
  const state = { n: 1 };
  let origInside;
  produce(state, (draft) => {
    draft.n = 42;
    origInside = original(draft).n;
  });
  lines.push("8 origInside=" + origInside);
}

process.stdout.write(lines.join("\n") + "\n");
