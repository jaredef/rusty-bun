// tslib ^2 — TypeScript compiler runtime helpers. Distinct axis: helper
// fns for emit-down-leveling (__extends, __assign, __awaiter, etc.).
import * as t from "tslib";

const lines = [];

// __extends
function Base() {}
Base.prototype.hello = function () { return "base"; };
function Derived() { Base.call(this); }
t.__extends(Derived, Base);
lines.push("1 ext=" + (new Derived()).hello());

// __assign (Object.assign-equivalent)
lines.push("2 " + JSON.stringify(t.__assign({ a: 1 }, { b: 2 }, { c: 3 })));

// __rest
const { a, ...rest } = { a: 1, b: 2, c: 3 };
lines.push("3 rest=" + JSON.stringify(rest));

// __spreadArray
lines.push("4 " + JSON.stringify(t.__spreadArray([1, 2], [3, 4], false)));

// __values + __read
const it = t.__values([10, 20, 30]);
const r = [];
let v;
while (!(v = it.next()).done) r.push(v.value);
lines.push("5 vals=" + JSON.stringify(r));

// __makeTemplateObject
const tmpl = t.__makeTemplateObject(["a", "b"], ["a", "b"]);
lines.push("6 tmpl=" + (Array.isArray(tmpl) && tmpl.raw && tmpl.raw.length === 2));

process.stdout.write(lines.join("\n") + "\n");
