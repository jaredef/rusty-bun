import { scrypt } from "scrypt-js";
const pw = new TextEncoder().encode("password");
const salt = new TextEncoder().encode("salt");
const key = await scrypt(pw, salt, 1024, 8, 1, 32);
process.stdout.write(JSON.stringify({ keyLen: key.length, firstByte: key[0] }) + "\n");
