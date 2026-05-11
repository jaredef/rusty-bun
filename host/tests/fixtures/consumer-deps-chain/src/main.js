// Tier-J consumer #22: vendored package-with-dependency chain (M9.bis).
//
// The structurally novel verification: colorize internally imports
// ansi-styles. When the loader resolves the `ansi-styles` bare specifier
// from inside node_modules/colorize/lib/, it must walk UP the directory
// tree past colorize's own (absent) node_modules to find the fixture's
// top-level node_modules/ansi-styles.
//
// This tests transitive bare-specifier resolution — the standard npm-flat-
// layout convention. Prior vendored fixtures (clsx, mri-shape, dotenv-
// shape) were single-package (no transitive deps).

import { wrap, bold, compose } from "colorize";
import styles from "ansi-styles";

async function selfTest() {
    const results = [];

    // 1. colorize.wrap uses ansi-styles internally — verify the codes are
    // the canonical ANSI codes (single-quote-delimited for the escape).
    const red = wrap("hello", "red");
    results.push(["wrap-red",
        red === "[31mhello[39m"]);

    // 2. Unknown color is a no-op pass-through.
    results.push(["wrap-unknown",
        wrap("plain", "fuchsia") === "plain"]);

    // 3. compose stacks multiple styles.
    const styled = compose("hi", "bold", "underline", "blue");
    // bold-then-underline-then-blue applied right-to-left in compose loop.
    const expected =
        "[34m" +    // blue open (outermost-applied last)
        "[4m" +     // underline open
        "[1m" +     // bold open (innermost-applied first)
        "hi" +
        "[22m" +    // bold close
        "[24m" +    // underline close
        "[39m";     // blue close
    results.push(["compose-stack", styled === expected]);

    // 4. Direct ansi-styles import from main.js works alongside transitive.
    results.push(["direct-import",
        styles.fg.red.open === "[31m" &&
        styles.fg.red.close === "[39m"]);

    // 5. The colorize package's bold helper produces the same code as
    // direct access through styles — confirms internal import resolved
    // to the same module instance (module-cache identity).
    const boldA = bold("x");
    const boldB = styles.bold.open + "x" + styles.bold.close;
    results.push(["module-cache-identity", boldA === boldB]);

    // 6. styles re-exported from colorize equals the directly-imported
    // ansi-styles — sharper module-cache-identity test.
    // (colorize re-exports styles via `export { styles }`.)
    const { styles: colorizeStyles } = await import("colorize");
    results.push(["reexported-styles-identity",
        colorizeStyles === styles]);

    // 7. ANSI ESC byte (0x1b) is preserved literally through the import
    // chain — no Unicode normalization, no escape-rewriting.
    const escByte = "";
    results.push(["esc-byte-preserved",
        red.charCodeAt(0) === 0x1b &&
        red.charCodeAt(0) === escByte.charCodeAt(0)]);

    // 8. The chain works for fg colors not used internally by colorize.
    results.push(["fg-magenta",
        styles.fg.magenta.open === "[35m"]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
