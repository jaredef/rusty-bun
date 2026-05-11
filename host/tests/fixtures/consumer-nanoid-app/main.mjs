// nanoid ^5 differential — IDs are non-deterministic, so the differential
// asserts structural properties (length, character-set membership, count
// of generated IDs) that match Bun's output line-for-line.
import { nanoid, customAlphabet, urlAlphabet } from "nanoid";

const lines = [];

async function main() {
  // 1: default nanoid length + URL-safe character set membership
  {
    const id = nanoid();
    const okLen = id.length === 21;
    // urlAlphabet is the canonical 64-char set nanoid draws from.
    const okSet = [...id].every(ch => urlAlphabet.includes(ch));
    lines.push("1 len=" + id.length + " okLen=" + okLen + " okSet=" + okSet);
  }

  // 2: explicit length
  {
    const id = nanoid(10);
    lines.push("2 len=" + id.length);
  }

  // 3: customAlphabet round-trip — only A-Z, fixed length
  {
    const gen = customAlphabet("ABCDEFGHIJ", 8);
    const id = gen();
    const okSet = [...id].every(ch => "ABCDEFGHIJ".includes(ch));
    lines.push("3 len=" + id.length + " okSet=" + okSet);
  }

  // 4: uniqueness across many calls (probabilistic, but extremely high prob)
  {
    const set = new Set();
    for (let i = 0; i < 500; i++) set.add(nanoid());
    lines.push("4 unique=" + (set.size === 500));
  }

  // 5: urlAlphabet length
  {
    lines.push("5 urlAlphabetLen=" + urlAlphabet.length);
  }
}

try {
  await main();
  process.stdout.write(lines.join("\n") + "\n");
} catch (e) {
  process.stdout.write("FATAL " + e.constructor.name + ": " + e.message + "\n");
}
