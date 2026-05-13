// Π2.6.eval-ESM: dynamic import of ESM source via data: URL.
//
// The prettier-class consumer pattern: bundled plugin source is
// loaded as a string, then evaluated via
// `import('data:text/javascript;base64,...')`. The data: URL goes
// through the resolver as-is and through the loader as inline ESM
// source. Top-level imports + exports work.
//
// Fixture exercises the full pattern: a string of ESM source with
// named exports, a default export, and a function call, evaluated
// via the data: URL trick.

const pluginSource = `
export const name = "test-plugin";
export const version = "1.0.0";
export function transform(x) { return x.toUpperCase(); }
export default { name, version, transform };
`.trim();

const b64 = Buffer.from(pluginSource).toString("base64");
const dataUrl = "data:text/javascript;base64," + b64;
const plugin = await import(dataUrl);

const result = plugin.transform("hello");
const defaultName = plugin.default.name;

process.stdout.write(JSON.stringify({
  name: plugin.name,
  version: plugin.version,
  transformed: result,
  defaultName,
  defaultIsObject: typeof plugin.default === "object",
}) + "\n");
