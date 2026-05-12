import equal from "fast-deep-equal";

const lines = [];

// 1: primitives
lines.push("1 " + equal(1, 1) + " " + equal("a", "a") + " " + equal(1, "1") + " " + equal(null, undefined));

// 2: arrays
lines.push("2 " + equal([1,2,3], [1,2,3]) + " " + equal([1,2,3], [1,2,4]) + " " + equal([1,[2,3]], [1,[2,3]]));

// 3: objects
lines.push("3 " + equal({a:1,b:2}, {b:2,a:1}) + " " + equal({a:1}, {a:1,b:2}));

// 4: nested
lines.push("4 " + equal({a:{b:{c:[1,2,3]}}}, {a:{b:{c:[1,2,3]}}}) + " " + equal({a:{b:{c:[1,2,3]}}}, {a:{b:{c:[1,2,4]}}}));

// 5: Date / RegExp
lines.push("5 dates=" + equal(new Date(0), new Date(0)) + " regex=" + equal(/abc/g, /abc/g) + " regexDiff=" + equal(/abc/g, /abc/));

// 6: NaN
lines.push("6 nan=" + equal(NaN, NaN));

// 7: typed arrays
lines.push("7 " + equal(new Uint8Array([1,2,3]), new Uint8Array([1,2,3])) + " " + equal(new Uint8Array([1,2,3]), new Uint8Array([1,2,4])));

process.stdout.write(lines.join("\n") + "\n");
