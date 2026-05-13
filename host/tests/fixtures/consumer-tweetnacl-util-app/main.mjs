import nacl from "tweetnacl-util";

const enc = nacl.encodeBase64(new Uint8Array([1, 2, 3, 4]));
const dec = nacl.decodeBase64("AQIDBA==");

const utf8 = nacl.encodeUTF8(new Uint8Array([72, 101, 108, 108, 111]));
const utf8back = nacl.decodeUTF8("Hello");

process.stdout.write(JSON.stringify({
  enc,
  decLen: dec.length,
  decFirst: dec[0],
  utf8,
  utf8backLen: utf8back.length,
}) + "\n");
