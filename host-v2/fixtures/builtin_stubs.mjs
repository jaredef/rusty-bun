// Tier-Ω.5.r: smoke fixture for node:http + node:crypto stubs
// (success path — see builtin_stubs_throw.mjs for the http.request
// stub-error path).
import http from "node:http";
import crypto from "node:crypto";

console.log("http keys non-empty:", Object.keys(http).length > 0);
console.log("crypto.createHash typeof:", typeof crypto.createHash);

const uuid = crypto.randomUUID();
console.log("uuid typeof:", typeof uuid);
console.log("uuid sample:", uuid);

console.log("http.request typeof:", typeof http.request);
console.log("status 200:", http.STATUS_CODES["200"]);
console.log("status 404:", http.STATUS_CODES["404"]);
