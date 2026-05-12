// yaml ^2 — YAML 1.2 parser. Tests parse + stringify + Document API.
import YAML from "yaml";

const lines = [];

async function main() {
  // 1: parse simple YAML
  {
    const obj = YAML.parse("name: alice\nage: 30\ntags:\n  - dev\n  - cli");
    lines.push("1 name=" + obj.name + " age=" + obj.age + " tags=" + JSON.stringify(obj.tags));
  }

  // 2: stringify object
  {
    const yaml = YAML.stringify({ a: 1, b: [2, 3], c: { x: true } }).trim();
    lines.push("2 " + yaml.replace(/\n/g, " | "));
  }

  // 3: anchors + aliases
  {
    const src = "common: &c\n  ttl: 60\n  retry: 3\napi:\n  <<: *c\n  host: api.test\n";
    // Note: yaml@2 doesn't merge with << by default; parse to see structure.
    const obj = YAML.parse(src);
    lines.push("3 commonTtl=" + obj.common.ttl + " apiHost=" + obj.api.host);
  }

  // 4: numbers + booleans + null
  {
    const obj = YAML.parse("a: 1\nb: 3.14\nc: true\nd: null\ne: ~");
    lines.push("4 a=" + obj.a + " b=" + obj.b + " c=" + obj.c + " d=" + obj.d + " e=" + obj.e);
  }

  // 5: invalid YAML throws
  {
    try { YAML.parse("a: : invalid"); lines.push("5 NOT_THROWN"); }
    catch (e) { lines.push("5 threw=" + (e instanceof Error)); }
  }
}

try {
  await main();
  process.stdout.write(lines.join("\n") + "\n");
} catch (e) {
  process.stdout.write("FATAL " + e.constructor.name + ": " + e.message + "\n");
}
