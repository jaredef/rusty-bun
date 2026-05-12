// bn.js ^5 — bignum used widely in elliptic/crypto/web3 libs.
import BN from "bn.js";

const lines = [];

lines.push("1 " + new BN(123).add(new BN(456)).toString());
lines.push("2 " + new BN("ffffffff", 16).toString(16));
lines.push("3 " + new BN(10).pow(new BN(20)).toString());
lines.push("4 " + new BN(7).mul(new BN(13)).toString());
lines.push("5 " + new BN(100).mod(new BN(7)).toString());
lines.push("6 " + new BN("1234567890abcdef", 16).toString(10));

process.stdout.write(lines.join("\n") + "\n");
