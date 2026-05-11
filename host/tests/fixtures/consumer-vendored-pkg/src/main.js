// Tier-J consumer #15: vendored real-shape npm package.
//
// Imports clsx from node_modules (vendored verbatim from
// github.com/lukeed/clsx, MIT). The library code was not written for
// this engagement — it's actual published npm code shape.
//
// Tests verify clsx's README examples run identically on Bun 1.3.11
// and rusty-bun-host. The library uses:
//   - var declarations (not const/let — older style preserved in vendored source)
//   - typeof + Array.isArray for runtime type discrimination
//   - for-in loops over plain objects
//   - Conditional string concatenation (&&)
//   - arguments object iteration
//   - Default export + named export
//
// Tests the module loader's package.json "exports" field with
// conditional resolution (import vs require vs default).

import clsx from "clsx";
// Also test the named-export form.
import { clsx as clsxNamed } from "clsx";

async function selfTest() {
    const results = [];

    // 1. Default import works.
    results.push(["default-import", typeof clsx === "function"]);

    // 2. Named import works.
    results.push(["named-import",
        typeof clsxNamed === "function" && clsxNamed === clsx]);

    // 3. Simple string concat.
    results.push(["strings",
        clsx("foo", "bar") === "foo bar"]);

    // 4. Falsy values are skipped.
    results.push(["falsy-skipped",
        clsx("foo", false, null, undefined, 0, "bar", "") === "foo bar"]);

    // 5. Object form: keys with truthy values are included.
    results.push(["object-form",
        clsx({ foo: true, bar: false, baz: true }) === "foo baz"]);

    // 6. Array form: nested arrays are flattened.
    results.push(["array-form",
        clsx(["foo", "bar", false, "baz"]) === "foo bar baz"]);

    // 7. Deeply nested mix.
    results.push(["nested-mix",
        clsx("a", ["b", { c: true, d: false }, ["e", { f: true }]]) === "a b c e f"]);

    // 8. Numbers are stringified; top-level args are space-joined.
    results.push(["numbers",
        clsx(1, 2, 3) === "1 2 3"]);

    // 9. Real-world usage: conditional UI class composition.
    const isActive = true;
    const isDisabled = false;
    const variant = "primary";
    const cls = clsx(
        "btn",
        `btn-${variant}`,
        { "btn-active": isActive, "btn-disabled": isDisabled },
        isActive && "ring",
    );
    results.push(["conditional-ui",
        cls === "btn btn-primary btn-active ring"]);

    // 10. Empty call returns empty string.
    results.push(["empty-call", clsx() === ""]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
