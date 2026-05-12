// HMAC: generate → sign → verify
const hmacKey = await crypto.subtle.generateKey(
  { name: "HMAC", hash: "SHA-256" }, true, ["sign", "verify"]);
const msg = new TextEncoder().encode("hello");
const sig = await crypto.subtle.sign({ name: "HMAC" }, hmacKey, msg);
const ok = await crypto.subtle.verify({ name: "HMAC" }, hmacKey, sig, msg);

// AES-GCM: generate → encrypt → decrypt
const aesKey = await crypto.subtle.generateKey(
  { name: "AES-GCM", length: 256 }, true, ["encrypt", "decrypt"]);
const iv = crypto.getRandomValues(new Uint8Array(12));
const pt = new TextEncoder().encode("secret");
const ct = await crypto.subtle.encrypt({ name: "AES-GCM", iv }, aesKey, pt);
const dt = await crypto.subtle.decrypt({ name: "AES-GCM", iv }, aesKey, ct);
const aesRound = new TextDecoder().decode(dt) === "secret";

// Export the HMAC key as jwk
const jwk = await crypto.subtle.exportKey("jwk", hmacKey);

process.stdout.write(JSON.stringify({
  hmacVerified: ok,
  aesRound,
  jwkKty: jwk.kty,
  jwkAlg: jwk.alg,
  hasJwkK: typeof jwk.k === "string",
}) + "\n");
