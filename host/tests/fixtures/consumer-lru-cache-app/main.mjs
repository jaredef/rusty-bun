import { LRUCache } from "lru-cache";

const c = new LRUCache({ max: 3 });

c.set("a", 1);
c.set("b", 2);
c.set("c", 3);

const beforeOverflow = { a: c.get("a"), size: c.size };

c.set("d", 4);  // Evicts oldest (which was "b" since "a" was just accessed)

const afterOverflow = {
  hasA: c.has("a"),
  hasB: c.has("b"),
  hasC: c.has("c"),
  hasD: c.has("d"),
  size: c.size,
};

c.delete("c");
const afterDelete = { size: c.size, hasC: c.has("c") };

process.stdout.write(JSON.stringify({
  beforeOverflow,
  afterOverflow,
  afterDelete,
}) + "\n");
