// ECDSA: generate → sign → verify. Slow on Pi; ignored by default.
const ec = await crypto.subtle.generateKey(
  { name: "ECDSA", namedCurve: "P-256" }, true, ["sign", "verify"]);
const msg = new TextEncoder().encode("hello");
const sig = await crypto.subtle.sign(
  { name: "ECDSA", hash: "SHA-256" }, ec.privateKey, msg);
const ok = await crypto.subtle.verify(
  { name: "ECDSA", hash: "SHA-256" }, ec.publicKey, sig, msg);

process.stdout.write(JSON.stringify({
  verified: ok,
  privType: ec.privateKey.type,
  pubType: ec.publicKey.type,
  curve: ec.privateKey.algorithm.namedCurve,
}) + "\n");
