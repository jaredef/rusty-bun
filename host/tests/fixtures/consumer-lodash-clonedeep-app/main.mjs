import cloneDeep from "lodash.clonedeep";
const a = { x: [1, 2, { y: 3 }] };
const b = cloneDeep(a);
b.x[2].y = 99;
process.stdout.write(JSON.stringify({ original: a.x[2].y, clone: b.x[2].y }) + "\n");
