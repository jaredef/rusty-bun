// diff ^7 — JS diff library. Pure-JS, distinct axis (text diffing).
import { diffChars, diffWords, diffLines, createPatch, applyPatch } from "diff";

const lines = [];

// 1: char diff
{
  const d = diffChars("abc", "abd");
  lines.push("1 parts=" + d.length + " hasRemove=" + d.some(p => p.removed) + " hasAdd=" + d.some(p => p.added));
}

// 2: word diff
{
  const d = diffWords("hello brave world", "hello new world");
  const summary = d.map(p => (p.added ? "+" : p.removed ? "-" : "=") + p.value.trim()).filter(s => s.length > 1).join("|");
  lines.push("2 " + summary);
}

// 3: line diff
{
  const d = diffLines("a\nb\nc\n", "a\nB\nc\n");
  lines.push("3 chunks=" + d.length + " modified=" + d.filter(p => p.added || p.removed).length);
}

// 4: createPatch
{
  const p = createPatch("file.txt", "line1\nline2\n", "line1\nLINE2\n", "old", "new");
  // Just check shape: starts with Index, contains @@, contains -line2 and +LINE2
  lines.push("4 hasIndex=" + p.startsWith("Index:") +
             " hasMarker=" + p.includes("@@") +
             " hasMinus=" + p.includes("-line2") +
             " hasPlus=" + p.includes("+LINE2"));
}

// 5: applyPatch round-trip
{
  const orig = "x\ny\nz\n";
  const mod = "x\nY\nz\n";
  const p = createPatch("f.txt", orig, mod);
  const applied = applyPatch(orig, p);
  lines.push("5 roundTrip=" + (applied === mod));
}

// 6: identical inputs
{
  const d = diffChars("same", "same");
  lines.push("6 unchanged=" + (d.length === 1 && !d[0].added && !d[0].removed));
}

process.stdout.write(lines.join("\n") + "\n");
