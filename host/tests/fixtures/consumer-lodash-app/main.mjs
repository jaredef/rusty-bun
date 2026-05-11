// lodash ^4 — kitchen-sink utility (~17K LOC of CJS). Tests the
// CJS-in-ESM bridge at production scale. Exercises collection ops,
// functional helpers, object/string utils, and Lodash's chain API.
import _ from "lodash";

const lines = [];

// 1: collection ops
lines.push("1 sum=" + _.sum([1, 2, 3, 4, 5]) +
           " uniq=" + JSON.stringify(_.uniq([1, 2, 2, 3, 3, 4])) +
           " chunk=" + JSON.stringify(_.chunk([1,2,3,4,5,6,7], 3)));

// 2: object utils
lines.push("2 keys=" + JSON.stringify(_.keys({a:1,b:2,c:3})) +
           " pick=" + JSON.stringify(_.pick({a:1,b:2,c:3}, ["a","c"])) +
           " merge=" + JSON.stringify(_.merge({a:{x:1}}, {a:{y:2}})));

// 3: functional
{
  const debounced = _.debounce(() => "x", 0);
  const sq = _.curry((a, b, c) => a + b + c);
  lines.push("3 curry=" + sq(1)(2)(3) +
             " memoize=" + _.memoize((n) => n * 2)(5) +
             " partial=" + _.partial((a,b,c) => a+b+c, 10, 20)(30));
}

// 4: chain API
{
  const r = _.chain([1, 2, 3, 4, 5])
    .filter(n => n % 2 === 1)
    .map(n => n * n)
    .reduce((a, b) => a + b, 0)
    .value();
  lines.push("4 chain=" + r);
}

// 5: deep + isEqual + cloneDeep
{
  const a = { x: { y: [1, 2, 3], z: new Date(0) } };
  const b = _.cloneDeep(a);
  const eq = _.isEqual(a, b);
  b.x.y.push(4);
  const after = _.isEqual(a, b);
  lines.push("5 cloneEq=" + eq + " afterMutEq=" + after);
}

// 6: strings + camelCase + kebabCase
lines.push("6 camel=" + _.camelCase("hello-world foo") +
           " kebab=" + _.kebabCase("helloWorldFoo") +
           " capitalize=" + _.capitalize("HELLO world"));

// 7: range + groupBy
lines.push("7 range=" + JSON.stringify(_.range(1,6)) +
           " groupBy=" + JSON.stringify(_.groupBy([1.2, 1.5, 2.1, 2.5], Math.floor)));

process.stdout.write(lines.join("\n") + "\n");
