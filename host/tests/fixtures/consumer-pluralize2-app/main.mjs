// inflection ^3 — Rails-style string inflection (pluralize/singularize/camelize/etc),
// distinct API surface from pluralize.
import * as inf from "inflection";

const lines = [];
lines.push("1 " + inf.pluralize("octopus"));
lines.push("2 " + inf.singularize("children"));
lines.push("3 " + inf.camelize("hello_world"));
lines.push("4 " + inf.underscore("HelloWorld"));
lines.push("5 " + inf.dasherize("hello_world"));
lines.push("6 " + inf.titleize("man_from_the_boondocks"));

process.stdout.write(lines.join("\n") + "\n");
