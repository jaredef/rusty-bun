// mixme ^2 — recursive immutable merge.
import { merge, mutate, snake_case, camelize, clone, is_object_literal } from "mixme";

const lines = [];
lines.push("1 " + JSON.stringify(merge({ a: 1 }, { b: 2 }, { c: 3 })));
lines.push("2 " + JSON.stringify(merge({ x: { y: 1 } }, { x: { z: 2 } })));
lines.push("3 " + JSON.stringify(clone({ a: { b: 1 } })));
lines.push("4 " + is_object_literal({}) + "/" + is_object_literal([]) + "/" + is_object_literal(null));
lines.push("5 " + JSON.stringify(snake_case({ aKey: 1, bNestedKey: { cKey: 2 } })));
lines.push("6 " + JSON.stringify(camelize({ a_key: 1, b_nested_key: 2 })));

process.stdout.write(lines.join("\n") + "\n");
