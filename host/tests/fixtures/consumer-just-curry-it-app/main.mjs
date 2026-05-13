import curry from "just-curry-it";
const add = curry((a, b, c) => a + b + c);
process.stdout.write(JSON.stringify({ result: add(1)(2)(3) }) + "\n");
