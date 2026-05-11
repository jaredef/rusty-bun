// Tier-J consumer #21: env-file loader using vendored dotenv-shape (M9.bis).
//
// Library uses `import fs from "node:fs"` internally — tests that
// node:fs builtin resolution works from inside a third-party-package
// context, not just from the engagement's own fixtures.
//
// Bun-portable: real Bun executes the vendored library identically.

import dotenv, { parse, config } from "dotenv";
import fs from "node:fs";

// Write a small .env file to a deterministic temp path the fixture
// owns, exercise the library, then clean up.
const ENV_PATH = "/tmp/rusty-bun-fixture-env-" + process.argv[1].split("/").pop();

const ENV_CONTENT = `# A comment
NAME=Alice
PORT=8080
GREETING="hello, world!"
ESCAPED="line1\\nline2"
SINGLE='no escapes \\n here'
EMPTY=
WITH_EQUALS=key=value
export EXPORTED=from-shell
`;

async function selfTest() {
    const results = [];

    // Setup: write the file.
    fs.writeFileSync(ENV_PATH, ENV_CONTENT);

    // 1. parse() directly on the source content.
    const parsed = parse(ENV_CONTENT);
    results.push(["parse-direct",
        parsed.NAME === "Alice" &&
        parsed.PORT === "8080" &&
        parsed.GREETING === "hello, world!"]);

    // 2. Double-quoted strings interpret \n escapes.
    results.push(["parse-double-quote-escape",
        parsed.ESCAPED === "line1\nline2"]);

    // 3. Single-quoted strings preserve content literally (no \n interpretation).
    results.push(["parse-single-quote-literal",
        parsed.SINGLE === "no escapes \\n here"]);

    // 4. Empty values.
    results.push(["parse-empty", parsed.EMPTY === ""]);

    // 5. Values containing `=` (only first `=` separates key from value).
    results.push(["parse-equals-in-value", parsed.WITH_EQUALS === "key=value"]);

    // 6. `export FOO=...` shell-style prefix is stripped.
    results.push(["parse-export-prefix", parsed.EXPORTED === "from-shell"]);

    // 7. Comments and blank lines are ignored.
    results.push(["parse-comments-ignored",
        !("# A comment" in parsed) &&
        Object.keys(parsed).length === 8]);

    // 8. config() reads from file path; library uses node:fs internally.
    const { parsed: fromFile, error } = config({ path: ENV_PATH });
    results.push(["config-file-read",
        !error &&
        fromFile.NAME === "Alice" &&
        fromFile.PORT === "8080"]);

    // 9. config() returns error for missing file (graceful, not thrown).
    const missing = config({ path: ENV_PATH + ".doesnotexist" });
    results.push(["config-missing-file",
        missing.error instanceof Error &&
        Object.keys(missing.parsed).length === 0]);

    // 10. Default export shape (real dotenv has both default and named).
    results.push(["default-export-shape",
        typeof dotenv === "object" &&
        typeof dotenv.config === "function" &&
        typeof dotenv.parse === "function"]);

    // Cleanup.
    fs.unlinkSync(ENV_PATH);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
