// deep-freeze-es6 ^5 — recursive Object.freeze.
import deepFreeze from "deep-freeze-es6";

const lines = [];
const o = { a: 1, nested: { b: 2, arr: [1, 2, 3] } };
deepFreeze(o);
lines.push("1 frozen=" + Object.isFrozen(o));
lines.push("2 nestedFrozen=" + Object.isFrozen(o.nested));
lines.push("3 arrFrozen=" + Object.isFrozen(o.nested.arr));

let err1 = null;
try { o.a = 99; } catch (e) { err1 = e.constructor.name; }
lines.push("4 topErr=" + err1 + " val=" + o.a);

let err2 = null;
try { o.nested.b = 99; } catch (e) { err2 = e.constructor.name; }
lines.push("5 nestedErr=" + err2 + " val=" + o.nested.b);

const p = deepFreeze({ x: { y: { z: 1 } } });
lines.push("6 deepDeep=" + Object.isFrozen(p.x.y));

process.stdout.write(lines.join("\n") + "\n");
