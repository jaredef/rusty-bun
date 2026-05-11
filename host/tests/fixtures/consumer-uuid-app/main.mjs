// uuid ^11 — Tests v4 (random) structurally and v5 (SHA-1 namespace+name)
// for byte-identical deterministic output across runtimes.
import { v4, v5, validate, version, NIL, MAX } from "uuid";

const lines = [];

async function main() {
  // 1: v4 structural
  {
    const id = v4();
    lines.push("1 len=" + id.length + " v=" + version(id) + " valid=" + validate(id));
  }

  // 2: v5 deterministic with DNS namespace
  {
    const NS_DNS = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
    const a = v5("example.com", NS_DNS);
    const b = v5("example.com", NS_DNS);
    const c = v5("example.org", NS_DNS);
    lines.push("2 a=" + a + " same=" + (a === b) + " diff=" + (a !== c));
  }

  // 3: known v5 vector — uuid.v5("hello", v5.URL) is documented in tests
  {
    const NS_URL = "6ba7b811-9dad-11d1-80b4-00c04fd430c8";
    const known = v5("hello", NS_URL);
    lines.push("3 hello-url=" + known + " v=" + version(known));
  }

  // 4: validate NIL + MAX
  {
    lines.push("4 nil=" + NIL + " validNil=" + validate(NIL) + " validMax=" + validate(MAX));
  }

  // 5: invalid string rejected by validate
  {
    lines.push("5 " + validate("not-a-uuid") + " " + validate("ffffffff-ffff-ffff-ffff-ffffffffffff"));
  }
}

try {
  await main();
  process.stdout.write(lines.join("\n") + "\n");
} catch (e) {
  process.stdout.write("FATAL " + e.constructor.name + ": " + e.message + "\n");
}
