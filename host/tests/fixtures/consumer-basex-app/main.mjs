import baseX from "base-x";

const lines = [];

// 1: bs58 (bitcoin alphabet)
{
  const bs58 = baseX("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz");
  const bytes = new Uint8Array([1, 2, 3, 4, 5]);
  const enc = bs58.encode(bytes);
  const dec = bs58.decode(enc);
  const back = Array.from(dec);
  lines.push("1 enc=" + enc + " roundTripEq=" + (back.join(",") === "1,2,3,4,5"));
}

// 2: base62 round-trip
{
  const b62 = baseX("0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ");
  const bytes = new Uint8Array([255, 128, 64, 32, 16]);
  const enc = b62.encode(bytes);
  const back = Array.from(b62.decode(enc));
  lines.push("2 enc=" + enc + " ok=" + (back.join(",") === "255,128,64,32,16"));
}

// 3: empty
{
  const bs58 = baseX("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz");
  lines.push("3 enc=" + JSON.stringify(bs58.encode(new Uint8Array(0))));
}

// 4: leading zeros preserved
{
  const bs58 = baseX("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz");
  const enc = bs58.encode(new Uint8Array([0, 0, 0, 1]));
  const dec = Array.from(bs58.decode(enc));
  lines.push("4 enc=" + enc + " back=" + JSON.stringify(dec));
}

// 5: decodeUnsafe returns undefined for invalid chars
{
  const bs58 = baseX("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz");
  const r = bs58.decodeUnsafe("not-base58-!!!");
  lines.push("5 invalid=" + (r === undefined));
}

process.stdout.write(lines.join("\n") + "\n");
