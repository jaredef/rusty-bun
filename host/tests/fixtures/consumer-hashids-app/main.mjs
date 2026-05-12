// hashids ^2 — short reversible ID encoder (encodes integers ↔ short
// alphanumeric strings). Distinct axis. Tests deterministic encode/decode
// pair + salt-affected output + alphabet customization.
import Hashids from "hashids";

const lines = [];

// 1: encode/decode round-trip with default salt
{
  const h = new Hashids("test-salt");
  const id = h.encode(123);
  const back = h.decode(id);
  lines.push("1 id=" + id + " back=" + JSON.stringify(back));
}

// 2: encode multiple numbers
{
  const h = new Hashids("salt");
  const id = h.encode([1, 2, 3]);
  const back = h.decode(id);
  lines.push("2 id=" + id + " back=" + JSON.stringify(back));
}

// 3: salt changes output
{
  const a = new Hashids("salt-A");
  const b = new Hashids("salt-B");
  const ida = a.encode(42);
  const idb = b.encode(42);
  lines.push("3 ida=" + ida + " idb=" + idb + " differ=" + (ida !== idb));
}

// 4: minimum length padding
{
  const h = new Hashids("salt", 10);
  const id = h.encode(1);
  lines.push("4 len=" + id.length + " atLeast10=" + (id.length >= 10));
}

// 5: custom alphabet (must be >= 16 unique chars)
{
  const h = new Hashids("salt", 0, "abcdefghijklmnop");
  const id = h.encode(42);
  const ok = [...id].every(c => "abcdefghijklmnop".includes(c));
  lines.push("5 id=" + id + " inAlphabet=" + ok);
}

// 6: invalid decode throws
{
  const h = new Hashids("salt");
  try { h.decode("!!!not-valid!!!"); lines.push("6 NOT_THROWN"); }
  catch (e) { lines.push("6 threw=" + (e instanceof Error)); }
}

process.stdout.write(lines.join("\n") + "\n");
