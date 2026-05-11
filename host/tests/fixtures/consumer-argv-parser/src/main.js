// Tier-J consumer #16: argv parser using vendored mri-shape library.
//
// Bigger than clsx: ~150 LOC ESM with helpers + main API. Same
// principle: library code was not written for this engagement.
//
// Tests exercise mri's documented behaviors:
//   - Simple flag parsing
//   - Aliases (short/long)
//   - Boolean flags
//   - String flags
//   - Defaults
//   - Positional args (_ key)
//   - --no-flag negation
//   - -- separator
//   - = value form

import mri from "mri";

async function selfTest() {
    const results = [];

    // 1. Simple long flags + value.
    const a = mri(["--name", "Alice", "--port", "8080"]);
    results.push(["simple-long",
        a.name === "Alice" && a.port === 8080 && JSON.stringify(a._) === "[]"]);

    // 2. Equals form.
    const b = mri(["--name=Bob", "--port=3000"]);
    results.push(["equals-form",
        b.name === "Bob" && b.port === 3000]);

    // 3. Short flag with value.
    const c = mri(["-n", "Carol"]);
    results.push(["short-flag", c.n === "Carol"]);

    // 4. Aliases.
    const d = mri(["-n", "Dave"], { alias: { n: "name" } });
    results.push(["alias", d.n === "Dave" && d.name === "Dave"]);

    // 5. Boolean flag.
    const e = mri(["--verbose"], { boolean: ["verbose"] });
    results.push(["boolean", e.verbose === true]);

    // 6. --no- negation.
    const f = mri(["--no-color"]);
    results.push(["no-flag", f.color === false]);

    // 7. String flag with default.
    const g = mri(["--quiet"], {
        string: ["mode"],
        default: { mode: "auto" },
    });
    results.push(["default-string",
        g.mode === "auto" && g.quiet === true]);

    // 8. Positional args.
    const h = mri(["build", "src", "dist", "--watch"]);
    results.push(["positionals",
        JSON.stringify(h._) === '["build","src","dist"]' &&
        h.watch === true]);

    // 9. -- separator passes through unparsed.
    const i = mri(["--mode", "fast", "--", "--mode", "slow"]);
    results.push(["dash-dash",
        i.mode === "fast" &&
        JSON.stringify(i._) === '["--mode","slow"]']);

    // 10. Mixed short flags + long + positional + alias.
    const j = mri(
        ["-v", "build", "--output", "dist", "src"],
        { alias: { v: "verbose" }, boolean: ["verbose"] }
    );
    results.push(["mixed",
        j.v === true && j.verbose === true &&
        j.output === "dist" &&
        JSON.stringify(j._) === '["build","src"]']);

    // 11. Repeated flag accumulates into array.
    const k = mri(["--tag", "a", "--tag", "b", "--tag", "c"]);
    results.push(["repeated-flag",
        JSON.stringify(k.tag) === '["a","b","c"]']);

    // 12. Numeric coercion of values that look like numbers.
    const l = mri(["--count", "42", "--name", "alice"]);
    results.push(["numeric-coercion",
        l.count === 42 && typeof l.count === "number" &&
        l.name === "alice" && typeof l.name === "string"]);

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
