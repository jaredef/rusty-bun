// Tier-J consumer #53: DNS resolution composition.
//
// Tier-Π1.2 round. Validates Bun.dns + node:dns + dns/promises wired
// against std::net::ToSocketAddrs (sync, Tier-3 divergence from Bun's
// async c-ares — recorded per seed C1 three-tier authority taxonomy).
//
// Strategy: use hostnames that resolve identically under both
// implementations to avoid c-ares-cache-dependence. "localhost" is the
// canonical stable choice (always 127.0.0.1 / ::1 per RFC 6761). For
// NXDOMAIN, use the reserved ".invalid" TLD (RFC 2606) which is
// guaranteed to never resolve.

import nodeDns from "node:dns";
import { lookup as dnsLookup, resolve4 as dnsResolve4 } from "node:dns/promises";

async function selfTest() {
    const results = [];

    // 1. Bun.dns.lookup("localhost") returns array with at least one entry.
    const bunResult = await Bun.dns.lookup("localhost");
    results.push(["bun-dns-lookup-localhost",
        Array.isArray(bunResult) && bunResult.length >= 1 &&
        typeof bunResult[0].address === "string" &&
        (bunResult[0].family === 4 || bunResult[0].family === 6)]);

    // 2. Bun.dns.lookup with family:4 returns only IPv4.
    const v4Only = await Bun.dns.lookup("localhost", { family: 4 });
    results.push(["bun-dns-lookup-family-4",
        Array.isArray(v4Only) && v4Only.length >= 1 &&
        v4Only.every(r => r.family === 4) &&
        v4Only[0].address === "127.0.0.1"]);

    // 3. node:dns.lookup callback form (default: first address + family).
    const cbResult = await new Promise((resolve, reject) => {
        nodeDns.lookup("localhost", (err, address, family) => {
            if (err) reject(err);
            else resolve({ address, family });
        });
    });
    results.push(["node-dns-lookup-callback",
        typeof cbResult.address === "string" &&
        (cbResult.family === 4 || cbResult.family === 6)]);

    // 4. node:dns.lookup with {all: true} returns array.
    const allResult = await new Promise((resolve, reject) => {
        nodeDns.lookup("localhost", { all: true }, (err, addrs) => {
            if (err) reject(err);
            else resolve(addrs);
        });
    });
    results.push(["node-dns-lookup-all",
        Array.isArray(allResult) && allResult.length >= 1 &&
        allResult.every(a => typeof a.address === "string" && typeof a.family === "number")]);

    // 5. node:dns/promises.lookup returns Promise<{address, family}>.
    const promResult = await dnsLookup("localhost");
    results.push(["node-dns-promises-lookup",
        typeof promResult.address === "string" &&
        (promResult.family === 4 || promResult.family === 6)]);

    // 6. node:dns/promises.resolve4 returns array of IPv4 strings.
    const v4Arr = await dnsResolve4("localhost");
    results.push(["node-dns-promises-resolve4",
        Array.isArray(v4Arr) && v4Arr.length >= 1 &&
        v4Arr.includes("127.0.0.1")]);

    // 7. NXDOMAIN: .invalid is reserved (RFC 2606) and must fail to resolve.
    let nxThrown = false;
    try {
        await Bun.dns.lookup("nonexistent-host-12345.invalid");
    } catch (e) {
        nxThrown = true;
    }
    results.push(["bun-dns-nxdomain-throws", nxThrown]);

    // 8. node:dns.lookup ENOTFOUND error code on NXDOMAIN.
    const nodeErr = await new Promise((resolve) => {
        nodeDns.lookup("nonexistent-host-12345.invalid", (err) => {
            resolve(err);
        });
    });
    results.push(["node-dns-enotfound-code",
        nodeErr != null && nodeErr.code === "ENOTFOUND"]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
