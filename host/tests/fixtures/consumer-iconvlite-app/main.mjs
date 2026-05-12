import iconv from "iconv-lite";

const stages = {};
try {
  const utf8Buf = iconv.encode("Hello", "utf8");
  stages.utf8 = Array.from(utf8Buf);
  stages.utf8Round = iconv.decode(utf8Buf, "utf8");

  const latin1Buf = iconv.encode("café", "latin1");
  stages.latin1 = Array.from(latin1Buf);
  stages.latin1Round = iconv.decode(latin1Buf, "latin1");

  const utf16leBuf = iconv.encode("AB", "utf16le");
  stages.utf16le = Array.from(utf16leBuf);
  stages.utf16leRound = iconv.decode(utf16leBuf, "utf16le");

  stages.supported = ["utf8", "utf-8", "ascii", "latin1", "win1252", "utf16le"]
    .map(e => [e, iconv.encodingExists(e)]);
} catch (e) {
  stages.error = (e && (e.stack || e.message) || String(e));
}
process.stdout.write(JSON.stringify(stages) + "\n");
