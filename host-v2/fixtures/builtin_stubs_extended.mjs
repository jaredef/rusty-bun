// Tier-Ω.5.s: smoke fixture for the five new node-builtin stubs.
import a from "node:assert";
import https from "node:https";
import s from "node:stream";
import url from "node:url";
import util from "node:util";

// node:assert — ok(true) returns undefined.
a.ok(true);
console.log("assert.ok(true) ok");

// node:https — request is a function (stub, but typeof check works).
console.log("https.request typeof:", typeof https.request);

// node:stream — Readable is a function.
console.log("stream.Readable typeof:", typeof s.Readable);

// node:url — fileURLToPath round-trips.
console.log("fileURLToPath:", url.fileURLToPath("file:///tmp/x"));

// node:util — format does %s substitution.
console.log("util.format:", util.format("hi %s", "world"));
