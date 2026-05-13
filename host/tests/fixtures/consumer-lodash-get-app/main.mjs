import get from "lodash.get";
process.stdout.write(JSON.stringify({ r: get({a:{b:{c:42}}}, "a.b.c") }) + "\n");
