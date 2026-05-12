// lru-cache ^11 — LRU cache primitive. Distinct data-structure axis,
// used by basically every npm package's cache layer.
import { LRUCache } from "lru-cache";

const lines = [];

// 1: basic set/get + size
{
  const c = new LRUCache({ max: 3 });
  c.set("a", 1);
  c.set("b", 2);
  c.set("c", 3);
  lines.push("1 size=" + c.size + " a=" + c.get("a") + " b=" + c.get("b") + " c=" + c.get("c"));
}

// 2: eviction at max capacity
{
  const c = new LRUCache({ max: 2 });
  c.set("a", 1);
  c.set("b", 2);
  c.set("c", 3);  // evicts a
  lines.push("2 hasA=" + c.has("a") + " hasB=" + c.has("b") + " hasC=" + c.has("c"));
}

// 3: LRU recency — get a, then set c, b should evict (not a)
{
  const c = new LRUCache({ max: 2 });
  c.set("a", 1);
  c.set("b", 2);
  c.get("a");  // a becomes MRU
  c.set("c", 3);  // evicts b
  lines.push("3 hasA=" + c.has("a") + " hasB=" + c.has("b") + " hasC=" + c.has("c"));
}

// 4: delete + clear
{
  const c = new LRUCache({ max: 5 });
  c.set("x", 10);
  c.set("y", 20);
  const d = c.delete("x");
  lines.push("4 deleted=" + d + " size=" + c.size);
  c.clear();
  lines.push("4b clearSize=" + c.size);
}

// 5: ttl-based expiry (without real timers; just check shape)
{
  const c = new LRUCache({ max: 5, ttl: 60000 });
  c.set("k", "v");
  lines.push("5 has=" + c.has("k") + " size=" + c.size);
}

// 6: forEach + keys + values
{
  const c = new LRUCache({ max: 5 });
  c.set("a", 1); c.set("b", 2); c.set("c", 3);
  const keys = Array.from(c.keys()).sort();
  const vals = Array.from(c.values()).sort();
  lines.push("6 keys=" + JSON.stringify(keys) + " vals=" + JSON.stringify(vals));
}

// 7: dump + load (serialization)
{
  const c = new LRUCache({ max: 5 });
  c.set("a", 1); c.set("b", 2);
  const dump = c.dump();
  const c2 = new LRUCache({ max: 5 });
  c2.load(dump);
  lines.push("7 c2.a=" + c2.get("a") + " c2.b=" + c2.get("b") + " c2.size=" + c2.size);
}

process.stdout.write(lines.join("\n") + "\n");
