// jose v5 production JWT differential. Uses HS256 (HMAC-SHA-256) which
// rusty-bun-host's crypto.subtle has supported since the E.8 partial
// closure on 2026-05-11.

import { SignJWT, jwtVerify, errors } from "jose";

const secret = new TextEncoder().encode("a-strong-secret-32-bytes-long!!");
const lines = [];

async function main() {
// 1: sign + verify round-trip
{
  const jwt = await new SignJWT({ sub: "user-1", scope: "read" })
    .setProtectedHeader({ alg: "HS256", typ: "JWT" })
    .setIssuedAt(1700000000)
    .setExpirationTime(9999999999)
    .setIssuer("https://example.test")
    .sign(secret);
  const parts = jwt.split(".");
  lines.push("1 parts=" + parts.length);
  const { payload, protectedHeader } = await jwtVerify(jwt, secret, {
    issuer: "https://example.test",
  });
  lines.push("2 alg=" + protectedHeader.alg + " sub=" + payload.sub +
             " scope=" + payload.scope + " iat=" + payload.iat +
             " exp=" + payload.exp);
}

// 3: tampered payload -> verification fails
{
  const jwt = await new SignJWT({ admin: false })
    .setProtectedHeader({ alg: "HS256" })
    .setIssuedAt(1700000000)
    .setExpirationTime(9999999999)
    .sign(secret);
  const parts = jwt.split(".");
  // Flip a payload character: change "false" -> "true!" same length.
  const decoded = atob(parts[1].replace(/-/g, "+").replace(/_/g, "/").padEnd(parts[1].length + (4 - parts[1].length % 4) % 4, "="));
  const tampered = decoded.replace('"admin":false', '"admin":truee');
  const b64u = btoa(tampered).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  const bad = parts[0] + "." + b64u + "." + parts[2];
  try {
    await jwtVerify(bad, secret);
    lines.push("3 NOT_REJECTED");
  } catch (e) {
    lines.push("3 rejected " + (e instanceof errors.JWSSignatureVerificationFailed ? "SigFail" : "Other"));
  }
}

// 4: wrong secret -> verification fails
{
  const jwt = await new SignJWT({ x: 1 })
    .setProtectedHeader({ alg: "HS256" })
    .setIssuedAt(1700000000)
    .sign(secret);
  try {
    await jwtVerify(jwt, new TextEncoder().encode("wrong-secret-32-bytes-padding!!"));
    lines.push("4 NOT_REJECTED");
  } catch (e) {
    lines.push("4 rejected " + (e instanceof errors.JWSSignatureVerificationFailed ? "SigFail" : "Other"));
  }
}

// 5: malformed token -> failure
{
  try {
    await jwtVerify("not-a-jwt", secret);
    lines.push("5 NOT_REJECTED");
  } catch (e) {
    lines.push("5 rejected " + e.constructor.name);
  }
}

}

try {
  await main();
  process.stdout.write(lines.join("\n") + "\n");
} catch (e) {
  process.stdout.write("FATAL " + e.constructor.name + ": " + e.message + "\n");
}

