import CryptoJS from "crypto-js";

const md5 = CryptoJS.MD5("hello").toString();
const sha256 = CryptoJS.SHA256("hello").toString();
const hmac = CryptoJS.HmacSHA256("data", "secret").toString();

const wordArr = CryptoJS.enc.Utf8.parse("hello");
const base64 = CryptoJS.enc.Base64.stringify(wordArr);
const fromB64 = CryptoJS.enc.Utf8.stringify(CryptoJS.enc.Base64.parse(base64));

const aesKey = CryptoJS.enc.Utf8.parse("0123456789abcdef0123456789abcdef");
const aesIv = CryptoJS.enc.Utf8.parse("0123456789abcdef");
const ct = CryptoJS.AES.encrypt("secret", aesKey, { iv: aesIv });
const pt = CryptoJS.AES.decrypt(ct, aesKey, { iv: aesIv }).toString(CryptoJS.enc.Utf8);

process.stdout.write(JSON.stringify({
  md5, sha256,
  hmacLen: hmac.length,
  base64,
  fromB64,
  pt,
}) + "\n");
