// Demonstrates a richer set of wired pilots running through rusty-bun-host.
// Run: ./target/release/rusty-bun-host host/examples/runtime-demo.js

console.log("=== rusty-bun-host runtime demo ===");

// 1. console.log + JS pure-language sanity
console.log("Hello from rusty-bun-host. 1+2 =", 1 + 2);

// 2. atob / btoa
const encoded = btoa("Hello, world!");
console.log("btoa('Hello, world!') =", encoded);
console.log("atob(...)              =", atob(encoded));

// 3. path.*
console.log("path.basename('/usr/local/bin/node') =", path.basename("/usr/local/bin/node"));
console.log("path.normalize('/foo/bar//baz/..')   =", path.normalize("/foo/bar//baz/.."));

// 4. crypto
console.log("crypto.randomUUID()                     =", crypto.randomUUID());
console.log("crypto.subtle.digestSha256Hex('hello')  =", crypto.subtle.digestSha256Hex("hello"));

// 5. TextEncoder / TextDecoder
const enc = new TextEncoder();
const dec = new TextDecoder();
const bytes = enc.encode("héllo, мир! 🌍");
console.log("encode + decode unicode =", dec.decode(bytes));

// 6. Buffer
console.log("Buffer.byteLength('héllo') =", Buffer.byteLength("héllo"));
console.log("Buffer.encodeBase64(Buffer.from('hi')) =", Buffer.encodeBase64(Buffer.from("hi")));

// 7. URLSearchParams
const params = new URLSearchParams("?a=1&b=2&a=3");
params.append("c", "x y");
params.sort();
console.log("URLSearchParams.toString() =", params.toString());
console.log("params.getAll('a')         =", params.getAll("a"));

// 8. fs round-trip
const tmp = "/tmp/rusty-bun-host-demo-" + crypto.randomUUID();
fs.writeFileSync(tmp, "demo content");
console.log("fs.existsSync(tmp)         =", fs.existsSync(tmp));
console.log("fs.readFileSyncUtf8(tmp)   =", fs.readFileSyncUtf8(tmp));
fs.unlinkSync(tmp);
console.log("after unlink, existsSync   =", fs.existsSync(tmp));

console.log("\nAll wired pilots functional from JS.");
