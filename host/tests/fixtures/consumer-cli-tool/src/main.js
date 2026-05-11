// Tier-J consumer #17: CLI-tool shape exercising process.* surface.
//
// Phase-2-extension round: process was a known basin boundary closed by
// wiring in this same commit. This fixture verifies the wiring lands a
// J.1.a fixture on the newly-in-basin axis.
//
// Bun-portable: real Bun has full process; rusty-bun-host now wires
// argv/env/platform/arch/version/versions/cwd/exit/stdout.write/
// stderr.write/hrtime. Tests verify presence + shape, not specific
// values (which differ between runtimes).

import process from "node:process";

async function selfTest() {
    const results = [];

    // 1. process is an object.
    results.push(["process-object", typeof process === "object"]);

    // 2. process.argv is an array of strings.
    results.push(["argv-shape",
        Array.isArray(process.argv) &&
        process.argv.length >= 1 &&
        process.argv.every((a) => typeof a === "string")]);

    // 3. process.env is an object; PATH is a string.
    results.push(["env-shape",
        typeof process.env === "object" &&
        typeof process.env.PATH === "string"]);

    // 4. process.platform is one of the known platforms.
    const knownPlatforms = ["linux", "darwin", "win32", "freebsd", "openbsd"];
    results.push(["platform-known",
        knownPlatforms.includes(process.platform)]);

    // 5. process.arch is one of the known archs.
    const knownArchs = ["x64", "arm64", "arm", "ia32"];
    results.push(["arch-known", knownArchs.includes(process.arch)]);

    // 6. process.version is a string.
    results.push(["version-string", typeof process.version === "string"]);

    // 7. process.cwd() returns an absolute path string.
    const cwd = process.cwd();
    results.push(["cwd-absolute",
        typeof cwd === "string" && cwd.startsWith("/")]);

    // 8. process.versions is an object with at least one key.
    results.push(["versions-object",
        typeof process.versions === "object" &&
        Object.keys(process.versions).length >= 1]);

    // 9. process.hrtime() returns [seconds, nanoseconds].
    const hr = process.hrtime();
    results.push(["hrtime-tuple",
        Array.isArray(hr) && hr.length === 2 &&
        Number.isInteger(hr[0]) && Number.isInteger(hr[1])]);

    // 10. process.hrtime.bigint() returns a BigInt.
    const hrb = process.hrtime.bigint();
    results.push(["hrtime-bigint", typeof hrb === "bigint"]);

    // 11. process.stdout.write returns true (per Node spec).
    const writeResult = process.stdout.write("");  // empty write
    results.push(["stdout-write-result", writeResult === true]);

    // 12. process.env round-trip via PATH containing forward slash.
    results.push(["env-path-contents",
        process.env.PATH.length > 0 && process.env.PATH.includes("/")]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
