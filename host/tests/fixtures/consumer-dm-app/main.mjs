import deepmerge from "deepmerge";
const lines = [];

// 1: basic merge
lines.push("1 " + JSON.stringify(deepmerge({a:1,b:{x:1}}, {b:{y:2}, c:3})));

// 2: array concat (default)
lines.push("2 " + JSON.stringify(deepmerge({arr:[1,2]}, {arr:[3,4]})));

// 3: array overwrite via customizer
const overwrite = (dest, src) => src;
lines.push("3 " + JSON.stringify(deepmerge({arr:[1,2]}, {arr:[3,4]}, { arrayMerge: overwrite })));

// 4: deep nested
lines.push("4 " + JSON.stringify(deepmerge({a:{b:{c:{d:1}}}}, {a:{b:{c:{e:2}}}})));

// 5: all (merge multiple)
lines.push("5 " + JSON.stringify(deepmerge.all([{a:1}, {b:2}, {c:3}, {a:99}])));

process.stdout.write(lines.join("\n") + "\n");
