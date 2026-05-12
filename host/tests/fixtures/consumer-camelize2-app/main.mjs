import camelcaseKeys from "camelcase-keys";
const lines = [];
lines.push("1 " + JSON.stringify(camelcaseKeys({ foo_bar: 1, baz_qux: 2 })));
lines.push("2 " + JSON.stringify(camelcaseKeys({ a_b: { c_d: 1 } })));
lines.push("3 " + JSON.stringify(camelcaseKeys({ a_b: { c_d: 1 } }, { deep: true })));
lines.push("4 " + JSON.stringify(camelcaseKeys([{ a_b: 1 }, { c_d: 2 }])));
lines.push("5 " + JSON.stringify(camelcaseKeys({ HelloWorld: 1, foo_bar: 2 }, { pascalCase: true })));
lines.push("6 " + JSON.stringify(camelcaseKeys({ a_b: 1 }, { exclude: ["a_b"] })));
process.stdout.write(lines.join("\n") + "\n");
