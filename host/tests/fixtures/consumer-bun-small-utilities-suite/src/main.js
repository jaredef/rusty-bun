// Tier-J consumer #61: Bun namespace small utilities. Tier-Π4 round.

async function selfTest() {
    const results = [];

    // 1. Bun.deepEquals.
    results.push(["bun-deepequals",
        Bun.deepEquals({ a: [1, 2] }, { a: [1, 2] }) === true &&
        Bun.deepEquals({ a: 1 }, { a: 2 }) === false]);

    // 2. Bun.inspect renders objects.
    const ins = Bun.inspect({ x: 1 });
    results.push(["bun-inspect", typeof ins === "string" && ins.length > 0]);

    // 3. Bun.escapeHTML.
    results.push(["bun-escapehtml",
        Bun.escapeHTML("<script>x</script>") === "&lt;script&gt;x&lt;/script&gt;"]);

    // 4. Bun.fileURLToPath / Bun.pathToFileURL round-trip.
    const u = Bun.pathToFileURL("/tmp/x");
    const p = Bun.fileURLToPath(u);
    results.push(["bun-fileurl-roundtrip", u.protocol === "file:" && p === "/tmp/x"]);

    // 5. Bun.CryptoHasher SHA-256.
    const h = new Bun.CryptoHasher("sha256");
    h.update("abc");
    const hex = await h.digest("hex");
    // SHA-256("abc") = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
    results.push(["bun-cryptohasher-sha256",
        hex === "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"]);

    // 6. Bun.Glob match.
    const g = new Bun.Glob("src/**/*.ts");
    results.push(["bun-glob-match",
        g.match("src/lib/foo.ts") === true &&
        g.match("src/main.ts") === true &&
        g.match("README.md") === false]);

    // 7. Bun.sleep resolves after a delay.
    const start = Date.now();
    await Bun.sleep(1);
    const elapsed = Date.now() - start;
    results.push(["bun-sleep", elapsed >= 0]);

    // 8. Bun.nanoseconds returns a number monotonically increasing.
    const n1 = Bun.nanoseconds();
    const n2 = Bun.nanoseconds();
    results.push(["bun-nanoseconds",
        typeof n1 === "number" && typeof n2 === "number" && n2 >= n1]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
