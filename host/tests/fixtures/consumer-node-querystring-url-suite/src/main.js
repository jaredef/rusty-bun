// Tier-J consumer #60: node:querystring + node:url full.
// Tier-Π3.11 closure round.

import * as qs from "node:querystring";
import * as urlMod from "node:url";

async function selfTest() {
    const results = [];

    // 1. querystring.parse basic.
    const p1 = qs.parse("a=1&b=2&c=3");
    results.push(["qs-parse-basic", p1.a === "1" && p1.b === "2" && p1.c === "3"]);

    // 2. querystring.parse multi-value.
    const p2 = qs.parse("k=1&k=2&k=3");
    results.push(["qs-parse-multi-value",
        Array.isArray(p2.k) && p2.k.length === 3 && p2.k[0] === "1" && p2.k[2] === "3"]);

    // 3. querystring.parse plus-as-space decoding.
    const p3 = qs.parse("greet=hello+world");
    results.push(["qs-parse-plus-as-space", p3.greet === "hello world"]);

    // 4. querystring.stringify basic + array.
    const s4 = qs.stringify({ a: "1", b: ["x", "y"] });
    results.push(["qs-stringify-basic",
        s4 === "a=1&b=x&b=y"]);

    // 5. url.parse returns legacy shape with protocol/host/pathname/search.
    const u5 = urlMod.parse("https://user@example.com:8080/api/v1?q=1#frag", true);
    results.push(["url-parse-legacy",
        u5.protocol === "https:" &&
        u5.hostname === "example.com" &&
        u5.port === "8080" &&
        u5.pathname === "/api/v1" &&
        u5.search === "?q=1" &&
        u5.hash === "#frag" &&
        typeof u5.query === "object" && u5.query.q === "1"]);

    // 6. url.format roundtrips with the parsed object's most relevant fields.
    const f6 = urlMod.format({
        protocol: "http:",
        hostname: "host.example",
        port: 1234,
        pathname: "/x",
        search: "?a=1",
    });
    results.push(["url-format-basic", f6 === "http://host.example:1234/x?a=1"]);

    // 7. url.fileURLToPath round-trips with pathToFileURL.
    const file = urlMod.pathToFileURL("/tmp/example.txt");
    const path = urlMod.fileURLToPath(file);
    results.push(["url-file-roundtrip",
        file.protocol === "file:" && path === "/tmp/example.txt"]);

    // 8. url.resolve relative-to-base.
    const r8 = urlMod.resolve("http://example.com/dir/a", "b");
    results.push(["url-resolve", r8 === "http://example.com/dir/b"]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
