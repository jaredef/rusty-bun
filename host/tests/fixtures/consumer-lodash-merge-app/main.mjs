import merge from "lodash.merge";
process.stdout.write(JSON.stringify(merge({a:{b:1}}, {a:{c:2}})) + "\n");
