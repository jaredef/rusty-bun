// tweetnacl ^1 — NaCl crypto port (Ed25519 sig, X25519 box, etc).
// Distinct axis from rusty-web-crypto's RFC algorithms.
import nacl from "tweetnacl";

const lines = [];

// Deterministic seed
const seed = new Uint8Array(32);
for (let i = 0; i < 32; i++) seed[i] = i;

const kp = nacl.sign.keyPair.fromSeed(seed);
const msg = new Uint8Array([1, 2, 3, 4, 5]);
const sig = nacl.sign.detached(msg, kp.secretKey);
lines.push("1 sigLen=" + sig.length);
lines.push("2 pubLen=" + kp.publicKey.length);
lines.push("3 verify=" + nacl.sign.detached.verify(msg, sig, kp.publicKey));

const bad = new Uint8Array(msg);
bad[0] ^= 1;
lines.push("4 badVerify=" + nacl.sign.detached.verify(bad, sig, kp.publicKey));

// Hash
const h = nacl.hash(new Uint8Array([1, 2, 3]));
lines.push("5 hashLen=" + h.length);

// secretbox round-trip
const key = new Uint8Array(32); for (let i = 0; i < 32; i++) key[i] = i * 7 % 256;
const nonce = new Uint8Array(24); for (let i = 0; i < 24; i++) nonce[i] = i;
const ct = nacl.secretbox(msg, nonce, key);
const pt = nacl.secretbox.open(ct, nonce, key);
lines.push("6 rt=" + (pt && pt.length === msg.length && pt[0] === 1));

process.stdout.write(lines.join("\n") + "\n");
