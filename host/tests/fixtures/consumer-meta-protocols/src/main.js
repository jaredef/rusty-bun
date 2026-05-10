// Tier-J consumer #12: meta-protocols exercise (spec-first M9).
//
// Tests in-basin axes not yet covered:
//   - Symbol.hasInstance (custom instanceof predicate)
//   - AsyncGenerator's .throw() protocol
//   - structuredClone on graphs with circular references
//   - Regex sticky flag /y with .lastIndex anchoring
//   - String.prototype.replaceAll
//   - JSON.parse with reviver function
//   - Array.prototype.flat / flatMap composition with depth
//   - Symbol.iterator on plain object (manual iterable protocol)

async function selfTest() {
    const results = [];

    // 1. Symbol.hasInstance — custom instanceof check.
    class EvenChecker {
        static [Symbol.hasInstance](x) {
            return typeof x === "number" && x % 2 === 0;
        }
    }
    results.push(["symbol-hasInstance",
        (4 instanceof EvenChecker) === true &&
        (5 instanceof EvenChecker) === false &&
        ("foo" instanceof EvenChecker) === false]);

    // 2. AsyncGenerator .throw() — error injection into generator.
    async function* counterGen() {
        let i = 0;
        try {
            while (true) {
                yield i++;
            }
        } catch (e) {
            yield "caught:" + e.message;
        }
    }
    const gen = counterGen();
    const r1 = await gen.next();         // {value: 0, done: false}
    const r2 = await gen.next();         // {value: 1, done: false}
    const r3 = await gen.throw(new Error("stop"));  // caught → "caught:stop"
    results.push(["async-generator-throw",
        r1.value === 0 && r2.value === 1 && r3.value === "caught:stop"]);

    // 3. structuredClone on graph with circular references.
    const a = { name: "node-a" };
    a.self = a;          // direct cycle
    a.indirect = { back: a };  // indirect cycle through inner object
    const cloned = structuredClone(a);
    results.push(["structuredClone-circular",
        cloned !== a &&
        cloned.self === cloned &&
        cloned.indirect.back === cloned &&
        cloned.name === "node-a"]);

    // 4. Regex sticky /y flag — anchored at .lastIndex.
    const re = /\w+/y;
    re.lastIndex = 4;
    const m1 = re.exec("foo bar baz");
    // Sticky regex matches only at lastIndex; "bar" starts at 4, matches.
    results.push(["regex-sticky",
        m1 !== null && m1[0] === "bar" && re.lastIndex === 7]);

    // 5. String.prototype.replaceAll (string + regex forms).
    const s1 = "a-b-c-d".replaceAll("-", "_");
    const s2 = "foo123bar456".replaceAll(/\d+/g, "#");
    results.push(["replaceAll",
        s1 === "a_b_c_d" && s2 === "foo#bar#"]);

    // 6. JSON.parse with reviver — transform values during parse.
    const reviver = (key, value) => {
        if (key === "secret") return "REDACTED";
        if (typeof value === "number") return value * 2;
        return value;
    };
    const parsed = JSON.parse('{"a":1,"b":{"c":2},"secret":"shh"}', reviver);
    results.push(["json-reviver",
        parsed.a === 2 && parsed.b.c === 4 && parsed.secret === "REDACTED"]);

    // 7. Array.flat with depth.
    const nested = [1, [2, [3, [4, [5]]]]];
    const flat1 = nested.flat();
    const flat2 = nested.flat(2);
    const flatAll = nested.flat(Infinity);
    results.push(["array-flat-depth",
        JSON.stringify(flat1) === "[1,2,[3,[4,[5]]]]" &&
        JSON.stringify(flat2) === "[1,2,3,[4,[5]]]" &&
        JSON.stringify(flatAll) === "[1,2,3,4,5]"]);

    // 8. Symbol.iterator on plain object — manual iterable protocol.
    const range = {
        from: 1, to: 5,
        [Symbol.iterator]() {
            let cur = this.from;
            const end = this.to;
            return {
                next() {
                    return cur <= end
                        ? { value: cur++, done: false }
                        : { value: undefined, done: true };
                }
            };
        }
    };
    const collected = [...range];
    results.push(["symbol-iterator-plain",
        collected.join(",") === "1,2,3,4,5"]);

    // 9. JSON.stringify with replacer + space + reviver round-trip.
    const orig = { x: 1, y: 2, z: [3, 4] };
    const stringified = JSON.stringify(orig, null, 2);
    // Reviver doubles numbers on parse.
    const dbl = JSON.parse(stringified, (k, v) => typeof v === "number" ? v * 2 : v);
    results.push(["json-roundtrip-reviver",
        dbl.x === 2 && dbl.y === 4 && dbl.z[0] === 6 && dbl.z[1] === 8]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

if (typeof process !== "undefined" && process.stdout && process.stdout.write) {
    process.stdout.write(summary + "\n");
} else {
    globalThis.__esmResult = summary;
}
