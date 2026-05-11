// Tier-J consumer #19: data validator (M9.bis-native).
//
// In-basin axes not yet exercised:
//   - Proxy with multiple traps: set, has, deleteProperty, defineProperty
//   - Reflect.construct for dynamic instantiation
//   - Reflect.apply for explicit function invocation
//   - Number.parseInt with radix; Number.parseFloat
//   - String.fromCharCode / String.fromCodePoint
//   - String.prototype.codePointAt
//   - String.prototype.normalize (Unicode normalization)
//   - Date.parse + toISOString roundtrip
//   - Array.prototype.copyWithin

class ValidatorError extends Error {
    constructor(field, message) {
        super(`${field}: ${message}`);
        this.field = field;
        this.name = "ValidatorError";
    }
}

class NumberRule {
    constructor(min, max) {
        this.min = min;
        this.max = max;
    }
    check(value) {
        if (typeof value !== "number" || !Number.isFinite(value)) {
            throw new ValidatorError("number", "not a finite number");
        }
        if (value < this.min || value > this.max) {
            throw new ValidatorError("number", `out of range [${this.min}, ${this.max}]`);
        }
        return value;
    }
}

class StringRule {
    constructor(minLen) { this.minLen = minLen; }
    check(value) {
        if (typeof value !== "string") {
            throw new ValidatorError("string", "not a string");
        }
        if (value.length < this.minLen) {
            throw new ValidatorError("string", `length < ${this.minLen}`);
        }
        return value;
    }
}

// Proxy with multi-trap. Records all access/mutation events for inspection.
function makeAuditedTarget(initial) {
    const events = [];
    const proxy = new Proxy(initial, {
        get(t, key) {
            if (key === "__events") return events;
            events.push({ op: "get", key });
            return Reflect.get(t, key);
        },
        set(t, key, value) {
            events.push({ op: "set", key, value });
            return Reflect.set(t, key, value);
        },
        has(t, key) {
            events.push({ op: "has", key });
            return Reflect.has(t, key);
        },
        deleteProperty(t, key) {
            events.push({ op: "delete", key });
            return Reflect.deleteProperty(t, key);
        },
        defineProperty(t, key, desc) {
            events.push({ op: "define", key });
            return Reflect.defineProperty(t, key, desc);
        },
    });
    return proxy;
}

async function selfTest() {
    const results = [];

    // 1. Reflect.construct — dynamic class instantiation.
    const rules = { number: NumberRule, string: StringRule };
    const numRule = Reflect.construct(rules.number, [0, 100]);
    const strRule = Reflect.construct(rules.string, [3]);
    results.push(["reflect-construct",
        numRule instanceof NumberRule && strRule instanceof StringRule &&
        numRule.min === 0 && strRule.minLen === 3]);

    // 2. Reflect.apply — explicit function invocation with this/args.
    const result = Reflect.apply(numRule.check, numRule, [42]);
    results.push(["reflect-apply", result === 42]);

    // 3. Custom error subclass round-trips through validator.
    let caught = null;
    try { numRule.check(999); } catch (e) { caught = e; }
    results.push(["custom-error-subclass",
        caught instanceof ValidatorError &&
        caught.field === "number" &&
        caught.name === "ValidatorError"]);

    // 4. Proxy multi-trap: set, get, has, delete, define.
    const t = makeAuditedTarget({ a: 1 });
    t.b = 2;                            // set
    const v = t.a;                      // get
    const has = "b" in t;               // has
    delete t.a;                         // deleteProperty
    Object.defineProperty(t, "c", { value: 3, enumerable: true });  // defineProperty
    const events = t.__events;          // get
    results.push(["proxy-multi-trap",
        v === 1 && has === true &&
        events.some((e) => e.op === "set" && e.key === "b") &&
        events.some((e) => e.op === "delete" && e.key === "a") &&
        events.some((e) => e.op === "define" && e.key === "c") &&
        events.some((e) => e.op === "has" && e.key === "b")]);

    // 5. Number.parseInt with radix.
    results.push(["parseInt-radix",
        Number.parseInt("ff", 16) === 255 &&
        Number.parseInt("1010", 2) === 10 &&
        Number.parseInt("777", 8) === 511]);

    // 6. Number.parseFloat with scientific notation.
    results.push(["parseFloat-scientific",
        Number.parseFloat("3.14") === 3.14 &&
        Number.parseFloat("1.5e3") === 1500 &&
        Number.parseFloat("0.1") === 0.1]);

    // 7. String.fromCharCode + .codePointAt for ASCII roundtrip.
    const ch = String.fromCharCode(65, 66, 67);
    results.push(["fromCharCode-codePointAt",
        ch === "ABC" &&
        "A".codePointAt(0) === 65 &&
        "B".codePointAt(0) === 66]);

    // 8. String.fromCodePoint + codePointAt for supplementary plane.
    const emoji = String.fromCodePoint(0x1F600);
    results.push(["supplementary-codepoint",
        emoji === "😀" &&
        emoji.codePointAt(0) === 0x1F600]);

    // 9. String.prototype.normalize — NFC for combining marks.
    const composed = "é";              // single code-point
    const decomposed = "é";      // e + combining acute
    results.push(["string-normalize",
        composed !== decomposed &&
        composed.normalize("NFC") === decomposed.normalize("NFC")]);

    // 10. Date.parse + toISOString roundtrip.
    const isoIn = "2024-01-15T10:23:45.000Z";
    const ms = Date.parse(isoIn);
    const isoOut = new Date(ms).toISOString();
    results.push(["date-iso-roundtrip", isoIn === isoOut]);

    // 11. Array.prototype.copyWithin — in-place internal copy.
    const arr = [1, 2, 3, 4, 5];
    arr.copyWithin(0, 3, 5);  // copy indices 3..5 to position 0
    results.push(["array-copyWithin",
        JSON.stringify(arr) === "[4,5,3,4,5]"]);

    // 12. Composition: validate a record via Reflect.apply over rule map.
    const record = { name: "Alice", age: 30 };
    const ruleMap = { name: strRule, age: numRule };
    let allValid = true;
    for (const field of Object.keys(record)) {
        try {
            Reflect.apply(ruleMap[field].check, ruleMap[field], [record[field]]);
        } catch (e) {
            allValid = false;
            break;
        }
    }
    results.push(["composition-validate", allValid]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
