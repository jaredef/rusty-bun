import { base16, base32, base64, base64url } from "rfc4648";

const lines = [];
const bytes = new Uint8Array([72, 101, 108, 108, 111]);  // "Hello"

lines.push("1 b16=" + base16.stringify(bytes));
lines.push("2 b32=" + base32.stringify(bytes));
lines.push("3 b64=" + base64.stringify(bytes));
lines.push("4 b64url=" + base64url.stringify(bytes));

// round-trips
const back16 = Array.from(base16.parse(base16.stringify(bytes)));
const back32 = Array.from(base32.parse(base32.stringify(bytes)));
const back64 = Array.from(base64.parse(base64.stringify(bytes)));
lines.push("5 rt16=" + (back16.join() === "72,101,108,108,111") +
           " rt32=" + (back32.join() === "72,101,108,108,111") +
           " rt64=" + (back64.join() === "72,101,108,108,111"));

// loose / strict
try { base32.parse("not-base32-!!!"); lines.push("6 NOT_THROWN"); }
catch (e) { lines.push("6 threw=" + (e instanceof Error)); }

process.stdout.write(lines.join("\n") + "\n");
