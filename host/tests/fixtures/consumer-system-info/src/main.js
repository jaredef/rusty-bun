// Tier-J consumer #11: system-info reporter (spec-first M9 authoring).
//
// Uses node:os — the apparatus boundary being closed this round.
// Exercises shapes consumer code typically needs from node:os:
//   - os.platform() / os.arch() / os.type() — runtime identification
//   - os.tmpdir() / os.homedir() — well-known paths
//   - os.hostname() — host identity
//   - os.EOL — line-ending constant
//
// Bun-portable: real Bun has full node:os; rusty-bun-host needs the
// wiring this round adds. The fixture's output is identity-stable
// across the two runtimes for the constants we test (platform name,
// architecture string), since both query the same OS underneath.

import os from "node:os";

async function selfTest() {
    const results = [];

    // 1. platform() returns a known string.
    const platform = os.platform();
    results.push(["platform-is-known-string",
        typeof platform === "string" &&
        ["linux", "darwin", "win32", "freebsd", "openbsd", "sunos", "aix"].includes(platform)]);

    // 2. arch() returns a known string.
    const arch = os.arch();
    results.push(["arch-is-known-string",
        typeof arch === "string" &&
        ["x64", "arm64", "arm", "ia32", "mips", "mipsel", "ppc", "ppc64", "s390", "s390x"].includes(arch)]);

    // 3. tmpdir() returns an absolute path string.
    const tmp = os.tmpdir();
    results.push(["tmpdir-is-absolute",
        typeof tmp === "string" && tmp.length > 0 && tmp.startsWith("/")]);

    // 4. homedir() returns an absolute path string (POSIX).
    const home = os.homedir();
    results.push(["homedir-is-absolute",
        typeof home === "string" && home.length > 0 && home.startsWith("/")]);

    // 5. hostname() returns a non-empty string.
    const host = os.hostname();
    results.push(["hostname-nonempty",
        typeof host === "string" && host.length > 0]);

    // 6. EOL is "\n" on POSIX.
    results.push(["eol-posix", os.EOL === "\n"]);

    // 7. type() returns a string (e.g., "Linux", "Darwin").
    const type = os.type();
    results.push(["type-is-string",
        typeof type === "string" && type.length > 0]);

    // 8. Cross-pilot composition: build a report string using template literals
    // and node:os values. Output is deterministic on a given host so a
    // differential between Bun and rusty-bun-host should match.
    const report = `platform=${platform} arch=${arch} EOL-bytes=${os.EOL.length}`;
    results.push(["composition-report",
        report.startsWith("platform=") && report.includes(" arch=") && report.endsWith("=1")]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
