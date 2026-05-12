import dotenv from "dotenv";
import path from "node:path";

const url = import.meta.url;
const __dirname = path.dirname(url.startsWith("file://") ? url.slice(7) : url);
const res = dotenv.config({ path: path.join(__dirname, ".env"), quiet: true });
const parsed = res.parsed || {};

process.stdout.write(JSON.stringify({
  foo: parsed.FOO,
  num: parsed.NUM,
  quoted: parsed.QUOTED,
  hasEmpty: "EMPTY" in parsed,
  multi: parsed.MULTI,
  envFoo: process.env.FOO,
  importMetaUrl: import.meta.url,
}) + "\n");
